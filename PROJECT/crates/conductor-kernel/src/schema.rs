//! Schema versioning for persisted records (P1-W10-T01 / Blueprint §4.8,
//! §4.8.1).
//!
//! Design decision, stated explicitly because it is load-bearing:
//!
//! `schema_version` is a SIBLING of a record's content, written by wrapping
//! the content at the point of serialization — never a field ON `Event` or
//! `Anchor` themselves, and NEVER an input to `Event::computed_hash()` or
//! `Anchor::computed_hash()`. Tamper-evidence (the hash chain) and
//! wire-format versioning (this module) are deliberately independent axes.
//! If `schema_version` participated in the content hash, then the day a v1
//! record is migrated to v2 (Blueprint §4.8.1: "inevitable"), its hash would
//! change even though its semantic content didn't — silently breaking every
//! downstream `previous_event_hash` link that pointed at the old hash. A
//! migration must be able to change JSON SHAPE without ever touching what
//! the chain has already committed to. `Versioned<T>` below is what keeps
//! those two concerns from ever being able to collide.
//!
//! Scope per the P1-W10-T01 contract: every currently-persisted record type
//! is audited and covered (not just one struct) — see the callers of
//! `encode_versioned`/`decode_versioned` in `event_log.rs`, `chain_anchor.rs`,
//! and `persist.rs::AtomicStore` (which in turn covers `outbox.rs`,
//! `idempotency_store.rs`, and `sequence.rs` — all three build on
//! `AtomicStore` and inherit versioning with no changes of their own beyond
//! routing their own direct-read bypass paths through this module too, since
//! those reads skip `AtomicStore::update` entirely).

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// This build's own schema version — what every NEW write is stamped with.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
/// Oldest version this build can still read WITHOUT running a migration
/// first. Equal to `CURRENT_SCHEMA_VERSION` until a v2 migration exists
/// (§4.8.1: "Migrations run one version at a time" — there is nothing to
/// migrate FROM yet, so nothing older than 1 can exist legitimately).
pub const OLDEST_SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SchemaError {
    #[error(
        "record schema_version {found} is newer than this build supports (current: {current}) \
         — this build is older than the data; refusing to guess at an unknown future format"
    )]
    VersionNewerThanSupported { found: u32, current: u32 },
    #[error(
        "record schema_version {found} predates the oldest version this build can read without \
         a migration ({oldest}); no migration step exists yet to bring it forward — refusing to \
         silently reinterpret it"
    )]
    VersionOlderThanSupported { found: u32, oldest: u32 },
}

/// Fail-closed in both directions: a version from the future (this build is
/// behind) and a version too old to read without a migration this build
/// doesn't have. Never silently coerced to "probably fine."
pub fn check_schema_version(found: u32) -> Result<(), SchemaError> {
    if found > CURRENT_SCHEMA_VERSION {
        return Err(SchemaError::VersionNewerThanSupported {
            found,
            current: CURRENT_SCHEMA_VERSION,
        });
    }
    if found < OLDEST_SUPPORTED_SCHEMA_VERSION {
        return Err(SchemaError::VersionOlderThanSupported {
            found,
            oldest: OLDEST_SUPPORTED_SCHEMA_VERSION,
        });
    }
    Ok(())
}

/// One version-to-version transform, per Blueprint §4.8.1's `MigrationStep`
/// sketch. Deliberately a STUB today: `migration_path()` is empty because
/// `CURRENT_SCHEMA_VERSION == OLDEST_SUPPORTED_SCHEMA_VERSION == 1` — there
/// is no earlier version to migrate from yet. The shape exists now so the
/// day a v2 is needed, the step is additive (implement + register one
/// `MigrationStep`, one version at a time, never v1 -> v3 directly) rather
/// than retrofitted under pressure.
pub struct MigrationStep {
    pub from: u32,
    pub to: u32,
    pub migrate: fn(serde_json::Value) -> Result<serde_json::Value, SchemaError>,
}

/// Ordered, one-version-at-a-time migration registry. Empty until a second
/// schema version exists.
pub fn migration_path() -> Vec<MigrationStep> {
    Vec::new()
}

/// Wraps a record's content with a `schema_version` sibling field at
/// serialization time. `#[serde(flatten)]` keeps the on-disk shape flat —
/// `{"schema_version":1,"seq":1,"event_type":"...",...}` — rather than
/// nesting content under a `content` key, so existing JSONL-per-line and
/// whole-file-JSON consumers stay structurally familiar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Versioned<T> {
    pub schema_version: u32,
    #[serde(flatten)]
    pub content: T,
}

/// Borrowing counterpart used only for encoding, so callers don't need to
/// clone their content just to stamp a version on it.
#[derive(Serialize)]
struct VersionedRef<'a, T> {
    schema_version: u32,
    #[serde(flatten)]
    content: &'a T,
}

#[derive(Debug, thiserror::Error)]
pub enum VersionedDecodeError {
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Schema(#[from] SchemaError),
}

/// Serialize `content` to a compact JSON string wrapped in the current
/// schema version. Used for JSONL-per-line stores (event log, anchors).
pub fn encode_versioned<T: Serialize>(content: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(&VersionedRef {
        schema_version: CURRENT_SCHEMA_VERSION,
        content,
    })
}

/// Serialize `content` to pretty-printed JSON bytes wrapped in the current
/// schema version. Used for whole-file JSON stores (`persist::AtomicStore`).
pub fn encode_versioned_bytes<T: Serialize>(content: &T) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec_pretty(&VersionedRef {
        schema_version: CURRENT_SCHEMA_VERSION,
        content,
    })
}

/// Parse a schema-versioned JSON string, checking the version BEFORE
/// handing the content back. This is the one function every read path for
/// persisted data must go through — including bypass paths that don't go
/// through a generic store (e.g. `SequenceAllocator::current()`,
/// `IdempotencyStore::read_state()`) — or the version check has a hole.
pub fn decode_versioned<T: DeserializeOwned>(s: &str) -> Result<T, VersionedDecodeError> {
    let v: Versioned<T> = serde_json::from_str(s)?;
    check_schema_version(v.schema_version)?;
    Ok(v.content)
}

/// Byte-slice counterpart of `decode_versioned`, for whole-file stores that
/// read raw bytes off disk rather than a line of text.
pub fn decode_versioned_bytes<T: DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, VersionedDecodeError> {
    let v: Versioned<T> = serde_json::from_slice(bytes)?;
    check_schema_version(v.schema_version)?;
    Ok(v.content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Sample {
        a: u32,
        b: String,
    }

    #[test]
    fn encode_wraps_flat_with_current_version() {
        let s = Sample { a: 1, b: "x".into() };
        let json = encode_versioned(&s).expect("encode");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = value.as_object().expect("object");
        assert_eq!(obj.get("schema_version").unwrap(), &serde_json::json!(CURRENT_SCHEMA_VERSION));
        assert_eq!(obj.get("a").unwrap(), &serde_json::json!(1));
        assert_eq!(obj.get("b").unwrap(), &serde_json::json!("x"));
        assert!(
            obj.get("content").is_none(),
            "flatten must keep fields at the top level, not nested under 'content'"
        );
    }

    #[test]
    fn round_trip_preserves_content_exactly() {
        let s = Sample { a: 42, b: "hello".into() };
        let json = encode_versioned(&s).expect("encode");
        let back: Sample = decode_versioned(&json).expect("decode");
        assert_eq!(back, s);
    }

    #[test]
    fn version_from_the_future_is_rejected_fail_closed() {
        let json = r#"{"schema_version":999,"a":1,"b":"x"}"#;
        let err = decode_versioned::<Sample>(json).unwrap_err();
        assert!(matches!(
            err,
            VersionedDecodeError::Schema(SchemaError::VersionNewerThanSupported { found: 999, .. })
        ));
    }

    #[test]
    fn version_older_than_supported_is_rejected_fail_closed() {
        let json = r#"{"schema_version":0,"a":1,"b":"x"}"#;
        let err = decode_versioned::<Sample>(json).unwrap_err();
        assert!(matches!(
            err,
            VersionedDecodeError::Schema(SchemaError::VersionOlderThanSupported { found: 0, .. })
        ));
    }

    #[test]
    fn current_version_is_accepted() {
        assert!(check_schema_version(CURRENT_SCHEMA_VERSION).is_ok());
    }

    #[test]
    fn malformed_json_is_a_serde_error_not_a_schema_error() {
        // Distinguishing the two matters: "corrupt bytes" and "valid JSON,
        // unsupported version" call for different remediation, so they must
        // not collapse into one error variant.
        let err = decode_versioned::<Sample>("not json at all").unwrap_err();
        assert!(matches!(err, VersionedDecodeError::Serde(_)));
    }

    #[test]
    fn migration_path_is_empty_until_a_second_version_exists() {
        assert!(migration_path().is_empty());
        assert_eq!(CURRENT_SCHEMA_VERSION, OLDEST_SUPPORTED_SCHEMA_VERSION);
    }

    // -- VG-P1-W10-T01 proof 1: "every persisted record carries
    // schema_version (schema scan)" -- proven against every REAL store
    // type's REAL write path, not just the generic wrapper above. Each
    // store is exercised through its normal public API, then the raw bytes
    // it wrote are inspected directly (bypassing that store's own read
    // methods, so this can't pass merely because reading and writing agree
    // with each other while both skip versioning).

    fn scan_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ck_schema_scan_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("mkdir");
        d
    }

    fn assert_every_line_has_schema_version(path: &std::path::Path) {
        let content = std::fs::read_to_string(path).expect("read");
        let mut lines_checked = 0;
        for line in content.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(line).expect("valid json line");
            assert_eq!(
                v.get("schema_version"),
                Some(&serde_json::json!(CURRENT_SCHEMA_VERSION)),
                "line missing/wrong schema_version: {line}"
            );
            lines_checked += 1;
        }
        assert!(lines_checked > 0, "scan found no lines to check — test is vacuous");
    }

    fn assert_whole_file_has_schema_version(path: &std::path::Path) {
        let bytes = std::fs::read(path).expect("read");
        let v: serde_json::Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(
            v.get("schema_version"),
            Some(&serde_json::json!(CURRENT_SCHEMA_VERSION)),
            "store file missing/wrong schema_version: {v}"
        );
    }

    #[test]
    fn schema_scan_event_log_lines_carry_schema_version() {
        use crate::event::{ClockSource, EventContext, NewEvent};
        use crate::event_log::EventLog;

        let d = scan_dir("event_log");
        let path = d.join("events.jsonl");
        let mut log = EventLog::open(&path).expect("open");
        log.append(NewEvent {
            event_type: "t".into(),
            context: EventContext::for_mission("m-1"),
            payload: serde_json::json!({"x": 1}),
            timestamp_ms: 1,
            clock_source: ClockSource::Virtual,
        })
        .expect("append");
        assert_every_line_has_schema_version(&path);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn schema_scan_anchor_lines_carry_schema_version() {
        use crate::chain_anchor::{Anchor, AnchorStore};
        use crate::event::ClockSource;

        let d = scan_dir("anchor");
        let path = d.join("anchors.jsonl");
        let mut store = AnchorStore::open(&path).expect("open");
        let mut a = Anchor {
            chain_id: "c1".into(),
            up_to_seq: 1,
            tail_event_hash: "0".repeat(64),
            created_at_ms: 1,
            clock_source: ClockSource::Virtual,
            anchor_hash: String::new(),
        };
        store.write_anchor(&mut a).expect("write");
        assert_every_line_has_schema_version(&path);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn schema_scan_outbox_file_carries_schema_version() {
        use crate::outbox::Outbox;
        let d = scan_dir("outbox");
        let path = d.join("outbox.json");
        let ob = Outbox::new(&path);
        ob.begin_intent("i1", "k", &["a"]).expect("begin");
        assert_whole_file_has_schema_version(&path);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn schema_scan_idempotency_store_file_carries_schema_version() {
        use crate::idempotency_store::IdempotencyStore;
        let d = scan_dir("idempotency");
        let path = d.join("idempotency.json");
        let store = IdempotencyStore::new(&path);
        store.record_applied("k1", None).expect("record");
        assert_whole_file_has_schema_version(&path);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn schema_scan_sequence_allocator_file_carries_schema_version() {
        use crate::sequence::SequenceAllocator;
        let d = scan_dir("sequence");
        let path = d.join("seq.json");
        let alloc = SequenceAllocator::new(&path);
        alloc.next().expect("next");
        assert_whole_file_has_schema_version(&path);
        let _ = std::fs::remove_dir_all(&d);
    }

    // -- VG-P1-W10-T01 proof 2: "unknown version fails closed (rejection
    // test)" -- proven against real store read APIs, not just
    // decode_versioned() in isolation: a file bearing a from-the-future
    // schema_version must make the store's own public read/open method
    // fail, not silently succeed with garbage or defaults.

    #[test]
    fn event_log_open_rejects_unsupported_schema_version_fail_closed() {
        use crate::event_log::{EventLog, EventLogError};
        let d = scan_dir("reject_eventlog");
        let path = d.join("events.jsonl");
        std::fs::write(
            &path,
            format!(
                "{{\"schema_version\":999,\"seq\":1,\"event_type\":\"t\",\"context\":{{\"mission_id\":\"m\"}},\"payload\":{{}},\"timestamp_ms\":1,\"clock_source\":\"virtual\",\"previous_event_hash\":\"{}\",\"event_hash\":\"x\"}}\n",
                "0".repeat(66)
            ),
        )
        .expect("write future-versioned line");

        let err = EventLog::open(&path).unwrap_err();
        assert!(
            matches!(err, EventLogError::UnsupportedSchemaVersion { .. }),
            "opening a log with a from-the-future schema_version must fail closed, got {err:?}"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn atomic_store_update_rejects_unsupported_schema_version_fail_closed() {
        use crate::persist::{AtomicStore, PersistError};
        let d = scan_dir("reject_atomicstore");
        let path = d.join("store.json");
        std::fs::write(&path, r#"{"schema_version":999,"last_issued":5}"#).expect("seed future");

        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        struct S {
            last_issued: u64,
        }
        let store = AtomicStore::new(&path);
        let err = store.update(|c: Option<S>| c.unwrap_or_default()).unwrap_err();
        assert!(
            matches!(err, PersistError::Schema(SchemaError::VersionNewerThanSupported { found: 999, .. })),
            "AtomicStore::update over a from-the-future schema_version must fail closed, got {err:?}"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn sequence_allocator_current_rejects_unsupported_schema_version() {
        // Specifically targets the bypass-path gap this task exists to
        // close: current() does not go through AtomicStore::update, so it
        // needs its own enforcement, not a free ride from update()'s.
        use crate::sequence::SequenceAllocator;
        let d = scan_dir("reject_seq_bypass");
        let path = d.join("seq.json");
        std::fs::write(&path, r#"{"schema_version":999,"last_issued":5}"#).expect("seed future");
        let alloc = SequenceAllocator::new(&path);
        assert!(
            alloc.current().is_err(),
            "current() is a bypass read path (not through AtomicStore::update) and must still fail closed on an unsupported version"
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}
