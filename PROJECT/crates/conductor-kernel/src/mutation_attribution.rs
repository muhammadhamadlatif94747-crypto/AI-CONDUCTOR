//! Mutation attribution (P2-W05 / Blueprint §4.6).
//!
//! ```text
//! Attempt begins → capture baseline diff-state
//! Execution Harness operates → capture resulting diff
//! Compare baseline → result, line-region by line-region
//! Classify each changed region as:
//!    expected_conductor_change | unexpected_ai_change | user_change | conflict_region
//! ```
//!
//! Whole-file classification (P2-W03's `reconcile`) cannot answer *"did
//! the AI change line 40, or did the user"* when both changed the same
//! file -- it can only say the file changed. This module answers that
//! question at line-region granularity, which is what Blueprint §4.6 says
//! Invariant 6 ("user changes are protected") actually requires:
//! protection needs *attribution*, not just detection that a file
//! changed.
//!
//! Scope, stated explicitly (this is P2-W05 only): this module classifies
//! regions from three already-available content snapshots (baseline, AI
//! result, live-current). It does not decide what to *do* about a
//! `ConflictRegion` -- that policy (halt, don't auto-resolve) is P2-W06's
//! job. It also does not implement *how* a caller obtains three physical
//! snapshots from a real running attempt's lifecycle -- that's
//! orchestration wiring for whichever task drives the real lifecycle
//! (P2-W07 / Phase 5).
//!
//! Diffing is done with real `git diff --no-index`, not a hand-rolled
//! algorithm -- consistent with this crate's established discipline
//! (P2-W01's real `git worktree`) of using real Git behavior rather than
//! a reimplementation that could silently diverge from it.

use crate::reconciliation::ExpectedOperation;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum AttributionError {
    #[error("failed to write temp file for diffing: {0}")]
    TempFileWrite(#[source] std::io::Error),
    #[error("failed to invoke git diff --no-index: {0}")]
    GitInvocation(#[source] std::io::Error),
    #[error("git diff --no-index exited with unexpected status {0}: {1}")]
    UnexpectedGitExit(i32, String),
    #[error("git diff --no-index output was not valid UTF-8")]
    NonUtf8Output,
    #[error("could not parse a hunk header: {0:?}")]
    UnparseableHunkHeader(String),
}

/// Blueprint §4.6's four region classifications, reproduced verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionClassification {
    ExpectedConductorChange,
    UnexpectedAiChange,
    UserChange,
    ConflictRegion,
}

/// A classified change, anchored to baseline line numbers (1-indexed,
/// inclusive start, exclusive end -- `[start, end)`). `end == start`
/// represents a pure insertion at that baseline position (no baseline
/// lines were themselves changed; content was added there).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationAttribution {
    pub path: PathBuf,
    pub baseline_range: (usize, usize),
    pub classification: RegionClassification,
}

/// A single hunk's baseline-line span, as reported by `git diff --no-index
/// --unified=0`'s `@@ -start,count +... @@` header. `count == 0` means a
/// pure insertion (nothing removed from the baseline at this point).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HunkRange {
    start: usize,
    count: usize,
}

impl HunkRange {
    /// `[start, end)` for overlap purposes. A pure insertion (`count ==
    /// 0`) is widened to a single-line window at `start` rather than an
    /// empty window -- the conservative direction: this can only cause an
    /// extra region to be flagged as overlapping (and therefore, in this
    /// module's caller, ConflictRegion) that a stricter reading might
    /// not, never the reverse. Under-flagging a real conflict is exactly
    /// what Invariant 11 (no silent auto-resolution) forbids; over-
    /// flagging just costs an extra human look at something that turns
    /// out fine.
    fn overlap_window(&self) -> (usize, usize) {
        let width = self.count.max(1);
        (self.start, self.start + width)
    }

    fn overlaps(&self, other: &HunkRange) -> bool {
        let (a_start, a_end) = self.overlap_window();
        let (b_start, b_end) = other.overlap_window();
        a_start < b_end && b_start < a_end
    }
}

impl fmt::Display for HunkRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.start, self.count)
    }
}

/// Run `git diff --no-index --unified=0` between two content strings and
/// return the baseline-anchored hunk ranges (the `-start,count` side of
/// each hunk header). Exit code 0 (identical) yields an empty vec; exit
/// code 1 (files differ, the normal case) is parsed; any other exit code
/// is a real error.
fn diff_hunks(old_content: &str, new_content: &str) -> Result<Vec<HunkRange>, AttributionError> {
    let dir = std::env::temp_dir().join(format!(
        "ck_mutation_attribution_{}_{}",
        std::process::id(),
        nonce()
    ));
    std::fs::create_dir_all(&dir).map_err(AttributionError::TempFileWrite)?;
    let old_path = dir.join("baseline");
    let new_path = dir.join("candidate");

    write_file(&old_path, old_content)?;
    write_file(&new_path, new_content)?;

    let output = Command::new("git")
        .args(["diff", "--no-index", "--unified=0", "--"])
        .arg(&old_path)
        .arg(&new_path)
        .output()
        .map_err(AttributionError::GitInvocation)?;

    let _ = std::fs::remove_dir_all(&dir);

    match output.status.code() {
        Some(0) => Ok(Vec::new()),
        Some(1) => {
            let stdout = String::from_utf8(output.stdout).map_err(|_| AttributionError::NonUtf8Output)?;
            parse_hunk_headers(&stdout)
        }
        other => {
            let code = other.unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(AttributionError::UnexpectedGitExit(code, stderr))
        }
    }
}

fn write_file(path: &std::path::Path, content: &str) -> Result<(), AttributionError> {
    let mut f = std::fs::File::create(path).map_err(AttributionError::TempFileWrite)?;
    f.write_all(content.as_bytes()).map_err(AttributionError::TempFileWrite)?;
    Ok(())
}

fn nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Parses every `@@ -start[,count] +...` header out of unified diff
/// output, taking only the baseline (`-`) side -- that is the coordinate
/// system both the AI diff and the user diff share, since both are
/// computed against the same baseline content.
fn parse_hunk_headers(diff_output: &str) -> Result<Vec<HunkRange>, AttributionError> {
    let mut ranges = Vec::new();
    for line in diff_output.lines() {
        if !line.starts_with("@@ ") {
            continue;
        }
        // Format: "@@ -start[,count] +start2[,count2] @@ optional context"
        let after_at = line.trim_start_matches("@@ ");
        let old_field = after_at
            .split_whitespace()
            .next()
            .ok_or_else(|| AttributionError::UnparseableHunkHeader(line.to_string()))?;
        let old_field = old_field
            .strip_prefix('-')
            .ok_or_else(|| AttributionError::UnparseableHunkHeader(line.to_string()))?;

        let (start_str, count_str) = match old_field.split_once(',') {
            Some((s, c)) => (s, c),
            None => (old_field, "1"), // "@@ -N +..." means N,1
        };
        let start: usize = start_str
            .parse()
            .map_err(|_| AttributionError::UnparseableHunkHeader(line.to_string()))?;
        let count: usize = count_str
            .parse()
            .map_err(|_| AttributionError::UnparseableHunkHeader(line.to_string()))?;

        ranges.push(HunkRange { start, count });
    }
    Ok(ranges)
}

/// Classify every changed line-region of one file into Blueprint §4.6's
/// four categories, given the file's baseline content, what the attempt
/// produced (`ai_result`), and the live workspace's current content
/// (`live_current`) -- all as plain strings, not filesystem paths, so
/// callers can supply them however they obtained them (already read from
/// disk, held in memory from an earlier step, etc.).
///
/// `path` is recorded on each returned `MutationAttribution` for the
/// caller's convenience; it is not read from disk by this function.
pub fn attribute_mutations(
    path: impl Into<PathBuf>,
    baseline: &str,
    ai_result: &str,
    live_current: &str,
    expected: &ExpectedOperation,
) -> Result<Vec<MutationAttribution>, AttributionError> {
    let path = path.into();
    let ai_hunks = diff_hunks(baseline, ai_result)?;
    let user_hunks = diff_hunks(baseline, live_current)?;

    let is_expected_file = expected.files.contains(&path);
    let mut attributions = Vec::new();

    for ai_hunk in &ai_hunks {
        let overlapping_user_hunk = user_hunks.iter().find(|u| u.overlaps(ai_hunk));
        let classification = if overlapping_user_hunk.is_some() {
            RegionClassification::ConflictRegion
        } else if is_expected_file {
            RegionClassification::ExpectedConductorChange
        } else {
            RegionClassification::UnexpectedAiChange
        };
        attributions.push(MutationAttribution {
            path: path.clone(),
            baseline_range: ai_hunk.overlap_window(),
            classification,
        });
    }

    for user_hunk in &user_hunks {
        let overlaps_any_ai_hunk = ai_hunks.iter().any(|a| a.overlaps(user_hunk));
        if overlaps_any_ai_hunk {
            // Already recorded once as ConflictRegion from the AI-hunk
            // loop above; don't double-report the same overlapping
            // region from the user side too.
            continue;
        }
        attributions.push(MutationAttribution {
            path: path.clone(),
            baseline_range: user_hunk.overlap_window(),
            classification: RegionClassification::UserChange,
        });
    }

    Ok(attributions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn expected(files: &[&str]) -> ExpectedOperation {
        ExpectedOperation::new(files.iter().map(PathBuf::from), "test intent")
    }

    #[test]
    fn no_changes_anywhere_is_an_empty_result() {
        let content = "line1\nline2\nline3\n";
        let result = attribute_mutations("a.txt", content, content, content, &expected(&["a.txt"])).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn ai_only_change_in_an_expected_file_is_expected_conductor_change() {
        let baseline = "line1\nline2\nline3\n";
        let ai_result = "line1\nCHANGED\nline3\n";
        let live = baseline; // user made no changes
        let result = attribute_mutations("a.txt", baseline, ai_result, live, &expected(&["a.txt"])).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].classification, RegionClassification::ExpectedConductorChange);
    }

    #[test]
    fn ai_only_change_in_an_unexpected_file_is_unexpected_ai_change() {
        let baseline = "line1\nline2\nline3\n";
        let ai_result = "line1\nCHANGED\nline3\n";
        let live = baseline;
        // "a.txt" changed, but expected_operation only declared "other.txt"
        let result = attribute_mutations("a.txt", baseline, ai_result, live, &expected(&["other.txt"])).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].classification, RegionClassification::UnexpectedAiChange);
    }

    #[test]
    fn user_only_change_is_user_change() {
        let baseline = "line1\nline2\nline3\n";
        let ai_result = baseline; // AI made no changes to this file
        let live = "line1\nUSER EDITED\nline3\n";
        let result = attribute_mutations("a.txt", baseline, ai_result, live, &expected(&["a.txt"])).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].classification, RegionClassification::UserChange);
    }

    #[test]
    fn overlapping_regions_are_conflict_region() {
        let baseline = "line1\nline2\nline3\n";
        let ai_result = "line1\nAI CHANGED\nline3\n";
        let live = "line1\nUSER CHANGED\nline3\n"; // same line, different edit
        let result = attribute_mutations("a.txt", baseline, ai_result, live, &expected(&["a.txt"])).unwrap();
        assert_eq!(result.len(), 1, "overlapping region must be reported once, not twice");
        assert_eq!(result[0].classification, RegionClassification::ConflictRegion);
    }

    #[test]
    fn disjoint_regions_in_the_same_file_are_not_a_conflict() {
        let baseline = "line1\nline2\nline3\nline4\nline5\n";
        let ai_result = "line1\nAI CHANGED\nline3\nline4\nline5\n"; // line 2
        let live = "line1\nline2\nline3\nline4\nUSER CHANGED\n"; // line 5
        let result = attribute_mutations("a.txt", baseline, ai_result, live, &expected(&["a.txt"])).unwrap();
        assert_eq!(result.len(), 2, "two disjoint regions must be reported separately");
        assert!(result
            .iter()
            .any(|a| a.classification == RegionClassification::ExpectedConductorChange));
        assert!(result.iter().any(|a| a.classification == RegionClassification::UserChange));
    }

    #[test]
    fn adjacent_pure_insertions_at_the_same_baseline_line_are_treated_as_a_conflict() {
        // Both the AI and the user inserted new content right after the
        // same baseline line (a classic simultaneous-append collision).
        // Neither insertion "removes" any baseline line (count == 0 on
        // both sides), so this exercises the conservative overlap rule
        // for pure insertions specifically.
        let baseline = "line1\nline2\n";
        let ai_result = "line1\nline2\nAI ADDED\n";
        let live = "line1\nline2\nUSER ADDED\n";
        let result = attribute_mutations("a.txt", baseline, ai_result, live, &expected(&["a.txt"])).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].classification,
            RegionClassification::ConflictRegion,
            "two insertions anchored at the same baseline line must be conservatively flagged as a conflict"
        );
    }

    #[test]
    fn insertions_at_different_baseline_lines_are_not_a_conflict() {
        let baseline = "line1\nline2\nline3\n";
        let ai_result = "line1\nAI ADDED\nline2\nline3\n"; // inserted after line 1
        let live = "line1\nline2\nline3\nUSER ADDED\n"; // inserted after line 3
        let result = attribute_mutations("a.txt", baseline, ai_result, live, &expected(&["a.txt"])).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|a| a.classification != RegionClassification::ConflictRegion));
    }

    #[test]
    fn baseline_range_is_reported_in_baseline_line_coordinates() {
        let baseline = "a\nb\nc\nd\ne\n";
        let ai_result = "a\nb\nCHANGED\nd\ne\n"; // baseline line 3
        let result = attribute_mutations("f.txt", baseline, ai_result, baseline, &expected(&["f.txt"])).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].baseline_range, (3, 4));
    }

    #[test]
    fn path_is_carried_through_to_the_result() {
        let baseline = "x\n";
        let ai_result = "y\n";
        let result =
            attribute_mutations("nested/dir/file.txt", baseline, ai_result, baseline, &expected(&["nested/dir/file.txt"]))
                .unwrap();
        assert_eq!(result[0].path, PathBuf::from("nested/dir/file.txt"));
    }
}
