//! Evidence production (P4-W02 / Blueprint §4.5).
//!
//! Blueprint §4.5's correction: evidence isn't one universal trust
//! ladder — a stale, days-old CI result and a test that ran ninety
//! seconds ago against the exact current attempt are not comparable just
//! because both are "tests." [`Evidence`] models the full record
//! (provenance, freshness, execution binding, reproducibility,
//! integrity) so a Mission Acceptance Contract criterion (§8, P4-W03)
//! can require a *specific combination*, not just a minimum rung — and
//! [`Evidence::matches`] proves that check is real: wrong provenance,
//! staleness, or a mismatched `execution_id` each independently fail a
//! match, so a CI badge or an old test summary cannot satisfy a
//! criterion that actually asked for fresh, current-attempt evidence.
//!
//! # What's verbatim from the Blueprint and what's this task's own call
//!
//! [`EvidenceProvenance`] (six variants) and [`Reproducibility`]
//! (`Deterministic | Flaky | Unknown`) are reproduced exactly as
//! Blueprint §4.5 states them. [`ObservationType`] and
//! [`IntegrityLevel`] are referenced by §4.5's `Evidence` struct but
//! their variants are not enumerated anywhere read for this task — the
//! sets defined here are this task's own reasonable, minimal
//! interpretation, stated as such rather than presented as if found
//! verbatim in the text. `Evidence::result: bool` is also an addition
//! beyond Blueprint's minimal code snippet: evidence with no recorded
//! observed outcome can't usefully satisfy anything, and §8's Mission
//! Acceptance Contract criteria compare against an actual result.
//!
//! Freshness is stored as `produced_at_ms` (a point in time) rather than
//! Blueprint's literal `freshness: Duration` field, and computed on
//! demand against a caller-supplied `now_ms` — a fixed `Duration`
//! captured once would go stale the moment real time passes, which
//! defeats the entire purpose of a freshness check. No method in this
//! module reads the system clock directly (AC-12); every freshness
//! computation takes an explicit `now_ms`, the same pattern P3-W02's
//! `Clock` established.

use crate::verification::{StageOutcome, StageStatus, VerificationStage};
use std::time::Duration;

/// Blueprint §4.5, verbatim — six provenance classes, not rungs on one
/// ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceProvenance {
    ModelClaim,
    ProcessObservation,
    FilesystemObservation,
    LocalTestRunner,
    CiSystem,
    ExternalSystemObservation,
}

/// Blueprint §4.5, verbatim (`Deterministic | Flaky | Unknown`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reproducibility {
    Deterministic,
    Flaky,
    Unknown,
}

/// **Not verbatim from the Blueprint** — see module docs. A minimal,
/// explicitly-interpreted set covering the kinds of checks P4-W01's
/// pipeline stages and Blueprint §8's `verification.type` examples
/// (`test_command`, `dom_check`) actually produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationType {
    TestExecution,
    BuildExecution,
    LintExecution,
    FilesystemCheck,
    DiffCheck,
    DomCheck,
    ProcessObservation,
    ModelAssessment,
}

/// **Not verbatim from the Blueprint** — see module docs. Blueprint
/// §4.5's own field comment ("was this verified independently, or only
/// reported?") is treated as literally defining exactly these two
/// states, not extended with additional granularity that wasn't asked
/// for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityLevel {
    /// The producing process reported this result itself, with nothing
    /// external checking it.
    SelfReported,
    /// Something outside the reporting process independently confirmed
    /// it (e.g. the Verification Engine re-ran the check or inspected
    /// the actual artifact rather than trusting a reported exit code).
    IndependentlyVerified,
}

/// Blueprint §4.5's `Evidence` record.
#[derive(Debug, Clone)]
pub struct Evidence {
    pub provenance: EvidenceProvenance,
    pub observation_type: ObservationType,
    /// When this evidence was produced. Freshness is always computed on
    /// demand against a caller-supplied `now_ms` (see module docs for
    /// why a fixed `Duration` captured once wouldn't work).
    pub produced_at_ms: u64,
    /// Ties this evidence to a specific run, not just "a run once" —
    /// Blueprint §4.5's own phrasing.
    pub execution_id: String,
    /// e.g. `"npm test -- auth.test.ts"`, `"ci:build-4821"`.
    pub source: String,
    pub reproducibility: Reproducibility,
    pub integrity: IntegrityLevel,
    /// Addition beyond Blueprint's minimal snippet — see module docs.
    pub result: bool,
}

impl Evidence {
    /// How long ago this evidence was produced, relative to `now_ms`.
    /// Saturating: if `now_ms` is somehow earlier than `produced_at_ms`
    /// (e.g. a `VirtualClock` misuse), returns `Duration::ZERO` rather
    /// than panicking on subtraction overflow.
    pub fn freshness_relative_to(&self, now_ms: u64) -> Duration {
        Duration::from_millis(now_ms.saturating_sub(self.produced_at_ms))
    }

    pub fn is_fresh(&self, max_freshness: Duration, now_ms: u64) -> bool {
        self.freshness_relative_to(now_ms) <= max_freshness
    }

    /// Blueprint §8's `required_evidence` check, made real: `provenance`,
    /// `max_freshness_seconds`, and `execution_id` must ALL match, or
    /// this evidence does not satisfy the requirement. Each dimension
    /// fails independently — see
    /// `tests::wrong_provenance_alone_fails_the_match`,
    /// `tests::staleness_alone_fails_the_match`,
    /// `tests::wrong_execution_id_alone_fails_the_match` — not just a
    /// single combined pass/fail case.
    pub fn matches(
        &self,
        required_provenance: EvidenceProvenance,
        max_freshness: Duration,
        required_execution_id: &str,
        now_ms: u64,
    ) -> bool {
        self.provenance == required_provenance
            && self.is_fresh(max_freshness, now_ms)
            && self.execution_id == required_execution_id
            && self.result
    }

    /// Constructs real Evidence from a real P4-W01 pipeline stage
    /// result — connecting the pipeline's actual stage outcomes to real
    /// evidence records, not a separate, disconnected concept.
    /// `result` is set from the stage's actual `Passed`/`Failed` status
    /// (a `SkippedDueToEarlierFailure` stage is evidence of nothing
    /// having run, so it also maps to `result: false` — it cannot
    /// satisfy a requirement any more than an outright failure can).
    pub fn from_stage_outcome(
        outcome: &StageOutcome,
        provenance: EvidenceProvenance,
        produced_at_ms: u64,
        execution_id: impl Into<String>,
        source: impl Into<String>,
        reproducibility: Reproducibility,
        integrity: IntegrityLevel,
    ) -> Evidence {
        let observation_type = observation_type_for_stage(outcome.stage);
        let result = matches!(outcome.status, StageStatus::Passed);
        Evidence {
            provenance,
            observation_type,
            produced_at_ms,
            execution_id: execution_id.into(),
            source: source.into(),
            reproducibility,
            integrity,
            result,
        }
    }
}

fn observation_type_for_stage(stage: VerificationStage) -> ObservationType {
    match stage {
        VerificationStage::FilesystemEvidence => ObservationType::FilesystemCheck,
        VerificationStage::DiffVerification => ObservationType::DiffCheck,
        VerificationStage::Build => ObservationType::BuildExecution,
        VerificationStage::Tests => ObservationType::TestExecution,
        VerificationStage::LintTypecheck => ObservationType::LintExecution,
        VerificationStage::RequirementVerification
        | VerificationStage::MissionAcceptanceContractCheck
        | VerificationStage::CheckpointAccepted => ObservationType::ProcessObservation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(produced_at_ms: u64, execution_id: &str, provenance: EvidenceProvenance, result: bool) -> Evidence {
        Evidence {
            provenance,
            observation_type: ObservationType::TestExecution,
            produced_at_ms,
            execution_id: execution_id.to_string(),
            source: "npm test -- auth.test.ts".to_string(),
            reproducibility: Reproducibility::Deterministic,
            integrity: IntegrityLevel::IndependentlyVerified,
            result,
        }
    }

    #[test]
    fn freshness_is_computed_exactly() {
        let e = evidence(1_000, "attempt-1", EvidenceProvenance::LocalTestRunner, true);
        assert_eq!(e.freshness_relative_to(1_000), Duration::ZERO);
        assert_eq!(e.freshness_relative_to(6_000), Duration::from_millis(5_000));
    }

    #[test]
    fn freshness_saturates_rather_than_panicking_if_now_precedes_produced_at() {
        let e = evidence(10_000, "attempt-1", EvidenceProvenance::LocalTestRunner, true);
        assert_eq!(e.freshness_relative_to(1_000), Duration::ZERO);
    }

    #[test]
    fn is_fresh_distinguishes_fresh_from_stale_at_the_exact_boundary() {
        let e = evidence(0, "attempt-1", EvidenceProvenance::LocalTestRunner, true);
        let max = Duration::from_secs(300);
        assert!(e.is_fresh(max, 300_000), "exactly at the boundary must count as fresh");
        assert!(!e.is_fresh(max, 300_001), "one millisecond past the boundary must count as stale");
    }

    #[test]
    fn matches_succeeds_when_every_dimension_is_correct() {
        let e = evidence(0, "attempt-1", EvidenceProvenance::LocalTestRunner, true);
        assert!(e.matches(EvidenceProvenance::LocalTestRunner, Duration::from_secs(300), "attempt-1", 60_000));
    }

    #[test]
    fn wrong_provenance_alone_fails_the_match() {
        let e = evidence(0, "attempt-1", EvidenceProvenance::CiSystem, true);
        assert!(!e.matches(EvidenceProvenance::LocalTestRunner, Duration::from_secs(300), "attempt-1", 60_000));
    }

    #[test]
    fn staleness_alone_fails_the_match() {
        let e = evidence(0, "attempt-1", EvidenceProvenance::LocalTestRunner, true);
        let ten_minutes_later = 600_000;
        assert!(!e.matches(EvidenceProvenance::LocalTestRunner, Duration::from_secs(300), "attempt-1", ten_minutes_later));
    }

    #[test]
    fn wrong_execution_id_alone_fails_the_match() {
        let e = evidence(0, "attempt-1", EvidenceProvenance::LocalTestRunner, true);
        assert!(!e.matches(EvidenceProvenance::LocalTestRunner, Duration::from_secs(300), "attempt-STALE-PRIOR-RUN", 60_000));
    }

    #[test]
    fn a_failed_underlying_result_fails_the_match_even_with_correct_provenance_and_freshness() {
        let e = evidence(0, "attempt-1", EvidenceProvenance::LocalTestRunner, false);
        assert!(
            !e.matches(EvidenceProvenance::LocalTestRunner, Duration::from_secs(300), "attempt-1", 60_000),
            "correct provenance/freshness/execution_id must not paper over a failed result"
        );
    }

    #[test]
    fn from_stage_outcome_reflects_a_passed_stage_as_true_result() {
        let outcome = StageOutcome {
            stage: VerificationStage::Tests,
            status: StageStatus::Passed,
            detail: "passed".to_string(),
        };
        let e = Evidence::from_stage_outcome(
            &outcome,
            EvidenceProvenance::LocalTestRunner,
            1_000,
            "attempt-1",
            "cargo test",
            Reproducibility::Deterministic,
            IntegrityLevel::IndependentlyVerified,
        );
        assert!(e.result);
        assert_eq!(e.observation_type, ObservationType::TestExecution);
    }

    #[test]
    fn from_stage_outcome_reflects_a_failed_stage_as_false_result() {
        let outcome = StageOutcome {
            stage: VerificationStage::Build,
            status: StageStatus::Failed { reason: "compile error".to_string() },
            detail: "compile error".to_string(),
        };
        let e = Evidence::from_stage_outcome(
            &outcome,
            EvidenceProvenance::LocalTestRunner,
            1_000,
            "attempt-1",
            "cargo build",
            Reproducibility::Deterministic,
            IntegrityLevel::IndependentlyVerified,
        );
        assert!(!e.result);
    }

    #[test]
    fn from_stage_outcome_reflects_a_skipped_stage_as_false_result() {
        // A stage skipped because an earlier one failed is evidence of
        // nothing having run -- it must not be treated as if it passed.
        let outcome = StageOutcome {
            stage: VerificationStage::LintTypecheck,
            status: StageStatus::SkippedDueToEarlierFailure,
            detail: "skipped: an earlier stage failed".to_string(),
        };
        let e = Evidence::from_stage_outcome(
            &outcome,
            EvidenceProvenance::LocalTestRunner,
            1_000,
            "attempt-1",
            "cargo clippy",
            Reproducibility::Deterministic,
            IntegrityLevel::IndependentlyVerified,
        );
        assert!(!e.result, "a skipped stage must not be treated as a passing result");
    }
}
