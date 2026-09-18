# Architecture Contract AC-04 — Tamper-Evident Event Chain
# Task: P0-W04-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-04
task_id: P0-W04-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §4.9 Event Log as Hash Chain (incl. anchors & threat model)"
  - "Blueprint §15 Crash Recovery (chain verification on load)"
  - "Build Protocol §13 (event chain verification)"
  - "Phase Manifest P0-C04"
```

## 1. Purpose (required proof #3 — threat model)

The event log makes every state transition observable (Invariant 8). To make that
guarantee meaningful, the log must be **tamper-evident** against accidental and
software-level corruption. The protection scope, stated exactly per Blueprint v2.4:

```text
Accidental corruption (crash mid-write, disk error, bug mutating a past event)
    → PROTECTED: hash chain + anchor reliably detect this.
Compromised dependency/process silently editing history at runtime
    → PARTIALLY protected: detectable iff the anchor's storage location is genuinely
      separate and checked on load.
User/administrator with full filesystem access intentionally rewriting both
log and anchor
    → NOT protected. This is tamper-evidence, not forensic-grade security against
      whoever controls the machine.
```

No artifact, doc, or UI may overstate this guarantee.

## 2. Event record shape (required proof #1)

Every event in the append-only log carries:

```json
{
  "event_id": "evt-...",
  "sequence_number": <monotonic integer>,
  "previous_event_hash": "sha256:...",
  "event_hash": "sha256:...",
  "type": "<event type>",
  "payload": { "...": "..." }
}
```

- `event_hash` is computed over the event's own content plus the previous event's
  `event_hash` (hash chain).
- The log is append-only: no update, no delete, no rewrite. Corrections happen by
  appending new events (e.g., reconciliation outcomes), never by editing history.

## 3. Anchors (required proof #2)

A chain alone cannot detect truncation of its own head. Therefore:

```json
{
  "anchor_id": "anchor-...",
  "genesis_hash": "sha256:...",
  "event_count": <int>,
  "last_event_hash": "sha256:...",
  "checkpoint_hash": "sha256:..."
}
```

Rules:
- An anchor is persisted to a storage location separate from the event log itself
  (different file, ideally different write path), on a regular cadence — at minimum
  after every checkpoint.
- Diagnostics verify the live chain against the nearest known-good anchor, not only
  against itself. Truncation manifests as a mismatch vs the anchor.
- On load (Blueprint §15): verify chain integrity from genesis/anchor before any
  state reconstruction; a failed verification blocks automatic recovery and raises
  the discrepancy to the human (fail-closed).

## 4. What the chain does NOT do

It does not encrypt, does not authorize transitions (AC-01 governs authority), does
not replace checkpoints (AC-03), and does not protect secrets (there are none in
events). It provides detection + proof of intact history for surviving events.

## 5. Non-goals

Outbox mechanics (P1-W06), event-type taxonomy enumeration (defined incrementally
per-phase with their gates), UI presentation of history.

## 6. Invariants preserved

5, 6, 8, 9, 10. None weakened.
