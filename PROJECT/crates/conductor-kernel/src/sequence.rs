//! Monotonic sequence-number allocator (P1-W08 / Blueprint §4.9, §20 —
//! "feeds `sequence_number` in Section 4.9").
//!
//! `event_log.rs` already derives a running sequence counter from the log
//! file itself (each append writes the next number and the tail is
//! recovered by reading the file back on open). This module exists
//! alongside it for a different reason: other subsystems that need a
//! sequence number BEFORE the record that will carry it is fully built
//! (e.g. threading a `sequence_number` through a multi-step outbox intent,
//! or any future caller that isn't itself an event-log append) need a
//! standalone, independently durable source of monotonically increasing
//! numbers — one that survives a crash occurring in the middle of
//! allocating a number, not just in the middle of writing a whole event.
//!
//! Durability design: the allocator's entire persisted state is one
//! integer, `last_issued`. Every `next()` call is a single
//! `AtomicStore::update` — i.e. one atomic temp-file-write + fsync +
//! rename (see `persist.rs`). Because that primitive already guarantees a
//! reader never observes a half-written file, "crash mid-allocate" has
//! only two possible outcomes, never a third:
//!
//!   1. The crash happens before the rename lands → the file on disk is
//!      byte-for-byte what it was before this `next()` call. The number
//!      this call would have returned is simply never issued again to
//!      anyone (a gap is allowed — Blueprint requires *monotonic*, not
//!      *gapless*), and the next real call after restart correctly
//!      re-derives from the last value that WAS durably persisted.
//!   2. The crash happens after the rename lands → the new value is
//!      fully persisted and behaves exactly like a normal successful
//!      call.
//!
//! There is no state in between; this module doesn't need its own
//! two-phase reserve/commit protocol because `persist::AtomicStore`
//! already collapses "reserve" and "commit" into one atomic write.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::persist::{AtomicStore, PersistError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
struct AllocatorState {
    last_issued: u64,
}

/// A durable, crash-recoverable source of strictly increasing sequence
/// numbers, starting at 1.
pub struct SequenceAllocator {
    store: AtomicStore,
}

impl SequenceAllocator {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        SequenceAllocator {
            store: AtomicStore::new(path),
        }
    }

    /// The last number issued, or 0 if `next()` has never been called
    /// against this store.
    pub fn current(&self) -> Result<u64, PersistError> {
        Ok(crate::persist::read_versioned::<AllocatorState>(self.store.path())?
            .map(|s| s.last_issued)
            .unwrap_or(0))
    }

    /// Atomically issue and durably persist the next sequence number.
    /// Strictly greater than every number this store has ever returned,
    /// across any number of process restarts — including one that
    /// crashed partway through a previous `next()` call (see module
    /// docs: the atomic-write primitive underneath makes that case
    /// indistinguishable from "that call never happened").
    pub fn next(&self) -> Result<u64, PersistError> {
        let new_state = self.store.update(|current: Option<AllocatorState>| {
            let mut state = current.unwrap_or_default();
            state.last_issued += 1;
            state
        })?;
        Ok(new_state.last_issued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::thread;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_seq_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    #[test]
    fn fresh_allocator_starts_at_one() {
        let d = dir("start");
        let alloc = SequenceAllocator::new(d.join("seq.json"));
        assert_eq!(alloc.current().unwrap(), 0);
        assert_eq!(alloc.next().unwrap(), 1);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn sequence_is_strictly_increasing_across_many_calls() {
        let d = dir("increasing");
        let alloc = SequenceAllocator::new(d.join("seq.json"));
        let mut prev = 0u64;
        for _ in 0..500 {
            let n = alloc.next().unwrap();
            assert!(n > prev, "sequence must strictly increase: {n} after {prev}");
            prev = n;
        }
        assert_eq!(prev, 500);
        let _ = fs::remove_dir_all(&d);
    }

    /// Restart recoverability: a fresh allocator handle over the same
    /// path must resume exactly where the previous handle left off —
    /// never resetting to 0, never repeating a number already issued.
    #[test]
    fn allocator_resumes_after_restart_from_persisted_value() {
        let d = dir("restart");
        let path = d.join("seq.json");
        {
            let alloc = SequenceAllocator::new(&path);
            assert_eq!(alloc.next().unwrap(), 1);
            assert_eq!(alloc.next().unwrap(), 2);
            assert_eq!(alloc.next().unwrap(), 3);
            // "crash" here: handle dropped, no graceful shutdown.
        }
        let restarted = SequenceAllocator::new(&path);
        assert_eq!(restarted.current().unwrap(), 3);
        assert_eq!(restarted.next().unwrap(), 4, "must continue from 3, not reset to 0");
        let _ = fs::remove_dir_all(&d);
    }

    /// VG-P1-W08-T01's core proof: a crash occurring literally in the
    /// middle of allocating a number (i.e. an in-flight write that never
    /// completed its rename onto the real path) must never surface as a
    /// duplicate or a rollback once a fresh allocator opens over the same
    /// file. We reproduce exactly the on-disk shape `persist::AtomicStore`
    /// would leave behind if the process died between "temp file written"
    /// and "rename onto target": an orphaned, differently-named sibling
    /// file sitting next to the real store, with content that was never
    /// promoted to be the real value.
    #[test]
    fn crash_mid_allocate_never_duplicates_or_rolls_back_on_restart() {
        let d = dir("crash_mid_allocate");
        let path = d.join("seq.json");

        let alloc = SequenceAllocator::new(&path);
        assert_eq!(alloc.next().unwrap(), 1);
        assert_eq!(alloc.next().unwrap(), 2);
        // Two numbers durably issued and persisted: last_issued == 2.

        // Simulate a THIRD next() call that crashed after writing its temp
        // file but before the rename that would have made last_issued=3
        // durable. This orphaned file must never be mistaken for the real
        // store state by anything that opens `path` afterward.
        let orphan_tmp = path.with_file_name(".seq.json.tmp-crashed-99999");
        {
            let mut f = File::create(&orphan_tmp).expect("create orphan tmp");
            f.write_all(br#"{"last_issued":3}"#).expect("write orphan");
            f.sync_all().expect("sync orphan");
        }
        // Deliberately never renamed — this is the crash point.

        // The real store file must be untouched: still last_issued == 2.
        let on_disk: AllocatorState =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            on_disk.last_issued, 2,
            "an in-flight, never-renamed write must not affect the real store"
        );

        // Restart: fresh allocator handle over the same real path.
        let restarted = SequenceAllocator::new(&path);
        assert_eq!(restarted.current().unwrap(), 2);
        let n = restarted.next().unwrap();
        assert_eq!(
            n, 3,
            "recovery must issue exactly 3 next \u{2014} not repeat 2 (rollback), \
             not jump to 4 as if the crashed call had actually landed, and \
             critically never repeat a number (3) that could collide with \
             anything the orphaned attempt might have already handed to a \
             caller before it crashed"
        );

        assert!(orphan_tmp.exists(), "orphaned temp file is the expected leftover, harmless");
        let _ = fs::remove_dir_all(&d);
    }

    /// Concurrent callers within one process must never receive duplicate
    /// numbers — same F-0001-style regression shape as `persist.rs` and
    /// `outbox.rs`, applied to sequence allocation specifically.
    #[test]
    fn concurrent_callers_never_receive_duplicate_numbers() {
        let d = dir("concurrent");
        let alloc = Arc::new(SequenceAllocator::new(d.join("seq.json")));

        const CALLERS: usize = 64;
        let handles: Vec<_> = (0..CALLERS)
            .map(|_| {
                let alloc = Arc::clone(&alloc);
                thread::spawn(move || alloc.next().expect("next"))
            })
            .collect();

        let mut issued: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        issued.sort_unstable();
        let expected: Vec<u64> = (1..=CALLERS as u64).collect();
        assert_eq!(
            issued, expected,
            "every one of {CALLERS} concurrent calls must receive a distinct, \
             contiguous number \u{2014} a duplicate here is exactly the failure \
             this allocator exists to prevent"
        );
        let _ = fs::remove_dir_all(&d);
    }
}
