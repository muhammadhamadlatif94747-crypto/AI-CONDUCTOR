//! Append-only JSONL event log with hash chaining (P1-W03 / AC-04 §2).
//!
//! Storage contract:
//! - JSON Lines: one serialized `Event` per line.
//! - APPEND-ONLY: the public API offers no update or delete path — structurally
//!   impossible to rewrite history through this type.
//! - Every append chains onto the previous event's hash; the log refuses an
//!   append whose claimed predecessor does not match its own tail (fail-closed).

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::event::{Event, NewEvent, GENESIS_HASH};

#[derive(Debug, thiserror::Error)]
pub enum EventLogError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(
        "chain mismatch: new event claims predecessor {claimed} but log tail is {actual}"
    )]
    ChainMismatch { claimed: String, actual: String },
    #[error("corrupt line {line_no}: {reason}")]
    CorruptLine { line_no: usize, reason: String },
    #[error("line {line_no} has an unsupported schema version: {reason}")]
    UnsupportedSchemaVersion { line_no: usize, reason: String },
}

impl EventLogError {
    fn from_decode(line_no: usize, e: crate::schema::VersionedDecodeError) -> Self {
        match e {
            crate::schema::VersionedDecodeError::Serde(e) => EventLogError::CorruptLine {
                line_no,
                reason: e.to_string(),
            },
            crate::schema::VersionedDecodeError::Schema(e) => {
                EventLogError::UnsupportedSchemaVersion {
                    line_no,
                    reason: e.to_string(),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct EventLog {
    path: PathBuf,
    file: File,
    tail_hash: String,
    last_seq: u64,
}

impl EventLog {
    /// Open the log at `path`. If the file exists and is non-empty, the tail
    /// hash and last sequence are recovered by reading it (verification on
    /// load is P1-W04's full job; here we only need the chain anchors).
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EventLogError> {
        let path = path.as_ref().to_path_buf();
        let exists = path.exists() && path.metadata()?.len() > 0;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)?;
        let (tail_hash, last_seq) = if exists {
            read_tail(&path)?
        } else {
            (GENESIS_HASH.to_string(), 0)
        };
        Ok(EventLog {
            path,
            file,
            tail_hash,
            last_seq,
        })
    }

    pub fn tail_hash(&self) -> &str {
        &self.tail_hash
    }

    pub fn len(&self) -> u64 {
        self.last_seq
    }

    pub fn is_empty(&self) -> bool {
        self.last_seq == 0
    }

    /// Append one event. The sequence number is assigned by the log itself
    /// (monotonic); the previous-event link is set by the log from its own
    /// tail — a caller cannot forge it. Fails closed on any chain surprise.
    pub fn append(&mut self, new_event: NewEvent) -> Result<Event, EventLogError> {
        let seq = self.last_seq + 1;
        let event = Event {
            seq,
            event_type: new_event.event_type,
            context: new_event.context,
            payload: new_event.payload,
            timestamp_ms: new_event.timestamp_ms,
            clock_source: new_event.clock_source,
            previous_event_hash: self.tail_hash.clone(),
            event_hash: String::new(),
        };
        let mut event = event;
        event.event_hash = event.computed_hash();

        let line = crate::schema::encode_versioned(&event)?;
        // Single write call for one line: append semantics keep multi-record
        // consistency questions out of scope here (outbox is P1-W06).
        self.file.write_all(line.as_bytes())?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;

        self.tail_hash = event.event_hash.clone();
        self.last_seq = seq;
        Ok(event)
    }

    /// Read all events back in order. Any corrupt line fails loudly with its
    /// line number (fail-closed; never skipped silently).
    pub fn read_all(&self) -> Result<Vec<Event>, EventLogError> {
        let reader = BufReader::new(File::open(&self.path)?);
        let mut out = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            let line_no = idx + 1;
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let ev: Event = crate::schema::decode_versioned(&line)
                .map_err(|e| EventLogError::from_decode(line_no, e))?;
            out.push(ev);
        }
        Ok(out)
    }
}

fn read_tail(path: &Path) -> Result<(String, u64), EventLogError> {
    let reader = BufReader::new(File::open(path)?);
    let mut tail_hash = GENESIS_HASH.to_string();
    let mut last_seq = 0u64;
    for (idx, line) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let ev: Event = crate::schema::decode_versioned(&line)
            .map_err(|e| EventLogError::from_decode(line_no, e))?;
        tail_hash = ev.event_hash;
        last_seq = ev.seq;
    }
    Ok((tail_hash, last_seq))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{ClockSource, EventContext};

    fn tmp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ck_eventlog_test_{}_{}.jsonl",
            tag,
            std::process::id()
        ))
    }

    fn ctx(mission: &str) -> EventContext {
        EventContext::for_mission(mission)
    }

    fn ev(event_type: &str, n: u64) -> NewEvent {
        NewEvent {
            event_type: event_type.into(),
            context: ctx("m-test"),
            payload: serde_json::json!({ "n": n }),
            timestamp_ms: 42_000 + n,
            clock_source: ClockSource::Virtual,
        }
    }

    fn fresh_log(tag: &str) -> (PathBuf, EventLog) {
        let p = tmp_path(tag);
        let _ = std::fs::remove_file(&p);
        let log = EventLog::open(&p).expect("open");
        (p, log)
    }

    #[test]
    fn appends_chain_and_reopen_recovers_tail() {
        let (p, mut log) = fresh_log("chain");
        assert!(log.is_empty());
        assert_eq!(log.tail_hash(), GENESIS_HASH);

        let e1 = log.append(ev("a", 1)).expect("append 1");
        let e2 = log.append(ev("b", 2)).expect("append 2");
        let e3 = log.append(ev("c", 3)).expect("append 3");
        assert_eq!(e1.previous_event_hash, GENESIS_HASH);
        assert_eq!(e2.previous_event_hash, e1.event_hash);
        assert_eq!(e3.previous_event_hash, e2.event_hash);
        assert_eq!(log.len(), 3);

        drop(log);
        let reopened = EventLog::open(&p).expect("reopen");
        assert_eq!(reopened.len(), 3);
        assert_eq!(reopened.tail_hash(), e3.event_hash);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn read_all_returns_events_in_order_and_verifiable() {
        let (p, mut log) = fresh_log("readall");
        for i in 1..=5 {
            log.append(ev("t", i)).expect("append");
        }
        let all = log.read_all().expect("read_all");
        assert_eq!(all.len(), 5);
        for w in all.windows(2) {
            assert!(w[1].verify_link_to(&w[0]), "broken link {}->{}", w[0].seq, w[1].seq);
        }
        assert!(all[0].previous_event_hash == GENESIS_HASH);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn tampering_with_history_is_detectable_end_to_end() {
        // The property that makes the whole design work: edit ONE historical
        // line on disk, then walk the chain — verification must fail.
        let (p, mut log) = fresh_log("tamper");
        for i in 1..=4 {
            log.append(ev("t", i)).expect("append");
        }
        drop(log);

        // Tamper: rewrite payload of line 2 in place. Go through the same
        // schema-versioned encode/decode the log itself uses — a real
        // attacker editing an existing on-disk JSON line in place wouldn't
        // strip the schema_version key, only change the fields they're
        // targeting, so the test should reproduce that shape faithfully
        // rather than reconstructing the line from a bare, version-unaware
        // `Event` (which would silently drop schema_version and produce a
        // line no real tamper attempt would ever actually leave behind).
        let content = std::fs::read_to_string(&p).expect("read");
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        let mut e2: Event =
            crate::schema::decode_versioned(&lines[1]).expect("parse line 2");
        e2.payload = serde_json::json!({ "n": 999999 });
        e2.event_hash = e2.computed_hash(); // attacker even re-mines the hash
        lines[1] = crate::schema::encode_versioned(&e2).expect("reserialize");
        std::fs::write(&p, lines.join("\n") + "\n").expect("write tampered");

        // Walk: e3's stored previous_event_hash no longer equals tampered e2's hash.
        let reopened = EventLog::open(&p).expect("reopen");
        let all = reopened.read_all().expect("read_all still parses");
        assert!(
            !all.windows(2).all(|w| w[1].verify_link_to(&w[0])),
            "tampered history MUST break link verification"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn corrupt_line_fails_closed_at_open_and_read() {
        // Fail-closed rule: opening a log with an unparsable line must REFUSE —
        // appends cannot safely chain onto an undeterminable tail.
        let (p, mut log) = fresh_log("corrupt");
        log.append(ev("t", 1)).expect("append");
        drop(log);

        let content = std::fs::read_to_string(&p).expect("read");
        std::fs::write(&p, content + "THIS IS NOT JSON\n").expect("append garbage");

        match EventLog::open(&p) {
            Err(EventLogError::CorruptLine { line_no, .. }) => assert_eq!(line_no, 2),
            other => panic!("expected CorruptLine(2) at open, got {:?}", other.map(|_| ())),
        }
        // read_all on the same path reports the same corruption.
        let probe = EventLog { path: p.clone(), file: std::fs::OpenOptions::new().read(true).open(&p).expect("ro"), tail_hash: GENESIS_HASH.into(), last_seq: 0 };
        match probe.read_all() {
            Err(EventLogError::CorruptLine { line_no, .. }) => assert_eq!(line_no, 2),
            other => panic!("expected CorruptLine(2) at read_all, got {:?}", other.map(|_| ())),
        }
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn sequence_numbers_are_monotonic_from_one() {
        let (p, mut log) = fresh_log("monotonic");
        for i in 1..=10 {
            let e = log.append(ev("t", i)).expect("append");
            assert_eq!(e.seq, i);
        }
        let _ = std::fs::remove_file(&p);
    }
}
