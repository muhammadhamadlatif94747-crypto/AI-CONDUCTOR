//! Atomic, single-writer persistence primitive (P1-W05 / Blueprint §4.8,
//! F-0001 regression requirement).
//!
//! Two independent guarantees, composed:
//!
//! 1. ATOMICITY: a write either fully lands or the prior file is left
//!    completely untouched — a reader can never observe a partially-written
//!    file. Implemented via temp-file-in-the-same-directory + fsync +
//!    rename (Blueprint §4.8: "JSON files written atomically (temp file +
//!    rename)"). The target path is never opened for writing at all; only
//!    the temp file is, so there is no code path through which a partial
//!    write could land in the target.
//! 2. SINGLE-WRITER SERIALIZATION: concurrent callers updating the same
//!    logical store are serialized through one in-process mutex, so a
//!    read-modify-write sequence can never race with another and silently
//!    lose the earlier writer's change. This is the direct fix for F-0001
//!    (two parallel same-file edits; one silently dropped because each was
//!    computed from a stale read of the file).
//!
//! Scope note: this module — like `event_log.rs` and `chain_anchor.rs` — is
//! I/O by design. See the corrected crate-level doc comment in `lib.rs`.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Schema(#[from] crate::schema::SchemaError),
}

impl From<crate::schema::VersionedDecodeError> for PersistError {
    fn from(e: crate::schema::VersionedDecodeError) -> Self {
        match e {
            crate::schema::VersionedDecodeError::Serde(e) => PersistError::Serde(e),
            crate::schema::VersionedDecodeError::Schema(e) => PersistError::Schema(e),
        }
    }
}

/// Write `bytes` to `path` atomically.
///
/// Sequence: create a temp file IN THE SAME DIRECTORY as `path` (so the
/// final rename is a same-filesystem operation the OS can make atomic) →
/// write the full contents → fsync the temp file's data → rename onto
/// `path` (atomically replaces any existing file) → best-effort fsync of
/// the parent directory so the rename itself survives a crash immediately
/// after. If the process dies at any point before the rename, `path` is
/// left exactly as it was; the only visible leftover is an orphaned temp
/// file, never a corrupted target.
pub fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> Result<(), PersistError> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty());
    if let Some(dir) = dir {
        fs::create_dir_all(dir)?;
    }
    let tmp_path = tmp_path_for(path);
    {
        let mut f = File::create(&tmp_path)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;
    if let Some(dir) = dir {
        if let Ok(dir_file) = File::open(dir) {
            let _ = dir_file.sync_all(); // best-effort; not all platforms support fsync on a dir handle
        }
    }
    Ok(())
}

/// Serialize `value` to pretty JSON and write it atomically to `path`.
pub fn atomic_write_json<T: serde::Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), PersistError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    atomic_write_bytes(path, &bytes)
}

/// Deterministic per-process temp-file name so repeated calls from the same
/// process reuse (and overwrite) the same staging file rather than leaking
/// a new one on every write.
fn tmp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("store");
    path.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()))
}

/// A JSON-backed store whose updates are serialized behind one in-process
/// mutex, closing F-0001's root cause: two writers computing a new value
/// from a stale read of the same file, with the second write silently
/// clobbering the first.
pub struct AtomicStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl AtomicStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        AtomicStore {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read-modify-write under the single-writer lock. `f` receives the
    /// current parsed value (`None` if the store file does not exist yet)
    /// and returns the value to persist. The read, the caller's merge
    /// logic, and the write all happen while holding the lock — no other
    /// call to `update` on THIS `AtomicStore` can interleave, so no update
    /// can be computed from a state that another concurrent update has
    /// already superseded.
    ///
    /// Scope: this serializes writers *within one process* that share this
    /// `AtomicStore` handle. It does not by itself arbitrate two separate
    /// OS processes writing the same path — that needs an OS-level file
    /// lock, out of scope for P1-W05 (single-process kernel); tracked as a
    /// follow-on if/when multi-process writers to the same store exist.
    pub fn update<T, F>(&self, f: F) -> Result<T, PersistError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        F: FnOnce(Option<T>) -> T,
    {
        let _guard = self.lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let current: Option<T> = read_versioned(&self.path)?;
        let new_value = f(current);
        let bytes = crate::schema::encode_versioned_bytes(&new_value)?;
        atomic_write_bytes(&self.path, &bytes)?;
        Ok(new_value)
    }
}

/// Read a schema-versioned whole-file JSON record, or `None` if the file
/// doesn't exist yet. Shared by `AtomicStore::update` and by every module
/// that reads one of its stores WITHOUT going through `update` (e.g.
/// `SequenceAllocator::current()`, `IdempotencyStore::read_state()`) — those
/// bypass paths matter exactly as much for schema-version fail-closed
/// behavior as `update` does; a version check that only lived inside
/// `update` would leave every read-only accessor able to silently load an
/// unsupported version.
pub fn read_versioned<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Option<T>, PersistError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(crate::schema::decode_versioned_bytes(&bytes)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_persist_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    // -- Atomicity --

    #[test]
    fn atomic_write_round_trips_and_replaces_prior_content() {
        let d = dir("roundtrip");
        let p = d.join("store.json");
        atomic_write_bytes(&p, b"{\"v\":1}").expect("write 1");
        assert_eq!(fs::read_to_string(&p).unwrap(), "{\"v\":1}");

        atomic_write_bytes(&p, b"{\"v\":2,\"bigger\":true}").expect("write 2");
        assert_eq!(fs::read_to_string(&p).unwrap(), "{\"v\":2,\"bigger\":true}");

        // No leftover temp file after a successful write.
        let tmp = tmp_path_for(&p);
        assert!(!tmp.exists(), "temp file must not survive a successful write");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn target_is_never_touched_before_the_final_rename() {
        // Proves "no partial state observable": we replicate everything
        // atomic_write_bytes does EXCEPT the final rename (simulating a
        // crash right before it), then assert the target file — which a
        // reader could open at any moment — still holds its original,
        // complete content, not a truncated or half-written mix.
        let d = dir("crash_before_rename");
        let p = d.join("store.json");
        atomic_write_bytes(&p, b"original-complete-content").expect("seed");

        let tmp_path = tmp_path_for(&p);
        {
            let mut f = File::create(&tmp_path).expect("create tmp");
            f.write_all(b"new-content-that-never-lands").expect("write tmp");
            f.sync_all().expect("sync tmp");
        }
        // Deliberately do NOT rename — this is the "crash" point.

        assert_eq!(
            fs::read_to_string(&p).unwrap(),
            "original-complete-content",
            "target must be untouched until the rename actually happens"
        );
        assert!(tmp_path.exists(), "orphaned temp file is the only visible leftover");

        // A subsequent real write still succeeds cleanly and replaces both
        // the target and the leftover temp file.
        atomic_write_bytes(&p, b"recovered-write").expect("recover");
        assert_eq!(fs::read_to_string(&p).unwrap(), "recovered-write");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn atomic_write_creates_parent_directories() {
        let d = dir("mkdirs");
        let p = d.join("nested").join("deeper").join("store.json");
        atomic_write_bytes(&p, b"x").expect("write with mkdir");
        assert_eq!(fs::read_to_string(&p).unwrap(), "x");
        let _ = fs::remove_dir_all(&d);
    }

    // -- AtomicStore / single-writer serialization --

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq)]
    struct Ledger {
        entries: Vec<u32>,
    }

    #[test]
    fn update_initializes_from_none_when_store_does_not_exist() {
        let d = dir("init_none");
        let store = AtomicStore::new(d.join("ledger.json"));
        let result: Ledger = store
            .update(|current: Option<Ledger>| {
                assert!(current.is_none(), "fresh store must hand back None");
                Ledger { entries: vec![1] }
            })
            .expect("update");
        assert_eq!(result.entries, vec![1]);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn update_reads_back_the_previously_written_value() {
        let d = dir("read_back");
        let store = AtomicStore::new(d.join("ledger.json"));
        store
            .update(|_: Option<Ledger>| Ledger { entries: vec![1] })
            .expect("first");
        let second = store
            .update(|current: Option<Ledger>| {
                let mut l = current.expect("must see the first write");
                l.entries.push(2);
                l
            })
            .expect("second");
        assert_eq!(second.entries, vec![1, 2]);
        let _ = fs::remove_dir_all(&d);
    }

    /// F-0001 regression test: many threads concurrently issue
    /// read-modify-write updates against the SAME `AtomicStore`. Before the
    /// single-writer lock existed (the original bug), two writers computing
    /// their new value from the same stale read would silently clobber one
    /// another and one entry would be lost. With serialization, every
    /// writer's entry must survive.
    #[test]
    fn f0001_regression_no_lost_update_under_concurrent_writers() {
        let d = dir("f0001");
        let store = Arc::new(AtomicStore::new(d.join("ledger.json")));
        store
            .update(|_: Option<Ledger>| Ledger::default())
            .expect("seed");

        const WRITERS: u32 = 64;
        let handles: Vec<_> = (0..WRITERS)
            .map(|i| {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    store
                        .update(move |current: Option<Ledger>| {
                            let mut l = current.expect("seeded above");
                            l.entries.push(i);
                            l
                        })
                        .expect("concurrent update");
                })
            })
            .collect();
        for h in handles {
            h.join().expect("writer thread panicked");
        }

        let bytes = fs::read(store.path()).expect("read final ledger");
        let final_ledger: Ledger = serde_json::from_slice(&bytes).expect("parse final ledger");

        let mut seen: Vec<u32> = final_ledger.entries.clone();
        seen.sort_unstable();
        let expected: Vec<u32> = (0..WRITERS).collect();
        assert_eq!(
            seen, expected,
            "every writer's entry must be present exactly once — F-0001 is a LOST update, \
             so a missing id here is the exact regression this test exists to catch"
        );
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn corrupt_existing_file_surfaces_as_error_not_silent_data_loss() {
        let d = dir("corrupt");
        let p = d.join("ledger.json");
        fs::write(&p, b"THIS IS NOT JSON").expect("seed garbage");
        let store = AtomicStore::new(p);
        let result = store.update(|_: Option<Ledger>| Ledger::default());
        assert!(
            matches!(result, Err(PersistError::Serde(_))),
            "a corrupt existing file must fail closed, never be silently treated as empty"
        );
        let _ = fs::remove_dir_all(&d);
    }
}
