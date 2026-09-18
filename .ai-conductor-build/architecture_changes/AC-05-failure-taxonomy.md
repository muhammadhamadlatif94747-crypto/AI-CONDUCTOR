# Architecture Contract AC-05 — Failure Taxonomy & Structured Failure Classification
# Task: P0-W05-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-05
task_id: P0-W05-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §9 (Recovery & Handoff Protocol incl. Structured Failure Classification)"
  - "Blueprint §11 (Failure Taxonomy & Regression Corpus)"
  - "Phase Manifest P0-C05"
```

## 1. Failure taxonomy classes (required proof #1)

Seven classes, exactly as Blueprint §11 defines. Classes are the fine-grained
*underlying detail* recorded on every failure:

| Class | Examples |
|---|---|
| PROCESS | app restart, Aider crash, Conductor crash |
| NETWORK | timeout, disconnect, delayed response |
| PROVIDER | 429, quota exhausted, 5xx, malformed response |
| EXECUTION | partial file edit, tool failure, unexpected exit |
| PERSISTENCE | checkpoint write failure, state write interrupted |
| RECOVERY | retry after partial success, handoff after partial success, resume after restart |
| CONFLICT | user edit overlapping an in-flight attempt, simultaneous external Git operation |

**CONFLICT is an explicit class** (required by P0-C05), not folded into EXECUTION or
NETWORK. A CONFLICT-classified failure always corresponds to `ReconciliationOutcome.Conflict`
(AC-01) and triggers the Invariant 11 halt — automatic recovery is never attempted.

## 2. Structured FailureCategory (required proof #2)

`Failed` alone is too coarse for recovery decisions. Rather than inventing new
lifecycle states (which would re-create the exact AttemptStatus/ReconciliationOutcome
conflation fixed in v2.1), structured failure metadata is attached to every `Failed`
attempt:

```rust
enum FailureCategory {
    FailedClean,          // failed before any change was made — safe, simple retry
    FailedWithChanges,    // failed after partial changes landed — needs targeted repair
    FailedVerification,   // changes landed but didn't pass the Acceptance Contract
    FailedInfrastructure, // the failure was ours (process/network/persistence), not the model's
}
```

Binding rules:
- **FailureCategory is the coarse bucket the Recovery Protocol branches on**;
  taxonomy classes ride along as detail. Both are recorded; neither replaces `AttemptStatus`.
- Exactly one FailureCategory per Failed attempt; the primary class(es) plus
  evidence references accompany it.
- Category assignment is Conductor-attributed from evidence (AC-02 provenance),
  never from a provider/model claim alone. Ambiguous attribution resolves
  conservatively toward the category demanding more caution, with uncertainty surfaced.

## 3. Recovery consequence map

How each outcome/category branches (Blueprint §9). No other automatic actions exist:

```text
Failed/FailedClean           → new Attempt, different (or recovered) combo,
                               expanded handoff package attached
Failed/FailedWithChanges     → targeted repair request describing exactly the gap,
                               not a full redo
Failed/FailedVerification    → repair against the specific Acceptance Contract gaps
Failed/FailedInfrastructure  → fix/recover our side first; retry is not the model's burden
Unknown                      → NEVER auto-retry. Reconciliation runs first, always.
Conflict (any path)          → mission PAUSED. No automatic action. Person chooses.
```

## 4. Boundary with adjacent enums

- `FailureCategory` attaches ONLY to `Failed`. It never creates new `AttemptStatus`
  values and never merges with `ReconciliationOutcome`.
- `CancellationOutcome` (CancelledClean | CancelledWithChanges | CancellationUnknown,
  Blueprint §14) is its own enum for cancelled attempts — not a FailureCategory value.
- `Unknown` outcomes are not failures and receive no FailureCategory; they route to
  reconciliation.

## 5. Regression corpus rule

Every real bug becomes a permanent numbered entry in `.ai-conductor-build/failure_cases/`,
each with its own regression test that runs forever (e.g. F-0001 already recorded).
A class or category with no corpus entry is permitted; a reproduced bug without one is not.

## 6. Non-goals

Reconciliation algorithms (P2-W03), retry/backoff policy internals (P1/P3),
cancellation sequencing mechanics (P4+), UI presentation of failures.

## 7. Invariants preserved

1, 2, 3, 4, 5, 8, 11. None weakened.
