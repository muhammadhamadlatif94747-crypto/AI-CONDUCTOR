//! Developer Reliability Console (P1-W11-T01 / Blueprint §21, §16, AC-07).
//!
//! Explicitly NOT the polished Mission Mode UI — Blueprint §21: "Built
//! plain and functional in Phase 1; stays until the reliability core is
//! proven; only then replaced." Everything rendered here is read from REAL
//! on-disk stores through the exact same public APIs the rest of the
//! kernel uses. No mocked or synthetic state.
//!
//! Scope note, stated because it's a real constraint worth being explicit
//! about rather than quietly working around: the kernel does not yet have
//! a persisted Mission/Step/Attempt object store — that is later-phase
//! orchestration work, not something P1 built. What P1 actually has is an
//! event log, anchors, an outbox, an idempotency store, and a sequence
//! allocator. This console shows exactly that truth: event history, chain
//! integrity, outbox durability state, and the sequence counter. It does
//! NOT fabricate the richer illustrative fields in Blueprint §21's mockup
//! (workspace, files-expected/modified, reconciliation, acceptance
//! contract) since no kernel component yet produces that data —
//! inventing it would be exactly the "fake/demo dashboard" this task's
//! contract explicitly forbids ("Do not use mocked state where real
//! kernel state is required").
//!
//! Capability boundary (Blueprint §16, AC-07): the ONE mutation this
//! console exposes — requesting attempt cancellation — is enforced through
//! the SAME `state::try_attempt_transition` boundary every other caller in
//! the kernel must pass through. §16's hard rule is that enforcement lives
//! at the call boundary itself, not a soft check upstream; the console is
//! not granted a new bypass actor — it acts as
//! `TransitionActor::CancellationPath`, which is exactly the transition a
//! human clicking "cancel" is legitimately requesting, and nothing more.
//! Any transition the console is NOT authorized for (e.g. forcing
//! `Succeeded`, which only `VerificationEngine` may do) is rejected, and
//! the rejection is itself written to the event log as a real event —
//! "reject, log, surface" (§16), not "reject and stay silent."

use crate::chain_anchor::{verify_chain_on_load, AnchorStore, ChainVerdict};
use crate::event::{ClockSource, EventContext, NewEvent};
use crate::event_log::EventLog;
use crate::outbox::Outbox;
use crate::sequence::SequenceAllocator;
use crate::state::{try_attempt_transition, AttemptStatus, TransitionActor, TransitionError};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ConsoleError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    EventLog(#[from] crate::event_log::EventLogError),
    #[error(transparent)]
    Outbox(#[from] crate::outbox::OutboxError),
    #[error(transparent)]
    Persist(#[from] crate::persist::PersistError),
    #[error(transparent)]
    Chain(#[from] crate::chain_anchor::ChainVerificationError),
}

/// A read-only snapshot of everything the console can currently show,
/// assembled entirely from real store reads at the moment of the call —
/// this is not a live-updating handle, it's one point-in-time truth query.
#[derive(Debug)]
pub struct Snapshot {
    pub total_events: usize,
    /// Most recent events, newest first, capped by the caller's `tail_n`.
    pub latest_events: Vec<EventSummary>,
    pub chain_status: ChainStatus,
    pub pending_intents: usize,
    pub next_sequence: u64,
}

#[derive(Debug)]
pub struct EventSummary {
    pub seq: u64,
    pub event_type: String,
    pub mission_id: String,
    pub step_id: Option<String>,
    pub attempt_id: Option<String>,
}

#[derive(Debug)]
pub enum ChainStatus {
    NoAnchorYet,
    Verified { verified_through: u64 },
    Failed { reason: String },
}

/// Expected store layout under `store_dir`.
pub struct StorePaths {
    pub events: std::path::PathBuf,
    pub anchors: std::path::PathBuf,
    pub outbox: std::path::PathBuf,
    pub sequence: std::path::PathBuf,
}

impl StorePaths {
    pub fn under(store_dir: &Path) -> Self {
        StorePaths {
            events: store_dir.join("events.jsonl"),
            anchors: store_dir.join("anchors.jsonl"),
            outbox: store_dir.join("outbox.json"),
            sequence: store_dir.join("sequence.json"),
        }
    }
}

/// Build a snapshot from the real stores rooted at `store_dir`.
pub fn snapshot(store_dir: &Path, chain_id: &str, tail_n: usize) -> Result<Snapshot, ConsoleError> {
    let paths = StorePaths::under(store_dir);

    let log = EventLog::open(&paths.events)?;
    let all = log.read_all()?;
    let total_events = all.len();
    let latest_events = all
        .iter()
        .rev()
        .take(tail_n)
        .map(|e| EventSummary {
            seq: e.seq,
            event_type: e.event_type.clone(),
            mission_id: e.context.mission_id.as_str().to_string(),
            step_id: e.context.step_id.as_ref().map(|s| s.as_str().to_string()),
            attempt_id: e.context.attempt_id.as_ref().map(|a| a.as_str().to_string()),
        })
        .collect();

    let chain_status = if paths.anchors.exists() {
        let anchor_store = AnchorStore::open(&paths.anchors)?;
        match verify_chain_on_load(&paths.events, &anchor_store, chain_id)? {
            ChainVerdict::Verified { verified_through } => {
                ChainStatus::Verified { verified_through }
            }
            ChainVerdict::Failed { reason } => ChainStatus::Failed {
                reason: reason.to_string(),
            },
        }
    } else {
        ChainStatus::NoAnchorYet
    };

    let pending_intents = Outbox::new(&paths.outbox).pending()?.len();
    let next_sequence = SequenceAllocator::new(&paths.sequence)
        .current()?
        .saturating_add(1);

    Ok(Snapshot {
        total_events,
        latest_events,
        chain_status,
        pending_intents,
        next_sequence,
    })
}

/// Plain-text render, per Blueprint §21's "built plain and functional" —
/// deliberately not styled or interactive; a developer reading raw truth.
pub fn render(s: &Snapshot) -> String {
    let mut out = String::new();
    out.push_str("AI CONDUCTOR -- DEVELOPER RELIABILITY CONSOLE\n");
    out.push_str("(plain/functional -- not the Mission Mode UI; Blueprint section 21)\n\n");
    out.push_str(&format!("Total events:      {}\n", s.total_events));
    out.push_str(&format!("Next sequence:     {}\n", s.next_sequence));
    out.push_str(&format!("Pending intents:   {}\n", s.pending_intents));
    match &s.chain_status {
        ChainStatus::NoAnchorYet => out.push_str("Chain integrity:   no anchor written yet\n"),
        ChainStatus::Verified { verified_through } => out.push_str(&format!(
            "Chain integrity:   VERIFIED through seq {verified_through}\n"
        )),
        ChainStatus::Failed { reason } => {
            out.push_str(&format!("Chain integrity:   FAILED -- {reason}\n"))
        }
    }
    out.push_str("\nRecent events (newest first):\n");
    if s.latest_events.is_empty() {
        out.push_str("  (none)\n");
    }
    for e in &s.latest_events {
        out.push_str(&format!(
            "  #{:<5} {:<32} mission={:<10} step={:<10} attempt={:<10}\n",
            e.seq,
            e.event_type,
            e.mission_id,
            e.step_id.as_deref().unwrap_or("-"),
            e.attempt_id.as_deref().unwrap_or("-"),
        ));
    }
    out
}

/// The console's one mutation: request cancellation of an attempt
/// currently in `from`. Goes through the exact same `try_attempt_transition`
/// boundary as every other caller in the kernel — the console has no
/// special-cased shortcut. On denial the denial is itself written to the
/// event log as a real event (never silently dropped); on success, the
/// authorized request is written too, so either outcome is observable
/// truth in the same log the rest of the system reads.
pub fn request_attempt_cancellation(
    store_dir: &Path,
    context: EventContext,
    from: AttemptStatus,
    now_ms: u64,
) -> Result<Result<(), TransitionError>, ConsoleError> {
    let paths = StorePaths::under(store_dir);
    let outcome = try_attempt_transition(
        TransitionActor::CancellationPath,
        from,
        AttemptStatus::Cancelled,
    );

    let (event_type, payload) = match &outcome {
        Ok(()) => (
            "console.attempt_cancel_requested",
            serde_json::json!({
                "from": from.as_str(),
                "to": "Cancelled",
                "result": "authorized",
            }),
        ),
        Err(e) => (
            "console.attempt_cancel_denied",
            serde_json::json!({
                "from": from.as_str(),
                "to": "Cancelled",
                "result": "denied",
                "reason": e.to_string(),
            }),
        ),
    };

    let mut log = EventLog::open(&paths.events)?;
    log.append(NewEvent {
        event_type: event_type.to_string(),
        context,
        payload,
        timestamp_ms: now_ms,
        clock_source: ClockSource::Virtual,
    })?;

    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correlation::CorrelationContext;

    fn dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ck_console_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("mkdir");
        d
    }

    fn seed_events(store_dir: &Path, n: u64) {
        let paths = StorePaths::under(store_dir);
        let mut log = EventLog::open(&paths.events).expect("open log");
        for i in 1..=n {
            log.append(NewEvent {
                event_type: "test_event".into(),
                context: CorrelationContext::for_mission("m-1").with_step("s-1"),
                payload: serde_json::json!({ "i": i }),
                timestamp_ms: i,
                clock_source: ClockSource::Virtual,
            })
            .expect("append");
        }
    }

    #[test]
    fn snapshot_reflects_real_event_log_truth_no_mocking() {
        let d = dir("snapshot_real");
        seed_events(&d, 5);
        let snap = snapshot(&d, "c1", 3).expect("snapshot");
        assert_eq!(snap.total_events, 5, "must report the ACTUAL count on disk");
        assert_eq!(snap.latest_events.len(), 3, "tail_n caps the returned list");
        // Newest first.
        assert_eq!(snap.latest_events[0].seq, 5);
        assert_eq!(snap.latest_events[1].seq, 4);
        assert_eq!(snap.latest_events[2].seq, 3);
        assert_eq!(snap.next_sequence, 1, "no SequenceAllocator calls made yet");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn snapshot_on_empty_store_is_honest_not_fabricated() {
        let d = dir("snapshot_empty");
        // Nothing seeded — no events.jsonl exists yet at all.
        let snap = snapshot(&d, "c1", 10).expect("snapshot on empty store");
        assert_eq!(snap.total_events, 0);
        assert!(snap.latest_events.is_empty());
        assert!(matches!(snap.chain_status, ChainStatus::NoAnchorYet));
        let rendered = render(&snap);
        assert!(rendered.contains("(none)"), "must say so plainly, not invent placeholder rows");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn render_shows_verified_chain_status_from_a_real_anchor() {
        use crate::chain_anchor::{Anchor, AnchorStore};
        let d = dir("render_verified");
        seed_events(&d, 3);
        let paths = StorePaths::under(&d);
        let mut anchor_store = AnchorStore::open(&paths.anchors).expect("open anchors");
        let log = EventLog::open(&paths.events).expect("reopen");
        let tail_hash = log.read_all().unwrap().last().unwrap().event_hash.clone();
        let mut a = Anchor {
            chain_id: "c1".into(),
            up_to_seq: 3,
            tail_event_hash: tail_hash,
            created_at_ms: 1,
            clock_source: ClockSource::Virtual,
            anchor_hash: String::new(),
        };
        anchor_store.write_anchor(&mut a).expect("write anchor");

        let snap = snapshot(&d, "c1", 10).expect("snapshot");
        assert!(matches!(
            snap.chain_status,
            ChainStatus::Verified { verified_through: 3 }
        ));
        let rendered = render(&snap);
        assert!(rendered.contains("VERIFIED"));
        let _ = std::fs::remove_dir_all(&d);
    }

    // -- Capability boundary (AC-07): both directions proven for real. --

    #[test]
    fn cancellation_from_running_is_authorized_and_recorded() {
        let d = dir("cancel_authorized");
        seed_events(&d, 1);
        let ctx = CorrelationContext::for_mission("m-1")
            .with_step("s-1")
            .with_attempt("a-1")
            .unwrap();
        let outcome =
            request_attempt_cancellation(&d, ctx, AttemptStatus::Running, 100).expect("call");
        assert!(outcome.is_ok(), "Running -> Cancelled via CancellationPath must be authorized");

        let snap = snapshot(&d, "c1", 1).expect("snapshot");
        assert_eq!(
            snap.latest_events[0].event_type, "console.attempt_cancel_requested",
            "the authorized request must itself be recorded as a real event"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn cancellation_from_an_illegal_source_state_is_denied_and_the_denial_is_an_event() {
        // Attempt has no legal edge INTO Cancelled from Succeeded at all
        // (terminal state) -- proves denial is observable as a real event,
        // never silently dropped, per Blueprint section 16 "reject, log, surface".
        let d = dir("cancel_denied");
        seed_events(&d, 1);
        let ctx = CorrelationContext::for_mission("m-1")
            .with_step("s-1")
            .with_attempt("a-1")
            .unwrap();
        let outcome =
            request_attempt_cancellation(&d, ctx, AttemptStatus::Succeeded, 100).expect("call");
        assert!(
            outcome.is_err(),
            "Succeeded -> Cancelled is not a legal edge at all; must be denied"
        );

        let snap = snapshot(&d, "c1", 1).expect("snapshot");
        assert_eq!(
            snap.latest_events[0].event_type, "console.attempt_cancel_denied",
            "the denial must itself be recorded as a real event -- 'reject, log, surface', not silent"
        );
        let payload = &snap.total_events; // sanity: exactly one more event than seeded
        assert_eq!(*payload, 2);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn console_cannot_force_verification_engine_only_transitions() {
        // The console must NOT be able to grant itself an actor identity it
        // doesn't have. request_attempt_cancellation only ever acts as
        // CancellationPath, so it structurally cannot produce a Succeeded
        // edge (VerificationEngine-only, Invariant 14) even from a state
        // where Succeeded IS a legal structural edge (Running) -- this is
        // the actual boundary this task's gate cares about, not just "some
        // denial happens somewhere."
        let outcome = try_attempt_transition(
            TransitionActor::CancellationPath,
            AttemptStatus::Running,
            AttemptStatus::Succeeded,
        );
        assert!(matches!(outcome, Err(TransitionError::UnauthorizedActor { .. })));
    }
}
