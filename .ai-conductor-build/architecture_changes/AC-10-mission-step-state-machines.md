# Architecture Contract AC-10 — Mission & Step State Machines
# Task: P0-W10-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-10
task_id: P0-W10-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §4.2.1 (Mission and Step State Machines)"
  - "Blueprint §17 (single authoritative executor), §28.13 (recovery budget)"
  - "Blueprint §8 / AC-08 (mission-level acceptance evaluation)"
  - "Phase Manifest P0-C10"
```

## 1. MissionStatus (required)

```rust
enum MissionStatus {
    Draft,       // contract being defined, nothing executing yet
    Planning,    // decomposing into Steps
    Running,     // at least one Step is Running
    Paused,      // person-initiated pause; resumable
    Blocked,     // system-initiated halt — Conflict, budget exhaustion, escalation
    Succeeded,   // every required Step Succeeded AND the Mission Acceptance Contract passed as a whole
    Failed,      // could not complete within Recovery Budget (§28.13)
    Cancelled,   // person-initiated termination
}
```

## 2. StepStatus (required)

```rust
enum StepStatus {
    Pending,
    Running,
    Blocked,     // waiting on a Mission-level pause/block, not a failure of its own
    Succeeded,
    Failed,
    Skipped,     // explicitly not required for this Mission's contract
}
```

## 3. Transition tables (required)

```text
Mission: Draft → Planning → Running → { Succeeded | Failed | Cancelled }
         Running ⇄ Paused                          (person-initiated, reversible)
         Running → Blocked → { Running | Failed | Cancelled }    (system-initiated)

Step:    Pending → Running → { Succeeded | Failed | Skipped }
         Running → Blocked → Running               (mirrors an owning Mission pause/block)
```

Blocked-entry sources (system-initiated): Conflict per AC-01/AC-09 (Invariant 11),
recovery-budget exhaustion (→ Mission Failed when exhausted, §28.13), human escalation
(§28.19), repository-missing fail-closed (Blueprint §5 table).

## 4. Transition authority and independence from Attempt

- Only the single authoritative executor (§17) may mutate `MissionStatus`/`StepStatus`;
  every transition is itself an event (AC-04 chain).
- These are **independent lifecycle machines** (P0-C10): they are defined by their own
  enums and tables above, not derived from Attempt's. A Step AGGREGATES attempts
  (its Attempt lifecycle stays governed by AC-01); a Mission's Succeeded is NOT implied
  by its last Step's Attempt succeeding.
- Mission-level acceptance re-evaluation: `MissionStatus::Succeeded` requires every
  blocking criterion in the Mission Acceptance Contract (AC-08) to have passed at the
  Mission level — some criteria (e.g. diff-scope "only expected files changed") are
  naturally mission-scoped rather than per-step.
- Skipped exists ONLY by explicit contract declaration — never inferred.

## 5. Non-goals

Attempt lifecycle (AC-01), model-checking method selection (§26 note), UI rendering,
planning/decomposition algorithms.

## 6. Invariants preserved

1, 2, 5, 8. None weakened.
