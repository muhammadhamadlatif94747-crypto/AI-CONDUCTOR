//! Baseline capture (P2-W02 / Blueprint §4.4, §4.6).
//!
//! Blueprint §4.4's evidence shape records `"baseline": {"App.tsx":
//! "sha256:abc...", ...}` up front, before an attempt runs, precisely so a
//! later comparison answers "did the change we intended actually happen"
//! rather than just "did something change." §4.6 phrases it as two
//! separate steps -- "Attempt begins → capture baseline diff-state" then
//! later "Aider operates → capture resulting diff" -- and this module is
//! the first of those two steps only.
//!
//! Scope, stated explicitly (this is P2-W02 only): this module proves a
//! workspace's starting content is durably recorded and can be reliably
//! compared against a later state. It does NOT classify what a change
//! means (`expected_conductor_change` / `unexpected_ai_change` /
//! `user_change` / `conflict_region` -- P2-W05), does NOT reconcile
//! (P2-W03), and does NOT implement the two-phase merge re-check itself
//! (P2-W04+, Invariant 13) -- it produces the primitive that re-check
//! will later use.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::event::hex_encode;

#[derive(Debug, thiserror::Error)]
pub enum BaselineError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("path {0} is not inside the workspace root it claims to belong to")]
    OutsideWorkspace(PathBuf),
}

/// A file's path relative to the workspace root it was captured from --
/// deliberately not absolute, so a baseline captured in one workspace is
/// meaningfully comparable to a re-capture in a different, but
/// logically-corresponding, workspace (e.g. the live repo vs. an attempt
/// workspace checked out from the same commit).
pub type RelativePath = PathBuf;

/// sha256 of one file's full content, hex-encoded -- same hashing already
/// used elsewhere in this crate (event.rs, chain_anchor.rs), reused rather
/// than introducing a second hashing convention.
pub type FileHash = String;

/// The recorded starting content of every tracked file in a workspace, at
/// the moment an attempt begins.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Baseline {
    pub entries: BTreeMap<RelativePath, FileHash>,
}

/// How one path's content compares between two baselines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathChange {
    Added,
    Removed,
    Modified { before: FileHash, after: FileHash },
    Unchanged,
}

impl Baseline {
    /// Walk `workspace_root` and record a content hash for every regular
    /// file found, keyed by its path relative to `workspace_root`. `.git`
    /// at the workspace root is excluded -- it is not tracked-file content
    /// an attempt is meant to change, and its internals mutate as a side
    /// effect of ordinary Git operations unrelated to the attempt's own
    /// edits. Excluded regardless of whether it is a directory (the main
    /// repository's form) or a gitlink file (the form `git worktree add`
    /// produces in a linked working tree).
    ///
    /// This is a pure read: it never creates, modifies, or deletes
    /// anything in `workspace_root`. Symlinks are not followed (captured
    /// as their own file type is not attempted here; if present they are
    /// simply not regular files and are skipped) -- avoids the baseline
    /// silently walking outside the workspace via a symlink target.
    pub fn capture(workspace_root: impl AsRef<Path>) -> Result<Self, BaselineError> {
        let root = workspace_root.as_ref();
        let mut entries = BTreeMap::new();
        Self::walk(root, root, &mut entries)?;
        Ok(Baseline { entries })
    }

    fn walk(
        root: &Path,
        dir: &Path,
        entries: &mut BTreeMap<RelativePath, FileHash>,
    ) -> Result<(), BaselineError> {
        let read_dir = std::fs::read_dir(dir).map_err(|source| BaselineError::Io {
            path: dir.to_path_buf(),
            source,
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|source| BaselineError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|source| BaselineError::Io {
                path: path.clone(),
                source,
            })?;

            let is_git_root_entry =
                path.file_name().map(|n| n == ".git").unwrap_or(false) && path.parent() == Some(root);
            if is_git_root_entry {
                // In the main repository `.git` is a directory; inside a
                // `git worktree`-created checkout it is instead a small
                // *file* pointing at the real metadata directory elsewhere
                // (a "gitdir:" gitlink). Either form is Git's own
                // bookkeeping, not tracked-file content an attempt is
                // meant to change, so both are excluded here regardless
                // of which one `file_type` reports.
                continue;
            }

            if file_type.is_dir() {
                Self::walk(root, &path, entries)?;
            } else if file_type.is_file() {
                let content = std::fs::read(&path).map_err(|source| BaselineError::Io {
                    path: path.clone(),
                    source,
                })?;
                let hash = hex_encode(Sha256::digest(&content).as_slice());
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| BaselineError::OutsideWorkspace(path.clone()))?
                    .to_path_buf();
                entries.insert(relative, hash);
            }
            // Symlinks and other non-regular file types are deliberately
            // skipped: capturing their target content risks walking
            // outside the workspace root, and the Conductor never manages
            // non-regular-file content per Blueprint §4.6's edge-case
            // table style (mirrors the `.gitignore`d-file exclusion
            // there: things outside the tracked-content model are left
            // alone, not force-fit into it).
        }
        Ok(())
    }

    /// Compare this baseline (the "before") against `other` (the
    /// "after"), reporting every path that differs. Paths present and
    /// identical in both are omitted -- callers that need the full
    /// picture including `Unchanged` should use `diff_full`.
    pub fn diff(&self, other: &Baseline) -> BTreeMap<RelativePath, PathChange> {
        self.diff_full(other)
            .into_iter()
            .filter(|(_, change)| !matches!(change, PathChange::Unchanged))
            .collect()
    }

    /// Same as `diff`, but includes `Unchanged` entries for every path
    /// present in either baseline -- useful when a caller needs to
    /// enumerate the full set of tracked paths, not just what moved.
    pub fn diff_full(&self, other: &Baseline) -> BTreeMap<RelativePath, PathChange> {
        let mut result = BTreeMap::new();

        for (path, before_hash) in &self.entries {
            match other.entries.get(path) {
                None => {
                    result.insert(path.clone(), PathChange::Removed);
                }
                Some(after_hash) if after_hash == before_hash => {
                    result.insert(path.clone(), PathChange::Unchanged);
                }
                Some(after_hash) => {
                    result.insert(
                        path.clone(),
                        PathChange::Modified {
                            before: before_hash.clone(),
                            after: after_hash.clone(),
                        },
                    );
                }
            }
        }
        for path in other.entries.keys() {
            if !self.entries.contains_key(path) {
                result.insert(path.clone(), PathChange::Added);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn workspace(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_baseline_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    #[test]
    fn captures_a_hash_for_every_regular_file() {
        let ws = workspace("basic_capture");
        fs::write(ws.join("a.txt"), b"hello").unwrap();
        fs::create_dir_all(ws.join("sub")).unwrap();
        fs::write(ws.join("sub/b.txt"), b"world").unwrap();

        let baseline = Baseline::capture(&ws).expect("capture");
        assert_eq!(baseline.entries.len(), 2);
        assert!(baseline.entries.contains_key(&PathBuf::from("a.txt")));
        assert!(baseline.entries.contains_key(&PathBuf::from("sub/b.txt")));
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn dot_git_directory_is_excluded() {
        let ws = workspace("exclude_git");
        fs::create_dir_all(ws.join(".git")).unwrap();
        fs::write(ws.join(".git/HEAD"), b"ref: refs/heads/main\n").unwrap();
        fs::write(ws.join("tracked.txt"), b"content").unwrap();

        let baseline = Baseline::capture(&ws).expect("capture");
        assert_eq!(baseline.entries.len(), 1);
        assert!(baseline.entries.contains_key(&PathBuf::from("tracked.txt")));
        assert!(
            !baseline.entries.keys().any(|p| p.starts_with(".git")),
            ".git internals must never appear in a baseline"
        );
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn dot_git_as_a_gitlink_file_is_also_excluded() {
        // A `git worktree add` checkout has `.git` as a small FILE
        // ("gitdir: ...") rather than a directory. The exclusion must not
        // be conditioned on `.git` being a directory.
        let ws = workspace("exclude_git_gitlink_file");
        fs::write(ws.join(".git"), b"gitdir: /somewhere/else/.git/worktrees/x\n").unwrap();
        fs::write(ws.join("tracked.txt"), b"content").unwrap();

        let baseline = Baseline::capture(&ws).expect("capture");
        assert_eq!(baseline.entries.len(), 1);
        assert!(baseline.entries.contains_key(&PathBuf::from("tracked.txt")));
        assert!(!baseline.entries.contains_key(&PathBuf::from(".git")));
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn identical_content_produces_the_same_hash_deterministically() {
        let ws = workspace("deterministic_hash");
        fs::write(ws.join("a.txt"), b"same content").unwrap();
        let b1 = Baseline::capture(&ws).expect("capture 1");
        let b2 = Baseline::capture(&ws).expect("capture 2");
        assert_eq!(b1, b2, "capturing the same unchanged content twice must be identical");
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn a_single_byte_change_produces_a_different_hash() {
        let ws = workspace("single_byte_sensitivity");
        fs::write(ws.join("a.txt"), b"content-A").unwrap();
        let before = Baseline::capture(&ws).expect("capture before");

        fs::write(ws.join("a.txt"), b"content-B").unwrap();
        let after = Baseline::capture(&ws).expect("capture after");

        assert_ne!(
            before.entries.get(&PathBuf::from("a.txt")),
            after.entries.get(&PathBuf::from("a.txt")),
            "a single-byte content change must change the file's hash"
        );
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn capture_is_a_pure_read_and_never_mutates_the_workspace() {
        let ws = workspace("pure_read");
        fs::write(ws.join("a.txt"), b"unchanged").unwrap();
        let before_meta = fs::metadata(ws.join("a.txt")).unwrap().len();

        let _ = Baseline::capture(&ws).expect("capture");

        let after_meta = fs::metadata(ws.join("a.txt")).unwrap().len();
        assert_eq!(before_meta, after_meta);
        assert_eq!(
            fs::read(ws.join("a.txt")).unwrap(),
            b"unchanged",
            "capture must never modify file content"
        );
        // No new files must have appeared as a side effect of capturing.
        let baseline = Baseline::capture(&ws).unwrap();
        assert_eq!(baseline.entries.len(), 1);
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn recapture_with_no_changes_produces_an_empty_diff() {
        let ws = workspace("empty_diff");
        fs::write(ws.join("a.txt"), b"content").unwrap();
        fs::write(ws.join("b.txt"), b"more content").unwrap();

        let before = Baseline::capture(&ws).expect("capture before");
        let after = Baseline::capture(&ws).expect("capture after (no changes made)");

        assert!(
            before.diff(&after).is_empty(),
            "identical recapture must produce an empty diff"
        );
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn diff_reports_modified_added_and_removed_paths() {
        let ws = workspace("diff_categories");
        fs::write(ws.join("modified.txt"), b"original").unwrap();
        fs::write(ws.join("removed.txt"), b"will be removed").unwrap();
        fs::write(ws.join("unchanged.txt"), b"stays the same").unwrap();
        let before = Baseline::capture(&ws).expect("capture before");

        fs::write(ws.join("modified.txt"), b"changed").unwrap();
        fs::remove_file(ws.join("removed.txt")).unwrap();
        fs::write(ws.join("added.txt"), b"brand new").unwrap();
        let after = Baseline::capture(&ws).expect("capture after");

        let diff = before.diff(&after);
        assert_eq!(diff.len(), 3, "unchanged.txt must not appear in diff()");
        assert!(matches!(
            diff.get(&PathBuf::from("modified.txt")),
            Some(PathChange::Modified { .. })
        ));
        assert_eq!(diff.get(&PathBuf::from("removed.txt")), Some(&PathChange::Removed));
        assert_eq!(diff.get(&PathBuf::from("added.txt")), Some(&PathChange::Added));
        assert!(!diff.contains_key(&PathBuf::from("unchanged.txt")));

        let full = before.diff_full(&after);
        assert_eq!(
            full.get(&PathBuf::from("unchanged.txt")),
            Some(&PathChange::Unchanged),
            "diff_full must include Unchanged entries"
        );
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn modified_change_carries_both_before_and_after_hashes() {
        let ws = workspace("modified_carries_hashes");
        fs::write(ws.join("a.txt"), b"v1").unwrap();
        let before = Baseline::capture(&ws).expect("capture before");
        let expected_before_hash = before.entries.get(&PathBuf::from("a.txt")).unwrap().clone();

        fs::write(ws.join("a.txt"), b"v2").unwrap();
        let after = Baseline::capture(&ws).expect("capture after");
        let expected_after_hash = after.entries.get(&PathBuf::from("a.txt")).unwrap().clone();

        match before.diff(&after).get(&PathBuf::from("a.txt")) {
            Some(PathChange::Modified { before: b, after: a }) => {
                assert_eq!(b, &expected_before_hash);
                assert_eq!(a, &expected_after_hash);
            }
            other => panic!("expected Modified, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn baseline_from_a_worktree_created_by_p2_w01_captures_its_checked_out_content() {
        // Integration with worktree.rs: baseline capture operates on a
        // real AttemptWorkspace, not just an arbitrary directory, since
        // that is how it will actually be used.
        use crate::worktree::WorktreeManager;
        use std::process::Command as StdCommand;

        let live = workspace("worktree_integration_live");
        StdCommand::new("git")
            .args(["init", "--quiet", "--initial-branch=main"])
            .current_dir(&live)
            .output()
            .expect("git init");
        StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&live)
            .output()
            .unwrap();
        StdCommand::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&live)
            .output()
            .unwrap();
        fs::write(live.join("tracked.txt"), "original content\n").unwrap();
        StdCommand::new("git")
            .args(["add", "."])
            .current_dir(&live)
            .output()
            .unwrap();
        StdCommand::new("git")
            .args(["commit", "--quiet", "-m", "init"])
            .current_dir(&live)
            .output()
            .unwrap();

        let workspaces_root = live.parent().unwrap().join(format!(
            "ck_baseline_worktrees_{}",
            std::process::id()
        ));
        let mgr = WorktreeManager::new(&live, &workspaces_root);
        let ws = mgr.create("attempt-baseline-1", None).expect("create worktree");

        let baseline = Baseline::capture(&ws.path).expect("capture worktree baseline");
        assert!(baseline.entries.contains_key(&PathBuf::from("tracked.txt")));
        assert!(
            !baseline.entries.keys().any(|p| p.starts_with(".git")),
            ".git must be excluded from a worktree's baseline too"
        );

        mgr.remove(&ws).expect("cleanup worktree");
        let _ = fs::remove_dir_all(&live);
        let _ = fs::remove_dir_all(&workspaces_root);
    }
}
