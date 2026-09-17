//! Event model — the canonical event shape (AC-04 §2 / Blueprint §4.9).
//!
//! An event is an immutable record. `event_hash` is computed over the event's
//! own content PLUS `previous_event_hash`, which is what makes the log a
//! chain: mutating any historical event changes its hash and breaks every
//! successor link.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Canonical hierarchy threading (Phase Manifest P1-W09):
/// mission → step → attempt → event → provider_request → tool_call.
///
/// `EventContext` IS `correlation::CorrelationContext` — the event log does
/// not define its own identifier scheme; it reuses the one canonical type
/// (`correlation.rs`) that every hierarchy-aware module threads through.
/// Construct via `CorrelationContext::for_mission(..)` and the `with_*`
/// builder methods, which enforce that a child level is never set without
/// its parent (fail-closed; see `correlation::CorrelationError`).
pub type EventContext = crate::correlation::CorrelationContext;

/// A single append-only event record.
///
/// Field set per AC-04 §2: sequence number, event type, context, JSON payload,
/// timestamp + clock_source, previous_event_hash, event_hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Monotonic sequence number (allocator formalized in P1-W08; here it is
    /// assigned by the log itself).
    pub seq: u64,
    /// Event type name (e.g. "attempt_status_transition").
    pub event_type: String,
    /// Canonical hierarchy context.
    pub context: EventContext,
    /// Arbitrary structured payload; never contains credential material
    /// (Invariant 12 — enforcement at serialization paths lands with P1-W10
    /// schema work; the type here is payload-opaque by design).
    pub payload: serde_json::Value,
    /// Timestamp as recorded by the injected Clock abstraction (AC-12).
    /// The core never reads the system clock directly; the caller supplies it.
    pub timestamp_ms: u64,
    /// Which clock produced timestamp_ms (real | virtual) per Verification
    /// Gates §7 / AC-12 §3.
    pub clock_source: ClockSource,
    /// Hash of the immediately preceding event ("genesis" = all-zeros 64-hex).
    pub previous_event_hash: String,
    /// SHA-256 over this event's own content + previous_event_hash.
    pub event_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClockSource {
    Real,
    Virtual,
}

/// Hash of the genesis link (no predecessor).
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

impl Event {
    /// Recompute what this event's hash SHOULD be, from its own fields.
    ///
    /// Chaining rule (AC-04 §2): hash covers every semantic field of the event
    /// INCLUDING previous_event_hash, serialized canonically via serde_json so
    /// field order is deterministic.
    pub fn computed_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.seq.to_le_bytes());
        hasher.update(self.event_type.as_bytes());
        hasher.update(serde_json::to_vec(&self.context).expect("EventContext serializes"));
        hasher.update(serde_json::to_vec(&self.payload).expect("payload serializes"));
        hasher.update(self.timestamp_ms.to_le_bytes());
        hasher.update(self.clock_source.as_str().as_bytes());
        hasher.update(self.previous_event_hash.as_bytes());
        let digest = hasher.finalize();
        hex_encode(&digest)
    }

    /// Verify this single event's self-consistency:
    /// stored event_hash must equal the recomputed hash.
    pub fn verify_self(&self) -> bool {
        self.event_hash == self.computed_hash()
    }

    /// Verify the LINK to a predecessor: this event must claim the exact
    /// previous_event_hash it was chained onto.
    pub fn verify_link_to(&self, previous: &Event) -> bool {
        self.previous_event_hash == previous.event_hash && self.verify_self()
    }
}

impl ClockSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClockSource::Real => "real",
            ClockSource::Virtual => "virtual",
        }
    }
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Public hex helper shared by sibling modules (anchors reuse the same
/// encoding discipline as event hashes).
pub fn hex_encode_public(bytes: &[u8]) -> String {
    hex_encode(bytes)
}

/// Builder-side input: everything needed to construct a chained event.
#[derive(Debug, Clone)]
pub struct NewEvent {
    pub event_type: String,
    pub context: EventContext,
    pub payload: serde_json::Value,
    pub timestamp_ms: u64,
    pub clock_source: ClockSource,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> EventContext {
        EventContext::for_mission("m-1").with_step("s-1")
    }

    fn sample(prev: &str, seq: u64) -> Event {
        let new_ev = NewEvent {
            event_type: "test_event".into(),
            context: ctx(),
            payload: serde_json::json!({ "k": seq }),
            timestamp_ms: 1000 + seq,
            clock_source: ClockSource::Virtual,
        };
        Event {
            seq,
            event_type: new_ev.event_type,
            context: new_ev.context,
            payload: new_ev.payload,
            timestamp_ms: new_ev.timestamp_ms,
            clock_source: new_ev.clock_source,
            previous_event_hash: prev.to_string(),
            event_hash: String::new(),
        }
    }

    #[test]
    fn hash_is_deterministic_and_content_sensitive() {
        let e1 = sample(GENESIS_HASH, 1);
        let mut e2 = sample(GENESIS_HASH, 1);
        e2.event_hash = e1.computed_hash();
        assert_eq!(e1.computed_hash(), e2.event_hash);

        // Any semantic mutation changes the hash.
        let mut mutated = sample(GENESIS_HASH, 1);
        mutated.payload = serde_json::json!({ "k": 999 });
        assert_ne!(e1.computed_hash(), mutated.computed_hash());
    }

    #[test]
    fn previous_hash_is_part_of_the_hash_input() {
        let a = sample(GENESIS_HASH, 1);
        let chained_from_a = sample(&a.computed_hash(), 2);
        let chained_from_genesis = sample(GENESIS_HASH, 2);
        // Same content except predecessor ⇒ different hashes (this is what
        // makes tampering with ANY ancestor detectable in ALL descendants).
        assert_ne!(
            chained_from_a.computed_hash(),
            chained_from_genesis.computed_hash()
        );
    }

    #[test]
    fn self_verification_detects_mutation() {
        let mut e = sample(GENESIS_HASH, 1);
        e.event_hash = e.computed_hash();
        assert!(e.verify_self());

        e.timestamp_ms += 1; // tamper
        assert!(!e.verify_self());
    }

    #[test]
    fn link_verification_detects_broken_chain() {
        let e1 = sample(GENESIS_HASH, 1);
        let mut e1 = e1;
        e1.event_hash = e1.computed_hash();

        let mut e2 = sample(&e1.event_hash, 2);
        e2.event_hash = e2.computed_hash();
        assert!(e2.verify_link_to(&e1));

        // Forged predecessor claim: e3 claims to follow e1 but follows nothing real.
        let mut e3 = sample(&e1.event_hash, 3);
        e3.event_hash = e3.computed_hash();
        // e3 verifies against ITSELF but NOT as a successor of e2's world:
        assert!(!e3.verify_link_to(&e2));
    }

    fn full_ctx(tool_call: &str) -> EventContext {
        EventContext::for_mission("m-1")
            .with_step("s-1")
            .with_attempt("a-1")
            .unwrap()
            .with_provider_request("pr-1")
            .unwrap()
            .with_tool_call(tool_call)
            .unwrap()
    }

    #[test]
    fn event_context_carries_full_hierarchy_and_hash_is_sensitive_to_it() {
        // P1-W09: an event can carry correlation all the way down to a tool
        // call, and that provenance is part of what the event's hash covers
        // — tampering with which tool call an event belongs to is exactly
        // the kind of mutation the chain must catch.
        let mut e = sample(GENESIS_HASH, 1);
        e.context = full_ctx("tc-1");
        let hash_a = e.computed_hash();

        let mut e2 = sample(GENESIS_HASH, 1);
        e2.context = full_ctx("tc-2");
        let hash_b = e2.computed_hash();

        assert_ne!(hash_a, hash_b, "different tool_call_id must change the hash");

        // But identical context (including full lineage) reproduces the
        // same hash deterministically.
        let mut e3 = sample(GENESIS_HASH, 1);
        e3.context = full_ctx("tc-1");
        assert_eq!(hash_a, e3.computed_hash());
    }

    #[test]
    fn json_round_trip_preserves_fields() {
        let mut e = sample(GENESIS_HASH, 7);
        e.event_hash = e.computed_hash();
        let json = serde_json::to_string(&e).expect("serialize");
        let back: Event = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.computed_hash(), e.event_hash);
        assert_eq!(back.clock_source, ClockSource::Virtual);
    }
}
