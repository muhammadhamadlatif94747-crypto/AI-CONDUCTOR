# Architecture Contract AC-08 — Mission Acceptance Contract Format
# Task: P0-W08-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-08
task_id: P0-W08-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §8 (Mission Acceptance Contracts)"
  - "Blueprint §7 (RequirementVerificationClass, pipeline)"
  - "Blueprint §4.5 (Evidence provenance/freshness), §4.3 combination requirements"
  - "AC-02 (evidence model)"
  - "Phase Manifest P0-C08"
```

## 1. Criterion format (required — all four P0-C08 fields)

Decided **before** execution starts; never inferred afterward from model confidence.
Free-text criteria are invalid. Every criterion carries exactly these mandatory parts:

| Part | Rule |
|---|---|
| `criterion` | `id` + concrete `description` of an observable outcome |
| `required verification method` | `verification` object: typed `type`, executable `command`/check declaration, `expected_result`; type must map to a declared RequirementVerificationClass (§2) |
| `required evidence` | `required_evidence`: provenance from AC-02's six-class enum, `max_freshness_seconds`, `execution_id` (default `current_attempt`) |
| `evidence provenance/freshness expectations` | compared per §4.3 combination rule below; stale/foreign evidence can never satisfy a criterion |

Canonical shape (Blueprint §8 example is normative):

```json
{
  "id": "auth-03",
  "description": "Invalid credentials are rejected",
  "verification": {
    "type": "test_command",
    "command": "npm test -- auth.test.ts",
    "expected_result": "exit_code_0"
  },
  "required_evidence": {
    "provenance": "local_test_runner",
    "max_freshness_seconds": 300,
    "execution_id": "current_attempt"
  },
  "blocking": true
}
```

A full Mission Acceptance Contract is a **list of such criteria**, not prose.

## 2. Verification classes

Every criterion's `verification.type` resolves to exactly one class (Blueprint §7):

```text
MachineVerifiable   declared check executes; result authoritative
HumanVerifiable     explicit person confirmation step; never auto-passed
ModelAssisted       advisory input only; NEVER sufficient alone for blocking pass
```

ModelAssisted resolution rule: it may contribute evidence but must resolve into a
MachineVerifiable check or explicit HumanVerifiable confirmation before the
Verification Engine treats it as satisfied (Invariant 7).

## 3. Combination rule (§4.3 binding form)

A criterion may demand a specific conjunction:

```text
provenance = <declared class>
freshness  <= max_freshness_seconds
execution_id = current_attempt
result = pass
```

This kills the "CI badge or old test summary satisfies a fresh-attempt criterion" failure mode.

## 4. Execution semantics

```text
Model says "completed" → Verification Engine runs → each criterion's declared method
actually executes NOW → resulting Evidence record compared against required_evidence
PASS (all blocking criteria met) → Attempt = Succeeded + checkpoint accepted
  (Verification Engine ONLY — Invariant 14)
FAIL → the specific unmet criteria become the next repair request — never a vague "try again"
```

Non-blocking criteria report status without halting acceptance; blocking criteria gate it.
Verification commands execute inside the §7.1 sandbox (AC-07 §4) — network need must be
explicitly declared per criterion and is auditable.

## 5. Consistency with AC-02

Provenance values are the same six-class enum; missing state_binding ⇒ stale by default;
UNKNOWN evidence never satisfies any criterion. Freshness clocks follow the clock_source rules.

## 6. Non-goals

Pipeline stage internals (P4), repair-request generation mechanics, UI rendering of contracts,
sandbox implementation details (owned by AC-07 §4 reference).

## 7. Invariants preserved

2 (unaccepted work never presented as complete), 7 (model advisory), 14 (Succeeded via
Verification Engine only). None weakened.
