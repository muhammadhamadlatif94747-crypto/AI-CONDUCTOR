//! Event-chain anchoring + verification on load (P1-W04 / AC-04 §3).
//!
//! Anchor model (AC-04 §3):
//! - An ANCHOR is a separately-persisted record of the chain's state at a
//!   moment in time: { chain_id, up_to_seq, tail_event_hash, created_at_ms,
//!   clock_source, anchor_hash }.
//! - Anchors live in a DIFFERENT file than the event log. Compromising the
//!   log alone cannot forge a consistent anchor, and vice versa.
//! - Written at minimum after every checkpoint (cadence enforced by callers;
//!   `AnchorStore::write_anchor` is the only way to create one).
//! - On load, the live chain is walked and verified against the nearest
//!   anchor. ANY mismatch — broken link, mutated event, forged hash, or an
//!   anchor claiming more events than exist — fails closed and BLOCKS
//!   auto-recovery (recovery is deterministic only from verified state).

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::event::{ClockSource, Event, GENESIS_HASH};

#[derive(Debug, thiserror::Error)]
pub enum ChainVerificationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("corrupt log line {line_no}: {reason}")]
    CorruptLine { line_no: usize, reason: String },
    #[error("broken link at seq {seq}: stored prev={stored_prev} but expected {expected}")]
    BrokenLink {
        seq: u64,
        stored_prev: String,
        expected: String,
    },
    #[error("event self-hash mismatch at seq {seq}")]
    SelfHashMismatch { seq: u64 },
    #[error(
        "anchor claims up_to_seq={claimed} but log contains only {actual} events"
    )]
    AnchorAheadOfLog { claimed: u64, actual: u64 },
    #[error(
        "anchor belongs to chain '{found}' but this store verifies chain '{expected}'"
    )]
    ForeignChainAnchor { expected: String, found: String },
    #[error("line {line_no} has an unsupported schema version: {reason}")]
    UnsupportedSchemaVersion { line_no: usize, reason: String },
}

impl ChainVerificationError {
    fn from_decode(line_no: usize, e: crate::schema::VersionedDecodeError) -> Self {
        match e {
            crate::schema::VersionedDecodeError::Serde(e) => {
                ChainVerificationError::CorruptLine {
                    line_no,
                    reason: e.to_string(),
                }
            }
            crate::schema::VersionedDecodeError::Schema(e) => {
                ChainVerificationError::UnsupportedSchemaVersion {
                    line_no,
                    reason: e.to_string(),
                }
            }
        }
    }
}

/// A persisted snapshot of the chain's integrity position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub chain_id: String,
    /// The anchor vouches for all events with seq <= up_to_seq.
    pub up_to_seq: u64,
    /// event_hash of the event at up_to_seq (GENESIS_HASH when up_to_seq=0).
    pub tail_event_hash: String,
    pub created_at_ms: u64,
    pub clock_source: ClockSource,
    /// SHA-256 over the anchor's own fields — detects anchor tampering.
    pub anchor_hash: String,
}

impl Anchor {
    pub fn computed_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.as_bytes());
        hasher.update(self.up_to_seq.to_le_bytes());
        hasher.update(self.tail_event_hash.as_bytes());
        hasher.update(self.created_at_ms.to_le_bytes());
        hasher.update(self.clock_source.as_str().as_bytes());
        crate::event::hex_encode_public(&hasher.finalize())
    }

    pub fn verify_self(&self) -> bool {
        self.anchor_hash == self.computed_hash()
    }
}

/// Separate-storage anchor writer/reader.
pub struct AnchorStore {
    path: PathBuf,
}

impl AnchorStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(AnchorStore { path })
    }

    /// Persist one anchor (append-only JSONL, same discipline as the log).
    pub fn write_anchor(&mut self, anchor: &mut Anchor) -> Result<(), ChainVerificationError> {
        anchor.anchor_hash = anchor.computed_hash();
        let mut f = OpenOptions::new().create(true).append(true).open(&self.path)?;
        f.write_all(crate::schema::encode_versioned(anchor)?.as_bytes())?;
        f.write_all(b"\n")?;
        f.flush()?;
        Ok(())
    }

    /// The nearest anchor at or below `up_to_seq` (the one recovery would use),
    /// or None if no usable anchor exists.
    pub fn latest_anchor_at_or_before(
        &self,
        up_to_seq: u64,
    ) -> Result<Option<Anchor>, ChainVerificationError> {
        let mut best: Option<Anchor> = None;
        let f = File::open(&self.path)?;
        for (idx, line) in BufReader::new(f).lines().enumerate() {
            let line_no = idx + 1;
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let a: Anchor = crate::schema::decode_versioned(&line)
                .map_err(|e| ChainVerificationError::from_decode(line_no, e))?;
            if a.up_to_seq <= up_to_seq {
                match &best {
                    Some(b) if b.up_to_seq >= a.up_to_seq => {}
                    _ => best = Some(a),
                }
            }
        }
        Ok(best)
    }

    pub fn read_all(&self) -> Result<Vec<Anchor>, ChainVerificationError> {
        let mut out = Vec::new();
        let f = File::open(&self.path)?;
        for (idx, line) in BufReader::new(f).lines().enumerate() {
            let line_no = idx + 1;
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            out.push(
                crate::schema::decode_versioned(&line)
                    .map_err(|e| ChainVerificationError::from_decode(line_no, e))?,
            );
        }
        Ok(out)
    }
}

/// Verification verdict for a loaded chain.
///
/// Not `PartialEq`: the failure payload embeds IO/serde errors which are not
/// equatable; callers match on variants instead.
#[derive(Debug)]
pub enum ChainVerdict {
    /// Every event verified; chain intact through seq `verified_through`.
    Verified { verified_through: u64 },
    /// Chain does not match its anchors or itself. Auto-recovery MUST be blocked.
    Failed { reason: ChainVerificationError },
}

/// Verify the whole chain in `log_path` against its anchors.
///
/// Algorithm (fail-closed by construction):
/// 1. Parse every event strictly (corrupt line => fail).
/// 2. Walk the FULL chain from genesis, verifying self-hashes and links.
///    Anchoring never excuses us from re-verifying history.
/// 3. If any anchor exists for this chain_id, pick the one with the highest
///    up_to_seq. It must be self-consistent and must NOT claim more events
///    than the log holds (that would mean deletion after anchoring).
/// 4. Pin-check: the event at the governing anchor's up_to_seq must hash to
///    exactly the anchor's tail_event_hash. This is what makes truncation,
///    mutation, and re-mining of history detectable even without per-event
///    inclusion proofs.
pub fn verify_chain_on_load(
    log_path: &Path,
    anchors: &AnchorStore,
    chain_id: &str,
) -> Result<ChainVerdict, ChainVerificationError> {
    // 1. Parse every event strictly. A missing log = empty chain.
    let events = read_events(log_path)?;

    // 2. Full walk from genesis — no shortcuts.
    let mut expected_prev = GENESIS_HASH.to_string();
    for (idx, ev) in events.iter().enumerate() {
        if ev.seq != (idx as u64) + 1 {
            return Ok(ChainVerdict::Failed {
                reason: ChainVerificationError::BrokenLink {
                    seq: ev.seq,
                    stored_prev: ev.previous_event_hash.clone(),
                    expected: expected_prev.clone(),
                },
            });
        }
        if ev.previous_event_hash != expected_prev {
            return Ok(ChainVerdict::Failed {
                reason: ChainVerificationError::BrokenLink {
                    seq: ev.seq,
                    stored_prev: ev.previous_event_hash.clone(),
                    expected: expected_prev.clone(),
                },
            });
        }
        if !ev.verify_self() {
            return Ok(ChainVerdict::Failed {
                reason: ChainVerificationError::SelfHashMismatch { seq: ev.seq },
            });
        }
        expected_prev = ev.event_hash.clone();
    }

    // 3. Governing anchor = highest up_to_seq among self-consistent anchors.
    let governing = {
        let mut best: Option<Anchor> = None;
        for a in anchors.read_all()? {
            if !a.verify_self() {
                continue;
            }
            if a.chain_id != chain_id {
                // A foreign-chain anchor in this store means cross-chain
                // contamination — fail closed rather than silently ignore.
                return Ok(ChainVerdict::Failed {
                    reason: ChainVerificationError::ForeignChainAnchor {
                        expected: chain_id.to_string(),
                        found: a.chain_id.clone(),
                    },
                });
            }
            match &best {
                Some(b) if b.up_to_seq >= a.up_to_seq => {}
                _ => best = Some(a),
            }
        }
        best
    };

    if let Some(a) = governing {
        // Anchor claiming MORE events than the log holds means events were
        // deleted after anchoring — fail closed.
        if a.up_to_seq > events.len() as u64 {
            return Ok(ChainVerdict::Failed {
                reason: ChainVerificationError::AnchorAheadOfLog {
                    claimed: a.up_to_seq,
                    actual: events.len() as u64,
                },
            });
        }
        // Pin-check: the anchored tail must still be exactly there.
        if a.up_to_seq > 0 {
            let pinned = &events[(a.up_to_seq - 1) as usize];
            if pinned.event_hash != a.tail_event_hash {
                return Ok(ChainVerdict::Failed {
                    reason: ChainVerificationError::BrokenLink {
                        seq: pinned.seq,
                        stored_prev: pinned.previous_event_hash.clone(),
                        expected: format!("anchor tail {}", a.tail_event_hash),
                    },
                });
            }
        }
    }

    // All events verified; report how far integrity is proven.
    Ok(ChainVerdict::Verified {
        verified_through: events.len() as u64,
    })
}

fn read_events(log_path: &Path) -> Result<Vec<Event>, ChainVerificationError> {
    let mut out = Vec::new();
    if !log_path.exists() {
        return Ok(out);
    }
    let f = File::open(log_path)?;
    for (idx, line) in BufReader::new(f).lines().enumerate() {
        let line_no = idx + 1;
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let ev: Event = crate::schema::decode_versioned(&line)
            .map_err(|e| ChainVerificationError::from_decode(line_no, e))?;
        out.push(ev);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{hex_encode_public, NewEvent};
    use crate::event_log::EventLog;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_anchor_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("mkdir");
        d
    }

    fn ev(n: u64) -> NewEvent {
        NewEvent {
            event_type: "t".into(),
            context: crate::event::EventContext::for_mission("m"),
            payload: serde_json::json!({ "n": n }),
            timestamp_ms: 1000 + n,
            clock_source: ClockSource::Virtual,
        }
    }

    fn build_chain(d: &Path, n: u32) -> PathBuf {
        let log_path = d.join("events.jsonl");
        let mut log = EventLog::open(&log_path).expect("log");
        for i in 1..=n {
            log.append(ev(i as u64)).expect("append");
        }
        log_path
    }

    #[test]
    fn anchor_written_and_self_verifies() {
        let d = dir("anchor_ok");
        let mut store = AnchorStore::open(d.join("anchors.jsonl")).expect("store");
        let mut a = Anchor {
            chain_id: "c1".into(),
            up_to_seq: 5,
            tail_event_hash: "abc".repeat(21), // 63 chars, arbitrary for this test
            created_at_ms: 1234,
            clock_source: ClockSource::Virtual,
            anchor_hash: String::new(),
        };
        store.write_anchor(&mut a).expect("write");
        assert!(a.anchor_hash != String::new());
        let loaded = store.read_all().expect("read");
        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].verify_self());

        // Tamper with the anchor's fields → self-verification fails.
        let mut forged = loaded[0].clone();
        forged.up_to_seq = 99;
        assert!(!forged.verify_self());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn clean_chain_verifies_against_its_anchor() {
        let d = dir("clean");
        let log_path = build_chain(&d, 6);
        let mut store = AnchorStore::open(d.join("anchors.jsonl")).expect("store");
        let mut log = EventLog::open(&log_path).expect("reopen");
        // Simulate a checkpoint at seq 4: anchor captures the tail then.
        let tail_at_4 = {
            let all = log.read_all().unwrap();
            all[3].event_hash.clone()
        };
        let mut a = Anchor {
            chain_id: "c1".into(),
            up_to_seq: 4,
            tail_event_hash: tail_at_4,
            created_at_ms: 5000,
            clock_source: ClockSource::Virtual,
            anchor_hash: String::new(),
        };
        store.write_anchor(&mut a).expect("anchor");

        // Extend the chain AFTER anchoring (normal operation). Log now holds
        // 6 (pre-anchor) + 2 (post-anchor) = 8 events.
        log.append(ev(7)).expect("append");
        log.append(ev(8)).expect("append");

        let verdict =
            verify_chain_on_load(&log_path, &store, "c1").expect("verify runs");
        match verdict {
            // verify_chain_on_load walks the FULL live chain from genesis
            // (module doc: "no shortcuts") and reports how far integrity is
            // proven, which is the log's actual length — 8, not the anchor's
            // up_to_seq (4) or the pre-extension count (6).
            ChainVerdict::Verified { verified_through } => assert_eq!(verified_through, 8),
            other => panic!("expected Verified, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn deleted_event_is_detected_via_anchor_ahead_of_log() {
        let d = dir("truncation");
        let log_path = build_chain(&d, 4);
        let mut store = AnchorStore::open(d.join("anchors.jsonl")).expect("store");
        let mut a = Anchor {
            chain_id: "c1".into(),
            up_to_seq: 4,
            tail_event_hash: "x".into(),
            created_at_ms: 1,
            clock_source: ClockSource::Virtual,
            anchor_hash: String::new(),
        };
        store.write_anchor(&mut a).expect("anchor");

        // Attacker deletes the last event line.
        let content = std::fs::read_to_string(&log_path).unwrap();
        let truncated: String = content.lines().take(3).collect::<Vec<_>>().join("\n") + "\n";
        std::fs::write(&log_path, truncated).unwrap();

        let verdict = verify_chain_on_load(&log_path, &store, "c1").expect("runs");
        assert!(matches!(
            verdict,
            ChainVerdict::Failed {
                reason: ChainVerificationError::AnchorAheadOfLog { .. }
            }
        ));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn mutated_historical_event_fails_closed() {
        let d = dir("mutated");
        let log_path = build_chain(&d, 5);
        let mut store = AnchorStore::open(d.join("anchors.jsonl")).expect("store");
        let mut a = Anchor {
            chain_id: "c1".into(),
            up_to_seq: 5,
            tail_event_hash: "y".into(),
            created_at_ms: 1,
            clock_source: ClockSource::Virtual,
            anchor_hash: String::new(),
        };
        store.write_anchor(&mut a).expect("anchor");

        // Mutate event 2's payload without fixing hashes (realistic tamper).
        // Go through schema-versioned encode/decode so the tampered line
        // keeps its schema_version key, exactly as a real in-place edit
        // would — not reconstructed from a bare, version-unaware `Event`.
        let content = std::fs::read_to_string(&log_path).unwrap();
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        let mut e2: Event = crate::schema::decode_versioned(&lines[1]).unwrap();
        e2.payload = serde_json::json!({ "n": 42 });
        lines[1] = crate::schema::encode_versioned(&e2).unwrap();
        std::fs::write(&log_path, lines.join("\n") + "\n").unwrap();

        let verdict = verify_chain_on_load(&log_path, &store, "c1").expect("runs");
        assert!(
            matches!(verdict, ChainVerdict::Failed { .. }),
            "mutation MUST fail verification"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn wrong_chain_id_anchor_is_rejected() {
        let d = dir("wrongchain");
        let log_path = build_chain(&d, 2);
        let store = AnchorStore::open(d.join("anchors.jsonl")).expect("store");
        let mut a = Anchor {
            chain_id: "OTHER".into(),
            up_to_seq: 2,
            tail_event_hash: "z".into(),
            created_at_ms: 1,
            clock_source: ClockSource::Virtual,
            anchor_hash: String::new(),
        };
        let mut store_w = store;
        store_w.write_anchor(&mut a).expect("anchor");
        let verdict = verify_chain_on_load(&log_path, &store_w, "c1").expect("runs");
        assert!(matches!(verdict, ChainVerdict::Failed { .. }));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn hex_helper_matches_known_vector() {
        assert_eq!(
            hex_encode_public(&[0xde, 0xad, 0xbe, 0xef]),
            "deadbeef"
        );
    }
}
