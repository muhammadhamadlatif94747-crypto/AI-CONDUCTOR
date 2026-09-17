# AI CONDUCTOR
# Failure Corpus Specification — v1.0
## Permanent Failure Memory, Reproduction, and Regression System

**Authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.4.md`  
**Build procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.0.md`  
**Phase authorization:** `AI_CONDUCTOR_PHASE_MANIFEST_v1.0.md`  
**Task definition:** `AI_CONDUCTOR_TASK_CONTRACTS_v1.0.md`  
**Verification authority:** `AI_CONDUCTOR_VERIFICATION_GATES_v1.0.md`  
**State authority:** `AI_CONDUCTOR_BUILD_STATE_SPECIFICATION_v1.0.md`  
**Human authority:** Project owner  
**Primary artifact directory:** `.ai-conductor-build/failure_cases/`  
**Status:** Operational failure-memory specification

---

# 0. Purpose

The Failure Corpus is the permanent institutional memory of AI Conductor.

Its purpose is to ensure that every meaningful failure discovered during development, deterministic testing, simulation, chaos testing, real integration, provider use, Aider execution, recovery, handoff, verification, concurrency testing, security testing, or user operation can become a reproducible, reviewable, permanent regression safeguard.

The core principle is:

> **A bug discovered once must become a testable lesson, not a recurring surprise.**

The Failure Corpus prevents:

```text
 discover bug
     ↓
 fix bug
     ↓
 forget bug
     ↓
 later refactor
     ↓
 rediscover same bug
```

Instead:

```text
 discover
     ↓
 classify
     ↓
 reproduce
     ↓
 document
     ↓
 encode as regression scenario
     ↓
 fix
     ↓
 prove
     ↓
 retain permanently
```

---

# 1. Role in the Control Pack

```text
01 MASTER BLUEPRINT
        ↓
02 BUILD PROTOCOL
        ↓
03 PHASE MANIFEST
        ↓
04 TASK CONTRACTS
        ↓
05 VERIFICATION GATES
        ↓
06 BUILD STATE
        ↓
07 FAILURE CORPUS
        ↓
ACTUAL IMPLEMENTATION + EVIDENCE
```

The Failure Corpus does not replace task contracts, verification reports, event history, Build State, or architecture decisions. It is the regression and institutional-memory layer connecting failures to future proof.

---

# 2. What Counts as a Failure Corpus Entry

A failure should become a corpus case if it is:

1. a defect in implementation;
2. a violation or threatened violation of a core invariant;
3. a recovery mistake;
4. a reconciliation mistake;
5. persistence corruption;
6. a false-success condition;
7. a scope or security violation;
8. a provider/resource integration problem that could recur;
9. a concurrency race;
10. a meaningful test-discovered edge case;
11. a previously unknown environment behavior that affects correctness;
12. a regression;
13. a dangerous near-miss that reveals a missing safeguard.

Not every warning needs a corpus case. Harmless temporary noise, duplicate reports of an existing case, and failed experiments that were never part of an accepted design normally do not require permanent cases. When uncertain, record first and classify later.

---

# 3. Failure Case Identity

Every case receives an immutable ID:

```text
F-0001
F-0002
F-0003
...
```

IDs are never reused. If a case is superseded or invalidated, the historical ID remains.

---

# 4. Failure Case Directory Structure

```text
.ai-conductor-build/
└── failure_cases/
    ├── F-0001-timeout-after-partial-edit/
    │   ├── FAILURE.md
    │   ├── scenario.yaml
    │   ├── fixture/
    │   ├── expected/
    │   ├── replay/
    │   └── evidence/
    ├── F-0002-user-edit-conflict/
    │   ├── FAILURE.md
    │   ├── scenario.yaml
    │   └── ...
    └── index.yaml
```

The directory name should remain human-readable after the immutable numeric ID.

---

# 5. Canonical Failure Record

Each failure case contains:

```text
FAILURE.md
```

and:

```text
scenario.yaml
```

The human-readable record is for engineering understanding. The machine-readable scenario is for replay, regression, and automation.

---

# 6. Canonical Machine-Readable Schema

```yaml
failure_schema_version: 1
id: F-0001
title: "..."
status: discovered | reproducible | fixed | regression_protected |
        invalidated | superseded | accepted_risk
severity: low | medium | high | critical
priority: low | medium | high | critical
first_detected:
  phase: "P?"
  task_id: "..."
  attempt_id: "..."
  session_id: "..."
  timestamp: "..."
origin:
  source: deterministic_test | simulation | chaos | integration |
          provider | user_report | code_review | security_review |
          concurrency_test | production_observation | near_miss
classification:
  primary: PROCESS | NETWORK | PROVIDER | EXECUTION | PERSISTENCE |
           RECOVERY | CONFLICT | CANCELLATION | RESOURCE | SECURITY |
           VERIFICATION | CONTEXT | CONCURRENCY | SCOPE | OTHER
  secondary: []
affected:
  subsystem: "..."
  components: []
  invariants: []
  blueprint_sections: []
  phases: []
trigger:
  description: "..."
preconditions: []
stimulus: []
observed:
  state_before: {}
  event_sequence: []
  state_after: {}
  evidence: []
expected:
  state_after: {}
  reconciliation_outcome: "..."
  attempt_status: "..."
  recovery_action: "..."
  final_outcome: "..."
impact:
  user_data: none | possible | affected
  source_code: none | possible | affected
  credentials: none | possible | affected
  provider_quota: none | low | medium | high
  recovery_required: true
  silent_loss_possible: false
  silent_success_possible: false
reproduction:
  method: deterministic | scenario_dsl | chaos | integration | manual
  scenario_id: "..."
  seed: null
  virtual_clock: null
  required_environment: []
regression:
  test_ids: []
  automated: true
  mandatory_for_gates: true
resolution:
  task_id: "..."
  fix_commit: "..."
  verification_gate: "..."
  verification_report: "..."
  checkpoint: "..."
evidence:
  initial_report: "..."
  reproduction_report: "..."
  fix_report: "..."
history:
  related_cases: []
  supersedes: null
  superseded_by: null
lessons:
  root_cause: "..."
  prevention: "..."
  detection: "..."
  future_guidance: "..."
```

---

# 7. Failure Status Lifecycle

```text
DISCOVERED
    ↓
REPRODUCIBLE
    ↓
FIXED
    ↓
REGRESSION_PROTECTED
```

Alternative outcomes:

```text
DISCOVERED → INVALIDATED
DISCOVERED → ACCEPTED_RISK
FIXED → SUPERSEDED
```

`FIXED` does not mean the case is permanently protected. `REGRESSION_PROTECTED` requires the permanent regression mechanism to pass.

---

# 8. Failure Severity

## Low
Limited impact, such as presentation defects or harmless logging issues.

## Medium
Meaningful bounded behavior defects such as avoidable routing or recovery inefficiency.

## High
Reliability, integrity, or security risk such as partial work being incorrectly accepted or user changes being endangered.

## Critical
Potentially catastrophic conditions such as silent data loss, credential exposure, false success after failed work, destructive recovery, security-boundary bypass, or unrecoverable state corruption.

Severity reflects impact, not the effort required to fix the problem.

---

# 9. Failure Classification

Use structured primary classification:

```text
PROCESS
NETWORK
PROVIDER
EXECUTION
PERSISTENCE
RECOVERY
CONFLICT
CANCELLATION
RESOURCE
SECURITY
VERIFICATION
CONTEXT
CONCURRENCY
SCOPE
```

Use secondary classifications when a failure crosses boundaries.

---

# 10. Root Cause Classification

Where evidence supports it, classify the root cause as:

```text
SPECIFICATION
IMPLEMENTATION
STATE_MODEL
PERSISTENCE
CONCURRENCY
INTEGRATION
PROVIDER
ENVIRONMENT
TEST_GAP
OBSERVABILITY
SECURITY
HUMAN_WORKFLOW
TOOLING
UNKNOWN
```

Do not guess. `UNKNOWN` is valid and should remain visible until evidence supports a stronger conclusion.

---

# 11. Failure Reproduction Rule

Every corpus case should become reproducible where technically possible.

Preferred hierarchy:

```text
DETERMINISTIC
    ↓
SCENARIO DSL
    ↓
SIMULATED CHAOS
    ↓
REAL INTEGRATION
    ↓
MANUAL REPRODUCTION
```

Use the cheapest reproducible mechanism that still captures the actual failure.

---

# 12. Deterministic Failure Scenario

A deterministic case defines:

```text
initial state
stimulus
input/event ordering
virtual time
expected result
```

Example:

```yaml
id: F-0001
reproduction:
  method: deterministic
  scenario_id: timeout_after_partial_edit
  seed: 42
stimulus:
  - start_attempt
  - modify_file: "auth.ts"
  - provider_timeout
expected:
  attempt_status: "FAILED"
  reconciliation_outcome: "PARTIAL_CHANGE"
  verification: "NOT_SUCCEEDED"
  recovery: "TARGETED_REPAIR"
```

Exact state names must match the authoritative Blueprint implementation.

---

# 13. Scenario DSL Integration

Where the Blueprint's Scenario DSL is available:

```text
failure case
     ↓
scenario.yaml
     ↓
scenario runner
     ↓
expected result
     ↓
regression test
```

The corpus should not require a large custom test implementation for every case.

---

# 14. Chaos Failure Recording

For Chaos Mode failures record:

```text
chaos_seed
scenario_id
virtual_clock_seed/state
injection points
event sequence
expected outcome
actual outcome
```

A chaos failure without enough replay information remains observational until reproducibility is restored.

---

# 15. Real Integration Failure Recording

For real-provider or Aider failures preserve, where applicable:

```text
provider/model identity
provider capability snapshot
health state
quota/rate state
request/attempt identity
environment
repository/worktree identity
sanitized request metadata
response classification
process result
filesystem evidence
verification evidence
```

Never store credentials or credential-bearing raw requests merely for convenience.

---

# 16. Observational-Only Failures

Some external failures may not be reproducible. They may remain in the corpus as:

```yaml
status: discovered
reproduction:
  method: manual
  reproducible: false
```

Record the best-known trigger, observed evidence, expected safety behavior, and detection improvement.

---

# 17. Failure Case Example

```text
FAILURE ID: F-0001
TITLE: Timeout after partial edit

SEVERITY:
HIGH

PRIMARY CLASS:
NETWORK

SECONDARY:
EXECUTION, RECOVERY

DISCOVERY:
Phase P3, Chaos Mode

TRIGGER:
Provider timeout after a file mutation was already applied.

OBSERVED:
The file changed and the attempt became uncertain.

EXPECTED:
Attempt becomes UNKNOWN.
Reconciliation inspects the workspace.
Partial change is detected.
No blind retry occurs.
Targeted recovery is created.
Verification remains the only path to Succeeded.

ROOT CAUSE:
Recovery incorrectly treated an ambiguous outcome as an ordinary failure.

FIX:
...

REGRESSION:
Deterministic F-0001 scenario passes.

STATUS:
REGRESSION_PROTECTED
```

---

# 18. Failure Corpus Index

Maintain:

```text
failure_cases/index.yaml
```

Example:

```yaml
cases:
  - id: F-0001
    title: "Timeout after partial edit"
    severity: high
    primary: NETWORK
    status: REGRESSION_PROTECTED
    phases: [P3, P5, P8]
    invariants: [6, 14]
    regression_tests:
      - "SC-F-0001"
```

The index remains lightweight.

---

# 19. Duplicate Detection

Before opening a new case, search the corpus.

If the same underlying failure already exists, link to it. Create a new case only when trigger, root cause, impact, recovery behavior, or invariant impact differs materially.

---

# 20. Related Failures

Cases may reference related cases:

```yaml
related_cases:
  - "F-0012"
  - "F-0027"
```

Use this to connect recurring families such as state correctness, handoff, persistence, Git/worktree, provider reliability, security, verification, and concurrency.

---

# 21. Near-Miss Cases

A near miss is a dangerous condition that was successfully blocked by a safeguard.

Examples:

```text
capability enforcement blocked unauthorized tool call
user-change detector prevented overwrite
verification blocked false success
reconciliation detected stale state
```

Near misses should be retained when they validate or stress a critical safeguard.

---

# 22. Security Failure Cases

Security cases require:

```yaml
security:
  boundary: "..."
  attack_surface: "..."
  exploitability: low | medium | high | critical
  credential_exposure: false
  containment: "..."
  regression_required: true
```

Security cases are retained permanently unless formally superseded by a stronger case while preserving history.

---

# 23. Data-Loss Failure Cases

Any possibility of:

- user-data loss;
- source-code loss;
- silent overwrite;
- destructive reset;

is automatically at least `HIGH` severity.

The case must include the last-known-safe checkpoint, affected files, recovery outcome, reproduction, and regression protection.

A critical data-loss failure blocks release until its regression protection passes.

---

# 24. False-Success Failure Cases

These are first-class high-value cases.

Examples:

```text
model claims success but code is incomplete
Aider reports edit but verification fails
reconciliation sees expected change but acceptance criteria fail
test passed against stale code
```

The case should explicitly record:

```yaml
impact:
  silent_success_possible: true
```

These failures directly protect the trust boundary of AI Conductor.

---

# 25. Repeated-Failure Escalation

If the same failure recurs after being marked fixed, classify it as a regression and reconsider whether the underlying abstraction, invariant, test, or architecture is insufficient.

Do not repeatedly patch symptoms without reevaluating root cause.

---

# 26. Mandatory Regression Promotion

A meaningful failure should become automated regression protection when:

- reproduction is deterministic;
- the scenario can be encoded;
- expected safe behavior is clear.

Sequence:

```text
FAILURE CASE
    ↓
REPRODUCTION
    ↓
SCENARIO
    ↓
TEST
    ↓
FIX
    ↓
TEST PASSES
    ↓
REGRESSION_PROTECTED
```

Never mark protected before the regression actually passes.

---

# 27. Failure Invalidation

A case may be invalidated only when evidence proves that the original report or reproduction was incorrect, impossible, or duplicated by a stronger case.

Preserve the historical record and explain why it was invalidated.

---

# 28. Failure Supersession

A weaker observational case may be superseded by a deterministic case that captures the same underlying problem more reliably.

Example:

```text
F-001
manual timeout observation

F-017
deterministic timeout-after-write scenario
```

F-001 becomes `SUPERSEDED_BY F-017` while its original observation remains preserved.

---

# 29. Failure Case Review

High/critical cases should be reviewed for:

```text
root cause
invariant impact
whether architecture needs change
whether the regression is sufficient
whether recovery is safe
whether similar failure modes exist
```

If a failure exposes an architecture flaw:

```text
failure case
    ↓
architecture-change request
```

Do not hide architectural changes inside tests.

---

# 30. Failure-to-Invariant Mapping

Every serious case should identify affected invariants where applicable:

```yaml
invariants:
  - 6
  - 13
  - 14
```

This makes the corpus a map of what protects each core invariant.

---

# 31. Failure-to-Phase Mapping

Record every phase where a failure matters.

A Phase P3 problem may later matter in P5, P6, P8, or P10. This prevents the failure from disappearing from later acceptance gates.

---

# 32. Failure-to-Gate Mapping

Protected cases should reference relevant gates:

```yaml
regression_gates:
  - "VG-P3-..."
  - "VG-P8-..."
  - "VG-P10-..."
```

Critical failures should be included in release-gate evidence.

---

# 33. Failure-to-Task Mapping

Record both discovery and fixing tasks:

```yaml
discovered_by_task: "P3-W04-T02"
fixed_by_task: "P3-W05-T01"
```

This is traceability, not blame.

---

# 34. No-Blame Principle

Failure Corpus language should describe:

```text
system behavior
trigger
evidence
root cause
prevention
```

not judgments about the agent or developer.

The objective is to improve the system.

---

# 35. Failure Corpus and OX Alpha

OX Alpha should read relevant cases before modifying a subsystem.

Selection should use:

```text
current task
current subsystem
affected invariants
failure classifications
risk
related phases
critical prior cases
```

It should not load the complete corpus into every context window.

---

# 36. Failure Corpus and Engineering Discretion

The Failure Corpus tells OX Alpha:

```text
WHAT FAILED
WHAT MUST NEVER HAPPEN AGAIN
WHAT EVIDENCE PROVES THE FIX
```

It does not prescribe one implementation unless the architecture explicitly requires it.

Within authorized scope, OX Alpha may choose the best implementation strategy that preserves required safety properties.

Example:

```text
historical failure:
duplicate recovery after timeout

required prevention:
must not duplicate accepted side effect

possible implementations:
state-machine guard
idempotency ledger
reconciliation mechanism
command deduplication
```

The agent may select the better engineering solution when it remains within the normative contract.

---

# 37. Failure Corpus and AI-Assisted Coding

When fixing a known case, the Task Contract should reference it:

```yaml
failure_cases:
  - "F-0007"
```

The agent should:

```text
read failure
→ reproduce
→ inspect root cause
→ implement minimal safe fix
→ run historical regression
→ run new relevant tests
→ run broader regression
→ pass gate
```

The bug is not considered fixed merely because the original symptom disappears once.

---

# 38. Failure Corpus and Architecture Changes

If a failure reveals a weakness in:

```text
invariant
state model
security boundary
merge protocol
provider contract
```

then use:

```text
FAILURE CASE
    ↓
ARCHITECTURE CHANGE REQUEST
    ↓
human/authority decision
    ↓
Blueprint update if approved
    ↓
Task Contract update
    ↓
new regression protection
```

This keeps failure memory connected to architecture evolution.

---

# 39. Failure Corpus and Release Readiness

Before release:

```text
[ ] no open critical failure
[ ] no unprotected critical regression
[ ] all mandatory high-severity regression cases pass
[ ] recent regressions pass
[ ] security cases pass
[ ] data-loss cases pass
[ ] false-success cases pass
[ ] handoff/recovery cases pass
[ ] concurrency cases pass
```

Any exception must be explicit, reviewed, and recorded as accepted risk.

---

# 40. Failure Case Retention

Failure cases should normally be retained for the life of the product.

Repository size should be managed through compact evidence and references, not by deleting institutional memory.

---

# 41. Failure Case Privacy

Never store:

- API keys;
- OAuth tokens;
- refresh tokens;
- credentials;
- unnecessary private user data;
- secret-bearing environment dumps.

Redact tokens, authorization headers, cookies, provider credentials, and unnecessary sensitive paths or user content.

A failure case must preserve engineering reproducibility without preserving secrets.

---

# 42. Failure Corpus Validation

The corpus itself requires validation.

A validation command should detect:

```text
missing FAILURE.md
missing scenario.yaml
invalid schema
missing regression test
duplicate ID
invalid status
missing evidence
broken task/gate/Blueprint references
orphaned case
secret-like content
```

A corrupted corpus must not silently be treated as complete.

---

# 43. Failure Corpus Statistics

Useful diagnostic summaries include:

```text
total cases
open cases
protected cases
critical cases
repeat failures
failures by classification
failures by subsystem
failures by phase
failures by invariant
provider-specific failures
recovery failures
security failures
```

These are diagnostic, not product-quality claims.

---

# 44. Failure Corpus Query Model

Eventually provide operations such as:

```text
show failure F-0007
search failures "timeout"
list failures for invariant 14
list failures for reconciliation
list unprotected critical failures
list failures affecting Phase 8
run regression for F-0007
run all critical failures
```

The corpus should be useful during engineering, not merely archival.

---

# 45. Failure Learning Loop

```text
failure
  ↓
classification
  ↓
reproduction
  ↓
root-cause analysis
  ↓
fix
  ↓
regression
  ↓
architecture lesson if necessary
  ↓
future task context
```

A mature system becomes progressively harder to break because every important failure becomes part of its engineering memory.

---

# 46. Unknown Failure Rule

If a failure cannot yet be classified:

```yaml
primary: OTHER
root_cause: UNKNOWN
```

Do not force it into a misleading category.

If unknown behavior is safety-relevant, it remains blocking until understood or explicitly bounded by an approved policy.

---

# 47. Failure Escalation Rule

A failure should trigger architecture review when:

- it violates a core invariant;
- it exposes a missing state;
- its fix would bypass an invariant;
- the same class repeatedly recurs;
- recovery cannot be made deterministic;
- user data can be lost;
- a security boundary is involved;
- false success is possible.

A passing local test does not automatically prove the architecture is sound.

---

# 48. Failure Corpus Quality Principle

A good failure case tells a future agent:

```text
WHAT HAPPENED
WHY IT MATTERED
HOW TO REPRODUCE IT
WHAT MUST NEVER HAPPEN AGAIN
HOW WE KNOW IT IS FIXED
```

A case that only says `timeout bug fixed` is insufficient.

---

# 49. Failure Corpus Gate

The corpus has its own verification gate.

It passes only when:

```text
[ ] schema valid
[ ] no duplicate IDs
[ ] mandatory fields present
[ ] regression references resolve
[ ] protected cases are replayable where applicable
[ ] no protected critical case currently fails
[ ] no secret exposure detected
[ ] task/gate/Blueprint references resolve
```

---

# 50. Relationship to Build State

Build State should contain only lightweight references and summaries:

```json
"failures": {
  "open": ["F-0007"],
  "recent": ["F-0006"],
  "critical_open": 0
}
```

The complete failure record belongs in the corpus.

This keeps the persistent build state lightweight and fast to load.

---

# 51. Relationship to Verification Gates

Verification Gates should select relevant historical cases using:

```text
task subsystem
affected invariants
phase
risk
failure classification
```

A reconciliation task should automatically pull the relevant reconciliation, conflict, persistence, and recovery cases.

The corpus therefore becomes an input to future gates rather than an archive that can be forgotten.

---

# 52. Relationship to Phase Gates

Phase gates should require:

```text
mandatory historical cases for the phase
+
relevant critical/high cases
+
new cases discovered in the phase
```

to pass.

The Phase Manifest determines which cases are mandatory; the corpus supplies the cases and regression references.

---

# 53. Resource-Efficient Failure Testing

Failure replay follows:

```text
DETERMINISTIC
    ↓
SIMULATION
    ↓
REAL INTEGRATION
```

Do not consume real Gemini/Mistral quota to prove failure behavior that FakeProvider and the simulator can prove exactly.

Use real providers only when the failure specifically depends on real provider behavior.

---

# 54. Failure Corpus and Context Efficiency

When OX Alpha works on a task, retrieve only relevant cases.

For example:

```text
current task: provider handoff
        ↓
load:
provider failures
handoff failures
recovery failures
context-transfer failures
relevant invariants
critical related cases
```

Do not inject the complete historical corpus into every model context.

---

# 55. Failure Corpus and Safe Engineering Judgment

The corpus constrains the required safety property, not the implementation creativity.

Within a valid Task Contract, OX Alpha may use a better implementation than the original fix, provided that:

```text
required invariant remains true
regression case still passes
acceptance criteria still pass
scope remains authorized
security is preserved
```

If the better solution requires an architectural change, use the architecture-change protocol rather than silently changing the system.

---

# 56. OX Alpha Operating Instruction

When working on a task:

> Read only the Failure Corpus cases relevant to the active task, subsystem, invariants, risk, and known classifications.
>
> Treat historical failures as constraints on behavior, not commands to reproduce an exact historical implementation.
>
> Preserve the required safety property while using engineering judgment to choose the best implementation.
>
> If you discover a meaningful new failure, capture it accurately and create its regression scenario before calling the related protection complete.
>
> Do not delete, weaken, bypass, or silently downgrade a historical regression safeguard.
>
> If a historical case conflicts with the current Blueprint, do not resolve the conflict by guessing. Escalate through the architecture-change process.

---

# 57. Final Acceptance Checklist

```text
[ ] Every case has an immutable ID.
[ ] Human-readable and machine-readable representations are defined.
[ ] Failure classification is structured.
[ ] Severity is explicit.
[ ] Root cause may remain UNKNOWN without fabrication.
[ ] Reproduction hierarchy is defined.
[ ] Scenario DSL integration is defined.
[ ] Chaos metadata is preserved.
[ ] Real-provider failures can be sanitized and recorded.
[ ] Near misses are supported.
[ ] Security failures have stronger requirements.
[ ] Data-loss failures are automatically high/critical.
[ ] False-success failures are first-class.
[ ] Regression promotion is explicit.
[ ] Failure invalidation/supersession preserve history.
[ ] Related cases can be linked.
[ ] Cases map to invariants.
[ ] Cases map to phases.
[ ] Cases map to tasks and gates.
[ ] Build State only stores references/summaries.
[ ] Relevant cases can be selectively retrieved.
[ ] Corpus validation is defined.
[ ] Secret protection is defined.
[ ] Release gates can require critical cases.
[ ] OX Alpha retains engineering discretion while preserving required safety properties.
```

---

# 58. Final Principle

The Failure Corpus is not a museum of mistakes.

It is the project's **immune system**.

Every meaningful failure should make the system harder to break, easier to diagnose, cheaper to test, and less likely to repeat.

The intended loop is:

```text
DISCOVER
  ↓
REPRODUCE
  ↓
UNDERSTAND
  ↓
FIX
  ↓
VERIFY
  ↓
REGRESSION-PROTECT
  ↓
RETAIN
```

The objective is not to make the unverifiable claim that the product will have zero bugs.

The objective is:

> **Every important failure becomes increasingly difficult to repeat, increasingly easy to diagnose, and increasingly cheap to detect.**

And the control system must preserve OX Alpha's engineering ability while enforcing authority, safety, evidence, and scope boundaries.

Within authorized scope:

```text
the agent may think,
experiment,
design,
improve,
optimize,
and choose the better engineering solution.
```

The system constrains the **authority boundary and the proof**, not the agent's engineering capability.

---

**END — AI CONDUCTOR FAILURE CORPUS SPECIFICATION v1.0**
