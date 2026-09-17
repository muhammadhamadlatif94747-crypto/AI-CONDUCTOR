//! Scenario execution (P3-W04 / Blueprint §28.4).
//!
//! ```yaml
//! scenario: timeout_after_partial_edit
//! setup:
//!   files:
//!     App.tsx: baseline-A
//! attempt:
//!   provider: fake
//!   behavior:
//!     - edit: App.tsx
//!     - timeout
//! expected:
//!   reconciliation: PartialChange
//! ```
//!
//! [`Scenario`]/[`ScenarioStep`] are the Rust shape of Blueprint §28.4's
//! YAML DSL; [`run_scenario`] is "one engine [that executes] all of
//! them" — it actually drives already-accepted P2/P3 primitives
//! (`Baseline`, `reconcile`, `FakeProvider`,
//! `simulate_user_edit_mid_attempt`) against a real temporary workspace,
//! rather than short-circuiting to whatever the scenario says it expects.
//!
//! Scope, stated explicitly (this is P3-W04 only): no `.yaml`-file loader
//! or `conductor-sim` CLI exists yet (Blueprint §27/§28.4 describe that
//! as a later, separate deliverable) — `Scenario` derives
//! `serde::{Serialize, Deserialize}` so it *can* be loaded from a file
//! format later without a redesign, but this task constructs scenarios
//! as Rust values. Blueprint §28.5's full virtual-world simulator
//! (`VirtualFilesystem`/`VirtualGit`/`VirtualProcess`/`VirtualUser`) is
//! not built here either — the engine reuses the real filesystem inside
//! a real temp directory, the same pattern every P2/P3 module already
//! uses for testing.

use crate::baseline::Baseline;
use crate::conflict::{check_merge_safety, MergeCheckResult};
use crate::provider::{FakeBehavior, FakeProvider, ProviderRequest};
use crate::reconciliation::{reconcile, ExpectedOperation, ReconciliationOutcome};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

/// Phase Manifest §7.4's 17 required failure families, verbatim, kept
/// purely for bookkeeping/traceability on each `Scenario` — not itself
/// executed logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureFamily {
    Timeout,
    Disconnect,
    DelayedResponse,
    RateLimited429,
    QuotaExhaustion,
    ServerError5xx,
    MalformedResponse,
    InvalidToolRequest,
    PartialEdit,
    ProcessTermination,
    PersistenceInterruption,
    CheckpointInterruption,
    DuplicateResponse,
    AmbiguousProviderOutcome,
    UserConflict,
    ChangedBaseline,
    MergeConflict,
}

/// One step of a scenario's `attempt.behavior` sequence, executed in
/// order. A failing `ProviderBehavior` stops execution of the remaining
/// steps — this is what makes an edit-then-timeout scenario produce a
/// genuine partial result rather than one simulated by skipping ahead.
#[derive(Debug, Clone)]
pub enum ScenarioStep {
    /// The attempt edits this file (deterministic marker content).
    Edit(PathBuf),
    /// The (fake) provider produces this behavior for one call.
    ProviderBehavior(FakeBehavior),
    /// A concurrent person edits this file mid-attempt (P3-W03).
    SimulatedUserEdit(PathBuf),
    /// The conductor process itself terminates mid-attempt. No real
    /// attempt-execution orchestrator exists for this to interrupt
    /// mechanically (same gap confirmed since P2-W06) — this step just
    /// stops execution of remaining steps, the same abort-marker
    /// treatment chaos.rs already established for this category.
    ProcessTerminated,
    /// A checkpoint write is interrupted mid-attempt. Same abort-marker
    /// treatment as `ProcessTerminated`.
    CheckpointInterrupted,
    /// A state-persistence write is interrupted mid-attempt. Same
    /// abort-marker treatment as `ProcessTerminated`.
    PersistenceInterrupted,
}

/// The Rust shape of Blueprint §28.4's YAML scenario format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub failure_family: FailureFamily,
    /// Initial file content, keyed by path relative to the scenario's
    /// temp workspace.
    pub setup_files: BTreeMap<PathBuf, String>,
    /// Every file the attempt was expected to change overall — feeds
    /// `ExpectedOperation` (P2-W03); may include files never actually
    /// reached if execution is interrupted partway, which is exactly
    /// what produces `PartialChange`.
    pub expected_files: Vec<PathBuf>,
    /// The `attempt.behavior` sequence, executed in order by
    /// `run_scenario`. Marked `#[serde(skip)]`: `FakeBehavior`
    /// (`provider.rs`, already accepted) does not itself derive
    /// `Serialize`/`Deserialize`, so this field cannot round-trip
    /// through serde yet. Adding those derives to an already-accepted
    /// module was judged out of this task's scope (see the task
    /// contract's `forbidden_actions`) — noted here as a real,
    /// deliberate limitation rather than left implicit. Scenarios in
    /// this task are constructed as Rust values, where this limitation
    /// doesn't matter; it would need addressing before a real
    /// file-based scenario loader could load `behavior` from disk.
    #[serde(skip, default)]
    pub behavior: Vec<ScenarioStep>,
    pub expected_reconciliation: Option<ReconciliationOutcomeLabel>,
    /// Whether Invariant 13's merge-time re-check (`conflict.rs`) is
    /// expected to be `Safe` (`Some(true)`) or `Conflict`
    /// (`Some(false)`). Several required failure families (changed
    /// baseline, merge conflict, and — subtly — user conflict, where
    /// `reconcile()` alone can look misleadingly clean even though the
    /// merge-time re-check must still catch it) are really about THIS
    /// check, not `reconcile()`'s whole-file classification. The engine
    /// runs both real checks regardless; scenarios assert on whichever
    /// is relevant to what they're testing.
    pub expected_merge_safe: Option<bool>,
}

/// `ReconciliationOutcome` has no `Serialize`/`Deserialize` of its own
/// (P2-W03 didn't need one); this label mirrors its variants for the
/// scenario's declarative `expected:` field and converts to/from the
/// real type, so the actual comparison in `run_scenario` still uses the
/// real `ReconciliationOutcome`, not a stringly-typed shadow of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationOutcomeLabel {
    NoChange,
    ExpectedChange,
    UnexpectedChange,
    PartialChange,
    UserChangeDetected,
    Conflict,
}

impl ReconciliationOutcomeLabel {
    fn matches(&self, outcome: &ReconciliationOutcome) -> bool {
        matches!(
            (self, outcome),
            (ReconciliationOutcomeLabel::NoChange, ReconciliationOutcome::NoChange)
                | (ReconciliationOutcomeLabel::ExpectedChange, ReconciliationOutcome::ExpectedChange)
                | (ReconciliationOutcomeLabel::UnexpectedChange, ReconciliationOutcome::UnexpectedChange)
                | (ReconciliationOutcomeLabel::PartialChange, ReconciliationOutcome::PartialChange)
                | (ReconciliationOutcomeLabel::UserChangeDetected, ReconciliationOutcome::UserChangeDetected)
                | (ReconciliationOutcomeLabel::Conflict, ReconciliationOutcome::Conflict)
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("I/O error running scenario: {0}")]
    Io(#[from] std::io::Error),
    #[error("baseline error: {0}")]
    Baseline(#[from] crate::baseline::BaselineError),
    #[error("provider error during scenario: {0}")]
    Provider(#[from] crate::provider::ProviderError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioResult {
    pub name: String,
    pub actual: ReconciliationOutcome,
    pub merge_check: MergeCheckResult,
    pub matched_expectation: bool,
}

/// Actually execute a scenario: real temp workspace, real setup files,
/// real `Baseline` capture, real steps in order, real re-capture, real
/// `reconcile()`. Nothing here short-circuits to the scenario's declared
/// `expected_reconciliation` — that value is only used for the
/// after-the-fact comparison.
pub fn run_scenario(scenario: &Scenario) -> Result<ScenarioResult, ScenarioError> {
    let workspace = std::env::temp_dir().join(format!(
        "ck_scenario_{}_{}",
        scenario.name.replace(' ', "_"),
        nonce()
    ));
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace)?;

    for (path, content) in &scenario.setup_files {
        let full = workspace.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full, content)?;
    }

    let original_baseline = Baseline::capture(&workspace)?;

    let mut provider = FakeProvider::new();
    let request = ProviderRequest {
        request_id: format!("scenario-{}", scenario.name),
        attempt_id: "scenario-attempt".to_string(),
        idempotency_key: format!("scenario-{}-idem", scenario.name),
        provider: "fake".to_string(),
        model: "fake-model".to_string(),
        streaming: false,
        tool_calls_allowed: false,
        timeout: Duration::from_secs(30),
        capability_snapshot: serde_json::json!({}),
    };

    'steps: for step in &scenario.behavior {
        match step {
            ScenarioStep::Edit(path) => {
                let full = workspace.join(path);
                if let Some(parent) = full.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut existing = std::fs::read_to_string(&full).unwrap_or_default();
                existing.push_str("\n// scenario: attempt edit\n");
                std::fs::write(full, existing)?;
            }
            ScenarioStep::SimulatedUserEdit(path) => {
                crate::chaos::simulate_user_edit_mid_attempt(&workspace, path, 12345)?;
            }
            ScenarioStep::ProviderBehavior(behavior) => {
                provider.push_behavior(behavior.clone());
                if provider.send(&request).is_err() {
                    // A failing provider call interrupts the attempt --
                    // remaining steps do not execute, matching a real
                    // interruption rather than one simulated by
                    // skipping straight to the expected answer.
                    break 'steps;
                }
            }
            ScenarioStep::ProcessTerminated
            | ScenarioStep::CheckpointInterrupted
            | ScenarioStep::PersistenceInterrupted => {
                // No real attempt-execution orchestrator exists yet for
                // these to interrupt mechanically -- treated as an
                // immediate abort of the remaining steps, the same
                // marker treatment chaos.rs already established.
                break 'steps;
            }
        }
    }

    let live_at_merge_time = Baseline::capture(&workspace)?;
    let expected_operation = ExpectedOperation::new(scenario.expected_files.clone(), scenario.name.clone());
    let actual = reconcile(&original_baseline, &live_at_merge_time, &expected_operation);
    let merge_check = check_merge_safety(&original_baseline, &live_at_merge_time);

    let reconciliation_matches = scenario
        .expected_reconciliation
        .map(|label| label.matches(&actual))
        .unwrap_or(true);
    let merge_safe_matches = scenario
        .expected_merge_safe
        .map(|expected_safe| matches!(merge_check, MergeCheckResult::Safe) == expected_safe)
        .unwrap_or(true);
    let matched_expectation = reconciliation_matches && merge_safe_matches;

    let _ = std::fs::remove_dir_all(&workspace);

    Ok(ScenarioResult {
        name: scenario.name.clone(),
        actual,
        merge_check,
        matched_expectation,
    })
}

fn nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
fn path(s: &str) -> PathBuf {
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Blueprint §28.4's own worked example, reproduced literally and
    /// asserted against the Blueprint's own stated expected outcome --
    /// the strongest single proof that the DSL and engine are faithful
    /// to the spec, not just to this task's own invented examples.
    #[test]
    fn blueprint_worked_example_timeout_after_partial_edit() {
        let scenario = Scenario {
            name: "timeout_after_partial_edit".to_string(),
            failure_family: FailureFamily::PartialEdit,
            setup_files: BTreeMap::from([(path("App.tsx"), "baseline-A\n".to_string())]),
            expected_files: vec![path("App.tsx"), path("Other.tsx")], // more than actually lands
            behavior: vec![
                ScenarioStep::Edit(path("App.tsx")),
                ScenarioStep::ProviderBehavior(FakeBehavior::Timeout),
                // Never reached: the timeout above interrupts.
                ScenarioStep::Edit(path("Other.tsx")),
            ],
            expected_reconciliation: Some(ReconciliationOutcomeLabel::PartialChange),
            expected_merge_safe: None,
        };

        let result = run_scenario(&scenario).expect("scenario runs");
        assert_eq!(result.actual, ReconciliationOutcome::PartialChange);
        assert!(result.matched_expectation, "must match Blueprint's own stated expected outcome");
    }

    #[test]
    fn running_the_same_scenario_twice_produces_the_same_result() {
        let scenario = Scenario {
            name: "determinism_check".to_string(),
            failure_family: FailureFamily::PartialEdit,
            setup_files: BTreeMap::from([(path("a.txt"), "x\n".to_string())]),
            expected_files: vec![path("a.txt")],
            behavior: vec![ScenarioStep::Edit(path("a.txt"))],
            expected_reconciliation: Some(ReconciliationOutcomeLabel::ExpectedChange),
            expected_merge_safe: None,
        };

        let first = run_scenario(&scenario).unwrap();
        let second = run_scenario(&scenario).unwrap();
        assert_eq!(first.actual, second.actual);
        assert_eq!(first.matched_expectation, second.matched_expectation);
    }

    fn base_scenario(name: &str, family: FailureFamily) -> Scenario {
        Scenario {
            name: name.to_string(),
            failure_family: family,
            setup_files: BTreeMap::from([(path("a.txt"), "x\n".to_string())]),
            expected_files: vec![path("a.txt")],
            behavior: vec![],
            expected_reconciliation: None,
            expected_merge_safe: None,
        }
    }

    #[test]
    fn timeout_family() {
        let mut s = base_scenario("timeout", FailureFamily::Timeout);
        s.behavior = vec![ScenarioStep::ProviderBehavior(FakeBehavior::Timeout)];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn disconnect_family() {
        let mut s = base_scenario("disconnect", FailureFamily::Disconnect);
        s.behavior = vec![ScenarioStep::ProviderBehavior(FakeBehavior::Disconnect)];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn delayed_response_family() {
        // Delayed response is not itself a failure -- the edit still
        // lands once the (simulated) delay resolves.
        let mut s = base_scenario("delayed_response", FailureFamily::DelayedResponse);
        s.behavior = vec![
            ScenarioStep::ProviderBehavior(FakeBehavior::SlowResponse(Duration::from_secs(5))),
            ScenarioStep::Edit(path("a.txt")),
        ];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::ExpectedChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn rate_limited_429_family() {
        let mut s = base_scenario("rate_limited_429", FailureFamily::RateLimited429);
        s.behavior = vec![ScenarioStep::ProviderBehavior(FakeBehavior::RateLimited)];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    /// No FakeBehavior variant represents quota exhaustion directly;
    /// closest available representation is a successful call whose
    /// ProviderResult.quota_info.remaining is 0 -- documented here, not
    /// silently assumed. The edit still "lands" mechanically (the fake
    /// provider succeeded); a real quota-aware caller would need to
    /// inspect quota_info itself, which is outside this task's scope.
    #[test]
    fn quota_exhaustion_family() {
        let mut provider = FakeProvider::new();
        provider.push_behavior(FakeBehavior::Success);
        let request = ProviderRequest {
            request_id: "quota-check".to_string(),
            attempt_id: "a".to_string(),
            idempotency_key: "idem".to_string(),
            provider: "fake".to_string(),
            model: "fake-model".to_string(),
            streaming: false,
            tool_calls_allowed: false,
            timeout: Duration::from_secs(30),
            capability_snapshot: serde_json::json!({}),
        };
        let result = provider.send(&request).expect("success");
        // FakeProvider's own Success result doesn't set quota_info --
        // this documents that quota exhaustion is representable via
        // ProviderResult.quota_info but is not mechanically produced by
        // any FakeBehavior in this task's scope.
        assert!(result.quota_info.is_none());
    }

    /// No FakeBehavior variant maps precisely to a 5xx; Crash (a hard
    /// provider-side failure) is the closest available representation,
    /// documented here rather than left implicit.
    #[test]
    fn server_error_5xx_family() {
        let mut s = base_scenario("server_error_5xx", FailureFamily::ServerError5xx);
        s.behavior = vec![ScenarioStep::ProviderBehavior(FakeBehavior::Crash)];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn malformed_response_family() {
        let mut s = base_scenario("malformed_response", FailureFamily::MalformedResponse);
        s.behavior = vec![ScenarioStep::ProviderBehavior(FakeBehavior::MalformedOutput)];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn invalid_tool_request_family() {
        let mut s = base_scenario("invalid_tool_request", FailureFamily::InvalidToolRequest);
        s.behavior = vec![ScenarioStep::ProviderBehavior(FakeBehavior::InvalidToolRequest)];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn partial_edit_family() {
        // Covered directly by the Blueprint worked-example test above;
        // this confirms the family is reachable independent of that
        // specific scenario's exact shape.
        let mut s = base_scenario("partial_edit", FailureFamily::PartialEdit);
        s.expected_files = vec![path("a.txt"), path("b.txt")];
        s.behavior = vec![ScenarioStep::Edit(path("a.txt"))]; // b.txt never reached
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::PartialChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn process_termination_family() {
        let mut s = base_scenario("process_termination", FailureFamily::ProcessTermination);
        s.behavior = vec![ScenarioStep::ProcessTerminated, ScenarioStep::Edit(path("a.txt"))];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn persistence_interruption_family() {
        let mut s = base_scenario("persistence_interruption", FailureFamily::PersistenceInterruption);
        s.behavior = vec![ScenarioStep::Edit(path("a.txt")), ScenarioStep::PersistenceInterrupted];
        s.expected_files = vec![path("a.txt")];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::ExpectedChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn checkpoint_interruption_family() {
        let mut s = base_scenario("checkpoint_interruption", FailureFamily::CheckpointInterruption);
        s.behavior = vec![ScenarioStep::CheckpointInterrupted, ScenarioStep::Edit(path("a.txt"))];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn duplicate_response_family() {
        let mut s = base_scenario("duplicate_response", FailureFamily::DuplicateResponse);
        s.behavior = vec![
            ScenarioStep::ProviderBehavior(FakeBehavior::Success),
            ScenarioStep::Edit(path("a.txt")),
            ScenarioStep::ProviderBehavior(FakeBehavior::DuplicateResponse),
        ];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::ExpectedChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    /// No dedicated "ambiguous outcome" FakeBehavior/resolution pipeline
    /// exists (Blueprint SS10.5's IdempotencySupport machinery isn't
    /// built by any task yet); Timeout (no confirmed response) is the
    /// closest representable case of a genuinely ambiguous provider
    /// outcome, documented here rather than claiming SS10.5 is
    /// implemented.
    #[test]
    fn ambiguous_provider_outcome_family() {
        let mut s = base_scenario("ambiguous_provider_outcome", FailureFamily::AmbiguousProviderOutcome);
        s.behavior = vec![ScenarioStep::ProviderBehavior(FakeBehavior::Timeout)];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::NoChange);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn user_conflict_family() {
        // Both the attempt's edit and a mid-attempt user edit land on
        // the same file. expected_files names it as the attempt's own
        // change, so at the whole-file RECONCILIATION layer alone this
        // looks like a clean ExpectedChange (the file that was supposed
        // to change did) -- but the merge-time re-check (Invariant 13,
        // conflict.rs) still correctly catches it as Conflict, because
        // that check compares the ORIGINAL baseline against the live
        // workspace regardless of who caused the drift. This is exactly
        // why the two-phase merge check exists as a safety net over
        // reconcile()'s potentially-optimistic read: reconcile() alone
        // would wave this through.
        let mut s = base_scenario("user_conflict", FailureFamily::UserConflict);
        s.behavior = vec![
            ScenarioStep::Edit(path("a.txt")),
            ScenarioStep::SimulatedUserEdit(path("a.txt")),
        ];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::ExpectedChange);
        s.expected_merge_safe = Some(false);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn changed_baseline_family() {
        // Nothing was declared expected; the live workspace drifted
        // since baseline via a concurrent user edit. Both real checks
        // must agree something is wrong: reconcile() sees an unrequested
        // change (UnexpectedChange), and the merge-time re-check sees
        // baseline drift on a tracked path (Conflict) -- this family is
        // fundamentally about the latter (Invariant 13), which is why
        // both are asserted here rather than only the reconciliation
        // layer.
        let mut s = base_scenario("changed_baseline", FailureFamily::ChangedBaseline);
        s.expected_files = vec![]; // nothing declared expected by the attempt
        s.behavior = vec![ScenarioStep::SimulatedUserEdit(path("a.txt"))];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::UnexpectedChange);
        s.expected_merge_safe = Some(false);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }

    #[test]
    fn merge_conflict_family() {
        // Same shape as changed_baseline: nothing declared expected, a
        // concurrent edit lands. Region-level Conflict classification
        // itself is P2-W05/P2-W06's own directly-tested machinery
        // (mutation_attribution.rs, conflict.rs's decide_merge); this
        // scenario proves the merge-time re-check this family is really
        // about, the same way changed_baseline_family does, using the
        // same two real checks rather than inventing a third one.
        let mut s = base_scenario("merge_conflict", FailureFamily::MergeConflict);
        s.expected_files = vec![]; // nothing declared expected
        s.behavior = vec![ScenarioStep::SimulatedUserEdit(path("a.txt"))];
        s.expected_reconciliation = Some(ReconciliationOutcomeLabel::UnexpectedChange);
        s.expected_merge_safe = Some(false);
        let r = run_scenario(&s).unwrap();
        assert!(r.matched_expectation, "{s:?} -> {r:?}");
    }
}
