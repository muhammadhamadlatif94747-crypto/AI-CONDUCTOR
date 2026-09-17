# AI CONDUCTOR
# Task Contracts — v1.1 (Harness-Neutral)
## Bounded Execution Contracts for Reliable AI-Assisted Implementation

**Authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`  
**Build procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.1_HARNESS_NEUTRAL.md`  
**Phase authorization:** `AI_CONDUCTOR_PHASE_MANIFEST_v1.1_HARNESS_NEUTRAL.md`  
**Implementation agent:** OX Alpha / equivalent coding agent  
**Human authority:** Project owner  
**Status:** Operational task-contract specification

---

## 0. Purpose

This document defines the **Task Contract layer** of AI Conductor.

A Task Contract is the smallest authoritative execution boundary given to an AI coding agent. It converts an authorized phase objective into one bounded unit that can be:

- understood without reconstructing the whole project,
- implemented without guessing,
- tested deterministically where possible,
- verified with explicit evidence,
- reviewed against a finite acceptance set,
- checkpointed,
- safely interrupted,
- resumed by another agent,
- localized when something fails,
- prevented from silently expanding its scope.

The Task Contract is **not** an architecture document, phase roadmap, implementation plan for the entire project, or verification report.

It answers:

> **Exactly what may this agent change right now, what must it prove, and what must it leave untouched?**

The governing execution chain is:

```text
MASTER BLUEPRINT v2.4
        ↓
BUILD PROTOCOL v1.1
        ↓
PHASE MANIFEST v1.1
        ↓
TASK CONTRACT
        ↓
PRE-FLIGHT
        ↓
IMPLEMENTATION SLICE
        ↓
TEST / SIMULATION
        ↓
VERIFICATION
        ↓
ACCEPTANCE
        ↓
CHECKPOINT
        ↓
BUILD STATE
        ↓
NEXT TASK
```

The Blueprint defines what AI Conductor is.  
The Build Protocol defines how the agent works.  
The Phase Manifest defines what phase is authorized.  
**The Task Contract defines the exact unit currently authorized.**

---

# 1. Non-Negotiable Task Contract Rules

## 1.1 No task without phase authorization

A task may exist only if:

1. its phase is authorized by the active Phase Manifest;
2. its referenced Blueprint requirements are identifiable;
3. its scope is bounded;
4. its acceptance criteria are explicit.

A Task Contract never grants authority to enter a later phase.

---

## 1.2 The contract is a scope fence

The agent may modify only:

- explicitly allowed files;
- explicitly allowed generated artifacts;
- narrowly necessary test fixtures;
- narrowly necessary implementation support directly required by the contract.

Everything else is forbidden unless the contract is formally amended.

If an additional file, dependency, API, capability, schema, permission, or architectural rule becomes necessary:

```text
STOP
  ↓
record scope issue
  ↓
explain necessity
  ↓
identify exact affected surface
  ↓
assess risk
  ↓
request contract amendment / architecture decision
  ↓
resume only when authorized
```

---

## 1.3 No silent architecture changes

The Task Contract cannot override the Blueprint.

If implementation reveals:

- a contradiction,
- an insufficient invariant,
- an undefined state transition,
- an architectural incompatibility,
- a security boundary problem,
- a persistence guarantee that cannot be met,

the agent must use the Build Protocol's architecture-change procedure.

It must not "fix" the Blueprint by implementation.

---

## 1.4 No fake completion

A task is not complete because:

- code exists;
- files were created;
- a build command was assumed to pass;
- an AI model says it works;
- a previous session says it worked;
- a test was written but not executed;
- a provider returned HTTP 200;
- the UI looks correct.

The authoritative progression is:

```text
IMPLEMENTED
    ↓
TESTED
    ↓
VERIFIED
    ↓
ACCEPTED
    ↓
CHECKPOINTED
```

`ACCEPTED` requires executed evidence.

---

## 1.5 Unknown is a valid state

The agent must use explicit states such as:

- `UNKNOWN`
- `BLOCKED`
- `FAILED`
- `CONFLICT`
- `CANCELLED`

rather than inventing certainty.

`INFERRED` must never be represented as `VERIFIED`.

---

## 1.6 One invariant → one slice → one gate

Whenever practical:

```text
Blueprint requirement / invariant
        ↓
one bounded task
        ↓
one implementation slice
        ↓
deterministic proof
        ↓
simulation / failure proof
        ↓
verification
        ↓
checkpoint
```

Do not combine unrelated reliability contracts merely to reduce the number of tasks.

---

## 1.7 Smallest safe change

The agent must implement the smallest change that satisfies the contract.

Do not perform unrelated:

- cleanup,
- formatting churn,
- dependency upgrades,
- refactors,
- UI polish,
- speculative abstractions,
- performance work,
- architecture redesign.

A smaller reviewable diff is preferred over a clever large change.

---

## 1.8 Existing verified behavior is protected

A task must preserve:

- already-passing tests;
- established invariants;
- verified state semantics;
- security boundaries;
- existing accepted behavior;

unless the current contract explicitly changes them.

A regression is a task failure, not an acceptable side effect of "progress."

---

## 1.9 Evidence must match risk

Task risk determines proof strength.

```text
LOW
  deterministic checks

MEDIUM
  deterministic + integration/simulation as applicable

HIGH
  deterministic + simulation + targeted integration

CRITICAL
  all applicable verification levels + failure injection +
  architecture/security review + explicit human acceptance where required
```

Never use expensive real-provider testing to discover a deterministic logic error.

---

# 2. Task Identity and Naming

## 2.1 Canonical ID

Every task has a stable identifier:

```text
P<phase>-W<workstream>-T<task>
```

Examples:

```text
P0-W01-T01
P0-W02-T01
P1-W03-T04
P6-W02-T03
P10-W05-T02
```

Identifiers must not be silently reused for different work.

If scope changes materially, create a new version or successor task.

---

## 2.2 Optional task suffixes

For explicit sub-work only:

```text
P1-W03-T04-A
P1-W03-T04-B
```

Do not create subtask trees merely to make contracts look detailed.

A task should normally remain independently reviewable.

---

## 2.3 Contract version

Each contract has its own version:

```yaml
contract_version: 1
```

Changing acceptance semantics, scope, required evidence, risk, or implementation obligations requires a version change and an audit entry.

---

# 3. Required Task Contract Schema

The canonical machine-readable representation is YAML.

Every executable Task Contract must contain at least:

```yaml
contract_version: 1

id: P?-W??-T??
phase: P?

status: DRAFT

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §..."
  - "Build Protocol §..."
  - "Phase Manifest §..."

objective: >
  One precise sentence describing the bounded outcome.

allowed_files: []
forbidden_files: []

allowed_artifacts: []
forbidden_artifacts: []

allowed_commands: []
forbidden_commands: []

dependencies: []

preconditions: []

inputs: []

outputs: []

implementation_requirements: []

invariants_preserved: []

invariants_proven: []

required_tests: []

failure_scenarios: []

acceptance: []

non_goals: []

risk_level: low | medium | high | critical

resource_policy:
  network_required: false
  real_provider_allowed: false
  preferred_test_level: deterministic
  max_retry_policy: "per protocol"
  context_budget: "minimal sufficient"

done_when: []

checkpoint_requirements: []

resume_requirements: []

stop_conditions: []

escalation_conditions: []
```

The fields below strengthen this schema and should be used whenever applicable.

---

# 4. Field Definitions

## 4.1 `id`

Stable task identity.

Must correspond to the active phase and workstream.

---

## 4.2 `phase`

The authorized phase.

A task cannot declare a later phase merely because the implementation happens to touch infrastructure useful later.

---

## 4.3 `status`

Allowed task states:

```text
DRAFT
READY
BLOCKED
IN_PROGRESS
IMPLEMENTED
VERIFIED
ACCEPTED
FAILED
CANCELLED
SUPERSEDED
```

### Transition rule

```text
DRAFT
  ↓
READY
  ↓
IN_PROGRESS
  ↓
IMPLEMENTED
  ↓
VERIFIED
  ↓
ACCEPTED
```

Exceptional transitions:

```text
READY → BLOCKED
IN_PROGRESS → BLOCKED
IN_PROGRESS → FAILED
IMPLEMENTED → FAILED
VERIFIED → FAILED        (if later evidence invalidates it)
DRAFT/READY → CANCELLED
any historical version → SUPERSEDED
```

`ACCEPTED` is not a free-form status.

---

## 4.4 `authority`

Pins the contract to the exact governing versions.

This prevents an old task contract from being interpreted against a newer architecture without review.

---

## 4.5 `section_refs`

Every contract must identify the exact Blueprint and protocol material that authorizes the work.

References should be as narrow as practical.

Bad:

```text
Blueprint
```

Better:

```text
Blueprint §4.7 Attempt State Model
Blueprint §5.3 Reconciliation
Build Protocol §9 Task Contract Protocol
Build Protocol §21 Succeeded Is a Protected Transition
```

---

## 4.6 `objective`

One bounded outcome.

A good objective can be summarized in one sentence without "and also."

If it requires several unrelated outcomes, split the task.

---

## 4.7 `allowed_files`

The explicit mutation surface.

Examples:

```yaml
allowed_files:
  - "src/kernel/attempt.ts"
  - "src/kernel/attempt.test.ts"
  - "src/types/attempt.ts"
```

Use repository-relative paths.

If a file does not yet exist, the contract may authorize its creation.

---

## 4.8 `forbidden_files`

Files that must not be modified even if the agent considers them convenient.

Include sensitive or unrelated areas when useful.

Examples:

```yaml
forbidden_files:
  - "src/ui/**"
  - "package-lock.json"
  - ".env"
  - "production/**"
```

---

## 4.9 `allowed_artifacts`

Generated artifacts that may be produced.

Examples:

```yaml
allowed_artifacts:
  - ".ai-conductor-build/verification_reports/P0-W01-T01.md"
  - ".ai-conductor-build/task_contracts/P0-W01-T01.yaml"
```

---

## 4.10 `forbidden_artifacts`

Artifacts that must not be generated, especially:

- secrets,
- provider tokens,
- full environment dumps,
- user data,
- uncontrolled logs,
- huge diagnostic bundles.

---

## 4.11 `allowed_commands`

Commands needed to perform and prove the task.

Prefer exact commands where known.

Examples:

```yaml
allowed_commands:
  - "cargo test ..."
  - "cargo check"
  - "git diff --check"
```

If a command can mutate state, its mutation must be understood.

---

## 4.12 `forbidden_commands`

Commands prohibited for the task.

Examples:

```yaml
forbidden_commands:
  - "git reset --hard"
  - "git clean -fd"
  - "rm -rf ..."
```

Do not use destructive commands merely to recover from uncertainty.

---

## 4.13 `dependencies`

Only actual prerequisites.

For every dependency, identify:

```yaml
dependencies:
  - id: "D-..."
    reason: "..."
    status: available | unavailable | unknown
```

A dependency that is unavailable blocks the task rather than causing silent substitution.

---

## 4.14 `preconditions`

Facts that must be true before implementation begins.

Examples:

```yaml
preconditions:
  - "Previous task P0-W00-T00 is ACCEPTED."
  - "Repository baseline is recorded."
  - "No unresolved merge/rebase/cherry-pick exists."
  - "Required test command is executable."
```

The agent must verify preconditions; it must not assume them.

---

## 4.15 `inputs`

The minimum information or artifacts required.

Examples:

```yaml
inputs:
  - "Attempt state model"
  - "Reconciliation outcome enum"
  - "Existing persistence schema"
```

Do not request the entire repository when a small authoritative input set is sufficient.

---

## 4.16 `outputs`

Expected durable results.

Examples:

```yaml
outputs:
  - "Attempt transition implementation"
  - "Deterministic transition tests"
  - "Verification report"
```

Outputs must be distinguishable from claims.

---

## 4.17 `implementation_requirements`

These are obligations, not suggestions.

Examples:

```yaml
implementation_requirements:
  - "Preserve AttemptStatus and ReconciliationOutcome as distinct concepts."
  - "Do not allow reconciliation to emit Succeeded."
  - "Use injected Clock rather than wall-clock access."
```

---

## 4.18 `invariants_preserved`

List every relevant invariant that must remain true.

A task that touches a sensitive subsystem should explicitly state its preservation obligations.

---

## 4.19 `invariants_proven`

List the invariant(s) this task actually proves.

Do not list an invariant merely because the code mentions it.

---

## 4.20 `required_tests`

Tests must be executable and specific.

Prefer:

```yaml
required_tests:
  - "unit: transition from Running to FailedWithChanges"
  - "unit: illegal transition is rejected"
  - "property: no path allows reconciliation to emit Succeeded"
```

over:

```yaml
required_tests:
  - "test everything"
```

---

# 5. Test-Level Contract

Every task must declare the highest test level actually required.

## Level 1 — Deterministic

Use for pure logic such as:

- state transitions,
- failure classification,
- reconciliation,
- acceptance evaluation,
- command generation,
- resource scoring,
- capability decisions.

Requirements:

```text
no network
no real provider
no real wall clock
no nondeterministic randomness
```

---

## Level 2 — Simulation

Use:

- FakeProvider,
- VirtualClock,
- VirtualFilesystem,
- VirtualGit,
- VirtualProcess,
- VirtualUser,
- Scenario DSL,
- seeded chaos.

Simulation is preferred for failure behavior before real integrations.

---

## Level 3 — Real Integration

Use only when the contract requires real behavior:

- actual Aider,
- actual Git,
- actual OmniRoute,
- actual Gemini/Mistral,
- real OS process behavior,
- real browser/runtime behavior where applicable.

Real integrations are expensive and should not be the first proof of deterministic logic.

---

# 6. Failure Contract

Every reliability-relevant task must state which failures are intentionally tested.

Possible classes:

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
```

Examples:

```yaml
failure_scenarios:
  - "timeout after partial edit"
  - "duplicate provider response"
  - "restart during reconciliation"
  - "checkpoint interruption"
  - "changed Git baseline"
```

For each scenario, define:

```text
trigger
expected observed state
expected classification
expected recovery
expected evidence
expected final state
```

A discovered meaningful failure becomes a permanent regression scenario according to the Build Protocol.

---

# 7. Acceptance Contract

Acceptance is stricter than implementation.

Each acceptance item must be objectively answerable:

```yaml
acceptance:
  - id: AC-01
    criterion: "..."
    verification_method: "unit test ..."
    evidence_required: "test output ..."
    status: pending
```

### Required verification-method rule

Every acceptance criterion must state **how it will be verified**.

Allowed methods include:

```text
deterministic_test
simulation
integration_test
runtime_check
browser_check
diff_inspection
schema_check
requirement_check
security_review
manual_human_review
```

Do not use:

```text
"agent confirms"
"looks correct"
"probably works"
```

---

# 8. `done_when`

`done_when` is the final executable completion checklist.

A good task may look like:

```yaml
done_when:
  - "All required implementation files are within allowed_files."
  - "All required deterministic tests pass."
  - "Required failure scenario passes."
  - "No forbidden file changed."
  - "Acceptance criteria all PASS."
  - "Verification report recorded."
  - "Checkpoint recorded."
  - "BUILD_STATE updated."
```

A task is not accepted if any mandatory item is `UNKNOWN`, `BLOCKED`, or `FAIL`.

---

# 9. Risk Classification

## Low

Examples:

- isolated pure function;
- documentation-only change with no semantic impact;
- deterministic test fixture.

---

## Medium

Examples:

- persistence helper;
- state serialization;
- non-critical integration adapter.

---

## High

Examples:

- state machine;
- reconciliation;
- checkpointing;
- merge logic;
- provider handoff;
- capability enforcement;
- credential boundary.

---

## Critical

Examples:

- verification authority;
- `Succeeded` transition;
- security boundary;
- destructive filesystem/Git behavior;
- external side-effect idempotency;
- migration with potential data loss.

Risk classification controls proof strength; it does not permit shortcuts.

---

# 10. Resource-Efficiency Contract

AI Conductor is explicitly free-first and resource-efficient.

A Task Contract must avoid wasting:

- provider calls,
- context tokens,
- retries,
- network traffic,
- expensive real integration runs.

For each task, specify:

```yaml
resource_policy:
  network_required: true | false
  real_provider_allowed: true | false
  preferred_test_level: deterministic | simulation | integration
  context_budget: minimal | bounded | expanded
  retry_policy: "reference Build Protocol"
```

## Rules

1. Prefer local deterministic checks.
2. Use simulation before real providers.
3. Use real providers only when their behavior itself is under test.
4. Never retry blindly after an ambiguous side effect.
5. Never transmit the whole repository when a minimal context slice is sufficient.
6. Never spend provider quota to test logic that can be tested locally.
7. If a real integration fails due to network/provider conditions, preserve the failure evidence and classify it rather than repeatedly burning quota.

---

# 11. Pre-Flight Contract

Before implementation, OX Alpha must validate:

```text
[ ] Blueprint version matches
[ ] Build Protocol version matches
[ ] Phase Manifest version matches
[ ] Task Contract version is valid
[ ] Active phase matches task phase
[ ] Previous required task/gate is accepted
[ ] Repository is in expected state
[ ] Worktree is safe
[ ] Preconditions hold
[ ] Allowed files are known
[ ] Forbidden files are known
[ ] Required dependencies are available
[ ] Required commands are available
[ ] Required tests are known
[ ] Acceptance criteria are executable
[ ] No unresolved architecture decision blocks work
```

If any mandatory precondition is not established:

```text
BLOCKED
```

Do not code around it.

---

# 12. Implementation Protocol

Once pre-flight passes:

```text
1. Inspect only the necessary authoritative code.
2. Establish the smallest implementation plan.
3. Make the smallest safe change.
4. Avoid unrelated modifications.
5. Run cheapest disproof tests first.
6. Run required deterministic tests.
7. Run simulation/failure tests if required.
8. Run real integration only if required.
9. Inspect diff and scope.
10. Produce evidence.
11. Evaluate acceptance.
12. Checkpoint only after acceptance.
13. Update BUILD_STATE.
```

The agent must not continue into the next task merely because the current task appears easy.

---

# 13. Scope-Drift Detection

During implementation, continuously compare actual work against:

```text
allowed_files
allowed_commands
implementation_requirements
non_goals
section_refs
```

If drift occurs:

```text
SCOPE_VIOLATION
```

The agent must stop before making the out-of-scope mutation.

A necessary change can be authorized through contract amendment.

---

# 14. Contract Amendment Protocol

A Task Contract may require amendment when reality exposes a legitimate missing boundary.

An amendment must record:

```yaml
amendment:
  id: AM-0001
  reason: "..."
  discovered_by: "..."
  evidence:
    - "..."
  old_scope:
    - "..."
  new_scope:
    - "..."
  files_added:
    - "..."
  files_removed:
    - "..."
  acceptance_changes:
    - "..."
  invariant_impact:
    - "none | ..."
  risk_change:
    from: medium
    to: high
  architecture_change_required: false
  approval_status: pending | approved | rejected
```

### Important

If the amendment changes an architectural rule, invariant, state machine, security boundary, or externally visible contract, it is **not merely a task amendment**. It requires the Blueprint architecture-change protocol.

---

# 15. Dependency Request Boundary

A task must not install a new dependency simply because it is convenient.

If a dependency is necessary, create a dependency request containing:

```text
name
version
purpose
where used
why current dependencies / standard library are insufficient
size / footprint
license
maintenance health
security considerations
alternatives
why necessary now
```

Until approved:

```text
BLOCKED
```

unless the Build Protocol explicitly authorizes the dependency.

---

# 16. Verification Evidence Package

Every accepted task must leave a compact evidence package.

Recommended:

```text
.ai-conductor-build/
  task_contracts/
    <task-id>.yaml

  verification_reports/
    <task-id>.md

  checkpoints/
    <checkpoint-id>.md
```

The verification report should include:

```text
TASK
CONTRACT VERSION
REPOSITORY BASELINE
CHANGED FILES
COMMANDS EXECUTED
TESTS EXECUTED
TEST RESULTS
FAILURES
ACCEPTANCE RESULTS
EVIDENCE REFERENCES
ARCHITECTURE INTEGRITY RESULT
CHECKPOINT
FINAL STATUS
```

Evidence must be reproducible where practical.

---

# 17. Evidence Trust Classification

Task evidence uses the project's evidence trust model.

Every important observation should be classified:

```text
OBSERVED
VERIFIED
DOCUMENTED
INFERRED
UNKNOWN
```

### OBSERVED

Directly seen in:

- repository,
- command output,
- running process,
- provider response,
- filesystem,
- Git state.

### VERIFIED

Confirmed by reproducible controlled testing.

### DOCUMENTED

Established by authoritative documentation.

### INFERRED

Reasoned but not independently confirmed.

### UNKNOWN

Insufficient evidence.

Acceptance must not rely solely on `INFERRED`.

---

# 18. Checkpoint Contract

A task that changes implementation must identify the checkpoint obligation.

A checkpoint must preserve enough information to reconstruct:

```text
task identity
repository state
changed files
verified tests
acceptance result
next authorized action
```

Never mark the task accepted before the checkpoint requirement is satisfied.

---

# 19. Stop / Interruption Contract

A task may safely stop because of:

- context limit;
- network failure;
- provider failure;
- process interruption;
- resource exhaustion;
- test failure;
- architecture contradiction;
- scope conflict;
- dependency problem;
- human decision;
- cancellation.

Stopping is not automatically failure.

The distinction is:

```text
SAFE STOP
    ≠
TASK FAILURE
```

Before stopping, record:

```text
STATUS
COMPLETED
CHANGED FILES
TESTS RUN
TESTS PASSED
TESTS FAILED
KNOWN ERRORS
KNOWN UNKNOWNS
LAST VERIFIED CHECKPOINT
LAST VERIFIED COMMIT
EXACT UNFINISHED WORK
NEXT EXACT ACTION
DO NOT REPEAT
FILES INTENTIONALLY NOT TOUCHED
```

---

# 20. Resume Contract

When the human says:

> **Continue according to the blueprint.**

OX Alpha must not guess.

It must:

```text
1. Read BUILD_STATE.
2. Read active Phase Manifest.
3. Read active Task Contract.
4. Read latest STOP_REPORT.
5. Inspect actual repository.
6. Inspect latest checkpoint/commit.
7. Run minimum relevant tests.
8. Compare actual state with recorded state.
9. Reconcile discrepancies.
10. Determine exact unfinished work.
11. Run task pre-flight again.
12. Resume only the authorized task.
13. Verify.
14. Update BUILD_STATE.
15. Checkpoint if accepted.
16. Stop at the next safe boundary.
```

It must never:

- repeat an accepted task blindly;
- overwrite current code from memory;
- assume the previous agent was correct;
- skip verification;
- continue into the next task without authorization.

---

# 21. Task Recovery Decision Table

| Actual condition | Required action |
|---|---|
| Task incomplete, workspace consistent | Continue |
| Implementation complete, unverified | Verify |
| Verified, BUILD_STATE stale | Reconstruct from evidence, then update |
| Repository differs unexpectedly | Stop and reconcile |
| Active process state unknown | Reconcile first |
| User change conflicts with task | Block / human decision |
| Required dependency unavailable | Block / dependency request |
| Acceptance unclear | Block / contract correction |
| Architecture contradiction | Architecture-change protocol |
| Out-of-scope file required | Contract amendment |
| Test failure caused by task | Fix within scope or fail task |
| Network/provider transient failure | Classify; retry only under policy |
| Potential external side effect occurred but outcome unknown | Reconcile before retry |
| Context limit reached | Safe stop and resume later |

---

# 22. Cancellation Contract

Cancellation is not equivalent to failure.

If a task is cancelled:

```text
CANCELLED
```

must preserve:

- work already performed;
- evidence already collected;
- current repository state;
- checkpoint if valid;
- reason for cancellation;
- whether partial changes remain;
- whether cleanup was verified.

Never erase the audit trail to make cancellation look like non-execution.

---

# 23. External Side-Effect Rule

For tasks capable of causing external side effects:

```text
NEVER:
ambiguous outcome
    ↓
blind retry
```

Instead:

```text
ambiguous outcome
    ↓
reconcile
    ↓
determine whether side effect occurred
    ↓
apply idempotency policy
    ↓
only then retry / repair / hand off
```

This is especially important for:

- provider calls,
- Git operations,
- filesystem mutation,
- process execution,
- external APIs,
- persistence outbox operations.

---

# 24. Verification Authority Rule

No Task Contract may authorize a shortcut around the verification engine.

In particular:

```text
MODEL RESPONSE       ─┐
AIDER OUTPUT         ─┤
PROVIDER RESPONSE    ─┤
RECONCILIATION       ─┤
UI                   ─┤── cannot declare Succeeded
SKILL                ─┤
TASK CONTRACT        ─┘
                       ↓
               VERIFICATION ENGINE
                       ↓
                  Succeeded
```

The Task Contract may define what evidence the Verification Engine must evaluate; it cannot itself fabricate the result.

---

# 25. Security Boundary

Task Contracts must explicitly prevent accidental secret exposure.

Forbidden evidence includes:

- API keys;
- refresh tokens;
- OAuth credentials;
- environment secrets;
- private user data;
- credential-bearing provider requests.

Diagnostics and verification artifacts must use redaction or safe identifiers.

A task that would require secret material in logs is blocked until a safe evidence method exists.

---

# 26. Architecture Integrity Obligations

Before acceptance of any task touching core reliability behavior, verify:

```text
[ ] AttemptStatus remains distinct from ReconciliationOutcome
[ ] Succeeded remains Verification-Engine-only
[ ] Conflict cannot silently overwrite work
[ ] Two-phase merge remains intact
[ ] Event-chain integrity remains intact
[ ] Evidence provenance/freshness remains intact
[ ] Cancellation remains separately represented
[ ] External-side-effect idempotency remains intact
[ ] Correlation-ID semantics remain intact
[ ] Clock abstraction remains respected
[ ] Capability enforcement remains at the tool boundary
[ ] Persistence guarantees remain intact
[ ] Credentials remain outside logs/events/diagnostics
[ ] Recovery budget semantics remain intact
[ ] Multi-agent parallelism has not been introduced without authorization
```

A functional test pass does not override an architecture-integrity failure.

---

# 27. Task Completion Record

The final task result should be represented as:

```yaml
result:
  status: ACCEPTED
  task_id: P?-W??-T??
  contract_version: 1

  implementation:
    status: complete
    changed_files: []

  verification:
    status: passed
    tests_run: []
    failures: []

  acceptance:
    status: passed
    criteria:
      AC-01: PASS

  evidence:
    - type: test_output
      reference: "..."
    - type: diff
      reference: "..."

  checkpoint:
    status: recorded
    id: "..."

  build_state:
    status: updated

  next_authorized_action:
    task: "..."
```

This result belongs in the verification/reporting layer; it does not replace the original immutable contract.

---

# 28. Canonical Task Contract Template

Use this template when creating a new executable task:

```yaml
contract_version: 1

id: P0-W00-T00
phase: P0

status: DRAFT

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §..."
  - "Build Protocol §..."
  - "Phase Manifest §..."

objective: >
  ...

allowed_files:
  - "..."

forbidden_files:
  - "..."

allowed_artifacts:
  - "..."

forbidden_artifacts:
  - "..."

allowed_commands:
  - "..."

forbidden_commands:
  - "..."

dependencies:
  - id: "..."
    reason: "..."
    status: available

preconditions:
  - "..."

inputs:
  - "..."

outputs:
  - "..."

implementation_requirements:
  - "..."

invariants_preserved:
  - "..."

invariants_proven:
  - "..."

required_tests:
  - level: deterministic
    test: "..."
  - level: simulation
    test: "..."

failure_scenarios:
  - trigger: "..."
    expected_state: "..."
    expected_classification: "..."
    expected_recovery: "..."

acceptance:
  - id: AC-01
    criterion: "..."
    verification_method: "..."
    evidence_required: "..."
    status: pending

non_goals:
  - "..."

risk_level: medium

resource_policy:
  network_required: false
  real_provider_allowed: false
  preferred_test_level: deterministic
  max_retry_policy: "per Build Protocol"
  context_budget: minimal

done_when:
  - "..."
  - "..."
  - "..."

checkpoint_requirements:
  - "..."

resume_requirements:
  - "..."

stop_conditions:
  - "..."

escalation_conditions:
  - "..."
```

---

# 29. Phase 0 Initial Task Contract Registry

The Phase Manifest defines these initial planning identifiers:

```text
P0-W01-T01  Attempt/Reconciliation contract
P0-W02-T01  Evidence provenance/freshness contract
P0-W03-T01  Checkpoint contract
P0-W04-T01  Event-chain contract
P0-W05-T01  Failure taxonomy contract
P0-W06-T01  Skill contract
P0-W07-T01  Capability table + tool boundary contract
P0-W08-T01  Mission Acceptance Contract
P0-W09-T01  Two-phase merge + Git edge-case contract
P0-W10-T01  Mission/Step state-machine contract
P0-W11-T01  External-side-effect idempotency contract
P0-W12-T01  Clock trait contract
P0-W13-T01  Phase 1 task-contract preparation
P0-W14-T01  Phase 1 verification-gate preparation
```

These identifiers are planning entries only until the corresponding contracts are instantiated and authorized.

Phase 0 remains explicitly no-code. These tasks establish the contracts needed for safe implementation; they do not authorize implementation of later phases.

---

# 30. Phase 0 Contract Construction Order

OX Alpha should instantiate the Phase 0 contracts in dependency-aware order:

```text
P0-W01 Attempt / Reconciliation
        ↓
P0-W02 Evidence provenance / freshness
        ↓
P0-W03 Checkpoint
        ↓
P0-W04 Event chain
        ↓
P0-W05 Failure taxonomy
        ↓
P0-W06 Skill contract
        ↓
P0-W07 Capability / tool boundary
        ↓
P0-W08 Mission Acceptance
        ↓
P0-W09 Two-phase merge / Git edge cases
        ↓
P0-W10 Mission / Step state machines
        ↓
P0-W11 External-side-effect idempotency
        ↓
P0-W12 Clock trait
        ↓
P0-W13 Phase 1 task-contract preparation
        ↓
P0-W14 Phase 1 verification-gate preparation
```

If a dependency is discovered that is not represented in this order, the agent must document it rather than silently inventing a new architecture.

---

# 31. Example — Small Deterministic Contract

```yaml
contract_version: 1
id: P0-W12-T01
phase: P0

status: DRAFT

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §28 Implementation Reliability & Acceleration Layer"
  - "Build Protocol §16 Test Protocol"
  - "Build Protocol §9 Task Contract Protocol"

objective: >
  Define the injectable Clock contract required to prevent production wall-clock
  access from contaminating deterministic kernel behavior.

allowed_files:
  - ".ai-conductor-build/task_contracts/P0-W12-T01.yaml"
  - ".ai-conductor-build/verification_reports/P0-W12-T01.md"

forbidden_files:
  - "src/**"
  - "apps/**"
  - "ui/**"

allowed_artifacts:
  - ".ai-conductor-build/verification_reports/P0-W12-T01.md"

forbidden_artifacts:
  - ".env"
  - "*.secret"

allowed_commands:
  - "repository inspection commands"
  - "contract validation commands"

forbidden_commands:
  - "production execution commands"

dependencies: []

preconditions:
  - "Blueprint v2.4 is available."
  - "Build Protocol v1.1 is available."
  - "Phase 0 is active."

inputs:
  - "Blueprint clock requirements."

outputs:
  - "Explicit Clock contract."
  - "Contract verification evidence."

implementation_requirements:
  - "The contract must distinguish production clock access from deterministic test time."
  - "The contract must not authorize implementation of later phases."

invariants_preserved:
  - "Deterministic kernel behavior."
  - "Injected-clock hard rule."

invariants_proven:
  - "Clock access is explicitly contract-bound."

required_tests:
  - level: deterministic
    test: "Validate required clock contract fields."

failure_scenarios:
  - trigger: "Wall-clock dependency appears in a deterministic contract."
    expected_state: "BLOCKED"
    expected_classification: "Scope/architecture violation"
    expected_recovery: "Correct the contract or escalate."

acceptance:
  - id: AC-01
    criterion: "Clock contract explicitly defines the injectable boundary."
    verification_method: "requirement_check"
    evidence_required: "Rendered contract and verification report."
    status: pending

non_goals:
  - "Implement Clock trait."
  - "Modify runtime code."
  - "Introduce a time library."

risk_level: medium

resource_policy:
  network_required: false
  real_provider_allowed: false
  preferred_test_level: deterministic
  max_retry_policy: "none"
  context_budget: minimal

done_when:
  - "Contract is internally coherent."
  - "No implementation files were modified."
  - "Acceptance criteria pass."
  - "Verification report exists."
  - "BUILD_STATE is updated."

checkpoint_requirements:
  - "Record contract artifact and verification result."

resume_requirements:
  - "Reload this contract and verify its version."

stop_conditions:
  - "Blueprint requirement is ambiguous."
  - "Clock semantics conflict with another authoritative rule."

escalation_conditions:
  - "Architecture change appears necessary."
```

This example demonstrates the principle that **a task contract can be useful even when the task itself is specification work**.

---

# 32. Example — Implementation Contract Pattern

A normal implementation task should look like:

```yaml
contract_version: 1
id: P1-W03-T04
phase: P1

status: READY

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §..."
  - "Build Protocol §10 One Invariant → One Slice → One Gate"

objective: >
  Implement one bounded state-transition slice and prove its legal and illegal
  transitions without modifying unrelated state behavior.

allowed_files:
  - "src/kernel/..."
  - "tests/kernel/..."

forbidden_files:
  - "src/ui/**"
  - "provider/**"
  - "production/**"

allowed_artifacts:
  - ".ai-conductor-build/verification_reports/..."

allowed_commands:
  - "..."

forbidden_commands:
  - "destructive repository reset commands"

dependencies:
  - id: "previous accepted state-model task"
    reason: "..."
    status: available

preconditions:
  - "Previous task is ACCEPTED."
  - "Repository checkpoint matches BUILD_STATE."

inputs:
  - "Current state model."
  - "Relevant transition rules."

outputs:
  - "Implementation slice."
  - "Deterministic tests."

implementation_requirements:
  - "Only legal transitions are accepted."
  - "Illegal transitions fail explicitly."
  - "No unrelated state semantics change."

invariants_preserved:
  - "..."

invariants_proven:
  - "..."

required_tests:
  - level: deterministic
    test: "legal transition matrix"
  - level: deterministic
    test: "illegal transition matrix"
  - level: simulation
    test: "restart/replay if required"

failure_scenarios:
  - trigger: "unexpected transition request"
    expected_state: "unchanged"
    expected_classification: "invalid transition"
    expected_recovery: "none / caller correction"

acceptance:
  - id: AC-01
    criterion: "All defined legal transitions pass."
    verification_method: "deterministic_test"
    evidence_required: "test output"
    status: pending

  - id: AC-02
    criterion: "All defined illegal transitions are rejected."
    verification_method: "deterministic_test"
    evidence_required: "test output"
    status: pending

non_goals:
  - "Provider integration."
  - "UI implementation."
  - "Unrelated refactoring."

risk_level: high

resource_policy:
  network_required: false
  real_provider_allowed: false
  preferred_test_level: deterministic
  max_retry_policy: "per protocol"
  context_budget: minimal

done_when:
  - "Implementation is within allowed files."
  - "Required tests pass."
  - "Acceptance criteria pass."
  - "Architecture integrity audit passes."
  - "Checkpoint is recorded."
  - "BUILD_STATE is updated."
```

---

# 33. Task Contract Quality Gate

Before a Task Contract becomes `READY`, validate:

```text
IDENTITY
[ ] Unique task ID
[ ] Correct phase
[ ] Correct contract version

AUTHORITY
[ ] Blueprint version pinned
[ ] Protocol version pinned
[ ] Manifest version pinned
[ ] Relevant section references present

SCOPE
[ ] Objective is singular and bounded
[ ] allowed_files explicit
[ ] forbidden_files explicit where needed
[ ] allowed_commands explicit
[ ] forbidden_commands explicit
[ ] non-goals explicit

EXECUTION
[ ] Preconditions are verifiable
[ ] Inputs are sufficient
[ ] Outputs are concrete
[ ] Implementation requirements are testable
[ ] Dependencies are identified

PROOF
[ ] Required tests are executable
[ ] Failure scenarios identified where applicable
[ ] Every acceptance criterion has a verification method
[ ] Evidence requirements are explicit
[ ] done_when is finite

SAFETY
[ ] Risk level assigned
[ ] Resource policy defined
[ ] Secret boundary considered
[ ] External side effects considered
[ ] Stop conditions defined
[ ] Escalation conditions defined
[ ] Checkpoint requirements defined
[ ] Resume requirements defined
```

If a mandatory item is missing:

```text
DRAFT
```

not `READY`.

---

# 34. Task Contract Execution Gate

Before OX Alpha may mutate the repository:

```text
TASK STATUS = READY
        +
PHASE = AUTHORIZED
        +
PRECONDITIONS = PASS
        +
SCOPE = VALID
        +
DEPENDENCIES = AVAILABLE
        +
ACCEPTANCE = EXPLICIT
        +
REQUIRED TESTS = KNOWN
        ↓
AUTHORIZE IMPLEMENTATION
```

If any required component is `UNKNOWN`:

```text
BLOCKED
```

unless the contract explicitly identifies that unknown as the subject of investigation.

---

# 35. Task Acceptance Gate

A task can become `ACCEPTED` only if:

```text
[ ] Implementation requirements satisfied
[ ] No unauthorized file changed
[ ] No unauthorized dependency added
[ ] Required tests executed
[ ] Required tests passed
[ ] Required failure scenarios passed
[ ] Acceptance criteria all PASS
[ ] Evidence is real and traceable
[ ] No unresolved critical unknown remains
[ ] Architecture integrity passes
[ ] Checkpoint recorded
[ ] BUILD_STATE updated
```

The following are insufficient:

```text
"Looks good."
"Build succeeded."
"Tests were written."
"Model says done."
"Provider returned 200."
"UI appears correct."
```

---

# 36. Task Failure Gate

A task is `FAILED` when:

- required acceptance cannot be met within scope;
- implementation introduced a verified regression;
- required proof fails;
- an invariant is violated;
- unauthorized scope was changed and cannot be safely corrected;
- security boundary was violated;
- the task cannot safely proceed.

Failure must preserve evidence.

A failed task is not deleted merely because a later task fixes the problem.

---

# 37. Relationship to Phase Gates

Task acceptance is necessary but not sufficient for phase completion.

```text
ALL REQUIRED TASKS ACCEPTED
        +
PHASE VERIFICATION GATE PASSED
        +
ARCHITECTURE INTEGRITY AUDIT PASSED
        +
BUILD_STATE UPDATED
        +
CHECKPOINT RECORDED
        ↓
PHASE MAY ADVANCE
```

A task must never self-authorize phase advancement.

---

# 38. Relationship to Vertical Proof Slices

Task Contracts should support the Blueprint's vertical proof slices.

For example:

```text
VS0
minimal deterministic mission
→ multiple bounded tasks
→ one vertical proof gate

VS1
partial change + repair
→ bounded execution/reconciliation/recovery tasks

VS2
real Aider
→ isolation/execution/evidence/verification tasks

VS3
real provider
→ provider/resource/integration tasks

VS4
handoff
→ handoff/recovery/continuation tasks

VS5
gstack
→ skill/capability/runtime tasks
```

The Task Contract remains the atomic execution unit even when several contracts form one vertical slice.

---

# 39. Failure-Corpus Integration

If a task discovers a meaningful bug:

```text
task
  ↓
failure discovered
  ↓
classify
  ↓
capture deterministic scenario
  ↓
add regression case
  ↓
fix
  ↓
rerun regression
  ↓
accept task
```

The failure case should preserve:

```text
SCENARIO
INITIAL STATE
TRIGGER
OBSERVED FAILURE
EXPECTED BEHAVIOR
REGRESSION TEST
```

Never remove a regression safeguard merely because the current code now passes.

---

# 40. Context-Minimization Rule

The Task Contract is intentionally designed to reduce repeated context transmission.

When executing a task, OX Alpha should prioritize:

```text
1. current Task Contract
2. relevant Blueprint sections
3. relevant Build Protocol sections
4. relevant Phase Manifest section
5. current BUILD_STATE
6. relevant STOP_REPORT
7. relevant failure cases
8. actual repository files in scope
```

Do not repeatedly reread the entire project when the task can be safely understood from its bounded authoritative context.

This is a reliability and resource-efficiency measure, not permission to omit necessary context.

---

# 41. No Prompt-Only Safety

Task Contracts are not merely prompts.

Where a rule can be mechanically enforced, the eventual implementation should enforce it.

Examples:

```text
allowed_files
→ scope checker

capability requirements
→ tool-boundary enforcement

acceptance criteria
→ verification engine

task status
→ state machine

checkpoint requirement
→ checkpoint gate

phase authorization
→ phase gate

dependency approval
→ dependency gate
```

Prompt instructions alone are weaker than runtime enforcement.

---

# 42. Task Contract Immutability

Once a task becomes `IN_PROGRESS`, its original scope and acceptance semantics should be treated as an audit record.

If material changes are required:

```text
create amendment
or
create successor task
```

Do not silently rewrite history.

A verification report must identify the exact contract version it evaluated.

---

# 43. Human Decision Boundary

The human project owner is required when a decision affects:

- Blueprint architecture;
- invariants;
- state semantics;
- security boundaries;
- destructive recovery;
- unresolved ambiguity with material consequences;
- scope beyond the current task;
- new runtime dependency where approval is required;
- acceptance that cannot be objectively proven;
- unresolved conflict between authoritative sources.

The agent should not manufacture a decision simply to maintain momentum.

---

# 44. OX Alpha Operating Instruction

When given a Task Contract, OX Alpha must follow this exact operating posture:

> You are executing one bounded AI Conductor Task Contract.
>
> The Master Blueprint is the architecture authority.
>
> The Build Protocol is the implementation procedure.
>
> The Phase Manifest is the phase authorization.
>
> This Task Contract is the current execution boundary.
>
> Do not invent architecture.
>
> Do not expand scope silently.
>
> Do not modify files outside the allowed surface.
>
> Do not add dependencies without authorization.
>
> Do not treat model confidence as evidence.
>
> Do not claim completion without executed verification.
>
> Do not retry ambiguous external side effects blindly.
>
> Do not bypass the Verification Engine.
>
> Preserve all existing verified behavior and invariants.
>
> Prefer the cheapest reliable proof first.
>
> Use deterministic tests before simulation and simulation before real integrations where applicable.
>
> When something is unknown, mark it UNKNOWN and investigate.
>
> When something is blocked, remain BLOCKED rather than guessing.
>
> When the task is complete, verify it, produce evidence, checkpoint it, update BUILD_STATE, and stop.
>
> If interrupted, leave a precise stop state so another agent can resume without relying on your memory.
>
> If architecture or scope cannot be reconciled safely, stop and escalate.

---

# 45. "Continue According to the Blueprint" Task Instruction

When the human says:

> **Continue according to the blueprint.**

the Task Contract layer interprets this as:

```text
RECONSTRUCT
    ↓
VALIDATE
    ↓
RESUME EXACT UNFINISHED TASK
    ↓
VERIFY
    ↓
CHECKPOINT
    ↓
UPDATE STATE
    ↓
STOP
```

It does **not** mean:

```text
start over
guess what was intended
build whatever seems useful
skip the current gate
move to the next phase
```

The repository and recorded evidence are stronger than conversational memory.

---

# 46. Final Task Contract Principles

The Task Contract system exists to make OX Alpha **fast because the work is bounded**, not fast because safeguards are skipped.

The ideal task is:

```text
SMALL
  +
EXPLICIT
  +
AUTHORIZED
  +
TESTABLE
  +
REPLAYABLE
  +
FAILURE-AWARE
  +
RESOURCE-EFFICIENT
  +
CHECKPOINTABLE
  +
RESUMABLE
```

The objective is not to create thousands of bureaucratic documents.

The objective is to ensure that every meaningful mutation has:

```text
A KNOWN PURPOSE
A KNOWN SCOPE
A KNOWN AUTHORITY
A KNOWN PROOF
A KNOWN FAILURE BOUNDARY
A KNOWN RECOVERY PATH
A KNOWN CHECKPOINT
A KNOWN NEXT ACTION
```

That is what allows an AI coding agent to work continuously while remaining bounded, auditable, recoverable, and resistant to hallucinated completion.

---

# 47. Final Acceptance Checklist — This File

This Task Contract specification is considered ready for use when:

```text
[ ] Blueprint v2.4 is identified as architecture authority.
[ ] Build Protocol v1.1 is identified as procedure authority.
[ ] Phase Manifest v1.1 is identified as phase authority.
[ ] Task Contracts are explicitly subordinate to all three.
[ ] Every task has a bounded identity.
[ ] Scope fences are explicit.
[ ] Allowed and forbidden surfaces are represented.
[ ] Preconditions and dependencies are explicit.
[ ] Test levels are defined.
[ ] Failure scenarios are represented.
[ ] Acceptance criteria require explicit verification methods.
[ ] Evidence requirements are explicit.
[ ] Risk determines proof strength.
[ ] Resource efficiency is part of the contract.
[ ] Stop/resume semantics are explicit.
[ ] Cancellation is distinct from failure.
[ ] External side effects require reconciliation/idempotency.
[ ] Verification Engine remains the authority for Succeeded.
[ ] Architecture changes cannot be silently introduced.
[ ] Dependency changes cannot be silently introduced.
[ ] Checkpointing is part of acceptance.
[ ] BUILD_STATE is updated after acceptance.
[ ] Phase advancement remains outside task authority.
[ ] Vertical proof slices remain supported.
[ ] Failure-corpus integration remains supported.
[ ] Context minimization is explicitly encouraged without weakening proof.
[ ] Initial Phase 0 task IDs align with the Phase Manifest.
[ ] No task contract authorizes later-phase implementation.
[ ] No claim of correctness can be based solely on model output.
```

---

## Closing Principle

The Task Contract is the **execution atom** of AI Conductor.

The Blueprint says:

> **Build the right system.**

The Build Protocol says:

> **Build it reliably.**

The Phase Manifest says:

> **Build only what is authorized now.**

The Task Contract says:

> **Here is the exact smallest thing you may change, the exact boundary you must not cross, and the exact evidence you must produce before calling it done.**

Therefore:

```text
BLUEPRINT
   ↓
PROTOCOL
   ↓
PHASE
   ↓
TASK CONTRACT
   ↓
PREFLIGHT
   ↓
SMALLEST SAFE CHANGE
   ↓
TEST
   ↓
BREAK
   ↓
RECOVER
   ↓
VERIFY
   ↓
ACCEPT
   ↓
CHECKPOINT
   ↓
BUILD STATE
   ↓
NEXT TASK
```

**Fast execution comes from bounded work, deterministic proof, minimal context, safe automation, and immediate recovery—not from skipping controls.**

---

**END — AI CONDUCTOR TASK CONTRACTS v1.1**


---

# 39. Phase 5 Harness-Neutral Task Contract Registry Amendment

These contracts are the first executable contract set for the v2.5.1 Execution Harness architecture. They intentionally do not alter accepted P1/P2 work.

## P5-W01-T01 — Execution Adapter Contract

**Objective:** Implement the common Conductor-to-executor semantic boundary.

**Allowed scope:** execution adapter types/interfaces, lifecycle model, conformance fixtures and directly required tests.

**Must prove:**

```text
identity
capabilities
start
observe
cancel
collect evidence
finalize
```

**Must NOT:**

- create Succeeded directly;
- merge into the live workspace;
- bypass capability enforcement;
- assume ambiguous outcome is failure.

## P5-W02-T01 — Executor Capability Snapshot

**Objective:** Represent executor capabilities with version/commit, provenance and freshness.

**Must prove:** stale/unverified capability is not silently treated as current verified capability.

## P5-W03-T01 — Reference Executor Isolation

**Objective:** Run one supported real executor inside a Conductor-created isolated worktree.

**Must prove:** live workspace remains untouched during execution and rejected/failed attempts do not bypass isolation.

## P5-W04-T01 — Real Executor Lifecycle

**Objective:** Observe successful execution, failure, timeout, cancellation, crash and ambiguous termination.

**Must prove:** ambiguous outcomes become Unknown and enter reconciliation.

## P5-W05-T01 — Restart and Reconciliation

**Objective:** Resume safely after an executor interruption/restart.

**Must prove:** actual workspace state is reconciled before any continuation and accepted side effects are not blindly repeated.

## P5-W06-T01 — Aider Conformance

**Objective:** Validate retained Aider integration against the common Executor Conformance Suite.

**Condition:** only if Aider is retained as a supported executor.

## P5-W07-T01 — Jcode Evaluation

**Objective:** Evaluate the installed/version-pinned Jcode implementation against the same Executor Conformance Suite.

**Important:** this task evaluates Jcode; it does not assume Jcode is superior to Aider and does not make Jcode mandatory.

## P5-W08-T01 — Executor Selection / Reversibility

**Objective:** Select an eligible executor using explainable criteria without modifying reliability-kernel semantics.

**Must prove:** a different executor can execute the same bounded task contract without architectural rewrite.

### Common P5 non-goals

Do not add:

- broad swarm orchestration;
- learned routing;
- final UI;
- permanent provider lock-in;
- harness-specific acceptance authority.
