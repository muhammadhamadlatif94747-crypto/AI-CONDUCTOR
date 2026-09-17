//! Local idempotency-key store: "have I already applied this operation?"
//! (P1-W07 / Blueprint §10.5, §20; AC-11 §4).
//!
//! AC-11 §4 draws a hard line between this module and `idempotency`:
//!
//!   - This store answers ONE narrow, local question: did *this process*
//!     already record having applied a given idempotency key? It knows
//!     nothing about providers, networks, or dedup guarantees.
//!   - It carries NO `IdempotencySupport` field, accepts none as input,
//!     and returns none as output. `IdempotencyRecord` below has exactly
//!     two fields — `key` and `outcome` — and adding a classification
//!     field to it would be exactly the "store upgrades classification"
//!     failure AC-11 forbids. `tests::record_type_carries_no_classification_field`
//!     pins this down structurally.
//!   - Combining a local "already applied" answer with a provider's
//!     `IdempotencySupport` classification to decide whether a retry is
//!     actually safe is `idempotency::safe_to_skip_retry`'s job, not
//!     this module's.
//!
//! Built on `persist::AtomicStore`: durable, crash-recoverable, and
//! single-writer-serialized within this process, same as `outbox.rs`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::persist::{AtomicStore, PersistError};

/// A local record of one already-applied operation. Deliberately minimal:
/// just the key that identifies it, and an optional opaque outcome the
/// caller may want back on a later "did I already do this" check (e.g. a
/// result id to reuse rather than recomputing). No timestamp (no direct
/// system-clock reads permitted outside the Clock trait — AC-12, arrives
/// P3) and, per AC-11 §4, no idempotency-support classification field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyRecord {
    pub key: String,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
struct StoreState {
    records: BTreeMap<String, IdempotencyRecord>,
}

#[derive(Debug, thiserror::Error)]
pub enum IdempotencyStoreError {
    #[error(transparent)]
    Persist(#[from] PersistError),
    /// A caller tried to record a *different* outcome under a key that
    /// already has one recorded. Fail closed rather than silently
    /// overwrite — a changing outcome under the same key usually means
    /// the caller picked the wrong key, not that the old record is stale.
    #[error("key '{0}' already recorded with a different outcome")]
    ConflictingOutcome(String),
}

/// Local "already applied?" retry-detection store.
pub struct IdempotencyStore {
    store: AtomicStore,
}

impl IdempotencyStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        IdempotencyStore {
            store: AtomicStore::new(path),
        }
    }

    fn read_state(&self) -> Result<StoreState, IdempotencyStoreError> {
        Ok(crate::persist::read_versioned::<StoreState>(self.store.path())?.unwrap_or_default())
    }

    /// The core retry-detection check: has this key already been recorded
    /// as applied? This is a LOCAL answer only (see module docs) — it is
    /// never sufficient on its own to decide a retry is safe.
    pub fn has_already_applied(&self, key: &str) -> Result<bool, IdempotencyStoreError> {
        Ok(self.read_state()?.records.contains_key(key))
    }

    /// Full record lookup, if present.
    pub fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyStoreError> {
        Ok(self.read_state()?.records.get(key).cloned())
    }

    /// Durably record that `key` has now been applied, with an optional
    /// opaque outcome. Idempotent by design: recording the same key with
    /// the same outcome again (e.g. during outbox replay) is a safe
    /// no-op and returns the existing record unchanged. Recording the
    /// same key with a *different* outcome is refused — fail closed
    /// rather than silently overwrite prior bookkeeping.
    pub fn record_applied(
        &self,
        key: &str,
        outcome: Option<String>,
    ) -> Result<IdempotencyRecord, IdempotencyStoreError> {
        use std::cell::RefCell;
        use std::rc::Rc;

        let key_owned = key.to_string();
        let outcome_owned = outcome.clone();
        let conflict: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let conflict_slot = Rc::clone(&conflict);

        let result = self.store.update(move |current: Option<StoreState>| {
            let mut state = current.unwrap_or_default();
            match state.records.get(&key_owned) {
                Some(existing) if existing.outcome != outcome_owned => {
                    *conflict_slot.borrow_mut() = true;
                }
                Some(_) => {
                    // Same key, same outcome already recorded: no-op.
                }
                None => {
                    state.records.insert(
                        key_owned.clone(),
                        IdempotencyRecord {
                            key: key_owned.clone(),
                            outcome: outcome_owned.clone(),
                        },
                    );
                }
            }
            state
        })?;

        if *conflict.borrow() {
            return Err(IdempotencyStoreError::ConflictingOutcome(key.to_string()));
        }
        Ok(result
            .records
            .get(key)
            .cloned()
            .expect("just inserted or already present"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idempotency::{safe_to_skip_retry, IdempotencySupport};
    use std::fs;
    use std::path::PathBuf;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_idem_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    #[test]
    fn fresh_key_has_not_been_applied() {
        let d = dir("fresh");
        let store = IdempotencyStore::new(d.join("idem.json"));
        assert!(!store.has_already_applied("k1").unwrap());
        assert!(store.get("k1").unwrap().is_none());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn retry_detects_already_applied_locally() {
        let d = dir("retry_detect");
        let store = IdempotencyStore::new(d.join("idem.json"));
        store.record_applied("k1", Some("result-abc".to_string())).unwrap();

        // This is the exact scenario the task exists for: a caller about
        // to retry an operation first checks locally.
        assert!(store.has_already_applied("k1").unwrap());
        let record = store.get("k1").unwrap().unwrap();
        assert_eq!(record.outcome.as_deref(), Some("result-abc"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn recording_same_key_same_outcome_again_is_a_safe_no_op() {
        let d = dir("replay_safe");
        let store = IdempotencyStore::new(d.join("idem.json"));
        let first = store.record_applied("k1", Some("out".to_string())).unwrap();
        let second = store.record_applied("k1", Some("out".to_string())).unwrap();
        assert_eq!(first, second);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn recording_same_key_different_outcome_is_refused() {
        let d = dir("conflict");
        let store = IdempotencyStore::new(d.join("idem.json"));
        store.record_applied("k1", Some("out-a".to_string())).unwrap();
        let err = store
            .record_applied("k1", Some("out-b".to_string()))
            .unwrap_err();
        assert!(matches!(err, IdempotencyStoreError::ConflictingOutcome(ref k) if k == "k1"));
        // Original record must survive the refused conflicting write.
        let record = store.get("k1").unwrap().unwrap();
        assert_eq!(record.outcome.as_deref(), Some("out-a"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn different_keys_are_tracked_independently() {
        let d = dir("multi");
        let store = IdempotencyStore::new(d.join("idem.json"));
        store.record_applied("k1", None).unwrap();
        assert!(store.has_already_applied("k1").unwrap());
        assert!(!store.has_already_applied("k2").unwrap());
        let _ = fs::remove_dir_all(&d);
    }

    /// Crash/restart recoverability: a fresh `IdempotencyStore` handle
    /// over the same path must see everything a prior handle recorded —
    /// otherwise a restart would forget "already applied" and could
    /// cause a genuine double-execution, exactly what this store exists
    /// to prevent.
    #[test]
    fn recorded_keys_survive_a_restart() {
        let d = dir("restart");
        let path = d.join("idem.json");
        {
            let store = IdempotencyStore::new(&path);
            store.record_applied("k1", Some("done".to_string())).unwrap();
        }
        let restarted = IdempotencyStore::new(&path);
        assert!(restarted.has_already_applied("k1").unwrap());
        assert_eq!(
            restarted.get("k1").unwrap().unwrap().outcome.as_deref(),
            Some("done")
        );
        let _ = fs::remove_dir_all(&d);
    }

    /// Structural pin on AC-11 §4's separation requirement: `IdempotencyRecord`
    /// has exactly the two fields below. If a classification field were
    /// ever added to this type, this destructuring pattern would stop
    /// compiling (missing-field error) and fail the build — a compile-time
    /// trip wire, not just a comment, against the store starting to carry
    /// `IdempotencySupport`.
    #[test]
    fn record_type_carries_no_classification_field() {
        let IdempotencyRecord { key, outcome } = IdempotencyRecord {
            key: "k".to_string(),
            outcome: None,
        };
        assert_eq!(key, "k");
        assert_eq!(outcome, None);
    }

    /// End-to-end AC-11 separation proof, combining both modules exactly
    /// as a real caller would: a key IS recorded as locally applied, but
    /// the provider's classification is Unknown — `safe_to_skip_retry`
    /// must still say no. The local store never upgrades the
    /// classification just because it has a record.
    #[test]
    fn local_store_recording_never_makes_an_unknown_provider_safe_to_skip() {
        let d = dir("e2e_separation");
        let store = IdempotencyStore::new(d.join("idem.json"));
        store.record_applied("k1", Some("maybe-done".to_string())).unwrap();

        let applied_locally = store.has_already_applied("k1").unwrap();
        assert!(applied_locally);
        assert!(
            !safe_to_skip_retry(applied_locally, IdempotencySupport::Unknown),
            "AC-11 separation: a local record must never by itself make \
             an Unknown-classified provider safe to skip retrying"
        );
        assert!(
            safe_to_skip_retry(applied_locally, IdempotencySupport::ServerSideGuaranteed),
            "sanity check: the ONLY combination that is safe to skip"
        );
        let _ = fs::remove_dir_all(&d);
    }
}
