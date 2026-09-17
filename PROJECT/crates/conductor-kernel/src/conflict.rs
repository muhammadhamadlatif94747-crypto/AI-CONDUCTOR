//! Conflict detection (P2-W06 / Blueprint §5, §26.2, §5.1; Invariants 11
//! and 13).
//!
//! Two independent signals feed one decision:
//!
//! ```text
//! Invariant 13 (merge-time re-check)      P2-W05 (region attribution)
//!   original baseline                       AI diff vs user diff
//!         vs                                       vs
//!   live workspace, right                   overlapping baseline
//!   before merging                          line ranges
//!         │                                        │
//!         └──────────────┬─────────────────────────┘
//!                         ▼
//!                   decide_merge()
//!                         ▼
//!              Proceed  |  Halt(reason)
//! ```
//!
//! Phase Manifest §6.3 P2-W06: *"Conflict must halt rather than silently
//! resolve."* This module is what makes that a provable property of a
//! real function rather than a described intent: [`decide_merge`] is
//! structurally unable to return [`MergeDecision::Proceed`] when either
//! signal fires — there is no code path that ignores one signal because
//! the other looked clean.
//!
//! Scope, stated explicitly (this is P2-W06 only): this module decides
//! Proceed vs Halt. It does not perform the merge write itself, does not
//! drive any `MissionStatus` transition (state.rs currently has no
//! actor-authorization wrapper for mission-level transitions at all —
//! confirmed before writing this module), and does not implement the
//! four-way person decision UI that Blueprint pairs with "mission
//! PAUSED" — that's P2-W07's job, once there's a UI actually able to act
//! on a `Halt`.

use crate::baseline::{Baseline, PathChange, RelativePath};
use crate::mutation_attribution::{MutationAttribution, RegionClassification};

/// The result of Invariant 13's merge-time re-check: comparing the
/// originally-captured baseline against the live workspace's state right
/// before a merge would write to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeCheckResult {
    /// No path that existed in the original baseline was modified or
    /// removed since. Safe to proceed with the merge on the standard
    /// two-phase-locked path (Blueprint §26.2) — this function does not
    /// itself perform the merge; it only certifies this precondition.
    Safe,
    /// At least one baseline path was modified or removed in the live
    /// workspace since the baseline was captured. Per Blueprint §5.1's
    /// edge-case table, this is Conflict regardless of what the change
    /// was — "never merged blind."
    Conflict { changed_paths: Vec<RelativePath> },
}

/// Why [`decide_merge`] returned [`MergeDecision::Halt`] — carries the
/// actual evidence (not just a bare flag) so a future caller (P2-W07's
/// four-way UI) has something real to show the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictReason {
    /// Invariant 13's merge-time re-check found drift.
    MergeTimeDrift { changed_paths: Vec<RelativePath> },
    /// P2-W05's region attribution found the AI's change and a user
    /// change overlapping.
    RegionOverlap { regions: Vec<MutationAttribution> },
    /// Both signals fired independently.
    Both {
        changed_paths: Vec<RelativePath>,
        regions: Vec<MutationAttribution>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeDecision {
    Proceed,
    Halt(ConflictReason),
}

/// Invariant 13's re-check: compare `original_baseline` (captured at
/// attempt start) against `live_at_merge_time` (a fresh `Baseline::capture`
/// of the live workspace, taken immediately before merging).
///
/// Only `Modified` and `Removed` paths count as drift. `Added` paths —
/// files that exist in `live_at_merge_time` but not in
/// `original_baseline` — are deliberately excluded, per Blueprint §5.1's
/// table: *"File untracked, newly created since baseline | Not part of
/// the baseline comparison; left alone; merge proceeds on the files it
/// actually owns."* A file the conductor never knew about at baseline
/// time is not something its merge is entitled to have an opinion about.
pub fn check_merge_safety(original_baseline: &Baseline, live_at_merge_time: &Baseline) -> MergeCheckResult {
    let diff = original_baseline.diff(live_at_merge_time);
    let changed_paths: Vec<RelativePath> = diff
        .into_iter()
        .filter_map(|(path, change)| match change {
            PathChange::Modified { .. } | PathChange::Removed => Some(path),
            PathChange::Added | PathChange::Unchanged => None,
        })
        .collect();

    if changed_paths.is_empty() {
        MergeCheckResult::Safe
    } else {
        MergeCheckResult::Conflict { changed_paths }
    }
}

/// Combine Invariant 13's merge-time check with P2-W05's region
/// attribution into a single decision. Returns `Proceed` in exactly one
/// case: the merge-time check is `Safe` AND no attribution entry is
/// `ConflictRegion`. Every other combination returns `Halt`, carrying
/// whichever evidence actually fired (see
/// `tests::exhaustive_combination_coverage` for the direct proof of this
/// claim across all four combinations).
pub fn decide_merge(merge_check: &MergeCheckResult, attributions: &[MutationAttribution]) -> MergeDecision {
    let region_conflicts: Vec<MutationAttribution> = attributions
        .iter()
        .filter(|a| a.classification == RegionClassification::ConflictRegion)
        .cloned()
        .collect();

    match (merge_check, region_conflicts.is_empty()) {
        (MergeCheckResult::Safe, true) => MergeDecision::Proceed,
        (MergeCheckResult::Safe, false) => MergeDecision::Halt(ConflictReason::RegionOverlap {
            regions: region_conflicts,
        }),
        (MergeCheckResult::Conflict { changed_paths }, true) => {
            MergeDecision::Halt(ConflictReason::MergeTimeDrift {
                changed_paths: changed_paths.clone(),
            })
        }
        (MergeCheckResult::Conflict { changed_paths }, false) => MergeDecision::Halt(ConflictReason::Both {
            changed_paths: changed_paths.clone(),
            regions: region_conflicts,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutation_attribution::RegionClassification;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn baseline(entries: &[(&str, &str)]) -> Baseline {
        Baseline {
            entries: entries
                .iter()
                .map(|(p, h)| (PathBuf::from(p), h.to_string()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn region(classification: RegionClassification) -> MutationAttribution {
        MutationAttribution {
            path: PathBuf::from("a.txt"),
            baseline_range: (1, 2),
            classification,
        }
    }

    // -- check_merge_safety --

    #[test]
    fn identical_baselines_are_safe() {
        let a = baseline(&[("a.txt", "h1")]);
        let b = baseline(&[("a.txt", "h1")]);
        assert_eq!(check_merge_safety(&a, &b), MergeCheckResult::Safe);
    }

    #[test]
    fn a_modified_baseline_path_is_conflict() {
        let original = baseline(&[("a.txt", "h1")]);
        let live = baseline(&[("a.txt", "h2")]); // person edited it
        match check_merge_safety(&original, &live) {
            MergeCheckResult::Conflict { changed_paths } => {
                assert_eq!(changed_paths, vec![PathBuf::from("a.txt")]);
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    #[test]
    fn a_removed_baseline_path_is_conflict() {
        let original = baseline(&[("a.txt", "h1"), ("b.txt", "h2")]);
        let live = baseline(&[("a.txt", "h1")]); // b.txt deleted since baseline
        match check_merge_safety(&original, &live) {
            MergeCheckResult::Conflict { changed_paths } => {
                assert_eq!(changed_paths, vec![PathBuf::from("b.txt")]);
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    /// Blueprint §5.1's table: a newly-created untracked file is "not
    /// part of the baseline comparison; left alone." Proven directly,
    /// not just implemented.
    #[test]
    fn a_newly_added_path_alone_is_safe_not_conflict() {
        let original = baseline(&[("a.txt", "h1")]);
        let live = baseline(&[("a.txt", "h1"), ("new_untracked.txt", "h3")]);
        assert_eq!(
            check_merge_safety(&original, &live),
            MergeCheckResult::Safe,
            "a file the baseline never knew about must not trigger Conflict"
        );
    }

    #[test]
    fn an_added_path_alongside_a_real_modification_still_reports_only_the_modification() {
        let original = baseline(&[("a.txt", "h1")]);
        let live = baseline(&[("a.txt", "h2"), ("new_untracked.txt", "h3")]);
        match check_merge_safety(&original, &live) {
            MergeCheckResult::Conflict { changed_paths } => {
                assert_eq!(changed_paths, vec![PathBuf::from("a.txt")]);
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    // -- decide_merge: exhaustive combination coverage --

    #[test]
    fn exhaustive_combination_coverage() {
        let safe = MergeCheckResult::Safe;
        let conflict = MergeCheckResult::Conflict {
            changed_paths: vec![PathBuf::from("a.txt")],
        };
        let no_regions: Vec<MutationAttribution> = vec![];
        let with_region_conflict = vec![region(RegionClassification::ConflictRegion)];
        // A non-conflict attribution entry must not itself trigger a halt.
        let with_non_conflict_region = vec![region(RegionClassification::UserChange)];

        assert_eq!(decide_merge(&safe, &no_regions), MergeDecision::Proceed);
        assert_eq!(decide_merge(&safe, &with_non_conflict_region), MergeDecision::Proceed);

        assert!(matches!(
            decide_merge(&safe, &with_region_conflict),
            MergeDecision::Halt(ConflictReason::RegionOverlap { .. })
        ));
        assert!(matches!(
            decide_merge(&conflict, &no_regions),
            MergeDecision::Halt(ConflictReason::MergeTimeDrift { .. })
        ));
        assert!(matches!(
            decide_merge(&conflict, &with_region_conflict),
            MergeDecision::Halt(ConflictReason::Both { .. })
        ));
    }

    #[test]
    fn proceed_is_reachable_only_in_the_safe_and_no_conflict_cell() {
        // Direct restatement of the claim in decide_merge's doc comment,
        // as an assertion rather than prose: sweep every combination and
        // confirm Proceed shows up exactly once.
        let checks = [
            MergeCheckResult::Safe,
            MergeCheckResult::Conflict {
                changed_paths: vec![PathBuf::from("a.txt")],
            },
        ];
        let region_sets: [Vec<MutationAttribution>; 2] =
            [vec![], vec![region(RegionClassification::ConflictRegion)]];

        let mut proceed_count = 0;
        for check in &checks {
            for regions in &region_sets {
                if decide_merge(check, regions) == MergeDecision::Proceed {
                    proceed_count += 1;
                }
            }
        }
        assert_eq!(proceed_count, 1, "Proceed must be reachable in exactly one of the four combinations");
    }

    #[test]
    fn halt_reason_carries_the_actual_changed_paths_not_just_a_tag() {
        let conflict = MergeCheckResult::Conflict {
            changed_paths: vec![PathBuf::from("auth.ts"), PathBuf::from("config.rs")],
        };
        match decide_merge(&conflict, &[]) {
            MergeDecision::Halt(ConflictReason::MergeTimeDrift { changed_paths }) => {
                assert_eq!(changed_paths.len(), 2);
                assert!(changed_paths.contains(&PathBuf::from("auth.ts")));
                assert!(changed_paths.contains(&PathBuf::from("config.rs")));
            }
            other => panic!("expected MergeTimeDrift with real paths, got {other:?}"),
        }
    }

    #[test]
    fn halt_reason_carries_the_actual_conflicting_regions_not_just_a_tag() {
        let conflicting = region(RegionClassification::ConflictRegion);
        match decide_merge(&MergeCheckResult::Safe, &[conflicting.clone()]) {
            MergeDecision::Halt(ConflictReason::RegionOverlap { regions }) => {
                assert_eq!(regions, vec![conflicting]);
            }
            other => panic!("expected RegionOverlap with the real region, got {other:?}"),
        }
    }
}
