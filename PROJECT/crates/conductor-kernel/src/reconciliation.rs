//! The Reconciliation Engine (P2-W03 / Blueprint §4.3, §6).
//!
//! ```text
//! Mission state (what we think happened)
//!         +
//! Baseline diff-state (what existed before the attempt)   <- P2-W02
//!         +
//! Actual diff (what genuinely changed)
//!         ↓
//!   RECONCILIATION
//!         ↓
//! ReconciliationOutcome (Blueprint §4.3)
//! ```
//!
//! **Invariant 14, restated for this module specifically:** reconciliation
//! never sets `AttemptStatus::Succeeded`. It doesn't reference any
//! `AttemptStatus` at all -- it returns a `ReconciliationOutcome`, and
//! Blueprint §4.3 is explicit that even `ExpectedChange` is only
//! "advisory-grade evidence" until the (not-yet-built) Verification
//! Engine's full pipeline passes. Reconciliation observes; the
//! Verification Engine decides.
//!
//! Scope, stated explicitly (this is P2-W03 only): `ReconciliationOutcome`
//! is reproduced as the full six-variant enum from Blueprint §4.3 for
//! fidelity to the spec, but [`reconcile`] only ever produces four of
//! those six -- `NoChange`, `ExpectedChange`, `UnexpectedChange`,
//! `PartialChange`. The other two, `UserChangeDetected` and `Conflict`,
//! require data this crate does not have yet: attributing a changed
//! region to the user vs. the AI (Blueprint §4.6, P2-W05) and detecting
//! whether two changed regions overlap (P2-W06). Approximating either
//! without that data -- e.g. guessing "this looks like a user edit" from
//! file-level heuristics alone -- would be exactly the false-confidence
//! failure mode Blueprint §7's `RequirementVerificationClass` distinction
//! exists to prevent. Those branches are left for P2-W05/P2-W06 to wire
//! in once the data they need actually exists.
//!
//! **Restart-safety** (Blueprint §6's algorithm notes reconciliation
//! "must be safe to interrupt"): [`reconcile`] is a pure, deterministic
//! function of its three inputs, with no internal state and no I/O of its
//! own. That is the mechanism by which this module satisfies
//! restart-safety -- not by checkpointing intermediate steps of a
//! stateful process, but by having no stateful process to interrupt in
//! the first place. A crash mid-reconciliation means: nothing was written
//! yet, so a restart simply calls `reconcile` again against the same
//! durable inputs (a previously captured `Baseline` and a fresh
//! re-capture) and gets the identical answer. See
//! `tests::identical_inputs_after_a_simulated_restart_produce_the_identical_outcome`.

use crate::baseline::{Baseline, PathChange, RelativePath};
use crate::state::AttemptStatus;
use std::collections::BTreeSet;

/// Blueprint §4.3's ReconciliationOutcome, reproduced verbatim (six
/// variants) even though [`reconcile`] only ever produces four of them --
/// see the module-level scope note for why `UserChangeDetected` and
/// `Conflict` are not reachable from this function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconciliationOutcome {
    /// The filesystem matches the pre-attempt baseline exactly.
    NoChange,
    /// The actual diff matches the intended operation exactly: every
    /// expected file changed, and nothing else did.
    ExpectedChange,
    /// Something changed, but not what was requested -- a file outside
    /// the expected set changed (whether or not the expected files also
    /// changed).
    UnexpectedChange,
    /// Some, but not all, of the intended change landed -- a non-empty
    /// proper subset of the expected files changed, and nothing outside
    /// that set did.
    PartialChange,
    /// A change is attributable to the user, not the attempt.
    /// NOT PRODUCED BY [`reconcile`] IN THIS TASK -- requires mutation
    /// attribution (Blueprint §4.6, P2-W05). Included here only for
    /// fidelity to Blueprint §4.3's enum shape.
    UserChangeDetected,
    /// The attempt's change and a user change overlap on the same
    /// region. NOT PRODUCED BY [`reconcile`] IN THIS TASK -- requires
    /// region-overlap conflict detection (P2-W06). Included here only for
    /// fidelity to Blueprint §4.3's enum shape.
    Conflict,
}

/// Blueprint §4.4's `expected_operation` shape: which files an attempt
/// was expected to touch, and why.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpectedOperation {
    pub files: BTreeSet<RelativePath>,
    pub intent: String,
}

impl ExpectedOperation {
    pub fn new(files: impl IntoIterator<Item = RelativePath>, intent: impl Into<String>) -> Self {
        ExpectedOperation {
            files: files.into_iter().collect(),
            intent: intent.into(),
        }
    }
}

/// Blueprint §6's reconciliation algorithm (steps 1-5; steps 6-7 are out
/// of scope -- see module docs). Pure and deterministic: the same three
/// inputs always produce the same outcome, which is this module's whole
/// answer to "reconciliation must be safe to interrupt."
///
/// `before` is the Baseline captured at attempt start (P2-W02). `after`
/// is a fresh Baseline re-capture of the same workspace's current state.
/// `expected` is what the attempt was supposed to change.
pub fn reconcile(before: &Baseline, after: &Baseline, expected: &ExpectedOperation) -> ReconciliationOutcome {
    let diff = before.diff(after);

    if diff.is_empty() {
        return ReconciliationOutcome::NoChange;
    }

    let changed: BTreeSet<RelativePath> = diff.keys().cloned().collect();
    let changed_outside_expected: BTreeSet<&RelativePath> =
        changed.iter().filter(|p| !expected.files.contains(*p)).collect();

    if !changed_outside_expected.is_empty() {
        // Something changed that wasn't requested at all -- conservative
        // classification regardless of whether the expected files also
        // changed (Blueprint §4.3: "something changed, but not what was
        // requested").
        return ReconciliationOutcome::UnexpectedChange;
    }

    let changed_expected: BTreeSet<&RelativePath> = changed.iter().filter(|p| expected.files.contains(*p)).collect();

    if !expected.files.is_empty() && changed_expected.len() == expected.files.len() {
        ReconciliationOutcome::ExpectedChange
    } else {
        // changed_outside_expected is empty here, so every changed path
        // is in expected.files, and changed_expected is a non-empty
        // proper subset of it (diff was non-empty, so changed_expected
        // is non-empty too since changed_outside_expected is empty).
        ReconciliationOutcome::PartialChange
    }
}

/// The `PathChange` classification `diff()` would report for a given
/// path, exposed for callers that want the region-level detail behind an
/// outcome without recomputing the diff themselves.
pub fn change_for_path(before: &Baseline, after: &Baseline, path: &RelativePath) -> Option<PathChange> {
    before.diff_full(after).get(path).cloned()
}

/// Blueprint §4.3's outcome → next-action mapping (P2-W04).
///
/// Returns `None` for `ExpectedChange` specifically -- not
/// `Some(AttemptStatus::Succeeded)`, and not `Some(anything)`. Blueprint
/// §4.3 is explicit that `ExpectedChange` means only *"the diff we
/// intended appears to be present"* -- still advisory-grade evidence --
/// and that reconciliation's job "ends at producing the outcome;
/// acceptance is a separate decision made against real evidence" by the
/// (not-yet-built) Verification Engine. There is no `AttemptStatus` value
/// anywhere in this crate that means "reconciliation thinks this
/// succeeded"; modeling the deferral as anything other than `None` would
/// reintroduce the exact conflation Blueprint §4.3's own correction fixed
/// (Invariant 14).
///
/// Every other outcome maps to `Some(AttemptStatus::Failed)`, per
/// Blueprint §4.3's table. Each has different downstream handling --
/// targeted repair for `UnexpectedChange`/`PartialChange`, leaving the
/// user's change alone for `UserChangeDetected`, pausing the mission and
/// presenting the four-way choice for `Conflict` -- but routing to that
/// downstream handling is P2-W05/P2-W06/P2-W07's job; this function
/// returns only the `AttemptStatus`, deliberately not any mission-level
/// side effect.
pub fn recommended_attempt_status(outcome: &ReconciliationOutcome) -> Option<AttemptStatus> {
    match outcome {
        ReconciliationOutcome::ExpectedChange => None,
        ReconciliationOutcome::NoChange
        | ReconciliationOutcome::UnexpectedChange
        | ReconciliationOutcome::PartialChange
        | ReconciliationOutcome::UserChangeDetected
        | ReconciliationOutcome::Conflict => Some(AttemptStatus::Failed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn baseline(entries: &[(&str, &str)]) -> Baseline {
        Baseline {
            entries: entries
                .iter()
                .map(|(path, hash)| (PathBuf::from(path), hash.to_string()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn identical_baselines_produce_no_change() {
        let before = baseline(&[("a.txt", "hash-a"), ("b.txt", "hash-b")]);
        let after = baseline(&[("a.txt", "hash-a"), ("b.txt", "hash-b")]);
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "wire up a.txt");
        assert_eq!(reconcile(&before, &after, &expected), ReconciliationOutcome::NoChange);
    }

    #[test]
    fn exactly_the_expected_file_changing_is_expected_change() {
        let before = baseline(&[("a.txt", "hash-a1"), ("b.txt", "hash-b")]);
        let after = baseline(&[("a.txt", "hash-a2"), ("b.txt", "hash-b")]);
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "wire up a.txt");
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::ExpectedChange
        );
    }

    #[test]
    fn all_expected_files_changing_together_is_expected_change() {
        let before = baseline(&[("a.txt", "a1"), ("b.txt", "b1")]);
        let after = baseline(&[("a.txt", "a2"), ("b.txt", "b2")]);
        let expected = ExpectedOperation::new([PathBuf::from("a.txt"), PathBuf::from("b.txt")], "wire up both");
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::ExpectedChange
        );
    }

    #[test]
    fn one_of_two_expected_files_changing_is_partial_change() {
        let before = baseline(&[("a.txt", "a1"), ("b.txt", "b1")]);
        let after = baseline(&[("a.txt", "a2"), ("b.txt", "b1")]); // b.txt unchanged
        let expected = ExpectedOperation::new([PathBuf::from("a.txt"), PathBuf::from("b.txt")], "wire up both");
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::PartialChange
        );
    }

    #[test]
    fn an_unrequested_file_changing_is_unexpected_change() {
        let before = baseline(&[("a.txt", "a1"), ("other.txt", "o1")]);
        let after = baseline(&[("a.txt", "a1"), ("other.txt", "o2")]); // only other.txt changed
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "wire up a.txt");
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::UnexpectedChange
        );
    }

    #[test]
    fn expected_file_changing_alongside_an_unexpected_one_is_still_unexpected_change() {
        let before = baseline(&[("a.txt", "a1"), ("other.txt", "o1")]);
        let after = baseline(&[("a.txt", "a2"), ("other.txt", "o2")]); // both changed
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "wire up a.txt only");
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::UnexpectedChange,
            "an extra unrequested change must not be masked by the expected change also landing"
        );
    }

    #[test]
    fn a_change_with_an_empty_expected_operation_is_unexpected_change() {
        let before = baseline(&[("a.txt", "a1")]);
        let after = baseline(&[("a.txt", "a2")]);
        let expected = ExpectedOperation::default(); // nothing declared expected
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::UnexpectedChange,
            "nothing was declared expected, so any change is not what was requested"
        );
    }

    #[test]
    fn added_and_removed_files_are_classified_the_same_as_modified_files() {
        // Added
        let before = baseline(&[("a.txt", "a1")]);
        let after = baseline(&[("a.txt", "a1"), ("new.txt", "n1")]);
        let expected = ExpectedOperation::new([PathBuf::from("new.txt")], "add new.txt");
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::ExpectedChange
        );

        // Removed
        let before = baseline(&[("a.txt", "a1"), ("gone.txt", "g1")]);
        let after = baseline(&[("a.txt", "a1")]);
        let expected = ExpectedOperation::new([PathBuf::from("gone.txt")], "remove gone.txt");
        assert_eq!(
            reconcile(&before, &after, &expected),
            ReconciliationOutcome::ExpectedChange
        );
    }

    #[test]
    fn empty_baselines_with_nothing_expected_is_no_change() {
        let before = Baseline::default();
        let after = Baseline::default();
        let expected = ExpectedOperation::default();
        assert_eq!(reconcile(&before, &after, &expected), ReconciliationOutcome::NoChange);
    }

    /// Phase Manifest §6.4: "restart during reconciliation." Simulates a
    /// crash immediately after computing an outcome and a fresh restart
    /// re-deriving it from the same durable inputs -- since reconcile()
    /// is pure, this is simply calling it again.
    #[test]
    fn identical_inputs_after_a_simulated_restart_produce_the_identical_outcome() {
        let before = baseline(&[("a.txt", "a1"), ("b.txt", "b1")]);
        let after = baseline(&[("a.txt", "a2"), ("b.txt", "b1")]);
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "wire up a.txt");

        let outcome_before_crash = reconcile(&before, &after, &expected);
        // "Restart": no state carried over except the same durable inputs.
        let outcome_after_restart = reconcile(&before, &after, &expected);

        assert_eq!(outcome_before_crash, outcome_after_restart);
        assert_eq!(outcome_after_restart, ReconciliationOutcome::ExpectedChange);
    }

    #[test]
    fn change_for_path_exposes_the_underlying_classification() {
        let before = baseline(&[("a.txt", "a1"), ("b.txt", "b1")]);
        let after = baseline(&[("a.txt", "a2"), ("b.txt", "b1")]);

        assert!(matches!(
            change_for_path(&before, &after, &PathBuf::from("a.txt")),
            Some(PathChange::Modified { .. })
        ));
        assert_eq!(
            change_for_path(&before, &after, &PathBuf::from("b.txt")),
            Some(PathChange::Unchanged)
        );
        assert_eq!(change_for_path(&before, &after, &PathBuf::from("nonexistent.txt")), None);
    }

    #[test]
    fn reconcile_never_touches_any_attempt_status_type() {
        // Structural witness, not a runtime assertion: reconcile()'s
        // return type is ReconciliationOutcome, which has no variant
        // named Succeeded and no conversion to/from AttemptStatus exists
        // anywhere in this module. Invariant 14 is upheld by this
        // function simply having no capability to set that field, not by
        // a runtime check.
        let before = Baseline::default();
        let after = Baseline::default();
        let expected = ExpectedOperation::default();
        let outcome = reconcile(&before, &after, &expected);
        match outcome {
            ReconciliationOutcome::NoChange
            | ReconciliationOutcome::ExpectedChange
            | ReconciliationOutcome::UnexpectedChange
            | ReconciliationOutcome::PartialChange
            | ReconciliationOutcome::UserChangeDetected
            | ReconciliationOutcome::Conflict => {}
        }
    }

    // -- P2-W04: protected Succeeded transition --

    const ALL_OUTCOMES: [ReconciliationOutcome; 6] = [
        ReconciliationOutcome::NoChange,
        ReconciliationOutcome::ExpectedChange,
        ReconciliationOutcome::UnexpectedChange,
        ReconciliationOutcome::PartialChange,
        ReconciliationOutcome::UserChangeDetected,
        ReconciliationOutcome::Conflict,
    ];

    /// Phase Manifest §6.3 P2-W04's literal required direct test:
    /// "reconciliation result ≠ Succeeded transition." Exhaustive over
    /// every variant, not a spot-check.
    #[test]
    fn recommended_attempt_status_never_yields_succeeded_for_any_outcome() {
        for outcome in &ALL_OUTCOMES {
            assert_ne!(
                recommended_attempt_status(outcome),
                Some(AttemptStatus::Succeeded),
                "outcome {outcome:?} must never map to Succeeded"
            );
        }
    }

    #[test]
    fn expected_change_defers_rather_than_recommending_any_status() {
        assert_eq!(
            recommended_attempt_status(&ReconciliationOutcome::ExpectedChange),
            None,
            "ExpectedChange must defer to the Verification Engine, not propose a status itself"
        );
    }

    #[test]
    fn every_non_expected_change_outcome_recommends_failed() {
        for outcome in &ALL_OUTCOMES {
            if *outcome == ReconciliationOutcome::ExpectedChange {
                continue;
            }
            assert_eq!(
                recommended_attempt_status(outcome),
                Some(AttemptStatus::Failed),
                "outcome {outcome:?} must recommend Failed"
            );
        }
    }

    /// Ties reconciliation directly to the state engine's authorization
    /// gate (P1 state.rs) at the actual boundary where they meet: takes a
    /// real reconcile() ExpectedChange result -- the case closest to
    /// success -- and confirms only TransitionActor::VerificationEngine
    /// may turn it into a Succeeded transition; every other actor is
    /// structurally rejected.
    #[test]
    fn expected_change_can_only_become_succeeded_through_the_verification_engine_actor() {
        use crate::state::{try_attempt_transition, TransitionActor, TransitionError};

        let before = baseline(&[("a.txt", "a1")]);
        let after = baseline(&[("a.txt", "a2")]);
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "wire up a.txt");
        let outcome = reconcile(&before, &after, &expected);
        assert_eq!(outcome, ReconciliationOutcome::ExpectedChange);

        // Reconciliation itself proposes nothing.
        assert_eq!(recommended_attempt_status(&outcome), None);

        let all_actors = [
            TransitionActor::AuthoritativeExecutor,
            TransitionActor::CancellationPath,
            TransitionActor::VerificationEngine,
        ];
        for actor in all_actors {
            let result = try_attempt_transition(actor, AttemptStatus::Running, AttemptStatus::Succeeded);
            if actor == TransitionActor::VerificationEngine {
                assert!(result.is_ok(), "VerificationEngine must be authorized to produce Succeeded");
            } else {
                assert!(
                    matches!(result, Err(TransitionError::UnauthorizedActor { .. })),
                    "actor {actor:?} must be rejected for the Succeeded edge"
                );
            }
        }
    }
}
