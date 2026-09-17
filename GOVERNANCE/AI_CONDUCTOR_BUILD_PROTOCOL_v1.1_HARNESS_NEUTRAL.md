# AI CONDUCTOR
## Build Protocol — v1.1
### Operational Protocol for AI-Assisted Implementation — Harness-Neutral v1.1

**Authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`  
**Purpose:** Control how an AI coding agent (for example OX Alpha in Trae) implements, verifies, pauses, resumes, and evolves AI Conductor.

---

## 0. Document Status

| Field | Value |
|---|---|
| Protocol | AI Conductor Build Protocol |
| Version | 1.0 |
| Architecture authority | Master Blueprint v2.5.1 |
| Primary implementation agent | OX Alpha / equivalent coding agent |
| Human authority | Project owner |
| Scope | Entire AI Conductor implementation lifecycle |
| Status | Operational specification |

### Governing relationship

This document **does not replace** the Master Blueprint.

- **Master Blueprint v2.5.1** defines **what AI Conductor must be**.
- **This Build Protocol** defines **how an AI coding agent must build it**.
- **Phase Manifests** define **what is currently allowed to be built**.
- **Task Contracts** define **the exact current unit of work**.
- **Verification Gates** define **what proves the unit is complete**.
- **Build State** defines **where implementation currently stands**.
- **Failure Corpus** records failures that must never be forgotten.

When this protocol appears to conflict with the Master Blueprint, the agent must **stop and escalate**. It must not silently choose one.

---

# 1. Mission of the Build Process

The objective is not merely to produce compiling code.

The objective is to build a **lightweight, reliable, free-first AI engineering control plane** that combines:

- gstack as the engineering process/skill layer,
- Aider as an execution layer,
- OmniRoute as provider connectivity/routing infrastructure,
- AI Conductor as the control, state, reliability, recovery, verification, and resource-efficiency layer.

The system must remain faithful to the core product promise:

> **Reliable orchestration for unreliable AI resources.**

The resulting tool must minimize:

- lost work,
- duplicated work,
- silent overwrites,
- false completion,
- wasted provider quota,
- unnecessary context transmission,
- uncontrolled retries,
- architectural drift,
- unnecessary dependencies,
- manual recovery burden.

The implementation process itself must follow the same philosophy.

---

# 2. Non-Negotiable Agent Principles

These rules apply to every implementation session.

## 2.1 Blueprint authority

The Master Blueprint v2.5.1 is authoritative for architecture, invariants, state semantics, boundaries, and intended sequencing.

The agent must not silently invent an alternative architecture.

## 2.2 Model is not authority

The coding agent is an implementer and proposer, not the authority on correctness.

The agent may:

- propose an implementation,
- propose a refactor,
- propose a test,
- identify a missing dependency,
- identify a contradiction.

The agent may not silently declare an architectural rule invalid or change it.

## 2.3 No fake completion

The agent must never claim a task is complete because:

- the code looks complete,
- the files exist,
- compilation is assumed,
- a command was not actually run,
- a model said it worked,
- a previous session claimed it worked,
- documentation says it works.

A task is complete only when its task contract and verification gate pass.

## 2.4 Unknown is better than invented certainty

When something is not known, use an explicit `UNKNOWN` or `BLOCKED` state and investigate.

Never convert an assumption into a fact merely to keep progress moving.

## 2.5 Evidence before advancement

Every major implementation step must produce evidence appropriate to the task.

The stronger the risk, the stronger the evidence requirement.

## 2.6 Smallest safe change

Implement the smallest change that satisfies the current contract.

Do not make unrelated cleanup, aesthetic refactors, dependency upgrades, architecture changes, or speculative abstractions during a bounded task.

## 2.7 No silent scope expansion

If implementation requires a file, dependency, API, capability, or architectural change outside the task contract, stop and record it as a scope issue.

Do not silently expand scope.

## 2.8 Preserve working behavior

Existing passing tests, invariants, and verified behavior are protected assets.

A new task must not knowingly break them unless the task explicitly changes the contract and the change has been reviewed.

## 2.9 Never optimize away evidence

Do not remove tests, logs, safeguards, or validation merely because they slow development.

If a safeguard is expensive, measure it first and propose a justified optimization.

## 2.10 Build the core before the decoration

Reliability kernel, state, persistence, reconciliation, verification, isolation, and testing take priority over polished UI.

---

# 3. The Source-of-Truth Stack

The build is governed by a layered source of truth.

```text
MASTER BLUEPRINT v2.5.1
        ↓
BUILD PROTOCOL
        ↓
PHASE MANIFEST
        ↓
TASK CONTRACT
        ↓
VERIFICATION GATE
        ↓
BUILD STATE
        ↓
ACTUAL REPOSITORY + TEST EVIDENCE
```

The agent must resolve discrepancies upward, not downward.

### Priority order

1. Verified actual repository state and executed evidence
2. Human-approved architecture change decisions
3. Master Blueprint
4. Current Phase Manifest
5. Current Task Contract
6. Previous agent statements / chat history
7. Agent assumptions

Conversation memory is never stronger evidence than the repository or recorded state.

---

# 4. Build State Is the Memory of the Project

The project must maintain a machine-readable build state, for example:

```text
.ai-conductor-build/
    BUILD_STATE.json
    STOP_REPORT.md
    architecture_changes/
    dependency_requests/
    phase_manifests/
    task_contracts/
    verification_reports/
```

`BUILD_STATE.json` is the authoritative implementation-progress record.

### Minimum fields

```json
{
  "protocol_version": "1.0",
  "blueprint_version": "2.4",
  "current_phase": "P1",
  "current_task": "P1-S03-T04",
  "task_status": "in_progress",
  "completed_tasks": [],
  "blocked_tasks": [],
  "failed_tasks": [],
  "last_verified_checkpoint": "...",
  "last_verified_commit": "...",
  "tests_passed": 0,
  "tests_failed": 0,
  "known_issues": [],
  "known_unknowns": [],
  "pending_decisions": [],
  "next_authorized_action": "..."
}
```

### Build-state rules

- Update after every accepted task.
- Update before a safe stop.
- Update after any failed gate.
- Update after an architecture-change request.
- Never mark a task accepted without its gate.
- Never delete historical failure information merely to make the state cleaner.

---

# 5. The Agent Session Contract

Every OX Alpha session follows the same lifecycle.

```text
BOOT
 ↓
READ
 ↓
INSPECT
 ↓
VALIDATE
 ↓
IMPLEMENT
 ↓
TEST
 ↓
VERIFY
 ↓
CHECKPOINT
 ↓
UPDATE BUILD STATE
 ↓
STOP AT SAFE BOUNDARY
```

An agent must never jump directly from `BOOT` to large-scale implementation.

---

# 6. Session Boot Protocol

At the beginning of every session, the agent must read:

1. `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`
2. `AI_CONDUCTOR_BUILD_PROTOCOL_v1.1_HARNESS_NEUTRAL.md`
3. `BUILD_STATE.json`
4. the active Phase Manifest
5. the active Task Contract
6. the most recent STOP_REPORT, if present
7. relevant failure-corpus entries
8. relevant verification reports

Then it must inspect the actual repository state.

### Required boot report

Before changing code, produce internally or in the session log:

```text
BLUEPRINT VERSION: 2.4
PROTOCOL VERSION: 1.0
CURRENT PHASE: ...
CURRENT TASK: ...
TASK STATUS: ...
LAST VERIFIED CHECKPOINT: ...
LAST VERIFIED COMMIT: ...
KNOWN FAILURES: ...
KNOWN UNKNOWNS: ...
ALLOWED FILES: ...
FORBIDDEN FILES: ...
REQUIRED TESTS: ...
NEXT AUTHORIZED ACTION: ...
```

If these values cannot be reconstructed reliably, the agent must stop and reconcile the build state before coding.

---

# 7. Continue Protocol

When the human says:

> **Continue according to the blueprint.**

the agent must interpret that as a formal command, not as a conversational invitation to guess.

## Continue sequence

```text
1. Read BUILD_STATE
2. Read current Phase Manifest
3. Read current Task Contract
4. Read latest STOP_REPORT
5. Inspect repository
6. Inspect latest checkpoint/commit
7. Run the minimum relevant existing tests
8. Compare actual state against recorded state
9. Reconcile any discrepancy
10. Determine exact unfinished work
11. Run preflight
12. Resume only the authorized task
13. Verify
14. Update BUILD_STATE
15. Checkpoint if the gate passes
16. Stop at the next safe boundary
```

### Continue must never mean

- repeat the entire previous task,
- reread the entire conversation and guess,
- overwrite current code with an earlier mental model,
- restart a successful attempt blindly,
- skip the current task's verification gate.

### Resume decision table

| Observed state | Required action |
|---|---|
| Task incomplete, workspace consistent | Continue task |
| Task implementation complete but unverified | Run verification |
| Task verified but build state stale | Reconstruct state from evidence, then record |
| Repository differs unexpectedly | Stop and reconcile |
| Architecture changed externally | Stop; architecture-change protocol |
| Active process unknown | Reconciliation first |
| User/AI conflict detected | Block; human decision |
| Required dependency unavailable | Block; dependency request |
| Acceptance criteria unclear | Block; contract correction |

---

# 8. Safe Stop Protocol

The agent must be able to stop cleanly at any time.

A safe stop is preferred over speculative continuation.

Before stopping, the agent must leave:

### STOP_REPORT.md

```text
AI CONDUCTOR BUILD STOP REPORT

Blueprint: 2.4
Protocol: 1.0
Phase: ...
Task: ...

STATUS:
- complete / incomplete / blocked / failed / unknown

COMPLETED:
- ...

CHANGED FILES:
- ...

TESTS RUN:
- ...

TESTS PASSED:
- ...

TESTS FAILED:
- ...

KNOWN ERRORS:
- ...

KNOWN UNKNOWNS:
- ...

LAST VERIFIED CHECKPOINT:
- ...

LAST VERIFIED COMMIT:
- ...

NEXT EXACT ACTION:
- ...

DO NOT REPEAT:
- ...

FILES INTENTIONALLY NOT TOUCHED:
- ...
```

The goal is that a new agent can continue correctly without relying on the previous agent's memory.

---

# 9. Task Contract Protocol

Every implementation task must have a bounded contract.

### Required structure

```yaml
version: 1
id: P1-S03-T04
phase: P1
section_refs:
  - "Blueprint 4.2"
  - "Blueprint 3.14"

objective: "..."

allowed_files:
  - "..."

forbidden_files:
  - "..."

allowed_commands:
  - "..."

forbidden_actions:
  - "..."

dependencies:
  - "..."

implementation_requirements:
  - "..."

required_tests:
  - "..."

acceptance:
  - "..."

non_goals:
  - "..."

risk_level: low | medium | high | critical

done_when:
  - "..."
```

### Task sizing rule

A task should be small enough that:

- the agent can understand it without reading the whole project repeatedly,
- the task has a finite set of acceptance checks,
- the likely diff is reviewable,
- the task can be resumed after an interruption,
- failure can be localized.

If a task is too large, split it.

---

# 10. One Invariant → One Slice → One Gate

This is the preferred implementation rhythm.

```text
ONE INVARIANT
    ↓
ONE IMPLEMENTATION SLICE
    ↓
DETERMINISTIC TESTS
    ↓
SIMULATOR / CHAOS
    ↓
VERIFICATION GATE
    ↓
CHECKPOINT
    ↓
NEXT INVARIANT
```

Do not implement an entire reliability subsystem before testing its core contract.

---

# 11. Vertical Slice Strategy

The roadmap is phase-based, but implementation should also use vertical proof slices.

## Vertical Slice 0 — Minimal deterministic mission

A tiny fake task must:

```text
create mission
→ create attempt
→ FakeProvider acts
→ mutation occurs in virtual workspace
→ evidence captured
→ verification passes
→ checkpoint created
→ state persisted
→ simulated restart
→ state restored
```

Then inject failures:

- timeout,
- process crash,
- persistence interruption,
- duplicate provider response,
- checkpoint interruption.

No next major slice until this works.

## Vertical Slice 1 — Partial change + repair

Prove:

```text
partial edit
→ reconciliation
→ targeted repair
→ verification
→ checkpoint
```

## Vertical Slice 2 — Real Aider

Prove:

```text
worktree
→ Aider
→ evidence
→ verification
→ accepted merge
```

## Vertical Slice 3 — Real provider

Prove real Gemini/OmniRoute behavior and record real failure cases.

## Vertical Slice 4 — Handoff

Prove:

```text
Gemini
→ failure
→ reconciliation
→ handoff package
→ Mistral
→ continuation
→ verification
```

## Vertical Slice 5 — gstack

Prove a minimal backend skill sequence:

```text
Engineering Review
→ Build
→ Review
→ QA
```

with capability enforcement.

---

# 12. Evidence and Anti-Hallucination Protocol

Every non-trivial claim by the agent must be categorized internally as:

```text
OBSERVED
VERIFIED
DOCUMENTED
INFERRED
UNKNOWN
```

## OBSERVED
Directly seen in the repository, running program, command output, or current provider response.

## VERIFIED
Confirmed by a reproducible test or controlled experiment.

## DOCUMENTED
Confirmed by authoritative project/dependency/provider documentation.

## INFERRED
Reasoned from known evidence but not directly verified.

## UNKNOWN
Not enough evidence.

### Rules

- `UNKNOWN` may not silently become an implementation assumption.
- `INFERRED` may not be presented as `VERIFIED`.
- Provider capabilities must be verified before being relied upon.
- API behavior must not be invented.
- Test output must be real.
- Logs must not be fabricated.
- Benchmarks must not be fabricated.
- Security claims must not exceed the defined threat model.

When a required fact is unknown:

```text
UNKNOWN
→ investigate
→ record evidence
→ update contract if necessary
→ implement
```

---

# 13. Scope Fence Protocol

Every task has an explicit allowed surface.

The agent must not modify:

- files outside `allowed_files`, unless the task contract is updated,
- architecture outside the referenced sections,
- dependencies without a dependency request,
- schemas outside the current schema task,
- UI while implementing kernel work unless specifically required,
- production configuration unless explicitly authorized.

### Scope expansion

When the agent discovers that another file is necessary:

```text
STOP
 ↓
Explain why
 ↓
Identify exact additional file
 ↓
Identify risk
 ↓
Request scope update
 ↓
Resume only after authorization
```

Do not silently expand scope.

---

# 14. Architecture Change Protocol

A coding agent must never silently rewrite the blueprint while implementing it.

When the architecture appears insufficient or contradictory, create:

```text
architecture_changes/AC-XXXX.md
```

with:

```text
TITLE:

CURRENT RULE:

BLUEPRINT REFERENCES:

OBSERVED PROBLEM:

EVIDENCE:

WHY CURRENT DESIGN FAILS:

PROPOSED CHANGE:

ALTERNATIVES CONSIDERED:

INVARIANTS AFFECTED:

STATE MODEL AFFECTED:

TESTS AFFECTED:

SECURITY IMPACT:

PERFORMANCE IMPACT:

RECOMMENDATION:

STATUS: pending human decision
```

Until approved, implementation must remain within the existing contract whenever possible.

---

# 15. Dependency Change Protocol

No new runtime dependency is added casually.

For every proposed dependency, create:

```text
DEPENDENCY REQUEST

name:
version:
purpose:
where used:
why standard library/current dependencies are insufficient:
size/footprint:
license:
maintenance health:
security considerations:
alternatives:
reason it is necessary now:
```

### Kernel rule

The Conductor Kernel remains deliberately lightweight and dependency-stable.

A convenience dependency must not enter the kernel merely because it saves a small amount of code.

---

# 16. Test Protocol

Every task must state which test level applies.

## Level 1 — Deterministic

Use for:

- state transitions,
- failure classification,
- reconciliation,
- acceptance evaluation,
- command generation,
- resource scoring,
- capability decisions.

No network. No real time. No real provider.

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

## Level 3 — Real integration

Use:

- actual Aider,
- actual Git,
- actual OmniRoute,
- actual Gemini/Mistral,
- real OS process behavior.

Level 3 tests are fewer and more expensive.

### Test order

```text
cheapest deterministic test
→ simulation
→ real integration
```

Never use a real provider to discover logic that could have been caught deterministically.

---

# 17. Failure Injection Protocol

Every reliability feature must have an intentional failure test.

Required fault classes include:

### Process

- application restart,
- Aider crash,
- Conductor crash,
- child-process termination.

### Network

- timeout,
- disconnect,
- delayed response.

### Provider

- 429,
- quota exhausted,
- 5xx,
- malformed response,
- invalid tool request.

### Execution

- partial edit,
- unexpected exit,
- tool failure.

### Persistence

- state write failure,
- event interruption,
- checkpoint interruption,
- outbox interruption.

### Recovery

- retry after partial success,
- handoff after partial success,
- restart during reconciliation,
- duplicate response,
- ambiguous provider outcome.

### Conflict

- user edit during AI attempt,
- external Git operation,
- changed baseline,
- merge conflict.

Each discovered real failure becomes a permanent regression scenario.

---

# 18. Scenario DSL Rules

All important failure scenarios should be represented declaratively where practical.

Example:

```yaml
scenario: timeout_after_partial_edit
setup:
  files:
    App.tsx: baseline-A
attempt:
  provider: fake
  behavior:
    - edit: App.tsx
    - timeout
expected:
  reconciliation: PartialChange
  attempt_status: Failed
  failure_category: FailedWithChanges
  recovery: TargetedRepair
  workspace: Unmodified
```

### Scenario requirements

Every scenario must be:

- deterministic,
- replayable,
- versioned,
- associated with expected outcomes,
- runnable by the simulator,
- preserved if it represents a discovered bug.

---

# 19. Regression Corpus Protocol

Every meaningful bug becomes a permanent test.

Example:

```text
failure_cases/
  F-001-timeout-after-write/
  F-002-duplicate-provider-response/
  F-003-user-edit-conflict/
  F-004-checkpoint-crash/
  F-005-stale-capability/
```

Each case contains:

```text
SCENARIO
INITIAL STATE
TRIGGER
OBSERVED FAILURE
EXPECTED BEHAVIOR
REGRESSION TEST
```

A future developer must not remove a safeguard that exists solely because a recorded failure case proved it necessary without reviewing the case first.

---

# 20. Verification Gate Protocol

A task's verification gate must be explicit before implementation starts.

Typical ordering:

```text
file existence
→ diff scope
→ syntax
→ typecheck
→ build
→ unit tests
→ integration tests
→ runtime/browser checks
→ requirement checks
→ acceptance contract
```

Use the cheapest check capable of disproving success first.

### Important distinction

```text
Implemented ≠ Verified ≠ Accepted
```

### Implemented

Code exists.

### Verified

Required technical tests/evidence pass.

### Accepted

The task's acceptance contract passes and the result is checkpointed.

A task only becomes `Accepted` after verification.

---

# 21. Succeeded Is a Protected Transition

The agent must treat `Succeeded` as a guarded capability, not a field it may simply write.

Only the Verification Engine can create the transition to `AttemptStatus::Succeeded`.

Therefore:

- model response cannot succeed an attempt,
- Aider output cannot succeed an attempt,
- reconciliation cannot succeed an attempt,
- UI button cannot succeed an attempt,
- skill cannot succeed an attempt,
- provider response cannot succeed an attempt.

The verification pipeline must produce the necessary evidence first.

---

# 22. Git and Worktree Protocol

Every real coding attempt must follow the Master Blueprint's workspace isolation model.

### Rules

1. Never let Aider modify the user's live workspace directly.
2. Create an isolated worktree or supported isolated working copy.
3. Capture baseline before execution.
4. Record expected changes.
5. Perform work in isolation.
6. Verify.
7. Only accepted results may be merged.
8. Merge uses the expected-state conditional protocol.
9. Post-merge verification occurs before checkpoint acceptance.
10. If the baseline changed unexpectedly, surface `Conflict`.

### Unsupported repository conditions

Fail closed for unsupported conditions defined by the Blueprint, including:

- detached HEAD,
- in-progress rebase/merge/cherry-pick,
- unsupported submodule operations,
- unresolved conflict states,
- repository disappearance,
- external branch mutation that invalidates the baseline.

Do not improvise recovery semantics.

---

# 23. Provider/Resource Protocol

AI Conductor is free-first, not free-at-any-cost.

The objective is:

> maximize useful work while minimizing wasted quota, duplicated context, retries, and recovery cost.

### Provider selection must consider

- capability fit,
- health,
- quota headroom,
- rate-limit state,
- context fit,
- reliability,
- recovery cost,
- current mission requirements,
- evidence freshness.

Start with a transparent scoring formula.

Do not introduce learned routing before enough real data exists to justify it.

### Never assume

- a provider supports a capability,
- two providers have equivalent tool semantics,
- a local idempotency key makes a provider idempotent,
- a free quota will remain available,
- a provider's documented capability is current forever.

Unknown capability → verify or exclude from candidate routing.

---

# 24. Preflight Protocol

Before every expensive real attempt, run preflight.

Required checks:

```text
✓ provider capable
✓ provider not exhausted
✓ quota sufficient for expected work
✓ context within capability
✓ required files known
✓ workspace safe
✓ required checkpoint available
✓ skill has required capabilities
✓ acceptance criteria known
✓ current policy is valid
✓ recovery budget available
```

If a preflight check fails, do not spend the request.

---

# 25. Recovery Protocol

Recovery is not "try again."

Every recovery must state:

```text
WHAT FAILED
WHAT EVIDENCE SHOWED
WHAT IS ALREADY COMPLETE
WHAT REMAINS
WHAT IS DIFFERENT THIS TIME
WHY THE NEW ATTEMPT HAS A BETTER CHANCE
WHAT WILL NOT BE REPEATED
WHAT WILL VERIFY SUCCESS
```

### Recovery ordering

```text
classify
→ reconcile
→ choose targeted recovery
→ preflight
→ execute
→ verify
→ checkpoint
```

### Never do

```text
failure
→ same prompt
→ same model
→ same files
→ same approach
```

without an explicit human-requested repeat.

---

# 26. Recovery Budget

A mission has a bounded recovery budget.

Example policy:

```text
3 repair attempts
1 provider handoff
1 human escalation
```

When exhausted:

```text
MissionStatus::Blocked
```

with the exact reason and a safe checkpoint.

The point is to prevent an autonomous loop from wasting the day's provider quota.

---

# 27. Context-Efficiency Protocol

Context is a resource.

### Never resend unnecessarily

Avoid repeatedly sending:

- full conversation history,
- unrelated files,
- already-verified work,
- stale analysis,
- irrelevant logs.

### Prefer

```text
mission contract
+
current step
+
accepted changes
+
unresolved problem
+
relevant files
+
recent diff
+
failed verification evidence
+
explicit constraints
```

### Caching

Cache read-only analysis using fingerprints.

Never blindly cache mutation decisions.

### Incremental indexing

If one file changes, update the relevant index incrementally rather than rebuilding the entire project representation.

---

# 28. gstack Skill Implementation Protocol

gstack is the engineering process layer.

AI Conductor is the reliability/control layer.

Skills must be defined with:

```text
INPUT
OUTPUT
CAPABILITIES
FORBIDDEN CAPABILITIES
ALLOWED TOOLS
VERIFICATION
HANDOFF DATA
```

### Initial backend skill set

Build first:

- Office Hours,
- Engineering Review,
- Build,
- Review,
- QA,
- Debug.

Add other skills after the reliability substrate is proven.

### Skill rule

A skill must never rely on a prompt alone to enforce its permissions.

Tool execution must pass through the actual capability boundary.

---

# 29. Three Classes of Automation

Every action must belong to exactly one class.

## Deterministic automation

May run unattended.

Examples:

- state transitions,
- health calculations,
- preflight,
- reconciliation,
- simple bounded retries.

## AI-assisted automation

Model proposes; Conductor validates.

Examples:

- planning,
- repair,
- code review,
- context selection,
- decomposition.

## Human-required decisions

Never silently automated.

Examples:

- overlapping user/AI changes,
- production-affecting changes,
- security-sensitive exceptions,
- irreversible actions,
- exhausted recovery budget.

---

# 30. Safe Mode and Emergency Controls

Safe Mode is always available.

Safe Mode disables:

- automatic handoff,
- automatic repair,
- automatic merge,
- automatic destructive actions.

Each automation layer should also have an independent kill switch.

### Emergency rule

When unexpected behavior is observed during development:

```text
Disable automation
→ preserve state
→ export diagnostics
→ reproduce in simulator
→ fix
→ regression test
→ re-enable
```

Do not debug a dangerous autonomous loop by allowing it to continue running.

---

# 31. Dry-Run Protocol

Dry run must produce the commands the Kernel would emit without executing them.

Example:

```text
DRY RUN
Mission: Add authentication

Would:
1. Create checkpoint
2. Select provider
3. Create worktree
4. Run Engineering Review
5. Provide Aider selected context
6. Verify build/tests

No files modified.
```

Dry-run results should be auditable and should identify the selected policy and resource.

---

# 32. Shadow Mode Protocol

New intelligence must first run in shadow mode where practical.

Possible shadow candidates:

- routing policy,
- recovery policy,
- context selection,
- reconciliation implementation,
- health scoring.

Shadow code may observe and compare, but does not control active missions.

Changes in verdict must be recorded and reviewed before promotion.

---

# 33. Formal Model + Implementation Consistency

For the critical state machine:

```text
formal model
→ Rust implementation
→ property tests
→ simulator
→ real integration
```

If any level disagrees with the others:

```text
STOP
→ record discrepancy
→ determine authoritative semantics
→ update implementation/tests
```

Do not patch the symptom while leaving the models inconsistent.

---

# 34. Concurrency Protocol

One authoritative executor per mission.

State mutation must not be performed independently by:

- UI threads,
- provider health callbacks,
- arbitrary async tasks,
- Aider event handlers,
- timers.

They send commands/messages to the mission executor.

Concurrency-sensitive operations must have dedicated deterministic race tests.

Do not introduce multi-agent parallelism until the single-mission case is proven solid according to the Blueprint roadmap.

---

# 35. Persistence Protocol

Use:

- atomic file writes,
- append-only event log,
- hash chain,
- event anchor,
- outbox for multi-record local consistency,
- versioned schemas,
- migration backups.

External side effects are separate from local persistence guarantees.

A local idempotency record is not proof that an external action is safe to repeat.

---

# 36. Security Protocol

Credentials:

- live in the Credential Vault,
- never enter logs,
- never enter events,
- never enter diagnostic bundles,
- never enter mission ledgers,
- never enter model prompts,
- are masked in the UI,
- use structural redaction where possible.

Verification commands must execute inside the defined OS-level containment boundary.

Security claims must always match the documented threat model.

---

# 37. Diagnostic Protocol

Diagnostic bundles use an **allowlist**, not a denylist.

Allowed categories include, where safe:

- mission state,
- event log,
- failure classifications,
- sanitized provider metadata,
- attempt timeline,
- reconciliation results,
- Git metadata,
- test results,
- version information.

Never export:

- credentials,
- raw secrets,
- unapproved private source,
- unrestricted environment variables.

---

# 38. Benchmark Protocol

The project should measure, rather than assume, lightweight behavior.

Track:

- kernel state-transition latency,
- reconciliation latency,
- simulator throughput,
- context package size,
- handoff package size,
- recovery overhead,
- worktree creation time,
- mission startup time,
- UI idle memory,
- resource usage.

Do not invent target results before measurement.

Targets become meaningful once a baseline exists.

---

# 39. Architecture Integrity Audit Before Each Phase Gate

Before starting a new phase, verify:

```text
✓ no invariant was weakened
✓ no state machine was bypassed
✓ Succeeded authority remains intact
✓ capability boundaries remain intact
✓ no credentials leaked
✓ no scope violations remain
✓ prior failure cases still pass
✓ migration/version rules remain valid
✓ build state matches repository reality
```

A phase that passes functional tests but violates an invariant has **failed the gate**.

---

# 40. Phase Gate Protocol

A phase is complete only when:

```text
ALL TASKS ACCEPTED
+
ALL REQUIRED TESTS PASS
+
ALL REQUIRED INVARIANTS COVERED
+
NO BLOCKING UNKNOWNs
+
NO UNRESOLVED CRITICAL FAILURES
+
BUILD STATE UPDATED
+
CHECKPOINT CREATED
+
PHASE EXIT REPORT WRITTEN
```

### Phase Exit Report

```text
PHASE:

OBJECTIVE:

TASKS ACCEPTED:

TEST RESULTS:

INVARIANT COVERAGE:

FAILURE CASES ADDED:

BENCHMARK RESULTS:

KNOWN LIMITATIONS:

KNOWN UNKNOWNS:

DEPENDENCY CHANGES:

ARCHITECTURE CHANGES:

CHECKPOINT:

NEXT PHASE AUTHORIZED: YES / NO
```

---

# 41. Release Gates

Do not call the product production-ready merely because the UI looks complete.

Release readiness requires, at minimum:

- all core invariants tested,
- deterministic simulator suites passing,
- real integration suite passing,
- crash-recovery test passing,
- provider handoff passing,
- conflict protection passing,
- verification-only success enforcement passing,
- capability enforcement passing,
- credential leak tests passing,
- migration tests passing,
- diagnostic export sanitization tests passing,
- recovery budgets functioning,
- no known unresolved critical failure cases.

The final release decision must come from evidence, not from the coding agent's confidence.

---

# 42. Agent Prompt Template

Use this template when assigning a task to OX Alpha.

```text
You are implementing AI Conductor under:
- Master Blueprint: v2.5.1
- Build Protocol: v1.1

CURRENT PHASE:
<phase>

CURRENT TASK:
<task ID + title>

BLUEPRINT REFERENCES:
<sections>

OBJECTIVE:
<exact objective>

ALLOWED FILES:
<list>

FORBIDDEN FILES/ACTIONS:
<list>

KNOWN DEPENDENCIES:
<list>

REQUIRED IMPLEMENTATION:
<list>

REQUIRED TESTS:
<list>

DONE WHEN:
<acceptance conditions>

NON-GOALS:
<list>

RULES:
1. Do not invent behavior that is not verified.
2. Do not modify architecture outside the task.
3. Do not silently expand scope.
4. Do not claim a test passed unless you actually executed it.
5. Do not mark the task complete without the acceptance gate.
6. If blocked, stop and report the exact blocker.
7. If the blueprint appears contradictory, stop and create an architecture change request.
8. Update BUILD_STATE when the task is accepted.
```

---

# 43. Agent Completion Report Template

Every accepted task should leave:

```text
TASK COMPLETION REPORT

Task ID:
Task:
Blueprint references:

Implementation summary:

Files changed:

Tests executed:

Tests passed:

Tests failed:

Invariant coverage:

Acceptance criteria:

Evidence:

Known limitations:

Known unknowns:

Dependencies changed:

Failure cases added:

Checkpoint:

Next task:
```

No invented percentages.

No invented confidence scores.

No "looks good" as the primary evidence.

---

# 44. Agent Behavior at a Context/Token Limit

If the agent approaches its context or tool limit:

1. Stop starting new broad work.
2. Finish or safely suspend the smallest current action.
3. Persist current state.
4. Produce STOP_REPORT.
5. Record exact next action.
6. Do not claim completion if the gate has not passed.

The next agent continues via the Continue Protocol.

The goal is to make context limits ordinary interruptions, not project-threatening events.

---

# 45. Agent Behavior on Network/Provider Failure

If external access fails:

1. Classify the failure.
2. Do not fabricate a response.
3. Preserve local state.
4. Reconcile if an external side effect could have occurred.
5. Use the Provider/Idempotency rules.
6. Continue only if preflight permits.
7. Otherwise stop safely and record the next action.

No invented provider response may be used as evidence.

---

# 46. Agent Behavior on Build/Test Failure

A failed test is information, not permission to rewrite everything.

Protocol:

```text
failure
→ classify
→ identify smallest affected surface
→ reproduce
→ create/update scenario
→ repair only relevant scope
→ re-run focused tests
→ run regression tests
→ accept only when gate passes
```

Do not use broad rewrites to hide a localized failure.

---

# 47. Agent Behavior on Unexpected Repository State

Examples:

- user changes files,
- unrelated code appears modified,
- branch changed,
- dependency lock changed unexpectedly,
- generated files differ unexpectedly,
- another process is active.

Required response:

```text
STOP
→ record observed state
→ classify
→ do not overwrite
→ determine whether state is safe to continue
→ escalate if ambiguous
```

Never "clean up" unexpected user work merely to make the repository match the expected state.

---

# 48. Agent Behavior on Architecture Contradiction

If the agent detects a contradiction:

```text
STOP IMPLEMENTATION
        ↓
Record exact contradiction
        ↓
Identify impacted invariants
        ↓
Provide evidence
        ↓
Create architecture-change request
        ↓
Await human decision
```

It is better to stop for one clarification than to build a subsystem around the wrong interpretation.

---

# 49. Efficiency Rules for AI-Assisted Development

To reduce time and token waste:

### Prefer deterministic code over AI reasoning

If a rule can be represented as a pure function or table, implement it that way.

### Prefer schemas over prose

If a task has structured data, use machine-readable contracts.

### Prefer generated tests over remembered tests

Tie tests to invariants.

### Prefer targeted repairs over full restarts

Use failure evidence.

### Prefer local simulation over real provider calls

Use FakeProvider and VirtualWorld first.

### Prefer preflight before expensive work

Do not spend a request only to discover an obvious mismatch.

### Prefer caching observations

Do not recompute unchanged project analysis.

### Prefer small tasks

Keep coding-agent context narrow.

### Prefer explicit commands

Avoid hidden side effects.

### Prefer one source of truth

Do not duplicate schemas and rules across multiple places unnecessarily.

---

# 50. What the Agent Must NOT Do

Without explicit authorization, OX Alpha must not:

- rewrite the blueprint,
- weaken an invariant,
- disable safety mechanisms to make tests pass,
- remove failing tests to obtain green output,
- fabricate successful commands,
- fabricate provider behavior,
- invent undocumented APIs,
- commit secrets,
- expose credentials in logs,
- change unrelated modules during a bounded task,
- add large dependencies for convenience,
- introduce parallel agents early,
- optimize before correctness is established,
- make production-affecting changes without the required gate,
- claim "enterprise-ready" without evidence,
- claim "error-free" as an absolute property.

The system may aim for extremely high reliability, but no honest engineering process promises mathematical perfection of a large real-world application.

---

# 51. The Definition of a Good OX Alpha Session

A good session is not the one that writes the most code.

A good session is the one that leaves the project in a **more verified, more understandable, more recoverable state**.

At session end:

```text
BEFORE
unknown scope
unknown state

AFTER
bounded task
verified behavior
recorded evidence
updated state
safe checkpoint
clear next action
```

That is success.

---

# 52. The Governing Build Loop

This is the core operating loop for the entire project:

```text
                    ┌─────────────────────┐
                    │ MASTER BLUEPRINT    │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ PHASE MANIFEST      │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ TASK CONTRACT       │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ PRE-FLIGHT          │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ IMPLEMENT           │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ TEST / SIMULATE     │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ VERIFY              │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ CHECKPOINT          │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ UPDATE BUILD STATE  │
                    └──────────┬──────────┘
                               ↓
                    ┌─────────────────────┐
                    │ NEXT TASK / STOP    │
                    └─────────────────────┘
```

At any failure:

```text
             FAILURE
                ↓
             CLASSIFY
                ↓
          RECONCILE / TEST
                ↓
       ┌────────┴─────────┐
       ↓                  ↓
    RECOVER            ESCALATE
       ↓                  ↓
    VERIFY          HUMAN DECISION
       ↓
   CHECKPOINT
```

---

# 53. Final Operating Rule

The entire implementation protocol can be reduced to this:

> **Never ask the agent to "build the system." Ask it to prove one bounded truth at a time.**

And when a session ends:

> **Leave the repository, build state, evidence, checkpoint, and next action in a state where another agent can continue without trusting the previous agent's memory.**

And when uncertainty appears:

> **Stop, record it, investigate it, and only then proceed.**

And when a failure occurs:

> **Reproduce it, encode it, test it forever, and never solve the same uncertainty twice.**

---

# 54. Protocol Completion Checklist

Before using this protocol for the first OX Alpha build session, verify:

- [ ] Master Blueprint v2.5.1 is present.
- [ ] Build Protocol v1.1 is present.
- [ ] `.ai-conductor-build/BUILD_STATE.json` exists.
- [ ] Phase manifests exist for at least Phase 0 and Phase 1.
- [ ] Task contracts exist for every Phase 0 task.
- [ ] Verification gates exist for Phase 0.
- [ ] Failure-corpus directory exists.
- [ ] STOP_REPORT format is available.
- [ ] Architecture-change template exists.
- [ ] Dependency-request template exists.
- [ ] Current repository baseline/checkpoint is recorded.
- [ ] The OX Alpha session starts in a disposable branch/worktree where appropriate.
- [ ] No task begins without a bounded contract.
- [ ] No phase begins without its entry conditions.

---

## End of AI Conductor Build Protocol v1.1

**Authority:** AI Conductor Master Blueprint v2.5.1  
**Implementation principle:** one invariant → one slice → one gate  
**Continuation principle:** state, evidence, checkpoint, next action — never memory alone  
**Quality principle:** observable behavior beats model confidence  
**Efficiency principle:** minimum context, minimum duplicated work, minimum unnecessary requests  
**Safety principle:** unknown and conflict stop automation rather than inviting guesswork


---

# v1.1 Execution-Harness Amendment — Jcode / Aider / Future Executors

This amendment updates the operational protocol to match Blueprint v2.5.1. It does not invalidate previously accepted tasks.

## A. Harness-Neutral Rule

AI Conductor must not depend architecturally on Aider, Jcode, Claude Code, Codex, or any single execution harness.

All harnesses are accessed through the **Execution Adapter Contract**.

The Conductor owns:

```text
mission state
step/attempt state
capabilities
workspace authority
verification
reconciliation
recovery
acceptance
merge
persistence
evidence
```

The harness owns only delegated execution.

## B. Harness Selection

A supported execution request follows:

```text
Task Contract
  ↓
Capability / policy evaluation
  ↓
Executor selection
  ↓
Execution Adapter
  ↓
Harness
  ↓
Observed result/evidence
  ↓
Reconciliation
  ↓
Verification
```

Changing the selected harness must not require changes to the reliability kernel.

## C. No Harness Self-Certification

These statements are advisory only:

```text
"done"
"tests passed"
"commit complete"
"workspace safe"
"merge successful"
```

They must be checked against actual evidence.

No harness may:

- set `Succeeded` directly;
- bypass Capability Enforcement;
- authorize a Conductor merge;
- suppress an Unknown outcome;
- silently resolve a Conflict;
- write outside the authorized workspace.

## D. Harness Capability Claims

Executor capability records must include provenance/freshness where relevant:

```text
executor identity
version/commit
supported interfaces
workspace behavior
process behavior
supported tool classes
provider/model compatibility
interactive/non-interactive mode
capability verification source
verification timestamp
```

A capability not verified for the actual installed version is `UNVERIFIED`.

## E. External Harness Evaluation

When evaluating Jcode, Aider, or a future harness, do not ask whether it is generally "better."

Measure it against the same Conductor contract:

```text
startup
workspace confinement
mutation observation
failure classification
cancellation
timeout
crash
Unknown handling
restart/reconciliation
evidence collection
credential isolation
capability enforcement
Succeeded authority
merge authority
```

## F. Use Harness Strengths Without Importing Their Authority

A harness may provide useful features such as:

```text
memory
sessions
provider routing
skills
MCP
swarm/parallel execution
TUI
browser automation
Git helpers
```

These may be leveraged when useful, but Conductor must retain the authority boundary.

## G. Parallelism

The existence of a harness swarm does not authorize Conductor-wide parallel mutation.

Parallel work remains gated by the Blueprint's concurrency model, workspace isolation, merge protocol, and deterministic tests.

Do not adopt a harness's internal concurrency semantics merely because the harness supports them.

## H. Version Drift

When an external harness changes version:

```text
pin/reference version
→ run adapter conformance tests
→ run relevant failure corpus
→ compare capability observations
→ accept or reject upgrade
```

Do not silently upgrade a core execution dependency during an unrelated task.

## I. Adapter Failure

If a harness adapter fails, classify the layer:

```text
adapter defect
harness defect
provider failure
process failure
workspace failure
network failure
unknown outcome
```

Then reconcile before retrying whenever an external side effect may have occurred.

## J. Human Escalation

The agent should ask the human only for information or authority that cannot be discovered safely.

Examples:

```text
credentials
combo/model mapping
irreversible architecture choice
security decision
scope change
human acceptance
```

Do not block on information that can be discovered by inspecting the actual environment.
