# AI CONDUCTOR
# Build State Specification — v1.0
## Persistent Implementation Memory, Reconstruction, and Agent Continuity

**Authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.4.md`  
**Build procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.0.md`  
**Phase authorization:** `AI_CONDUCTOR_PHASE_MANIFEST_v1.0.md`  
**Task definition:** `AI_CONDUCTOR_TASK_CONTRACTS_v1.0.md`  
**Verification authority:** `AI_CONDUCTOR_VERIFICATION_GATES_v1.0.md`  
**Human authority:** Project owner  
**Primary runtime artifact:** `.ai-conductor-build/BUILD_STATE.json`  
**Status:** Operational state specification

---

# 0. Purpose

`BUILD_STATE.json` is the persistent machine-readable memory of the AI Conductor implementation process.

Its purpose is to let a new or resumed OX Alpha session determine, from durable evidence rather than conversational memory:

- what phase is authorized;
- what task is currently active;
- what work is accepted;
- what work failed or is blocked;
- what checkpoint is trustworthy;
- what tests have actually passed;
- what failures remain known;
- what unknowns remain unresolved;
- what decisions are pending;
- what the exact next authorized action is;
- whether the recorded state agrees with the actual repository.

The core principle is:

> **Build State records what the project has proved, not what an AI agent remembers saying.**

The actual repository and executed evidence remain the highest-trust implementation reality. Build State is the durable coordination memory that makes that reality usable across sessions.

---

# 1. Role in the Control Pack

The complete authority chain is:

```text
MASTER BLUEPRINT v2.4
        ↓
BUILD PROTOCOL v1.0
        ↓
PHASE MANIFEST v1.0
        ↓
TASK CONTRACT
        ↓
VERIFICATION GATE
        ↓
BUILD STATE
        ↓
ACTUAL REPOSITORY + EXECUTED EVIDENCE
```

The layers have different jobs.

| Layer | Answers |
|---|---|
| Master Blueprint | What must AI Conductor be? |
| Build Protocol | How must OX Alpha work? |
| Phase Manifest | What phase is authorized now? |
| Task Contract | What exact bounded change is authorized now? |
| Verification Gate | What proves that change is correct? |
| Build State | What has actually happened, and what is next? |
| Repository + evidence | What is physically and empirically true? |

Build State must never be used to override stronger evidence.

---

# 2. Core Design Principles

## 2.1 Evidence beats memory

If Build State says:

```text
task = ACCEPTED
```

but the latest repository does not contain the accepted checkpoint or the referenced evidence cannot be found, Build State is considered stale or inconsistent.

The agent must reconcile instead of assuming the state is correct.

---

## 2.2 State is durable; chat is transient

Conversation history is useful context.

It is never the authoritative source for:

- task completion,
- phase advancement,
- acceptance,
- checkpoint identity,
- repository state,
- test results.

A fresh OX Alpha session must be able to continue without reconstructing the project from prior chat.

---

## 2.3 State is append-aware

Historical outcomes must not be silently erased.

A task that previously failed and later succeeds should preserve the historical failure record.

A previous accepted gate that becomes invalid should be marked invalidated, not deleted.

---

## 2.4 State is reconstructable

Important Build State fields should have a durable evidence path.

For example:

```text
accepted_task
    ↓
verification_report
    ↓
checkpoint
    ↓
commit / repository identity
```

This means a state record can be independently checked.

---

## 2.5 State is minimal but sufficient

Build State should not become a second copy of the whole repository or full logs.

Store:

- references,
- identifiers,
- statuses,
- summaries,
- evidence links,
- next actions.

Large evidence remains in its proper artifact:

```text
verification_reports/
failure_cases/
checkpoints/
architecture_changes/
dependency_requests/
```

This keeps the state file lightweight.

---

# 3. Canonical File Layout

The implementation workspace should contain:

```text
.ai-conductor-build/
│
├── BUILD_STATE.json
├── STOP_REPORT.md
│
├── phase_manifests/
├── task_contracts/
├── verification_gates/
├── verification_reports/
├── checkpoints/
├── failure_cases/
├── architecture_changes/
├── dependency_requests/
├── handoffs/
├── diagnostics/
└── migrations/
```

The exact runtime layout may evolve only through controlled architecture/protocol change.

---

# 4. State Schema Versioning

Every Build State file must contain:

```json
"schema_version": "1.0"
```

This is separate from:

```json
"blueprint_version": "2.4"
"protocol_version": "1.0"
"phase_manifest_version": "1.0"
```

These version different contracts.

Example:

```json
{
  "schema_version": "1.0",
  "blueprint_version": "2.4",
  "protocol_version": "1.0",
  "phase_manifest_version": "1.0"
}
```

A schema migration must never silently mutate historical state.

---

# 5. Canonical Top-Level Schema

The minimum canonical shape is:

```json
{
  "schema_version": "1.0",

  "authority": {
    "blueprint_version": "2.4",
    "protocol_version": "1.0",
    "phase_manifest_version": "1.0",
    "task_contract_schema_version": "1.0",
    "verification_gate_version": "1.0"
  },

  "project": {},

  "current_phase": {},

  "current_task": {},

  "task_history": {},

  "phase_history": {},

  "checkpoints": {},

  "verification": {},

  "failures": {},

  "unknowns": {},

  "decisions": {},

  "dependencies": {},

  "repository": {},

  "runtime": {},

  "next_action": {},

  "session": {},

  "integrity": {},

  "statistics": {},

  "updated_at": "..."
}
```

Unknown additional fields may be introduced only through schema evolution.

---

# 6. Authority Block

Example:

```json
"authority": {
  "blueprint_version": "2.4",
  "protocol_version": "1.0",
  "phase_manifest_version": "1.0",
  "task_contract_schema_version": "1.0",
  "verification_gate_version": "1.0"
}
```

Purpose:

- detect contract drift;
- prevent an old Build State from being interpreted against a materially different implementation contract;
- allow controlled migration.

---

# 7. Project Identity

The project block identifies the implementation target.

Example:

```json
"project": {
  "project_id": "ac-project-001",
  "name": "AI Conductor",
  "root_path": "...",
  "repository_type": "git",
  "initial_repository_fingerprint": "...",
  "created_at": "...",
  "last_known_root_fingerprint": "..."
}
```

Do not place secrets here.

---

# 8. Current Phase

Example:

```json
"current_phase": {
  "id": "P1",
  "name": "State + Event + Persistence Core",
  "status": "IN_PROGRESS",
  "manifest_version": "1.0",
  "entry_checkpoint": "CK-P0-01",
  "entry_commit": "...",
  "tasks_total": 14,
  "tasks_accepted": 5,
  "tasks_blocked": 0,
  "tasks_failed": 0,
  "phase_gate_id": null,
  "phase_gate_status": "NOT_READY"
}
```

Allowed phase statuses:

```text
NOT_STARTED
IN_PROGRESS
BLOCKED
FAILED
GATE_PENDING
ACCEPTED
CANCELLED
```

---

# 9. Current Task

Example:

```json
"current_task": {
  "id": "P1-W03-T04",
  "contract_version": 1,
  "status": "IN_PROGRESS",
  "started_at": "...",
  "last_activity_at": "...",
  "risk_level": "HIGH",
  "attempt_id": "ATT-00042",
  "verification_gate_id": "VG-P1-W03-T04-01",
  "last_checkpoint": "CK-00041"
}
```

Allowed task statuses:

```text
DRAFT
READY
IN_PROGRESS
IMPLEMENTED
VERIFIED
ACCEPTED
BLOCKED
FAILED
CANCELLED
SUPERSEDED
```

`ACCEPTED` must only be recorded after the required Verification Gate passes.

---

# 10. Task History

Task history stores durable references to previous tasks.

Example:

```json
"task_history": {
  "P1-W03-T01": {
    "status": "ACCEPTED",
    "contract_version": 1,
    "gate_id": "VG-...",
    "verification_report": "verification_reports/...",
    "checkpoint": "CK-...",
    "commit": "...",
    "accepted_at": "..."
  }
}
```

History must retain:

- accepted tasks,
- failed tasks,
- blocked tasks,
- superseded tasks,
- invalidated accepted work.

Do not delete historical outcomes merely to make the state file smaller.

---

# 11. Phase History

Example:

```json
"phase_history": {
  "P0": {
    "status": "ACCEPTED",
    "gate_id": "VG-P0",
    "entry_checkpoint": "CK-00001",
    "exit_checkpoint": "CK-00012",
    "accepted_at": "..."
  }
}
```

A phase history entry must point to the gate and checkpoint that established acceptance.

---

# 12. Checkpoint Registry

Build State should reference checkpoints rather than embedding large checkpoint contents.

Example:

```json
"checkpoints": {
  "last_verified": "CK-P1-00041",
  "history": [
    {
      "id": "CK-P1-00041",
      "task_id": "P1-W03-T04",
      "kind": "task_acceptance",
      "commit": "...",
      "repository_fingerprint": "...",
      "verification_report": "verification_reports/...",
      "created_at": "..."
    }
  ]
}
```

Checkpoint categories may include:

```text
PHASE_ENTRY
TASK_ACCEPTANCE
RECOVERY_SAFE
PRE_RISK
PHASE_EXIT
RELEASE
```

The actual valid checkpoint semantics remain defined by the Blueprint.

---

# 13. Verification Registry

Example:

```json
"verification": {
  "last_gate": {
    "id": "VG-P1-W03-T04-01",
    "status": "PASSED",
    "report": "verification_reports/VG-P1-W03-T04-01.md",
    "commit": "...",
    "executed_at": "..."
  },

  "tests": {
    "passed": 124,
    "failed": 0,
    "blocked": 0,
    "unknown": 0
  }
}
```

Build State should store summary and references.

It should not become the full test output archive.

---

# 14. Failure Registry

Example:

```json
"failures": {
  "open": [
    "F-0007"
  ],

  "recent": [
    {
      "id": "F-0006",
      "classification": "PERSISTENCE",
      "status": "REGRESSION_FIXED",
      "task": "P1-W02-T03",
      "case": "failure_cases/F-0006/"
    }
  ],

  "counts": {
    "open": 1,
    "resolved": 14,
    "critical": 0
  }
}
```

The full failure case belongs in `failure_cases/`.

---

# 15. Unknowns Registry

This is critical.

Build State must preserve uncertainty explicitly.

Example:

```json
"unknowns": [
  {
    "id": "U-0003",
    "description": "Provider structured-output support not live-verified.",
    "scope": "P6",
    "severity": "MEDIUM",
    "blocking": true,
    "evidence": [],
    "next_action": "Run controlled capability probe.",
    "status": "OPEN"
  }
]
```

Allowed statuses:

```text
OPEN
INVESTIGATING
RESOLVED
REJECTED
SUPERSEDED
```

Rules:

- unknowns must not silently disappear;
- blocking unknowns prevent affected gates from passing;
- resolution must point to evidence.

---

# 16. Pending Decisions

Example:

```json
"decisions": {
  "pending": [
    {
      "id": "D-0004",
      "type": "ARCHITECTURE",
      "question": "...",
      "affected_sections": ["Blueprint §..."],
      "blocking": true,
      "record": "architecture_changes/AC-0004.md"
    }
  ]
}
```

Decisions may include:

```text
ARCHITECTURE
DEPENDENCY
SECURITY
SCOPE
ACCEPTANCE
RESOURCE
PRODUCT
```

A pending blocking decision prevents the affected task/phase from advancing.

---

# 17. Dependencies Registry

Record only state and references.

Example:

```json
"dependencies": {
  "active": [
    {
      "name": "serde",
      "version": "...",
      "status": "APPROVED",
      "request": null
    }
  ],
  "pending_requests": [
    "dependency_requests/DR-0002.md"
  ]
}
```

Do not store package metadata that can be obtained reliably from the repository unless it is needed for recovery decisions.

---

# 18. Repository State

Example:

```json
"repository": {
  "branch": "...",
  "worktree_id": "...",
  "head_commit": "...",
  "dirty": false,
  "changed_files_count": 0,
  "repository_fingerprint": "...",
  "last_inspected_at": "...",
  "external_change_detected": false
}
```

The exact repository fingerprint mechanism is implementation-defined by the Blueprint.

The important rule is:

> Build State must be able to detect when the repository no longer resembles the state it last recorded.

---

# 19. Runtime State

Runtime should record enough information to detect interrupted execution without turning Build State into a process log.

Example:

```json
"runtime": {
  "active_process": {
    "type": "aider",
    "pid": 12345,
    "attempt_id": "ATT-00042",
    "status": "UNKNOWN"
  }
}
```

A process ID is not proof that a process is safe to resume.

After restart:

```text
runtime state
+
actual OS observation
+
repository observation
+
attempt evidence
→ reconciliation
```

---

# 20. Next Authorized Action

This is one of the most important fields.

Example:

```json
"next_action": {
  "type": "EXECUTE_TASK",
  "task_id": "P1-W03-T04",
  "reason": "Previous verification failed; contract still active after approved repair.",
  "prerequisites": [
    "re-run preflight"
  ],
  "not_before": null
}
```

Allowed action types:

```text
CREATE_TASK
EXECUTE_TASK
VERIFY_TASK
RECONCILE
WAIT
HANDOFF
REQUEST_HUMAN_DECISION
REQUEST_ARCHITECTURE_CHANGE
REQUEST_DEPENDENCY
RUN_PHASE_GATE
ADVANCE_PHASE
SAFE_STOP
NONE
```

The next action must be **authorized by phase/task state**.

It is not merely a suggestion.

---

# 21. Session State

Example:

```json
"session": {
  "session_id": "SESSION-00017",
  "agent": "OX Alpha",
  "started_at": "...",
  "last_boot_at": "...",
  "last_stop_reason": "CONTEXT_LIMIT",
  "last_stop_report": "STOP_REPORT.md"
}
```

This is operational history.

It does not make an agent's statements authoritative.

---

# 22. Integrity Block

Example:

```json
"integrity": {
  "state_hash": "...",
  "last_event_sequence": 1842,
  "last_event_hash": "...",
  "anchor_reference": "...",
  "state_validated_at": "...",
  "validation_status": "VALID"
}
```

This must integrate with the Blueprint's event-chain and persistence design.

Do not invent a second competing integrity mechanism.

---

# 23. Statistics

Statistics are derived/summary information.

Examples:

```json
"statistics": {
  "tasks_accepted": 23,
  "tasks_failed": 4,
  "tasks_blocked": 1,
  "verification_pass_rate": 0.92,
  "provider_failures": 7,
  "recovery_count": 3,
  "handoff_count": 1
}
```

Statistics must never override authoritative task/gate history.

If a statistic disagrees with historical evidence:

```text
recalculate
```

Do not "fix" history to match statistics.

---

# 24. Timestamps and Clock Discipline

Build State may record timestamps.

But reliability-core semantics must respect the Blueprint's injectable-clock rules.

Where deterministic behavior depends on time, record:

```text
clock_source: real | virtual
```

Do not use an unqualified timestamp to make a state-machine decision where a virtual clock is required.

---

# 25. Build State Authority Rules

The following are authoritative within Build State:

```text
current authorized task
current phase status
pending decisions
blocking unknowns
next authorized action
historical references
```

The following are **derived**:

```text
task counts
statistics
last-known display summaries
```

The following are **not authoritative** if contradicted by stronger evidence:

```text
agent narrative
old chat text
unverified manual notes
stale status fields
```

---

# 26. Reconstruction Rule

If Build State and reality disagree:

```text
BUILD_STATE
    vs
ACTUAL REPOSITORY
    vs
EXECUTED EVIDENCE
```

the agent must not simply overwrite Build State.

It must perform reconstruction.

Order:

```text
1. inspect repository
2. inspect latest commit/checkpoint
3. inspect verification reports
4. inspect event history
5. inspect stop report
6. inspect current task contract
7. reconstruct highest-confidence state
8. record reconciliation
9. only then update BUILD_STATE
```

The reconciliation itself should leave an evidence trail.

---

# 27. Reconstructing a Completed Task

If Build State says:

```text
P1-W03-T04 = ACCEPTED
```

the agent should find:

```text
task contract
+
verification gate
+
verification report
+
checkpoint
+
repository identity
```

If all agree:

```text
ACCEPTED
```

If code is present but verification evidence is missing:

```text
IMPLEMENTED / UNVERIFIED
```

If evidence is stale:

```text
INVALIDATED
```

If repository state is contradictory:

```text
RECONCILIATION REQUIRED
```

---

# 28. Reconstructing an In-Progress Task

If:

```text
current_task = P1-W03-T04
status = IN_PROGRESS
```

the agent must determine:

```text
was code changed?
did tests run?
did verification run?
did the process terminate?
was the outcome unknown?
did the task cross a checkpoint?
```

It must not restart the whole task blindly.

---

# 29. Reconstructing After a Context Limit

The expected process is:

```text
READ BUILD_STATE
     ↓
READ CURRENT TASK
     ↓
READ STOP_REPORT
     ↓
INSPECT REPOSITORY
     ↓
INSPECT LAST CHECKPOINT
     ↓
RUN MINIMUM RELEVANT TESTS
     ↓
RECONCILE
     ↓
RESUME
```

The previous model's memory is optional context only.

---

# 30. Reconstructing After Network Failure

Network failure may mean:

```text
no action occurred
```

or:

```text
action may have occurred
```

depending on where failure happened.

Therefore:

```text
network error
    ↓
classify
    ↓
inspect provider attempt state
    ↓
reconcile external outcome if relevant
    ↓
only then retry/handoff
```

Do not let Build State simply change:

```text
RUNNING → FAILED
```

without understanding whether a side effect occurred.

---

# 31. Reconstructing After OX Alpha Stops Unexpectedly

If the agent stops without a clean STOP_REPORT:

```text
BUILD_STATE
+
repository
+
latest checkpoint
+
event history
+
process state
+
verification reports
```

must be reconciled.

The missing STOP_REPORT is itself recorded as an abnormal condition.

Do not guess the missing final state.

---

# 32. State Update Atomicity

Build State updates must follow the persistence guarantees defined by the Blueprint.

Prefer:

```text
write new state
→ validate
→ atomic replace
```

rather than:

```text
truncate
→ write
```

If the state write is interrupted, a recoverable previous version must remain available according to the project's persistence mechanism.

---

# 33. State History vs Current Projection

Treat Build State conceptually as:

```text
historical evidence/events
        ↓
current state projection
```

The current JSON should be compact and operational.

Historical event and evidence records remain outside it.

If the project uses event replay to reconstruct state, the replayed result must be comparable with the stored projection.

A projection mismatch should produce a reconciliation event.

---

# 34. Preventing Stale Build State

A session must consider Build State stale when:

- repository fingerprint changed unexpectedly;
- active worktree changed;
- accepted checkpoint is missing;
- referenced verification report is missing;
- referenced task contract version differs;
- architecture version changed;
- phase manifest changed;
- a relevant gate became invalid;
- a dependency migration occurred.

Stale state triggers:

```text
RECONCILIATION
```

not automatic continuation.

---

# 35. Build State and Scope

Build State must record the current task's scope reference:

```json
"current_task": {
  "id": "P1-W03-T04",
  "contract_version": 1
}
```

It must not duplicate the entire task contract.

The task contract remains authoritative for the current task's scope.

---

# 36. Build State and Verification

Build State may record:

```json
"verification": {
  "last_gate_id": "...",
  "last_gate_status": "PASSED"
}
```

but the verification report itself remains authoritative for the details.

If a verification report says `FAILED` while Build State says `PASSED`:

```text
RECONCILE
```

Do not change the report to match Build State.

---

# 37. Build State and Checkpoints

Build State should always know:

```text
last_verified_checkpoint
last_safe_recovery_checkpoint
last_phase_entry_checkpoint
```

where applicable.

These are references, not embedded snapshots.

---

# 38. Build State and Failure Memory

A task's status may become:

```text
ACCEPTED
```

while the project still has historical resolved failures.

This is normal.

Do not treat:

```text
failure history ≠ current failure
```

as a contradiction.

Open failures, unresolved critical failures, and historical resolved failures must remain distinguishable.

---

# 39. Build State and Unknowns

Open unknowns are acceptable if they are:

- non-blocking,
- explicitly documented,
- scoped,
- not falsely treated as verified.

Blocking unknowns prevent affected gates from passing.

Example:

```text
Provider capability unknown
→ P6 provider task blocked
→ unrelated deterministic kernel task may still continue
```

Use dependency-aware blocking rather than freezing the entire project unnecessarily.

---

# 40. Build State and Human Decisions

When the system reaches a human decision:

```json
"decisions": {
  "pending": [
    {
      "id": "D-0001",
      "status": "PENDING",
      "blocking": true,
      "options": [
        "..."
      ],
      "recommended": "...",
      "evidence": [
        "..."
      ]
    }
  ]
}
```

After the human decides, retain:

```text
question
options
decision
decision timestamp
decision authority
evidence considered
affected contracts
```

Do not rewrite the question out of history.

---

# 41. Build State and Agent Autonomy

The control system must **not unnecessarily restrict OX Alpha's engineering judgment**.

This is a deliberate principle.

## 41.1 The files constrain authority, not intelligence

Within an authorized Task Contract, OX Alpha may choose:

- a cleaner implementation,
- a safer algorithm,
- a simpler design,
- a better test strategy,
- a more robust error-handling method,
- an implementation detail the Blueprint intentionally leaves open,
- extra internal tests,
- a more efficient local refactor,
- a better debugging method,

provided that doing so:

1. stays within the allowed scope;
2. does not alter an architectural contract;
3. does not weaken invariants;
4. does not create an unapproved dependency;
5. does not introduce security risk;
6. does not change acceptance semantics;
7. preserves or improves verified behavior.

This is called the:

> **Authorized Engineering Discretion Zone.**

---

## 41.2 Do not force a poorer implementation for document conformity

The agent must not be told:

> "Implement exactly the illustrative code shown in a document."

Blueprint examples are contracts only where explicitly declared normative.

The agent should implement the **best correct solution consistent with the normative requirements**.

---

## 41.3 Improvements are welcome within the boundary

If OX Alpha discovers a better implementation that is:

```text
safer
simpler
faster
more maintainable
more testable
more reliable
lighter
```

it should be allowed to use it when no normative rule is violated.

It should record meaningful implementation decisions when they matter to future maintenance.

---

## 41.4 Discovery is not scope expansion

The agent may discover useful ideas while implementing a task.

It may record:

```text
improvement_candidate
future_optimization
architectural_observation
```

without implementing them.

This lets the agent preserve useful intelligence without derailing the current task.

---

## 41.5 Ask only where authority is genuinely required

The goal is not to make OX Alpha ask the human for every decision.

It should proceed independently when:

```text
architecture is clear
scope is clear
risk is controlled
verification is clear
decision does not cross a human boundary
```

It should ask/stop only when the decision is genuinely outside its authority.

This preserves the agent's strength and reduces unnecessary time consumption.

---

# 42. Next Authorized Action Semantics

`next_action` is not a natural-language instruction to the model.

It is a machine-readable authorization.

Example:

```json
"next_action": {
  "type": "EXECUTE_TASK",
  "task_id": "P1-W03-T04",
  "contract_version": 1,
  "authorized": true,
  "reason": "Task READY and all prerequisites satisfied."
}
```

If the agent finds a better method within that task:

```text
method may change
authorization does not
```

If the better method requires architectural change:

```text
authorization stops
→ architecture-change record
```

---

# 43. Build State and Continue Command

The phrase:

> **Continue according to the blueprint.**

must cause the following:

```text
1. Read Build State
2. Validate state integrity
3. Inspect actual repository
4. Validate current phase
5. Validate current task
6. Validate latest checkpoint
7. Validate relevant evidence
8. Reconcile discrepancies
9. Re-enter authorized engineering discretion zone
10. Run preflight
11. Continue the unfinished task
12. Verify
13. Update state
14. Checkpoint
15. Stop at the next safe boundary
```

The phrase must not cause:

```text
guess previous intention
repeat all previous work
start a random next task
skip a failed gate
```

---

# 44. Build State Integrity Validation

At boot and before critical advancement, validate:

```text
[ ] schema version supported
[ ] authority versions supported
[ ] current phase exists in manifest
[ ] current task exists
[ ] task contract version exists
[ ] referenced checkpoint exists
[ ] referenced verification report exists
[ ] repository identity is coherent
[ ] integrity references are valid
[ ] no impossible status combination exists
```

Examples of impossible combinations:

```text
phase=ACCEPTED
task currently FAILED
phase_gate_status=NOT_READY

task=ACCEPTED
verification_gate=FAILED

next_action=ADVANCE_PHASE
phase_gate=FAILED
```

Impossible combinations cause:

```text
STATE_INVALID
→ reconcile
```

---

# 45. State Invariants

The following are Build State invariants.

### BS-01

There is at most one active authorized task per serial mission executor.

### BS-02

An accepted task has a passing verification gate reference.

### BS-03

An accepted task has a valid checkpoint reference where required.

### BS-04

A phase cannot be accepted while a mandatory task is unresolved.

### BS-05

A blocking unknown cannot coexist with an accepted gate that depends on it.

### BS-06

A failed gate cannot be represented as accepted without an explicit new valid verification.

### BS-07

Historical task outcomes are not silently deleted.

### BS-08

Next authorized action must be compatible with current phase/task state.

### BS-09

Build State must never authorize a task outside the active Phase Manifest.

### BS-10

A repository mismatch cannot be silently ignored.

### BS-11

A stale verification reference cannot be treated as current.

### BS-12

Build State cannot grant authority to modify architecture beyond the Blueprint.

### BS-13

Build State cannot itself produce `AttemptStatus::Succeeded`.

### BS-14

Build State must preserve uncertainty when reality is uncertain.

### BS-15

Agent narrative cannot override verified repository/evidence state.

### BS-16

The agent's authorized engineering discretion remains available inside the active task boundary.

---

# 46. Error and Corruption Recovery

If `BUILD_STATE.json` is unreadable or structurally invalid:

```text
DO NOT REWRITE BLINDLY
```

Instead:

```text
1. preserve corrupt state
2. inspect backup/previous state if available
3. inspect event chain
4. inspect repository
5. inspect checkpoint
6. inspect task/verification artifacts
7. reconstruct state
8. record state-recovery event
9. write new validated projection
```

The recovery itself must remain auditable.

---

# 47. Schema Migration Rules

When the schema changes:

```text
old state
  ↓
backup
  ↓
validate old schema
  ↓
migration
  ↓
validate new schema
  ↓
compare critical fields
  ↓
accept migration
```

Never silently reinterpret old state using new field meanings.

A migration failure blocks continuation until state is recovered.

---

# 48. State Export / Diagnostic Safety

A diagnostic/exported Build State must:

- preserve references needed for debugging;
- remove or redact secrets;
- avoid copying full credential-bearing requests;
- avoid embedding unnecessary source code;
- identify the schema version;
- identify whether the state was validated.

The diagnostic representation is not itself authoritative.

---

# 49. Build State Statistics

Statistics are for observability, not control.

Useful metrics:

```text
tasks accepted
tasks failed
tasks blocked
gate pass rate
regression failures
recovery count
handoffs
average recovery overhead
provider failure count
average verification duration
```

Do not use statistics to silently change architecture or policy.

Policy decisions belong to explicit policy/configuration layers.

---

# 50. Final Build State Example

A minimal realistic state:

```json
{
  "schema_version": "1.0",

  "authority": {
    "blueprint_version": "2.4",
    "protocol_version": "1.0",
    "phase_manifest_version": "1.0",
    "task_contract_schema_version": "1.0",
    "verification_gate_version": "1.0"
  },

  "project": {
    "project_id": "ac-001",
    "name": "AI Conductor"
  },

  "current_phase": {
    "id": "P1",
    "name": "State + Event + Persistence Core",
    "status": "IN_PROGRESS",
    "phase_gate_status": "NOT_READY"
  },

  "current_task": {
    "id": "P1-W03-T04",
    "contract_version": 1,
    "status": "IN_PROGRESS",
    "attempt_id": "ATT-0042",
    "verification_gate_id": "VG-P1-W03-T04-01"
  },

  "task_history": {
    "P0-W01-T01": {
      "status": "ACCEPTED",
      "gate_id": "VG-P0-W01-T01-01",
      "checkpoint": "CK-P0-01"
    }
  },

  "checkpoints": {
    "last_verified": "CK-P1-0041"
  },

  "verification": {
    "last_gate": {
      "id": "VG-P1-W03-T04-01",
      "status": "PASSED"
    }
  },

  "failures": {
    "open": [],
    "recent": []
  },

  "unknowns": [],

  "decisions": {
    "pending": []
  },

  "dependencies": {
    "pending_requests": []
  },

  "repository": {
    "branch": "ai-conductor/p1",
    "head_commit": "...",
    "worktree_id": "...",
    "dirty": true
  },

  "runtime": {
    "active_process": null
  },

  "next_action": {
    "type": "EXECUTE_TASK",
    "task_id": "P1-W03-T04",
    "authorized": true
  },

  "session": {
    "session_id": "SESSION-0007",
    "agent": "OX Alpha"
  },

  "integrity": {
    "validation_status": "VALID"
  }
}
```

This example is illustrative. Actual fields and values must match the implemented Blueprint schema and repository.

---

# 51. First-Boot BUILD_STATE

Before Phase 0 begins, the initial state should be minimal:

```json
{
  "schema_version": "1.0",

  "authority": {
    "blueprint_version": "2.4",
    "protocol_version": "1.0",
    "phase_manifest_version": "1.0",
    "task_contract_schema_version": "1.0",
    "verification_gate_version": "1.0"
  },

  "current_phase": {
    "id": "P0",
    "name": "Architecture Contracts",
    "status": "IN_PROGRESS"
  },

  "current_task": null,

  "task_history": {},
  "phase_history": {},
  "checkpoints": {},
  "verification": {},
  "failures": {
    "open": [],
    "recent": []
  },

  "unknowns": [],
  "decisions": {
    "pending": []
  },

  "dependencies": {
    "pending_requests": []
  },

  "repository": {},

  "runtime": {
    "active_process": null
  },

  "next_action": {
    "type": "CREATE_TASK",
    "task_id": "P0-W01-T01",
    "authorized": true
  },

  "session": {},

  "integrity": {
    "validation_status": "UNINITIALIZED"
  },

  "statistics": {
    "tasks_accepted": 0,
    "tasks_failed": 0,
    "tasks_blocked": 0
  }
}
```

---

# 52. Final Acceptance Checklist

The Build State Specification is ready for implementation when:

```text
[ ] Schema versioning is explicit.
[ ] Authority versions are recorded.
[ ] Current phase is explicit.
[ ] Current task is explicit.
[ ] Task history is preserved.
[ ] Phase history is preserved.
[ ] Checkpoints are referenced.
[ ] Verification evidence is referenced.
[ ] Failures are referenced.
[ ] Unknowns are first-class.
[ ] Decisions are first-class.
[ ] Dependencies are tracked.
[ ] Repository state is represented.
[ ] Runtime interruption state is represented.
[ ] Next action is explicit and authorized.
[ ] Integrity information is represented.
[ ] Statistics are clearly non-authoritative.
[ ] Reconstruction rules are explicit.
[ ] Context-limit recovery is explicit.
[ ] Network-failure recovery is explicit.
[ ] Crash recovery is explicit.
[ ] Schema migration is explicit.
[ ] Impossible-state validation exists.
[ ] Build State cannot authorize work outside the Phase Manifest.
[ ] Build State cannot declare Succeeded.
[ ] Evidence outranks stale state.
[ ] Human decisions are persisted.
[ ] Historical failures are not deleted.
[ ] Agent engineering discretion is preserved inside the authorized boundary.
[ ] The state remains lightweight and reference-based.
```

---

# 53. Operating Principle

The Build State file exists to solve one problem:

> **When the current AI agent disappears, the project must not disappear with it.**

A new OX Alpha session should be able to open:

```text
BUILD_STATE.json
+
current Task Contract
+
current Phase Manifest
+
latest verification evidence
+
repository
```

and know:

```text
WHERE WE ARE
WHAT IS ACTUALLY DONE
WHAT IS NOT DONE
WHAT FAILED
WHAT IS UNKNOWN
WHAT IS SAFE
WHAT IS AUTHORIZED
WHAT MUST HAPPEN NEXT
```

without needing the previous model's memory.

At the same time, the control system must never turn OX Alpha into a mechanical typist.

Inside a valid Task Contract:

```text
THE AGENT MAY THINK.
THE AGENT MAY DESIGN IMPLEMENTATION DETAILS.
THE AGENT MAY FIND A BETTER WAY.
THE AGENT MAY ADD BETTER TESTS.
THE AGENT MAY OPTIMIZE.
THE AGENT MAY IMPROVE ROBUSTNESS.
THE AGENT MAY DEBUG CREATIVELY.
```

The boundaries exist to protect:

```text
architecture
scope
security
evidence
state
user work
```

—not to suppress the agent's engineering ability.

That balance is intentional.

---

**END — AI CONDUCTOR BUILD STATE SPECIFICATION v1.0**
