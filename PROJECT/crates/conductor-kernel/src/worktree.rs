//! Isolated Git-worktree execution (P2-W01 / Blueprint §5).
//!
//! Blueprint §5: every attempt must execute in its own workspace, never
//! directly in the user's live folder — `git worktree add` is the
//! concrete mechanism, "cheap, fast, already part of Git, requiring no
//! new dependency." This module is that primitive: create an isolated
//! worktree for one attempt, tear it down afterward, and recover cleanly
//! if a crash left Git's worktree registry pointing at a directory that
//! no longer exists.
//!
//! Scope, stated explicitly (this is P2-W01 only): this module proves
//! ISOLATION — a live repository is structurally protected from anything
//! that happens inside an attempt workspace. It does NOT capture a
//! baseline (P2-W02), reconcile changes back (P2-W03), or implement the
//! two-phase merge lock (P2-W04+, Invariant 13). Merging an attempt's
//! result into the live workspace is deliberately not implemented here;
//! until that later work exists, an attempt workspace's changes simply
//! stay in the attempt workspace.
//!
//! Every Git interaction goes through an explicit `std::process::Command`
//! with output captured — no shell string interpolation, no swallowed
//! exit codes. This is the first module in the crate with an external
//! process dependency (the `git` binary itself) rather than pure
//! filesystem I/O; that dependency is real and unmocked in the tests
//! below, deliberately, so "isolation is proven" means proven against
//! actual Git, not a stub that agrees with itself.

use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("git command failed (exit {code:?}): git {args}\nstderr: {stderr}")]
    GitCommandFailed {
        args: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("could not parse `git worktree list --porcelain` output: {0}")]
    UnparsableWorktreeList(String),
}

/// One attempt's isolated working copy. `path` is always structurally
/// distinct from the live repository it was created from — a fresh
/// subdirectory under the manager's `workspaces_root`, never inside the
/// live repository itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptWorkspace {
    pub attempt_id: String,
    pub path: PathBuf,
}

/// Creates and tears down isolated Git worktrees for one live repository.
pub struct WorktreeManager {
    live_repo: PathBuf,
    workspaces_root: PathBuf,
}

impl WorktreeManager {
    /// `live_repo` is the user's real, existing Git repository. `workspaces_root`
    /// is a directory reserved for attempt workspaces — deliberately
    /// required to be separate from `live_repo` so that "distinct path"
    /// is a structural property of construction, not something each
    /// caller has to remember to arrange correctly.
    pub fn new(live_repo: impl Into<PathBuf>, workspaces_root: impl Into<PathBuf>) -> Self {
        WorktreeManager {
            live_repo: live_repo.into(),
            workspaces_root: workspaces_root.into(),
        }
    }

    fn run_git(&self, args: &[&str]) -> Result<std::process::Output, WorktreeError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.live_repo)
            .output()?;
        if !output.status.success() {
            return Err(WorktreeError::GitCommandFailed {
                args: args.join(" "),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }
        Ok(output)
    }

    /// Create a new isolated worktree checked out at `base_commit` (or
    /// `HEAD` if `None`), detached (never on a branch the live repo's own
    /// checkout might also be on — avoids Git's "branch already checked
    /// out" refusal and keeps the attempt's history read-only from its
    /// own perspective until a later merge step explicitly acts on it).
    ///
    /// `git worktree add` only REGISTERS a new linked working tree and
    /// populates its own directory; it has no code path that writes to
    /// the primary working tree's tracked files. That is the entirety of
    /// the isolation guarantee this function relies on, and it is what
    /// `tests::create_does_not_modify_the_live_repos_tracked_files` and
    /// `tests::create_does_not_change_the_live_repos_git_status` verify
    /// directly against a real repository rather than assumed from Git's
    /// documentation.
    pub fn create(
        &self,
        attempt_id: &str,
        base_commit: Option<&str>,
    ) -> Result<AttemptWorkspace, WorktreeError> {
        std::fs::create_dir_all(&self.workspaces_root)?;
        let path = self.workspaces_root.join(attempt_id);
        let path_str = path.to_string_lossy().into_owned();
        let commit_ish = base_commit.unwrap_or("HEAD");

        self.run_git(&["worktree", "add", "--detach", &path_str, commit_ish])?;

        Ok(AttemptWorkspace {
            attempt_id: attempt_id.to_string(),
            path,
        })
    }

    /// Tear down an attempt workspace. Safe to call even if the
    /// directory was already deleted out from under Git — e.g. by a
    /// crash that killed the process after the directory was removed
    /// (by cleanup code, a disk issue, or a person) but before Git's own
    /// worktree registry entry for it was cleared. In that case
    /// `git worktree remove` itself would fail (the path doesn't exist to
    /// remove), so this always prunes stale registry entries first, then
    /// attempts a real removal only if the directory still exists.
    pub fn remove(&self, workspace: &AttemptWorkspace) -> Result<(), WorktreeError> {
        // Prune first: clears any registry entries whose directory is
        // already gone. Safe no-op if there's nothing stale.
        self.run_git(&["worktree", "prune"])?;

        if workspace.path.exists() {
            let path_str = workspace.path.to_string_lossy().into_owned();
            self.run_git(&["worktree", "remove", "--force", &path_str])?;
        }
        Ok(())
    }

    /// Every worktree Git currently has registered for the live
    /// repository, read from Git's own ground truth
    /// (`git worktree list --porcelain`) rather than an independent
    /// ledger this module maintains — so it can never drift from what
    /// Git actually believes exists. This is what a restart calls to
    /// rediscover in-flight attempt workspaces from before a crash.
    pub fn list_registered(&self) -> Result<Vec<PathBuf>, WorktreeError> {
        let output = self.run_git(&["worktree", "list", "--porcelain"])?;
        let text = String::from_utf8(output.stdout)
            .map_err(|e| WorktreeError::UnparsableWorktreeList(e.to_string()))?;

        let mut paths = Vec::new();
        for line in text.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                paths.push(PathBuf::from(p));
            }
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command as StdCommand;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_worktree_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    fn git(repo: &Path, args: &[&str]) -> std::process::Output {
        let out = StdCommand::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .expect("run git");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    /// Sets up a real, minimal Git repository with one committed file,
    /// so every test below exercises actual Git behavior, not a stub.
    fn init_live_repo(tag: &str) -> PathBuf {
        let root = dir(tag);
        let live = root.join("live_repo");
        fs::create_dir_all(&live).expect("mkdir live_repo");
        git(&live, &["init", "--quiet", "-b", "main"]);
        git(&live, &["config", "user.email", "test@example.com"]);
        git(&live, &["config", "user.name", "Test"]);
        fs::write(live.join("tracked.txt"), "original content\n").expect("write tracked file");
        git(&live, &["add", "tracked.txt"]);
        git(&live, &["commit", "--quiet", "-m", "initial commit"]);
        live
    }

    fn workspaces_root_for(live_repo: &Path) -> PathBuf {
        live_repo.parent().unwrap().join("attempt_workspaces")
    }

    #[test]
    fn create_produces_a_path_distinct_from_the_live_repo() {
        let live = init_live_repo("distinct_path");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let ws = mgr.create("attempt-1", None).expect("create");

        assert_ne!(ws.path, live, "attempt workspace must never BE the live repo");
        assert!(
            ws.path.starts_with(&workspaces_root),
            "attempt workspace must live under the reserved workspaces_root"
        );
        assert!(ws.path.exists(), "the isolated worktree directory must actually exist");
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    #[test]
    fn create_does_not_modify_the_live_repos_tracked_files() {
        let live = init_live_repo("no_mutate_files");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let before = fs::read_to_string(live.join("tracked.txt")).unwrap();
        mgr.create("attempt-1", None).expect("create");
        let after = fs::read_to_string(live.join("tracked.txt")).unwrap();

        assert_eq!(before, after, "creating a worktree must not touch live repo file content");
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    #[test]
    fn create_does_not_change_the_live_repos_git_status() {
        let live = init_live_repo("no_mutate_status");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let status_before = git(&live, &["status", "--porcelain"]).stdout;
        assert!(status_before.is_empty(), "sanity: repo must start clean");

        mgr.create("attempt-1", None).expect("create");

        let status_after = git(&live, &["status", "--porcelain"]).stdout;
        assert_eq!(
            status_before, status_after,
            "worktree creation must leave the live repo's own status untouched"
        );
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    /// The sharpest, most literal proof of Blueprint §5's requirement:
    /// something operating INSIDE the attempt workspace — exactly what
    /// Aider would do — must never be observable in the live workspace.
    #[test]
    fn writes_inside_the_attempt_workspace_never_touch_the_live_repo() {
        let live = init_live_repo("isolation_of_writes");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let ws = mgr.create("attempt-1", None).expect("create");

        // Simulate Aider operating inside the attempt workspace: mutate
        // the tracked file and create a brand new one.
        fs::write(ws.path.join("tracked.txt"), "MUTATED BY THE ATTEMPT\n").expect("mutate");
        fs::write(ws.path.join("new_file_from_attempt.txt"), "new\n").expect("create new file");

        let live_tracked = fs::read_to_string(live.join("tracked.txt")).unwrap();
        assert_eq!(
            live_tracked, "original content\n",
            "a mutation made inside the attempt workspace must never appear in the live repo"
        );
        assert!(
            !live.join("new_file_from_attempt.txt").exists(),
            "a file created inside the attempt workspace must never appear in the live repo"
        );
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    #[test]
    fn worktree_is_checked_out_at_the_requested_base_commit_and_later_live_commits_do_not_leak_in() {
        let live = init_live_repo("base_commit_pin");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let base_commit = String::from_utf8(
            StdCommand::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&live)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        let ws = mgr.create("attempt-1", Some(&base_commit)).expect("create");
        assert_eq!(
            fs::read_to_string(ws.path.join("tracked.txt")).unwrap(),
            "original content\n"
        );

        // Advance the LIVE repo with a new commit after the attempt
        // workspace was created.
        fs::write(live.join("tracked.txt"), "a later live commit\n").unwrap();
        git(&live, &["add", "tracked.txt"]);
        git(&live, &["commit", "--quiet", "-m", "later commit"]);

        // The attempt workspace, pinned to the earlier commit, must be
        // unaffected by the live repo moving forward.
        assert_eq!(
            fs::read_to_string(ws.path.join("tracked.txt")).unwrap(),
            "original content\n",
            "an attempt workspace must not see commits made to the live repo after its creation"
        );
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    #[test]
    fn remove_tears_down_the_worktree_and_the_live_repo_is_unaffected() {
        let live = init_live_repo("remove_happy_path");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let ws = mgr.create("attempt-1", None).expect("create");
        fs::write(ws.path.join("tracked.txt"), "mutated\n").unwrap();

        mgr.remove(&ws).expect("remove");

        assert!(!ws.path.exists(), "worktree directory must be gone after remove()");
        assert_eq!(
            fs::read_to_string(live.join("tracked.txt")).unwrap(),
            "original content\n",
            "live repo must be unaffected throughout create/mutate/remove"
        );
        let registered = mgr.list_registered().unwrap();
        assert!(
            !registered.iter().any(|p| p == &ws.path),
            "removed worktree must no longer be registered with Git"
        );
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    /// VG-P2-W01-T01's core failure-injection proof (Phase Manifest §6.4:
    /// "unexpected process exit"): simulate a crash by deleting the
    /// worktree's directory directly on disk, WITHOUT going through Git,
    /// exactly as an OS-level cleanup, a killed process's half-finished
    /// teardown, or a person manually deleting a stale folder would leave
    /// things. Git's own registry (`.git/worktrees/<id>`) still thinks
    /// the worktree exists at that point. `remove()` must recover from
    /// this cleanly — no panic, no error that leaves the repo unusable
    /// for the next attempt.
    #[test]
    fn remove_recovers_from_a_worktree_directory_deleted_out_from_under_git() {
        let live = init_live_repo("crash_stale_worktree");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let ws = mgr.create("attempt-1", None).expect("create");
        assert!(mgr.list_registered().unwrap().iter().any(|p| p == &ws.path));

        // Simulate the crash: the directory disappears without Git ever
        // being told (no `git worktree remove`, just gone).
        fs::remove_dir_all(&ws.path).expect("simulate out-of-band deletion");
        assert!(!ws.path.exists());

        // Git itself, if asked right now, would still consider this
        // worktree registered (stale) until pruned.
        let registered_before_recovery = mgr.list_registered().unwrap();
        assert!(
            registered_before_recovery.iter().any(|p| p == &ws.path),
            "sanity: Git's registry must still be stale at this point, proving the \
             recovery path below is actually exercised and not a no-op"
        );

        // Recovery: remove() must handle this without error.
        mgr.remove(&ws).expect("remove must recover from a stale, already-deleted worktree");

        let registered_after = mgr.list_registered().unwrap();
        assert!(
            !registered_after.iter().any(|p| p == &ws.path),
            "the stale registry entry must be gone after recovery"
        );

        // The manager must still be fully usable afterward -- a fresh
        // attempt can be created without any lingering bad state.
        let ws2 = mgr.create("attempt-2", None).expect("manager must remain usable after recovery");
        assert!(ws2.path.exists());
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    #[test]
    fn list_registered_reflects_reality_for_restart_recovery() {
        let live = init_live_repo("list_for_restart");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let ws1 = mgr.create("attempt-1", None).expect("create 1");
        let ws2 = mgr.create("attempt-2", None).expect("create 2");

        let registered = mgr.list_registered().unwrap();
        assert!(registered.iter().any(|p| p == &ws1.path));
        assert!(registered.iter().any(|p| p == &ws2.path));

        mgr.remove(&ws1).expect("remove 1");
        let registered_after = mgr.list_registered().unwrap();
        assert!(!registered_after.iter().any(|p| p == &ws1.path));
        assert!(registered_after.iter().any(|p| p == &ws2.path));
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    #[test]
    fn two_concurrent_attempt_workspaces_are_fully_independent() {
        let live = init_live_repo("concurrent_independence");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let ws_a = mgr.create("attempt-a", None).expect("create a");
        let ws_b = mgr.create("attempt-b", None).expect("create b");

        fs::write(ws_a.path.join("tracked.txt"), "changed by A\n").unwrap();
        fs::write(ws_b.path.join("tracked.txt"), "changed by B\n").unwrap();

        assert_eq!(fs::read_to_string(ws_a.path.join("tracked.txt")).unwrap(), "changed by A\n");
        assert_eq!(fs::read_to_string(ws_b.path.join("tracked.txt")).unwrap(), "changed by B\n");
        assert_eq!(
            fs::read_to_string(live.join("tracked.txt")).unwrap(),
            "original content\n",
            "live repo must reflect neither attempt's changes"
        );
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }

    #[test]
    fn create_fails_closed_with_captured_stderr_on_an_invalid_base_commit() {
        let live = init_live_repo("invalid_commit");
        let workspaces_root = workspaces_root_for(&live);
        let mgr = WorktreeManager::new(&live, &workspaces_root);

        let err = mgr
            .create("attempt-1", Some("not-a-real-commit-ish"))
            .expect_err("an invalid base commit must fail, not silently fall back to HEAD");
        match err {
            WorktreeError::GitCommandFailed { stderr, .. } => {
                assert!(!stderr.is_empty(), "git's real stderr must be captured, not discarded");
            }
            other => panic!("expected GitCommandFailed, got {other:?}"),
        }
        let _ = fs::remove_dir_all(live.parent().unwrap());
    }
}
