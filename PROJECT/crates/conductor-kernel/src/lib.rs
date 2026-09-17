//! conductor-kernel — the deterministic reliability core of AI Conductor.
//!
//! Phase 1 scope (through P1-W10): the State Engine, the tamper-evident
//! event chain (log + anchors), cancellation-outcome typing, atomic/
//! single-writer persistence, the outbox/intent mechanism, idempotency
//! classification + local retry-detection, a monotonic sequence allocator,
//! the canonical correlation-ID hierarchy, and schema versioning on every
//! persisted record, per AC-01 / AC-04 / AC-10 / AC-11 / Blueprint section 4.2,
//! sections 4.7-4.9, 10.5, 20.
//!
//! Hard rules honored here:
//! - `state.rs` performs NO I/O of any kind — it is the pure decision core
//!   (deterministic; Blueprint section 26). This is a per-module guarantee, not a
//!   crate-wide one: `event_log.rs`, `chain_anchor.rs`, `persist.rs`,
//!   `outbox.rs`, `idempotency_store.rs`, and `sequence.rs` perform file
//!   I/O by design (that IS their job). An earlier version of this comment
//!   claimed "no I/O of any kind in this crate" as a hard rule; that was
//!   only ever true of `state.rs` in isolation (P1-W01, before the other
//!   modules existed) and was corrected here (takeover audit, P1-W04/W05)
//!   rather than left to mislead future readers.
//! - No direct system-clock reads will ever appear here (AC-12; Clock arrives with P3).
//! - `schema_version` (schema.rs) is NEVER an input to `Event::computed_hash()`
//!   or `Anchor::computed_hash()` — wire-format versioning and content
//!   tamper-evidence are independent axes on purpose (see schema.rs docs).

pub mod baseline;
pub mod cancellation;
pub mod chain_anchor;
pub mod chaos;
pub mod clock;
pub mod conflict;
pub mod conformance;
pub mod console;
pub mod correlation;
pub mod event;
pub mod event_log;
pub mod evidence;
pub mod execution_adapter;
pub mod executor_capability_snapshot;
pub mod idempotency;
pub mod idempotency_store;
pub mod mission_acceptance;
pub mod mutation_attribution;
pub mod outbox;
pub mod persist;
pub mod person_decision;
pub mod provider;
pub mod reconciliation;
pub mod schema;
pub mod scenario;
pub mod sequence;
pub mod state;
pub mod verification;
pub mod verification_engine;
pub mod worktree;
