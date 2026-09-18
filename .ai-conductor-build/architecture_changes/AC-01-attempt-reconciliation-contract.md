# Architecture Contract AC-01 — AttemptStatus & ReconciliationOutcome
# Task: P0-W01-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-01
task_id: P0-W01-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §4.2 (Attempt Status — Six States)"
  - "Blueprint §4.3 (Reconciliation Outcome — A Separate Enum)"
  - "Blueprint §4.4–4.5 (Evidence model consuming both)"
  - "Blueprint §7 (Verification Engine sole owner of Succeeded)"
  - "Blueprint Invariants 7, 11, 14"
  - "Phase Manifest P0-C01"
```

## 1. AttemptStatus — lifecycle only

Exactly six states:

```text
Pending | Running | Succeeded | Failed | Cancelled | Unknown
```

`Unknown` exists solely for ambiguous process outcomes (e.g., disconnect or crash
mid-attempt). It is resolved only by reconciliation — never by assumption,
timeout-guessing, or provider claims.

## 2. Structural transition table

Legal edges (all other edges are illegal and MUST be rejected by the State Engine):

```text
Pending → Running
Running → Succeeded | Failed | Unknown | Cancelled
Unknown → Succeeded | Failed
Succeeded, Failed, Cancelled → terminal (a retry creates a NEW Attempt)
```

Notes:
- `Unknown` may NOT transition to Running, Cancelled, or back to Pending.
- Terminal states accept no outgoing edges. A retry is a new Attempt with its own
  attempt_id, linked to the same Step; monotonic attempt numbering applies.
- Every accepted transition emits an event (Invariant 8): logged or it didn't happen.

## 3. ReconciliationOutcome — a separate enum

Exactly six outcomes:

```text
NoChange          filesystem matches pre-attempt baseline exactly
ExpectedChange    actual diff matches the intended operation
UnexpectedChange  something changed, but not what was requested
PartialChange     some but not all of the intended change landed
UserChangeDetected a change attributable to the user, not the attempt
Conflict          attempt change and user change overlap on the same region
```

## 4. Type separation rule (required proof #1)

`AttemptStatus` and `ReconciliationOutcome` are distinct types with distinct storage
fields. Neither may be assigned to, coerced into, or persisted as the other.
A reconciliation result is advisory-grade observation; it is never a lifecycle status.

## 5. Transition authorization (required proof #2 — Invariant 14)

Structural legality and authorization are separate concerns:

| Edge | Authorized by |
|---|---|
| Pending → Running | authoritative executor |
| Running → Unknown | authoritative executor, only on classified ambiguous process outcome |
| Running → Cancelled | cancellation path (detail recorded in CancellationOutcome per P1-W02) |
| Running → Failed | authoritative executor on classified failure |
| Unknown → Failed | authoritative executor applying a reconciliation classification below |
| Running → Succeeded | **Verification Engine ONLY** |
| Unknown → Succeeded | **Verification Engine ONLY**, after reconciliation produced `ExpectedChange` |

No executor, provider, Aider adapter, UI action, model response, or reconciliation
component may construct a `Succeeded` transition. (Direct negative test reserved to
P2-W04 / P4-W04; this contract reserves the authority now.)

Clarifying note (per Blueprint §4.3's own correction of §4.2's parenthetical):
"(only via reconciliation)" means the *decision input* comes from reconciliation;
the *write* of `Succeeded` still belongs exclusively to the Verification Engine.

## 6. Reconciliation outcome → consequence mapping (required proof #3)

```text
NoChange           → apply Failed (nothing happened; clean retry is safe)
ExpectedChange     → hand to Verification Engine; attempt holds current status until
                     verification decides (pass ⇒ Succeeded; fail ⇒ Failed)
UnexpectedChange   → apply Failed + flag targeted repair (never full redo)
PartialChange      → apply Failed + flag targeted repair
UserChangeDetected → apply Failed; the user's change is untouched; no action on it
Conflict           → apply Failed; Mission becomes Blocked; person chooses among
                     [Keep user version] [Keep AI version] [Merge] [Review diff]
```

Conflict is NEVER resolved automatically (Invariants 7, 11). All status writes above
flow through the single authoritative executor path so each emits its event.

## 7. Non-goals

Mission/Step state machines (AC-10), CancellationOutcome internals (P1-W02),
reconciliation engine mechanics (P2), verification pipeline mechanics (P4).

## 8. Invariants preserved

1, 2, 3, 4, 5, 6, 7, 8, 11, 14. None weakened.
