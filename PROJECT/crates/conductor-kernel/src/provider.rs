//! Provider abstraction & Fake Provider (P3-W01 / Blueprint §10.1, §10.2).
//!
//! `FakeProvider` is the deterministic, replayable foundation Phase 3's
//! later work packages build on: `VirtualClock` (P3-W02) will give
//! [`FakeBehavior::SlowResponse`]'s recorded `Duration` somewhere real to
//! resolve against; Chaos Mode (P3-W03) will layer seeded randomness over
//! this task's explicit script; the Scenario DSL (P3-W04) will drive
//! scripts from versioned scenario files. None of that exists yet — this
//! module is deliberately just the primitive: a provider whose behavior
//! is entirely, deterministically, explicitly controlled by whoever is
//! testing against it.
//!
//! **Deliberate, flagged deviation from Blueprint §10.1:** the Blueprint
//! specifies `async fn send()`. No async runtime exists anywhere in this
//! crate (every module from Phase 0 through Phase 2 is synchronous), and
//! adding a first-time `tokio`-equivalent dependency is a bigger
//! architectural decision than this task should make unilaterally.
//! `FakeProvider::send` is synchronous instead. Whichever future task
//! wires in a real, network-bound provider adapter will need real async
//! I/O and will need to revisit this — noted here rather than silently
//! diverging from the spec without saying so.

use std::collections::VecDeque;
use std::time::Duration;

/// Blueprint §10.1's thickened `ProviderRequest`. `capability_snapshot`
/// is typed as a `serde_json::Value` passthrough rather than Section
/// 10.3's full capability-registry shape — designing that registry is
/// not this task's job; a passthrough lets a later phase populate it
/// without a breaking change here.
#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub request_id: String,
    pub attempt_id: String,
    pub idempotency_key: String,
    pub provider: String,
    pub model: String,
    pub streaming: bool,
    pub tool_calls_allowed: bool,
    pub timeout: Duration,
    pub capability_snapshot: serde_json::Value,
}

/// Blueprint §10.1's `ProviderResult`, plus one addition this task needs:
/// `simulated_delay`, so [`FakeBehavior::SlowResponse`] has somewhere
/// real to record the duration it represents (not acted on here — see
/// module docs; P3-W02's `VirtualClock` is what will eventually resolve
/// it against real or virtual time).
#[derive(Debug, Clone)]
pub struct ProviderResult {
    pub provider_response_id: Option<String>,
    pub usage: Option<UsageInfo>,
    pub quota_info: Option<QuotaInfo>,
    pub rate_limit_info: Option<RateLimitInfo>,
    pub raw_metadata: serde_json::Value,
    pub sanitized_metadata: serde_json::Value,
    pub simulated_delay: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageInfo {
    pub tokens_used: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaInfo {
    pub remaining: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitInfo {
    pub retry_after: Option<Duration>,
    pub limit: Option<u64>,
}

/// Blueprint §10.2's fault menu, reproduced verbatim — ten variants, no
/// more, no fewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeBehavior {
    Success,
    Timeout,
    RateLimited,
    MalformedOutput,
    PartialOutput,
    Disconnect,
    SlowResponse(Duration),
    Crash,
    DuplicateResponse,
    InvalidToolRequest,
}

/// Every non-success `FakeBehavior` produces a distinguishable variant
/// here — a caller must be able to tell these apart programmatically,
/// not just by matching a string message.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderError {
    #[error("simulated timeout")]
    Timeout,
    #[error("simulated rate limit")]
    RateLimited,
    #[error("simulated malformed output")]
    MalformedOutput,
    #[error("simulated partial output: {partial:?}")]
    PartialOutput { partial: String },
    #[error("simulated disconnect")]
    Disconnect,
    #[error("simulated crash")]
    Crash,
    #[error("simulated invalid tool request: {details}")]
    InvalidToolRequest { details: String },
    /// `send` was called with no scripted behavior queued. Fail-closed —
    /// matching this crate's established discipline (baseline.rs,
    /// worktree.rs, mutation_attribution.rs) of never silently guessing
    /// a safe-looking default when the caller hasn't said what should
    /// happen.
    #[error("FakeProvider script exhausted: no behavior queued for this call")]
    ScriptExhausted,
    /// `FakeBehavior::DuplicateResponse` was scripted, but no prior
    /// successful result exists to duplicate. Fail-closed rather than
    /// fabricating a plausible-looking duplicate out of nothing.
    #[error("DuplicateResponse scripted but no prior successful result exists to duplicate")]
    NoDuplicateAvailable,
}

/// A script-driven, synchronous fake provider. Behaviors are queued
/// explicitly via [`FakeProvider::push_behavior`] and consumed in FIFO
/// order by [`FakeProvider::send`] — the ordering is the whole point:
/// deterministic and replayable, per Phase Manifest §7.5's P3 gate
/// definition.
#[derive(Debug, Default)]
pub struct FakeProvider {
    script: VecDeque<FakeBehavior>,
    last_success: Option<ProviderResult>,
}

impl FakeProvider {
    pub fn new() -> Self {
        FakeProvider {
            script: VecDeque::new(),
            last_success: None,
        }
    }

    /// Queue one behavior to be consumed by the next `send` call.
    /// Multiple calls queue multiple behaviors in the order pushed.
    pub fn push_behavior(&mut self, behavior: FakeBehavior) {
        self.script.push_back(behavior);
    }

    /// Convenience for queuing several behaviors in one call, in order.
    pub fn push_behaviors(&mut self, behaviors: impl IntoIterator<Item = FakeBehavior>) {
        for b in behaviors {
            self.push_behavior(b);
        }
    }

    /// Consume the next scripted behavior and produce its outcome.
    /// `request` is accepted (mirroring the real `ProviderAdapter`
    /// shape) but this fake does not inspect its contents to decide the
    /// outcome — the outcome is entirely determined by the script, which
    /// is the point: fully controllable, not content-dependent.
    pub fn send(&mut self, request: &ProviderRequest) -> Result<ProviderResult, ProviderError> {
        let behavior = self.script.pop_front().ok_or(ProviderError::ScriptExhausted)?;

        match behavior {
            FakeBehavior::Success => {
                let result = success_result(request, None);
                self.last_success = Some(result.clone());
                Ok(result)
            }
            FakeBehavior::SlowResponse(delay) => {
                let result = success_result(request, Some(delay));
                self.last_success = Some(result.clone());
                Ok(result)
            }
            FakeBehavior::DuplicateResponse => {
                self.last_success.clone().ok_or(ProviderError::NoDuplicateAvailable)
            }
            FakeBehavior::Timeout => Err(ProviderError::Timeout),
            FakeBehavior::RateLimited => Err(ProviderError::RateLimited),
            FakeBehavior::MalformedOutput => Err(ProviderError::MalformedOutput),
            FakeBehavior::PartialOutput => Err(ProviderError::PartialOutput {
                partial: "truncated simulated output".to_string(),
            }),
            FakeBehavior::Disconnect => Err(ProviderError::Disconnect),
            FakeBehavior::Crash => Err(ProviderError::Crash),
            FakeBehavior::InvalidToolRequest => Err(ProviderError::InvalidToolRequest {
                details: "simulated tool call the request did not authorize".to_string(),
            }),
        }
    }
}

fn success_result(request: &ProviderRequest, simulated_delay: Option<Duration>) -> ProviderResult {
    ProviderResult {
        provider_response_id: Some(format!("fake-response-{}", request.request_id)),
        usage: Some(UsageInfo { tokens_used: 0 }),
        quota_info: None,
        rate_limit_info: None,
        raw_metadata: serde_json::json!({}),
        sanitized_metadata: serde_json::json!({}),
        simulated_delay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str) -> ProviderRequest {
        ProviderRequest {
            request_id: id.to_string(),
            attempt_id: "a-1".to_string(),
            idempotency_key: format!("idem-{id}"),
            provider: "fake".to_string(),
            model: "fake-model".to_string(),
            streaming: false,
            tool_calls_allowed: false,
            timeout: Duration::from_secs(30),
            capability_snapshot: serde_json::json!({}),
        }
    }

    #[test]
    fn success_produces_an_ok_result_with_a_response_id() {
        let mut p = FakeProvider::new();
        p.push_behavior(FakeBehavior::Success);
        let result = p.send(&request("r1")).expect("success");
        assert_eq!(result.provider_response_id, Some("fake-response-r1".to_string()));
        assert_eq!(result.simulated_delay, None);
    }

    #[test]
    fn each_failure_behavior_produces_its_own_distinguishable_error_variant() {
        let cases = [
            (FakeBehavior::Timeout, ProviderError::Timeout),
            (FakeBehavior::RateLimited, ProviderError::RateLimited),
            (FakeBehavior::MalformedOutput, ProviderError::MalformedOutput),
            (FakeBehavior::Disconnect, ProviderError::Disconnect),
            (FakeBehavior::Crash, ProviderError::Crash),
        ];
        for (behavior, expected) in cases {
            let mut p = FakeProvider::new();
            p.push_behavior(behavior.clone());
            let err = p.send(&request("r1")).unwrap_err();
            assert_eq!(err, expected, "behavior {behavior:?} produced the wrong error variant");
        }
    }

    #[test]
    fn partial_output_carries_the_partial_content() {
        let mut p = FakeProvider::new();
        p.push_behavior(FakeBehavior::PartialOutput);
        match p.send(&request("r1")).unwrap_err() {
            ProviderError::PartialOutput { partial } => assert!(!partial.is_empty()),
            other => panic!("expected PartialOutput, got {other:?}"),
        }
    }

    #[test]
    fn invalid_tool_request_carries_details() {
        let mut p = FakeProvider::new();
        p.push_behavior(FakeBehavior::InvalidToolRequest);
        match p.send(&request("r1")).unwrap_err() {
            ProviderError::InvalidToolRequest { details } => assert!(!details.is_empty()),
            other => panic!("expected InvalidToolRequest, got {other:?}"),
        }
    }

    #[test]
    fn slow_response_records_the_delay_without_sleeping() {
        let mut p = FakeProvider::new();
        p.push_behavior(FakeBehavior::SlowResponse(Duration::from_secs(120)));
        let start = std::time::Instant::now();
        let result = p.send(&request("r1")).expect("slow response is still Ok");
        assert!(start.elapsed() < Duration::from_millis(50), "must not actually sleep");
        assert_eq!(result.simulated_delay, Some(Duration::from_secs(120)));
    }

    #[test]
    fn duplicate_response_replays_the_prior_successful_response_id() {
        let mut p = FakeProvider::new();
        p.push_behaviors([FakeBehavior::Success, FakeBehavior::DuplicateResponse]);
        let first = p.send(&request("r1")).expect("first success");
        let dup = p.send(&request("r2")).expect("duplicate replays prior result");
        assert_eq!(first.provider_response_id, dup.provider_response_id);
    }

    #[test]
    fn duplicate_response_with_nothing_to_duplicate_fails_closed() {
        let mut p = FakeProvider::new();
        p.push_behavior(FakeBehavior::DuplicateResponse);
        let err = p.send(&request("r1")).unwrap_err();
        assert_eq!(err, ProviderError::NoDuplicateAvailable);
    }

    #[test]
    fn an_empty_script_fails_closed_rather_than_defaulting_to_success() {
        let mut p = FakeProvider::new();
        let err = p.send(&request("r1")).unwrap_err();
        assert_eq!(err, ProviderError::ScriptExhausted);
    }

    #[test]
    fn scripted_behaviors_are_consumed_in_the_exact_order_queued() {
        let mut p = FakeProvider::new();
        p.push_behaviors([FakeBehavior::Success, FakeBehavior::Timeout, FakeBehavior::Success]);

        assert!(p.send(&request("r1")).is_ok());
        assert_eq!(p.send(&request("r2")).unwrap_err(), ProviderError::Timeout);
        assert!(p.send(&request("r3")).is_ok());
        // Script now exhausted -- proves nothing was silently reordered
        // or re-consumed.
        assert_eq!(p.send(&request("r4")).unwrap_err(), ProviderError::ScriptExhausted);
    }
}
