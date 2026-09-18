# Architecture Contract AC-03 — Checkpoint Semantics
# Task: P0-W03-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-03
task_id: P0-W03-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §4.7 (Checkpoint Semantics), §4.8 (Storage), §15 (Crash Recovery)"
  - "Build State Specification §12 (Checkpoint Registry)"
  - "Phase Manifest P0-C03"
```

## 1. What a checkpoint is (required proof #1)

A checkpoint is a **known-good state boundary** — the point a mission can safely be
resumed from after any interruption. It is not merely "a save"; it is a boundary that
has passed validation and is therefore trusted for recovery.

## 2. The four mandatory parts (required proof #1 continued)

Every checkpoint record MUST contain all four parts:

```text
CHECKPOINT
  1. workspace_snapshot_ref   Git commit hash of the workspace state at this boundary
  2. mission_state            full Mission/Step/Attempt status snapshot at this point
                              (statuses + correlation-ID chain per §4.1.1)
  3. verification_evidence    references to the Evidence records (AC-02) that justify
                              calling this boundary known-good
  4. resource_attempt_meta    provider/combo/attempt metadata needed to understand
                              what produced the work
```

Plus envelope fields: `checkpoint_id` (CK-...), `kind`, `schema_version`,
`created_at` (+ `clock_source`), `repository_fingerprint`.
No credentials ever inside a checkpoint record.

## 3. What makes a checkpoint valid

A checkpoint is VALID only when:

```text
V1  all four parts are present and parseable under its schema_version;
V2  workspace_snapshot_ref resolves to an existing, reachable snapshot
    (commit exists / content fingerprint matches);
V3  its verification_evidence passes AC-02 freshness & integrity rules —
    evidence whose state_binding no longer matches is invalid, making the
    checkpoint untrusted for recovery decisions;
V4  it was written through the atomic persistence path (temp file + rename,
    P1-W05) or via an outbox intent marked complete — never a partial write.
```

Invalidation: a checkpoint does not get deleted when superseded; it is retained in
history and marked superseded/invalidated (append-aware rule, Build State Spec §2.3).
Recovery must refuse to resume from an invalid checkpoint and fall back to the most
recent valid one; if none, surface UNKNOWN to the human — never guess.

## 4. How checkpoints are referenced

By `checkpoint_id` from BUILD_STATE (`checkpoints.last_verified`, task_history rows)
and from recovery logic. References carry enough identity to independently verify:
id → kind → commit/fingerprint → verification_report path → created_at. A reference
that cannot be resolved against durable artifacts makes the recorded build state
STALE — reconcile before continuing (Build State Spec §2.1).

## 5. How recovery uses checkpoints (required proof #2)

Startup recovery (Blueprint §15):

```text
load event log (verify hash chain vs latest anchor — AC-04)
→ find last valid checkpoint (validity rules §3)
→ reconstruct state = checkpoint snapshot + replay of subsequent events
→ detect unfinished outbox intents (Invariant 15) and complete/reconcile them
→ determine next safe action; never re-execute accepted side effects blindly
  (Invariant 3); ambiguous outcomes go to reconciliation, not retry
```

Recovery consumes checkpoints; it never rewrites them. A checkpoint is created only
at defined boundaries (phase entry, task acceptance, pre-risk, recovery-safe points),
never mid-mutation of the records it snapshots.

## 6. Non-goals

Event-chain internals (AC-04), outbox mechanism design (P1-W06), UI presentation.

## 7. Invariants preserved

1, 3, 5, 8, 9, 15. None weakened.
