//! External side-effect idempotency classification (AC-11; Blueprint
//! §10.5, Invariant 16).
//!
//! This module is deliberately small and deliberately separate from
//! `idempotency_store`. AC-11 §4 requires two distinct artifacts:
//!
//!   - `idempotency.rs` (this file) — WHAT a provider/capability promises
//!     about retry safety. Set once per provider/combo in the Capability
//!     Registry, from documented/confirmed provider behavior.
//!   - `idempotency_store.rs` — WHETHER *this process* has already
//!     attempted a given local operation. Set continuously, locally,
//!     every time an operation runs.
//!
//! These must never merge into one signal. Knowing "I already attempted
//! this locally" says nothing about whether the provider actually
//! deduplicated it — that gap is exactly what Invariant 16 exists to
//! close (Blueprint §10.5). `safe_to_skip_retry` below is the one place
//! the two signals are allowed to meet, and it is intentionally
//! conservative: local-applied alone is never enough.

/// What a provider/capability documents or confirms about how it treats
/// repeated requests bearing the same idempotency key.
///
/// Default is `Unknown` — matching Blueprint §10.3's staleness philosophy
/// of "don't assume." A provider/combo must be explicitly verified before
/// it may be classified as anything other than `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum IdempotencySupport {
    /// Provider documents/confirms it deduplicates by idempotency key —
    /// safe to retry automatically with the same key.
    ServerSideGuaranteed,
    /// Provider behavior on retry-with-same-key is not confirmed — treat
    /// as NOT safe to blind-retry. This is the default.
    Unknown,
    /// Provider explicitly does not deduplicate — a retry after an
    /// ambiguous outcome is a genuinely new side effect.
    NotIdempotent,
}

impl Default for IdempotencySupport {
    fn default() -> Self {
        IdempotencySupport::Unknown
    }
}

/// What to do about an ambiguous request outcome (timeout, disconnect, no
/// confirmed response), per Blueprint §10.5's decision tree. Deliberately
/// does NOT include a plain "skip / already done" branch reachable from
/// local bookkeeping alone — see `safe_to_skip_retry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// `ServerSideGuaranteed`: safe to retry automatically with the same
    /// idempotency key.
    RetryWithSameKey,
    /// `Unknown` / `NotIdempotent` and the provider can be asked whether
    /// the original request actually completed: query first, reconcile
    /// against that ground truth, then proceed.
    QueryProviderThenReconcile,
    /// `Unknown` / `NotIdempotent` and the provider cannot be queried:
    /// surface as Unknown to the person rather than silently risk a
    /// double-execution.
    SurfaceAsUnknownToHuman,
}

/// Blueprint §10.5's ambiguous-outcome decision rule, made explicit and
/// exhaustively testable rather than left as prose someone has to
/// remember correctly at each call site.
pub fn decide_on_ambiguous_outcome(
    support: IdempotencySupport,
    provider_is_queryable: bool,
) -> RetryDecision {
    match support {
        IdempotencySupport::ServerSideGuaranteed => RetryDecision::RetryWithSameKey,
        IdempotencySupport::Unknown | IdempotencySupport::NotIdempotent => {
            if provider_is_queryable {
                RetryDecision::QueryProviderThenReconcile
            } else {
                RetryDecision::SurfaceAsUnknownToHuman
            }
        }
    }
}

/// The one function where a local "already applied" signal (from
/// `idempotency_store`) and a provider's `IdempotencySupport`
/// classification are allowed to meet. It is intentionally narrow:
/// knowing the local process already attempted an operation is
/// insufficient on its own (AC-11 §2 — "generating a key never upgrades
/// a classification"). Skipping a retry as genuinely safe requires BOTH
/// the local record AND a provider-side guarantee that repeats are
/// harmless.
///
/// This function must never be satisfied by `applied_locally` alone —
/// that would be exactly the classification-upgrade AC-11 forbids. Every
/// branch below is exercised in `tests::` to keep it that way.
pub fn safe_to_skip_retry(applied_locally: bool, support: IdempotencySupport) -> bool {
    applied_locally && support == IdempotencySupport::ServerSideGuaranteed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_classification_is_unknown() {
        assert_eq!(IdempotencySupport::default(), IdempotencySupport::Unknown);
    }

    #[test]
    fn three_variants_are_distinct() {
        assert_ne!(IdempotencySupport::ServerSideGuaranteed, IdempotencySupport::Unknown);
        assert_ne!(IdempotencySupport::Unknown, IdempotencySupport::NotIdempotent);
        assert_ne!(
            IdempotencySupport::ServerSideGuaranteed,
            IdempotencySupport::NotIdempotent
        );
    }

    #[test]
    fn server_side_guaranteed_retries_with_same_key() {
        assert_eq!(
            decide_on_ambiguous_outcome(IdempotencySupport::ServerSideGuaranteed, true),
            RetryDecision::RetryWithSameKey
        );
        assert_eq!(
            decide_on_ambiguous_outcome(IdempotencySupport::ServerSideGuaranteed, false),
            RetryDecision::RetryWithSameKey,
            "guaranteed dedup means queryability is irrelevant"
        );
    }

    #[test]
    fn unknown_or_not_idempotent_queries_when_queryable() {
        assert_eq!(
            decide_on_ambiguous_outcome(IdempotencySupport::Unknown, true),
            RetryDecision::QueryProviderThenReconcile
        );
        assert_eq!(
            decide_on_ambiguous_outcome(IdempotencySupport::NotIdempotent, true),
            RetryDecision::QueryProviderThenReconcile
        );
    }

    #[test]
    fn unknown_or_not_idempotent_surfaces_to_human_when_not_queryable() {
        assert_eq!(
            decide_on_ambiguous_outcome(IdempotencySupport::Unknown, false),
            RetryDecision::SurfaceAsUnknownToHuman
        );
        assert_eq!(
            decide_on_ambiguous_outcome(IdempotencySupport::NotIdempotent, false),
            RetryDecision::SurfaceAsUnknownToHuman
        );
    }

    /// The AC-11 separation property, made concrete: for every one of the
    /// six (applied_locally × support) combinations, `safe_to_skip_retry`
    /// is true in EXACTLY one — applied_locally=true AND
    /// ServerSideGuaranteed. In particular, applied_locally=true can
    /// never by itself flip the result for Unknown or NotIdempotent —
    /// local bookkeeping never upgrades the provider's classification.
    #[test]
    fn local_applied_flag_never_upgrades_classification() {
        let supports = [
            IdempotencySupport::ServerSideGuaranteed,
            IdempotencySupport::Unknown,
            IdempotencySupport::NotIdempotent,
        ];
        for &support in &supports {
            for &applied_locally in &[true, false] {
                let expected =
                    applied_locally && support == IdempotencySupport::ServerSideGuaranteed;
                assert_eq!(
                    safe_to_skip_retry(applied_locally, support),
                    expected,
                    "applied_locally={applied_locally}, support={support:?}"
                );
            }
        }

        // Named cases for the two that matter most to catch a regression:
        assert!(
            !safe_to_skip_retry(true, IdempotencySupport::Unknown),
            "local 'already applied' must NOT be treated as safe-to-skip \
             when the provider's classification is Unknown"
        );
        assert!(
            !safe_to_skip_retry(true, IdempotencySupport::NotIdempotent),
            "local 'already applied' must NOT be treated as safe-to-skip \
             when the provider is known NOT to deduplicate"
        );
        assert!(
            !safe_to_skip_retry(false, IdempotencySupport::ServerSideGuaranteed),
            "a provider guarantee alone, with nothing applied locally yet, \
             is not 'skip' either — there is nothing to skip"
        );
    }
}
