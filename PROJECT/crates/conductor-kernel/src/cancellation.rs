//! Cancellation semantics (Blueprint §14, P1-W02).
//!
//! `CancelledClean | CancelledWithChanges | CancellationUnknown` are values of
//! THIS enum — never of `AttemptStatus`. `AttemptStatus` has exactly one
//! terminal cancellation value (`Cancelled`); the detail lives here, mirroring
//! the state/outcome separation pattern of AC-01.
//!
//! Type-level separation guarantee: this enum shares no constructor with
//! `AttemptStatus`, and no conversion exists in either direction. A caller
//! holding a `CancellationOutcome` cannot store it in an attempt-status field
//! without an explicit, reviewable decision.

use crate::state::TransitionActor;

/// The outcome of a completed cancellation sequence (Blueprint §14):
/// stop accepting actions → graceful termination → bounded wait → force-kill
/// → reconciliation → determine final outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationOutcome {
    /// Nothing landed; a clean retry later is safe.
    CancelledClean,
    /// Some partial change landed; needs the same targeted-repair path as
    /// PartialChange — not a blind clean retry.
    CancelledWithChanges,
    /// Reconciliation itself couldn't determine the result — surfaced to the
    /// person, never guessed.
    CancellationUnknown,
}

impl CancellationOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            CancellationOutcome::CancelledClean => "CancelledClean",
            CancellationOutcome::CancelledWithChanges => "CancelledWithChanges",
            CancellationOutcome::CancellationUnknown => "CancellationUnknown",
        }
    }

    /// Recovery consequence per Blueprint §14:
    /// Clean ⇒ safe retry path; WithChanges ⇒ targeted repair (same path as
    /// PartialChange); Unknown ⇒ surface to person, never auto-advance.
    pub fn is_clean_retry_safe(&self) -> bool {
        matches!(self, CancellationOutcome::CancelledClean)
    }

    /// Which actor may record each outcome onto a cancelled attempt's record.
    /// Reconciliation produces the classification; only the authoritative
    /// executor records it (single-writer discipline).
    pub fn recording_actor(&self) -> TransitionActor {
        TransitionActor::AuthoritativeExecutor
    }

    /// Every cancellation carries its own auditability markers (Blueprint §14):
    /// request id + requested_at + per-stage events. This type records the
    /// stage names so emitters cannot drift from the Blueprint's list.
    pub const AUDIT_STAGES: [&'static str; 5] = [
        "cancel_requested",
        "cancel_acknowledged",
        "process_exited",
        "reconciliation_completed",
        "final_cancellation_outcome_determined",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{try_attempt_transition, AttemptStatus};

    #[test]
    fn exactly_three_values_with_stable_identities() {
        let all = [
            CancellationOutcome::CancelledClean,
            CancellationOutcome::CancelledWithChanges,
            CancellationOutcome::CancellationUnknown,
        ];
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].as_str(), "CancelledClean");
        assert_eq!(all[1].as_str(), "CancelledWithChanges");
        assert_eq!(all[2].as_str(), "CancellationUnknown");
    }

    #[test]
    fn no_value_shares_identity_with_attempt_status() {
        use AttemptStatus::*;
        // AttemptStatus has exactly ONE cancellation value: Cancelled.
        // None of the three outcome strings collides with any status string.
        let status_strings = [Pending.as_str(), Running.as_str(), Succeeded.as_str(), Failed.as_str(), Cancelled.as_str(), Unknown.as_str()];
        for outcome in [
            CancellationOutcome::CancelledClean,
            CancellationOutcome::CancelledWithChanges,
            CancellationOutcome::CancellationUnknown,
        ] {
            for s in status_strings {
                assert_ne!(outcome.as_str(), s);
            }
        }
    }

    #[test]
    fn only_cancelled_status_exists_on_attempt_no_detail_leak_possible() {
        use AttemptStatus::*;
        // The attempt lifecycle offers exactly one cancellation terminal.
        // There is no AttemptStatus::CancelledClean/WithChanges/Unknown to leak into;
        // this compiles precisely because those variants do not exist there.
        assert!(!Running.can_transition_to(Cancelled) == false); // edge exists...
        assert!(
            try_attempt_transition(TransitionActor::CancellationPath, Running, Cancelled).is_ok()
        );
        // ...and no other status name encodes cancellation detail:
        for s in [Pending, Running, Succeeded, Failed, Unknown] {
            assert!(!s.as_str().contains("Cancelled") || s == Cancelled);
        }
    }

    #[test]
    fn recovery_consequences_match_blueprint() {
        assert!(CancellationOutcome::CancelledClean.is_clean_retry_safe());
        assert!(!CancellationOutcome::CancelledWithChanges.is_clean_retry_safe());
        assert!(!CancellationOutcome::CancellationUnknown.is_clean_retry_safe());
    }

    #[test]
    fn unknown_outcome_is_never_auto_advanceable() {
        // CancellationUnknown must be surfaced, never guessed: it has no
        // "safe" interpretation and no automatic retry path.
        assert!(!CancellationOutcome::CancellationUnknown.is_clean_retry_safe());
        // WithChanges routes to targeted repair like PartialChange (AC-05 §3),
        // NOT to a blind clean retry.
        assert_eq!(
            CancellationOutcome::CancelledWithChanges.is_clean_retry_safe(),
            false
        );
    }

    #[test]
    fn audit_stage_list_matches_blueprint_14_exactly() {
        assert_eq!(
            CancellationOutcome::AUDIT_STAGES,
            [
                "cancel_requested",
                "cancel_acknowledged",
                "process_exited",
                "reconciliation_completed",
                "final_cancellation_outcome_determined",
            ]
        );
        assert_eq!(CancellationOutcome::AUDIT_STAGES.len(), 5);
    }

    #[test]
    fn recording_actor_is_always_the_executor() {
        for outcome in [
            CancellationOutcome::CancelledClean,
            CancellationOutcome::CancelledWithChanges,
            CancellationOutcome::CancellationUnknown,
        ] {
            assert_eq!(outcome.recording_actor(), TransitionActor::AuthoritativeExecutor);
        }
    }
}
