//! Outbox / intent mechanism for multi-record writes (P1-W06 / Blueprint
//! Invariant 15, Section 20; VG-P1-W06-T01).
//!
//! Blueprint §Invariant 15 (as narrowed in v2.4): until SQLite provides real
//! multi-record transactions, any operation that must touch more than one
//! record (e.g. "mark attempt Succeeded" + "write the corresponding event"
//! + "update the checkpoint") is made durable via the outbox pattern:
//!
//!   1. Write an INTENT record first (durably, before any effect runs),
//!      naming every step the operation must perform.
//!   2. Perform each step. After each one succeeds, mark that step
//!      complete in the intent (durably) *before* moving to the next.
//!   3. Once every step is complete, retire the intent.
//!
//! If the process crashes at any point, startup recovery (`pending()`)
//! finds the intent exactly where it was left — with some prefix of steps
//! already marked complete — and the caller replays only the steps that
//! are NOT yet marked complete. A step's effect function is expected to be
//! idempotent-safe to call again only if it was never marked complete;
//! this module's guarantee is narrower and specific: it never loses an
//! intent, and it never re-reports a step as pending once that step has
//! been durably marked complete. It does not itself make arbitrary business
//! logic idempotent (that is Invariant 16 / the idempotency-key store,
//! P1-W07) — it guarantees the *bookkeeping* of "which steps are done"
//! survives a crash, which is what makes replay-until-idempotent possible
//! in the first place.
//!
//! Built on `persist::AtomicStore`, so every mutation here inherits that
//! primitive's two guarantees: no reader ever observes a half-written
//! outbox file, and concurrent in-process callers touching the same
//! `Outbox` are serialized (no lost update, per F-0001).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::persist::{AtomicStore, PersistError};

/// One step within a multi-record write, identified by a caller-chosen
/// name that is stable across the life of the intent (e.g. "write_event",
/// "update_checkpoint").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepRecord {
    pub name: String,
    pub completed: bool,
}

/// Durable status of an intent as a whole. `Pending` covers "not started"
/// and "partially applied" alike — the per-step `completed` flags are what
/// distinguish those cases; `status` only needs to say whether recovery
/// still has work to do.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentStatus {
    Pending,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Intent {
    pub id: String,
    pub kind: String,
    pub status: IntentStatus,
    pub steps: Vec<StepRecord>,
}

impl Intent {
    /// True once every named step is marked complete.
    pub fn all_steps_complete(&self) -> bool {
        self.steps.iter().all(|s| s.completed)
    }

    /// Steps not yet marked complete, in the original declared order —
    /// exactly what a recovering caller must replay.
    pub fn pending_steps(&self) -> Vec<&str> {
        self.steps
            .iter()
            .filter(|s| !s.completed)
            .map(|s| s.name.as_str())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
struct OutboxState {
    /// Keyed by intent id. `BTreeMap` for deterministic on-disk ordering
    /// (stable diffs, deterministic `pending()` iteration order).
    intents: BTreeMap<String, Intent>,
}

#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error("intent '{0}' does not exist")]
    UnknownIntent(String),
    #[error("intent '{0}' has no step named '{1}'")]
    UnknownStep(String, String),
    #[error("intent '{0}' already exists")]
    DuplicateIntent(String),
}

/// A durable, crash-recoverable outbox of in-flight multi-record writes.
pub struct Outbox {
    store: AtomicStore,
}

impl Outbox {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Outbox {
            store: AtomicStore::new(path),
        }
    }

    /// Step 1 of the pattern: durably record the intent — every step it
    /// will perform, none yet completed — BEFORE any effect runs. This is
    /// the write that makes the operation crash-recoverable at all: once
    /// this returns `Ok`, the intent survives a crash even if zero steps
    /// have run yet.
    pub fn begin_intent(
        &self,
        id: impl Into<String>,
        kind: impl Into<String>,
        step_names: &[&str],
    ) -> Result<Intent, OutboxError> {
        let id = id.into();
        let kind = kind.into();
        let steps: Vec<StepRecord> = step_names
            .iter()
            .map(|n| StepRecord {
                name: (*n).to_string(),
                completed: false,
            })
            .collect();

        let id_for_closure = id.clone();
        let duplicate = Rc::new(RefCell::new(false));
        let duplicate_slot = Rc::clone(&duplicate);
        let result = self.store.update(move |current: Option<OutboxState>| {
            let mut state = current.unwrap_or_default();
            if state.intents.contains_key(&id_for_closure) {
                *duplicate_slot.borrow_mut() = true;
                return state;
            }
            state.intents.insert(
                id_for_closure.clone(),
                Intent {
                    id: id_for_closure.clone(),
                    kind: kind.clone(),
                    status: IntentStatus::Pending,
                    steps: steps.clone(),
                },
            );
            state
        })?;

        if *duplicate.borrow() {
            return Err(OutboxError::DuplicateIntent(id));
        }
        Ok(result
            .intents
            .get(&id)
            .cloned()
            .expect("just inserted above"))
    }

    /// Step 2 of the pattern: durably mark one step of an existing intent
    /// complete, AFTER the caller has actually performed that step's
    /// effect. Marking a step complete that is already complete is a
    /// no-op (safe to call again during replay without double-marking).
    pub fn mark_step_complete(
        &self,
        intent_id: &str,
        step_name: &str,
    ) -> Result<Intent, OutboxError> {
        let intent_id_owned = intent_id.to_string();
        let step_name_owned = step_name.to_string();
        let error: Rc<RefCell<Option<OutboxError>>> = Rc::new(RefCell::new(None));
        let error_slot = Rc::clone(&error);

        let result = self.store.update(move |current: Option<OutboxState>| {
            let mut state = current.unwrap_or_default();
            let Some(intent) = state.intents.get_mut(&intent_id_owned) else {
                *error_slot.borrow_mut() = Some(OutboxError::UnknownIntent(intent_id_owned.clone()));
                return state;
            };
            let Some(step) = intent.steps.iter_mut().find(|s| s.name == step_name_owned) else {
                *error_slot.borrow_mut() = Some(OutboxError::UnknownStep(
                    intent_id_owned.clone(),
                    step_name_owned.clone(),
                ));
                return state;
            };
            step.completed = true;
            if intent.all_steps_complete() {
                intent.status = IntentStatus::Completed;
            }
            state
        })?;

        if let Some(e) = error.borrow_mut().take() {
            return Err(e);
        }
        Ok(result
            .intents
            .get(intent_id)
            .cloned()
            .expect("existence checked above"))
    }

    /// Step 3 of the pattern: retire a fully-completed intent so it no
    /// longer appears in `pending()`. Retiring an intent with incomplete
    /// steps is refused — retiring is only ever a bookkeeping cleanup
    /// after the durable record has already done its job, never a way to
    /// abandon unfinished work.
    pub fn retire_intent(&self, intent_id: &str) -> Result<(), OutboxError> {
        let intent_id_owned = intent_id.to_string();
        let error: Rc<RefCell<Option<OutboxError>>> = Rc::new(RefCell::new(None));
        let error_slot = Rc::clone(&error);

        self.store.update(move |current: Option<OutboxState>| {
            let mut state = current.unwrap_or_default();
            match state.intents.get(&intent_id_owned) {
                None => {
                    *error_slot.borrow_mut() = Some(OutboxError::UnknownIntent(intent_id_owned.clone()));
                }
                Some(intent) if !intent.all_steps_complete() => {
                    // Refuse silently-lossy retirement: leave state
                    // untouched and surface an error instead. This must
                    // never be reachable in correct callers (they only
                    // retire what mark_step_complete just reported as
                    // Completed), but the type system can't prove that,
                    // so the fail-closed check stays.
                    *error_slot.borrow_mut() = Some(OutboxError::UnknownStep(
                        intent_id_owned.clone(),
                        "<incomplete-steps-remain>".to_string(),
                    ));
                }
                Some(_) => {
                    state.intents.remove(&intent_id_owned);
                }
            }
            state
        })?;

        if let Some(e) = error.borrow_mut().take() {
            return Err(e);
        }
        Ok(())
    }

    /// Recovery entry point: every intent that is not fully complete,
    /// in deterministic id order. Startup recovery calls this once and
    /// replays each returned intent's `pending_steps()` — this is the
    /// mechanism that makes a crash mid-multi-record-write recoverable
    /// rather than silently half-applied or lost (VG-P1-W06-T01).
    pub fn pending(&self) -> Result<Vec<Intent>, OutboxError> {
        let state: OutboxState =
            crate::persist::read_versioned(self.store.path())?.unwrap_or_default();
        Ok(state
            .intents
            .into_values()
            .filter(|i| i.status == IntentStatus::Pending)
            .collect())
    }

    /// Look up a single intent by id, regardless of status. Mainly useful
    /// in tests and diagnostics.
    pub fn get(&self, intent_id: &str) -> Result<Option<Intent>, OutboxError> {
        let state: OutboxState =
            crate::persist::read_versioned(self.store.path())?.unwrap_or_default();
        Ok(state.intents.get(intent_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_outbox_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    #[test]
    fn begin_intent_records_all_steps_as_pending() {
        let d = dir("begin");
        let ob = Outbox::new(d.join("outbox.json"));
        let intent = ob
            .begin_intent("i1", "mark_attempt_succeeded", &["write_event", "update_checkpoint"])
            .expect("begin");
        assert_eq!(intent.status, IntentStatus::Pending);
        assert_eq!(intent.pending_steps(), vec!["write_event", "update_checkpoint"]);
        assert!(!intent.all_steps_complete());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn duplicate_intent_id_is_rejected_not_silently_merged() {
        let d = dir("dup");
        let ob = Outbox::new(d.join("outbox.json"));
        ob.begin_intent("i1", "k", &["a"]).expect("first");
        let err = ob.begin_intent("i1", "k", &["a", "b"]).unwrap_err();
        assert!(matches!(err, OutboxError::DuplicateIntent(ref id) if id == "i1"));
        // Original intent must be untouched by the rejected duplicate.
        let intent = ob.get("i1").unwrap().unwrap();
        assert_eq!(intent.steps.len(), 1, "rejected duplicate must not mutate the existing intent");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn happy_path_all_steps_complete_then_retire() {
        let d = dir("happy");
        let ob = Outbox::new(d.join("outbox.json"));
        ob.begin_intent("i1", "k", &["a", "b"]).expect("begin");
        ob.mark_step_complete("i1", "a").expect("mark a");
        let after_a = ob.get("i1").unwrap().unwrap();
        assert_eq!(after_a.status, IntentStatus::Pending, "still pending with b outstanding");

        let after_b = ob.mark_step_complete("i1", "b").expect("mark b");
        assert_eq!(after_b.status, IntentStatus::Completed);
        assert!(after_b.all_steps_complete());

        ob.retire_intent("i1").expect("retire");
        assert!(ob.get("i1").unwrap().is_none(), "retired intent must be gone");
        assert!(ob.pending().unwrap().is_empty());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn retiring_an_incomplete_intent_is_refused() {
        let d = dir("retire_refused");
        let ob = Outbox::new(d.join("outbox.json"));
        ob.begin_intent("i1", "k", &["a", "b"]).expect("begin");
        ob.mark_step_complete("i1", "a").expect("mark a");
        let err = ob.retire_intent("i1").unwrap_err();
        assert!(matches!(err, OutboxError::UnknownStep(_, _)));
        // Intent must still be present and still show b as pending —
        // the refused retirement must not have deleted or mutated it.
        let intent = ob.get("i1").unwrap().unwrap();
        assert_eq!(intent.pending_steps(), vec!["b"]);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn marking_an_already_complete_step_again_is_a_safe_no_op() {
        let d = dir("idempotent_mark");
        let ob = Outbox::new(d.join("outbox.json"));
        ob.begin_intent("i1", "k", &["a"]).expect("begin");
        ob.mark_step_complete("i1", "a").expect("first mark");
        let second = ob.mark_step_complete("i1", "a").expect("second mark, replay-safe");
        assert!(second.all_steps_complete());
        assert_eq!(second.status, IntentStatus::Completed);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn unknown_intent_and_unknown_step_are_reported_not_silently_ignored() {
        let d = dir("unknown");
        let ob = Outbox::new(d.join("outbox.json"));
        let err = ob.mark_step_complete("ghost", "a").unwrap_err();
        assert!(matches!(err, OutboxError::UnknownIntent(ref id) if id == "ghost"));

        ob.begin_intent("i1", "k", &["a"]).expect("begin");
        let err2 = ob.mark_step_complete("i1", "not_a_real_step").unwrap_err();
        assert!(matches!(err2, OutboxError::UnknownStep(ref i, ref s) if i == "i1" && s == "not_a_real_step"));
        let _ = fs::remove_dir_all(&d);
    }

    /// VG-P1-W06-T01's core proof: interruption mid-outbox → restart
    /// completes or reconciles the intent, never loses it. We simulate a
    /// crash by simply dropping the first `Outbox` handle after only some
    /// steps are marked complete (no in-memory state survives — the ONLY
    /// thing that can make recovery work is what's on disk), then opening
    /// a brand new `Outbox` over the same path exactly as a fresh process
    /// restart would, and confirming recovery sees precisely the
    /// unfinished work and nothing more.
    #[test]
    fn interruption_mid_outbox_is_recovered_and_replayed_exactly_once() {
        let d = dir("crash_recovery");
        let path = d.join("outbox.json");

        {
            let ob = Outbox::new(&path);
            ob.begin_intent(
                "mission-7-attempt-3-succeeded",
                "mark_attempt_succeeded",
                &["write_event", "update_checkpoint", "advance_step_state"],
            )
            .expect("begin");
            // Simulate the process performing the first step's effect and
            // durably recording it, then crashing before the rest run.
            ob.mark_step_complete("mission-7-attempt-3-succeeded", "write_event")
                .expect("mark step 1");
            // "Crash" here: `ob` is dropped with 2 of 3 steps still
            // pending. No cleanup, no graceful shutdown.
        }

        // Restart: brand new Outbox, brand new process in spirit — it
        // only has what `path` holds on disk.
        let recovered = Outbox::new(&path);
        let pending = recovered.pending().expect("pending after restart");
        assert_eq!(pending.len(), 1, "the interrupted intent must not be lost");
        let intent = &pending[0];
        assert_eq!(intent.id, "mission-7-attempt-3-succeeded");
        assert_eq!(
            intent.pending_steps(),
            vec!["update_checkpoint", "advance_step_state"],
            "recovery must resume from exactly where it left off — not \
             re-run the already-completed step, not skip an unfinished one"
        );

        // Caller's recovery logic replays only the pending steps.
        for step in intent.pending_steps() {
            recovered
                .mark_step_complete(&intent.id, step)
                .expect("replay step");
        }
        let final_intent = recovered.get(&intent.id).unwrap().unwrap();
        assert!(final_intent.all_steps_complete());
        assert_eq!(final_intent.status, IntentStatus::Completed);
        assert!(
            recovered.pending().unwrap().is_empty(),
            "once replayed, nothing should remain pending"
        );
        let _ = fs::remove_dir_all(&d);
    }

    /// A second, harsher interruption scenario: the crash happens with
    /// ZERO steps completed (crash right after `begin_intent`, before any
    /// effect ran at all). Recovery must still see the whole intent and
    /// replay every step — the outbox record itself, not any partial
    /// progress, is what makes the operation recoverable.
    #[test]
    fn interruption_before_any_step_runs_still_recovers_full_intent() {
        let d = dir("crash_before_any_step");
        let path = d.join("outbox.json");

        {
            let ob = Outbox::new(&path);
            ob.begin_intent("i1", "k", &["a", "b"]).expect("begin");
            // Crash immediately — no steps ever marked.
        }

        let recovered = Outbox::new(&path);
        let pending = recovered.pending().expect("pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].pending_steps(), vec!["a", "b"]);
        let _ = fs::remove_dir_all(&d);
    }

    /// Multiple in-flight intents at once — recovery must return exactly
    /// the incomplete ones, in deterministic order, and never conflate one
    /// intent's steps with another's.
    #[test]
    fn multiple_concurrent_intents_are_tracked_independently() {
        let d = dir("multi");
        let ob = Outbox::new(d.join("outbox.json"));
        ob.begin_intent("i1", "k", &["a"]).expect("begin i1");
        ob.begin_intent("i2", "k", &["a", "b"]).expect("begin i2");
        ob.mark_step_complete("i1", "a").expect("complete i1");
        ob.retire_intent("i1").expect("retire i1");

        let pending = ob.pending().expect("pending");
        assert_eq!(pending.len(), 1, "only i2 should remain pending");
        assert_eq!(pending[0].id, "i2");
        assert_eq!(pending[0].pending_steps(), vec!["a", "b"]);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn pending_on_a_store_that_has_never_been_written_is_empty_not_an_error() {
        let d = dir("never_written");
        let ob = Outbox::new(d.join("outbox.json"));
        assert!(ob.pending().expect("pending on fresh store").is_empty());
        let _ = fs::remove_dir_all(&d);
    }
}
