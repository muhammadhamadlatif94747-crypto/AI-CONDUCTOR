# AI CONDUCTOR
# Master Phase Manifest — v1.1 (Harness-Neutral)

**Authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`  
**Build procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.1_HARNESS_NEUTRAL.md`  
**Purpose:** Define the bounded implementation program that an AI coding agent may execute, phase by phase, without inventing architecture or expanding scope silently.

---

## 0. Document Status

| Field | Value |
|---|---|
| Manifest | AI Conductor Master Phase Manifest |
| Version | 1.0 |
| Architecture authority | Master Blueprint v2.4 |
| Build authority | Build Protocol v1.1 |
| Human authority | Project owner |
| Coverage | Phase 0 through Phase 11 |
| Current active phase | Controlled by `BUILD_STATE.json` |
| Status | Operational implementation manifest |

### Governing relationship

```text
MASTER BLUEPRINT v2.4
        ↓
BUILD PROTOCOL v1.1
        ↓
THIS MASTER PHASE MANIFEST
        ↓
ACTIVE PHASE MANIFEST
        ↓
TASK CONTRACT
        ↓
VERIFICATION GATE
        ↓
BUILD STATE
        ↓
ACTUAL REPOSITORY + EXECUTED EVIDENCE
```

This document does **not** redefine the architecture.

The Blueprint answers:

> What must AI Conductor be?

The Build Protocol answers:

> How must an AI coding agent build it?

This Manifest answers:

> What phase is currently authorized, what bounded outcome must that phase prove, and what may not be built yet?

A task contract answers:

> What exact small unit is the agent executing now?

---

# 1. Non-Negotiable Manifest Rules

## 1.1 No phase skipping

A phase may not begin until the previous phase's exit gate has passed and the result is checkpointed.

The Blueprint explicitly defines a sequential roadmap with a Definition of Done for every phase.

## 1.2 No silent phase expansion

The agent may not add work to the active phase merely because it appears useful.

If a newly discovered requirement belongs to a later phase:

```text
record discovery
→ do not implement it
→ continue current authorized scope
```

unless the project owner explicitly changes the manifest.

## 1.3 Blueprint contradictions stop execution

If the agent discovers that:

- the Blueprint contradicts itself,
- the current phase conflicts with a later architectural rule,
- a required behavior is undefined,
- an implementation assumption would alter an invariant,

the agent must stop and create the appropriate architecture/change record.

It must not resolve the conflict by preference.

## 1.4 Phase completion is evidence-based

A phase is not complete because:

- the listed files exist,
- the code compiles,
- the agent says it is finished,
- tests were assumed to pass,
- a later feature happens to work,
- the UI looks correct.

A phase is complete only when:

```text
ALL REQUIRED TASKS ACCEPTED
        +
ALL PHASE GATE TESTS PASS
        +
ARCHITECTURE INTEGRITY AUDIT PASSES
        +
BUILD STATE UPDATED
        +
CHECKPOINT RECORDED
```

## 1.5 One invariant → one slice → one gate

Within every phase:

```text
Blueprint requirement
        ↓
bounded task contract
        ↓
implementation slice
        ↓
deterministic proof
        ↓
simulation / failure proof where applicable
        ↓
verification gate
        ↓
checkpoint
```

Do not implement a whole subsystem before proving its critical contract.

## 1.6 No premature multi-agent parallelism

Multi-agent parallelism is explicitly out of scope for all listed phases.

It may only be reconsidered after Phase 10 is genuinely bulletproof and the project owner authorizes the change.

## 1.7 Core before polish

Reliability, state semantics, persistence, isolation, reconciliation, verification, security, simulation, and recovery take precedence over visual polish.

Phase 11 is intentionally reserved for the polished frontend.

---

# 2. Phase Control Record

The implementation agent must maintain an active phase record in `BUILD_STATE.json`.

Required fields:

```json
{
  "manifest_version": "1.0",
  "blueprint_version": "2.4",
  "protocol_version": "1.0",
  "current_phase": "P0",
  "phase_status": "not_started",
  "current_task": null,
  "accepted_tasks": [],
  "blocked_tasks": [],
  "failed_tasks": [],
  "phase_gate_status": "not_ready",
  "last_verified_checkpoint": null,
  "last_verified_commit": null,
  "next_authorized_action": null
}
```

The actual repository state and executed evidence outrank stale manifest state.

---

# 3. Global Phase Lifecycle

Every phase follows:

```text
PHASE ENTRY
    ↓
READ BLUEPRINT REFERENCES
    ↓
READ BUILD PROTOCOL
    ↓
READ CURRENT MANIFEST
    ↓
INSPECT REPOSITORY
    ↓
CREATE TASK CONTRACTS
    ↓
PREFLIGHT
    ↓
IMPLEMENT ONE SLICE
    ↓
TEST
    ↓
VERIFY
    ↓
CHECKPOINT
    ↓
UPDATE BUILD STATE
    ↓
NEXT TASK
    ↓
ALL TASKS ACCEPTED?
    │
    ├── NO → continue
    │
    └── YES
          ↓
ARCHITECTURE INTEGRITY AUDIT
          ↓
PHASE GATE
          ↓
CHECKPOINT
          ↓
AUTHORIZE NEXT PHASE
```

---

# 4. Phase 0 — Architecture Contracts

## 4.1 Status

**Code:** `P0`  
**Name:** Architecture Contracts  
**Blueprint:** Section 23 — Phase 0  
**Code allowed:** **No production implementation code**

Phase 0 is a contract-definition phase.

Its purpose is to remove ambiguity before implementation begins.

## 4.2 Objective

Create the precise contracts that later phases must implement without reinterpretation.

## 4.3 Authorized scope

Only architecture-contract artifacts may be created or edited.

Expected categories include:

```text
.ai-conductor-build/
    phase_manifests/
    task_contracts/
    verification_reports/
    architecture_changes/
    dependency_requests/
```

The exact repository layout remains subject to the Blueprint and actual repository baseline.

## 4.4 Required contracts

### P0-C01 — AttemptStatus / ReconciliationOutcome

Define them as separate concepts.

Required proof:

- Attempt status cannot be silently replaced by reconciliation classification.
- `Succeeded` remains controlled by verification.
- Reconciliation produces a reconciliation outcome rather than declaring success.

Blueprint references: Sections 4.2–4.3.

### P0-C02 — Evidence provenance and freshness

Define:

- what evidence is,
- where it came from,
- when it was produced,
- how freshness is represented,
- how later decisions know whether evidence is still trustworthy.

Blueprint reference: Section 4.5.

### P0-C03 — Checkpoint semantics

Define:

- what constitutes a checkpoint,
- what state it protects,
- what makes it valid,
- how it is referenced,
- how recovery uses it.

Blueprint reference: Section 4.7.

### P0-C04 — Tamper-evident event chain

Define:

- event format,
- sequence numbering,
- hash chaining,
- anchor behavior,
- verification-on-load semantics.

Blueprint reference: Section 4.9.

### P0-C05 — Failure taxonomy

Define both:

- failure taxonomy classes,
- structured `FailureCategory`.

`Conflict` must be represented explicitly.

Blueprint references: Sections 9 and 11.

### P0-C06 — Skill contract

Define the backend skill contract.

Blueprint reference: Section 22.

### P0-C07 — Capability table

Define:

- capability identities,
- skill/tool permissions,
- enforcement boundary,
- tool-execution-boundary location.

The manifest must not invent capabilities not defined by the Blueprint.

Blueprint reference: Section 16.

### P0-C08 — Mission Acceptance Contract

Define the acceptance-contract format.

Every criterion must include:

- criterion,
- required verification method,
- required evidence,
- evidence provenance/freshness expectations.

Blueprint reference: Section 8.

### P0-C09 — Two-phase merge contract

Define the merge protocol including Git edge cases.

The contract must preserve the Blueprint's conditional expected-state merge semantics.

Blueprint references: Invariant 13, Sections 5 and 5.1.

### P0-C10 — Mission and Step state machines

Define them independently of Attempt.

Blueprint reference: Section 4.2.1.

### P0-C11 — External-side-effect idempotency classification

Define the model required by Invariant 16.

Blueprint reference: Section 10.5.

### P0-C12 — Clock trait

Define the `Clock` abstraction.

Hard rule:

> No direct system-clock calls inside the reliability core.

Blueprint reference: Section 10.2.

## 4.5 P0 non-goals

Do not:

- implement the state engine,
- implement persistence,
- integrate Aider,
- integrate OmniRoute,
- integrate providers,
- build the frontend,
- build multi-agent execution,
- add speculative dependencies,
- create production UI.

## 4.6 P0 gate

Phase 0 passes only when:

- every required contract above exists,
- terminology is internally consistent,
- each contract has a Blueprint reference,
- no contract weakens an invariant,
- no unresolved architecture contradiction remains,
- task contracts for Phase 1 are ready,
- verification gates for Phase 1 are ready,
- build state records the completed architecture-contract checkpoint.

---

# 5. Phase 1 — State + Event + Persistence Core

## 5.1 Status

**Code:** `P1`  
**Blueprint:** Section 23 — Phase 1

## 5.2 Objective

Implement the foundational state, event, persistence, identity, and reliability-console substrate.

This phase establishes the memory and state integrity on which later recovery depends.

## 5.3 Required work packages

### P1-W01 — State Engine

Implement the State Engine with an enforced transition table.

Required proof:

- legal transitions are accepted,
- illegal transitions are rejected,
- transition semantics match the Blueprint,
- tests exercise invalid transitions.

### P1-W02 — CancellationOutcome

Implement `CancellationOutcome` as its own enum.

Do not leak cancellation detail into `AttemptStatus`.

Blueprint reference: Section 14.

### P1-W03 — Append-only event log

Implement:

- JSON Lines event log,
- append-only semantics,
- hash chaining.

Blueprint reference: Section 4.9.

### P1-W04 — Event-chain anchoring

Implement persisted anchor behavior and verification on load.

Blueprint reference: Section 4.9.

### P1-W05 — Atomic persistence

Use atomic file writes wherever state is persisted.

### P1-W06 — Outbox / intent mechanism

Implement the intent mechanism required for multi-record local consistency.

Blueprint reference: Invariant 15 / Section 20.

### P1-W07 — Idempotency-key store

Implement local retry-detection state.

Important:

A local idempotency record is not proof that an external action is safe to repeat.

### P1-W08 — Sequence allocator

Implement the monotonic event sequence-number allocator.

### P1-W09 — Correlation IDs

Thread the canonical hierarchy through:

```text
mission
→ step
→ attempt
→ event
→ provider request
→ tool call
```

Do not create independent ad hoc ID systems.

### P1-W10 — Schema versioning

Every persisted record must carry the required schema version.

### P1-W11 — Developer Reliability Console

Implement the plain functional developer console required by Blueprint Section 21.

It is not the final polished Mission Mode UI.

## 5.4 P1 non-goals

Do not:

- integrate real Aider,
- integrate real providers,
- implement full reconciliation,
- implement polished frontend,
- introduce multi-agent parallelism,
- optimize routing,
- add learned routing.

## 5.5 P1 gate

Required proof includes:

- deterministic transition tests,
- persistence restart tests,
- event-chain verification tests,
- atomic-write failure tests,
- outbox interruption tests,
- idempotency tests,
- sequence monotonicity tests,
- correlation-ID propagation tests,
- schema-version tests,
- existing regression corpus remains passing.

A phase-level architecture audit must confirm that no later integration has bypassed the State Engine.

---

# 6. Phase 2 — Workspace Safety, Git Worktree Isolation, Reconciliation

## 6.1 Status

**Code:** `P2`  
**Blueprint:** Section 23 — Phase 2

## 6.2 Objective

Make execution isolated, observable, reconcilable, and conflict-safe.

## 6.3 Required work packages

### P2-W01 — Isolated worktree execution

Every attempt must execute in an isolated worktree.

The live user workspace must not be directly modified by Aider.

### P2-W02 — Baseline capture

Capture the expected baseline before execution.

### P2-W03 — Diff-aware Reconciliation Engine

Implement and unit-test the reconciliation engine.

### P2-W04 — Protected Succeeded transition

The Reconciliation Engine must never set `Succeeded`.

Required direct test:

```text
reconciliation result
    ≠
Succeeded transition
```

Only the Verification Engine can create the `Succeeded` transition.

### P2-W05 — Mutation attribution

Implement mutation attribution according to Blueprint Section 4.6.

### P2-W06 — Conflict detection

Implement conflict detection.

Conflict must halt rather than silently resolve.

### P2-W07 — Person decision path

Implement the four-way person decision UI required by the Blueprint.

This is functional decision UI, not polished Mission Mode.

## 6.4 Required failure tests

At minimum, exercise:

- user edit during AI attempt,
- external Git operation,
- changed baseline,
- merge conflict,
- partial change,
- unexpected process exit,
- restart during reconciliation.

## 6.5 P2 gate

The phase is accepted only when:

- isolated execution is proven,
- reconciliation is deterministic,
- conflict never silently overwrites,
- `Succeeded` remains verification-owned,
- two-phase merge rules are respected,
- restart/reconciliation behavior is proven,
- regression corpus remains passing.

---

# 7. Phase 3 — Fake Provider + Deterministic Failure Simulation

## 7.1 Status

**Code:** `P3`

## 7.2 Objective

Create a deterministic virtual world in which reliability behavior can be tested without consuming real provider quota.

## 7.3 Required work packages

### P3-W01 — FakeProvider

Implement a controllable fake provider with the Blueprint-required fault menu.

### P3-W02 — VirtualClock

Implement the virtual clock required for instant deterministic time-dependent tests.

### P3-W03 — Chaos Mode

Implement deterministic chaos injection.

It must include simulated mid-attempt user edits.

### P3-W04 — Scenario execution

Where the Blueprint specifies the Scenario DSL and simulator components, connect the phase's failure scenarios to that machinery.

## 7.4 Required failure families

Cover at least:

- timeout,
- disconnect,
- delayed response,
- 429,
- quota exhaustion,
- 5xx,
- malformed response,
- invalid tool request,
- partial edit,
- process termination,
- persistence interruption,
- checkpoint interruption,
- duplicate response,
- ambiguous provider outcome,
- user conflict,
- changed baseline,
- merge conflict.

## 7.5 P3 gate

A failure scenario must be:

```text
deterministic
+ replayable
+ versioned
+ expected-outcome based
+ simulator-runnable
```

Every discovered meaningful bug must enter the permanent regression corpus.

No real provider is required to pass this phase.

---

# 8. Phase 4 — Verification Engine + Acceptance Contracts

## 8.1 Status

**Code:** `P4`

## 8.2 Objective

Make success a verified state rather than a model/provider assertion.

## 8.3 Required work packages

### P4-W01 — Verification pipeline

Implement the full verification pipeline defined by Blueprint Section 7.

### P4-W02 — Evidence production

Verification must use actual evidence with the required provenance/freshness semantics.

### P4-W03 — Mission Acceptance Contracts

Enforce Mission Acceptance Contracts before an Attempt may become `Succeeded`.

### P4-W04 — Protected success transition

Prove:

```text
Only Verification Engine
        ↓
AttemptStatus::Succeeded
```

## 8.4 Required tests

Use `FakeProvider` and deterministic/simulation layers first.

Test:

- passing acceptance criteria,
- failing acceptance criteria,
- stale evidence,
- insufficient evidence,
- wrong verification method,
- partial completion,
- false model success,
- reconciliation that reports a favorable outcome without verification.

## 8.5 P4 gate

No code path other than the Verification Engine may create `Succeeded`.

All required acceptance criteria must be tied to explicit verification methods and required evidence.

---

# 9. Phase 5 — Execution Harness Adapter + Real Executor Validation

## 9.1 Status

**Code:** `P5`

## 9.2 Objective

Establish the replaceable execution-harness boundary and validate at least one real executor under Conductor-controlled isolation, observation, recovery, and verification. Jcode and Aider are evaluated under the same contract.

## 9.3 Required work packages

### P5-W01 — Execution Adapter Contract

Implement the common semantic adapter boundary defined by the Blueprint and the dedicated Execution Adapter Contract.

### P5-W02 — Executor Capability Snapshot

Record executor identity, version/commit, capabilities, workspace behavior, lifecycle semantics, and verification provenance.

### P5-W03 — Reference Executor Isolation

Validate one real executor inside a Conductor-created isolated worktree.

### P5-W04 — Real Execution Lifecycle

Prove normal execution, timeout, cancellation, crash/abrupt termination, and ambiguous outcome handling.

### P5-W05 — Restart and Reconciliation

Prove restart after real executor interruption and reconcile actual workspace state before continuation.

### P5-W06 — Aider Conformance

If Aider is retained, validate it against the common executor conformance suite.

### P5-W07 — Jcode Evaluation

If the pinned/installed Jcode build is available, evaluate it against the same contract. Do not promote it to a supported executor without passing conformance and failure tests.

### P5-W08 — Executor Selection and Reversibility

Prove that changing executor does not require changing the Conductor reliability kernel and that selection is explainable.

## 9.4 P5 non-goals

Do not:

- make Jcode mandatory,
- make Aider mandatory,
- delegate verification authority to a harness,
- delegate merge authority to a harness,
- adopt harness-internal swarm semantics as Conductor semantics,
- build the final frontend,
- introduce learned routing before transparent routing is proven.

## 9.5 P5 gate

At least one real executor must demonstrate:

```text
Execution Contract
→ capability check
→ isolated worktree
→ execution
→ observation
→ interruption
→ persisted state
→ restart
→ reconciliation
→ evidence
→ verification
→ safe continuation
```

without losing, silently duplicating, silently overwriting, or falsely accepting work.

Aider and Jcode are each independent candidates; failure of one must not invalidate the adapter architecture or the other candidate.

---

# 10. Phase 6 — OmniRoute + One Real Provider (Gemini)

## 10.1 Status

**Code:** `P6`

## 10.2 Objective

Introduce real provider connectivity while preserving all deterministic safeguards.

## 10.3 Required work packages

### P6-W01 — OmniRoute integration

Integrate OmniRoute according to the Blueprint's provider/connectivity boundary.

### P6-W02 — Gemini integration

Integrate one real provider: Gemini.

### P6-W03 — Real quota/rate-limit observation

Observe and classify actual:

- quota exhaustion,
- rate limits,
- provider failures,
- response failures.

### P6-W04 — Failure capture

Every meaningful new real failure becomes a permanent `failure_cases` entry.

## 10.4 P6 preflight requirement

Every expensive real attempt must pass preflight before provider execution.

Preflight includes, as applicable:

- capability,
- health,
- quota,
- context fit,
- files,
- workspace,
- checkpoint,
- skill capability,
- acceptance criteria,
- policy validity,
- recovery budget.

## 10.5 P6 gate

Real provider behavior must not be inferred from documentation alone when live behavior is relevant.

Real observations must be recorded as evidence.

---

# 11. Phase 7 — Resource Intelligence + Capability Registry

## 11.1 Status

**Code:** `P7`

## 11.2 Objective

Introduce transparent provider/resource intelligence without prematurely creating a learned opaque router.

## 11.3 Required work packages

### P7-W01 — Capability Registry

Populate the Provider/Model Capability Registry for:

- Gemini,
- Mistral.

Capabilities must be verified before being relied upon.

### P7-W02 — Provider Health Ledger

Implement the Provider Health Ledger.

It must feed the required cost/usage information into the Mission Ledger.

### P7-W03 — Basic routing

Implement basic:

- quota-aware routing,
- cooldown-aware routing.

Start with a transparent scoring formula.

## 11.4 Explicit non-goal

Do not introduce learned routing before enough real data exists to justify it.

## 11.5 P7 gate

Routing decisions must be:

- explainable,
- evidence-backed,
- capability-aware,
- quota-aware,
- health-aware,
- compatible with current policy,
- testable deterministically.

---

# 12. Phase 8 — Multi-Provider Handoff

## 12.1 Status

**Code:** `P8`

## 12.2 Objective

Prove reliable provider-to-provider recovery.

## 12.3 Required work packages

### P8-W01 — Gemini → Mistral handoff

Implement the expanded handoff package required by the Blueprint.

### P8-W02 — Recovery accounting

Measure and log recovery overhead.

### P8-W03 — Targeted recovery

A handoff must carry enough evidence that the next provider does not need to rediscover the entire failed mission.

The recovery protocol must state:

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

## 12.4 Recovery budget

The Blueprint's policy-configurable Recovery Budget is authoritative.

The documented default example is:

```text
3 repair attempts
1 handoff
1 human escalation
```

When the applicable budget is exhausted, the mission must enter the defined blocked/human-decision path rather than looping.

## 12.5 Monotonic retry rule

A new recovery attempt must be meaningfully different.

An identical immediate retry is rejected unless a person explicitly requests it.

## 12.6 P8 gate

Prove:

```text
Gemini
  ↓
failure
  ↓
classification
  ↓
reconciliation
  ↓
handoff package
  ↓
Mistral
  ↓
continuation
  ↓
verification
```

The system must preserve already-verified work and avoid repeating unnecessary context/work.

---

# 13. Phase 9 — gstack Skill Runtime

## 13.1 Status

**Code:** `P9`

## 13.2 Objective

Introduce gstack as the engineering process/skill layer while preserving capability boundaries.

## 13.3 Required skills

Implement backend-level skill contracts for:

- Office Hours,
- Engineering Review,
- Build,
- Review,
- QA,
- Debug.

These names and roles are derived from the Blueprint roadmap.

## 13.4 Capability enforcement

The tool-execution boundary must enforce skill capabilities.

Required direct proof:

```text
Review-phase skill
    ↓
attempts file write
    ↓
capability enforcement
    ↓
DENIED
```

Do not rely on the model simply following instructions.

## 13.5 P9 non-goals

Do not:

- allow skills to bypass Conductor verification,
- allow skills to bypass capability enforcement,
- allow skills to write outside authorized boundaries,
- introduce multi-agent parallelism.

## 13.6 P9 gate

All required skills execute through the defined skill contract and capability system.

Forbidden operations are blocked by the enforcement boundary, not merely by prompt wording.

---

# 14. Phase 10 — Full Mission Orchestration

## 14.1 Status

**Code:** `P10`

## 14.2 Objective

Connect the proven components into the complete mission lifecycle.

## 14.3 Required end-to-end flow

```text
Mission
  ↓
Acceptance Contract
  ↓
gstack workflow
  ↓
Resource Scheduler
  ↓
Preflight
  ↓
Provider selection
  ↓
Isolated execution
  ↓
Evidence capture
  ↓
Verification
  ↓
Recovery if necessary
  ↓
Checkpoint
  ↓
Mission completion / blocked-human decision
```

## 14.4 Required crash recovery

Implement/prove the watchdog and crash-recovery behavior defined by Blueprint Section 15.

Required proof:

> real forced-restart test.

## 14.5 Required safety properties

End-to-end execution must preserve:

- state-machine correctness,
- isolated workspace,
- verification-owned success,
- reconciliation semantics,
- evidence provenance/freshness,
- recovery budget,
- monotonic retry,
- capability enforcement,
- persistence integrity,
- human escalation,
- Safe Mode.

## 14.6 P10 gate

A real mission must survive the defined end-to-end failure paths and either:

```text
verify + checkpoint + complete
```

or:

```text
safe recovery
```

or:

```text
blocked + human decision
```

It must never silently claim success.

## 14.7 Parallelism boundary

Multi-agent parallelism remains explicitly out of scope until Phase 10 is bulletproof and separately authorized.

---

# 15. Phase 11 — Polished Frontend

## 15.1 Status

**Code:** `P11`

## 15.2 Objective

Build the polished Mission Mode frontend over the already-proven reliability substrate.

## 15.3 Required behavior

The frontend must render real tested state.

It must not create an alternative state machine.

The UI is a projection/control surface over authoritative backend state.

## 15.4 Required Mission Mode principles

The frontend should expose the actual states and evidence already produced by the backend, including where applicable:

- mission state,
- current step,
- current attempt,
- provider/resource information,
- recovery state,
- acceptance progress,
- evidence,
- events,
- human-decision state,
- blocked state.

## 15.5 UI safety rule

UI controls must not be treated as authoritative transitions.

For example:

```text
UI says "success"
        ≠
AttemptStatus::Succeeded
```

The backend's Verification Engine remains authoritative.

## 15.6 P11 non-goals

Do not use visual polish to hide:

- incomplete reliability,
- missing evidence,
- unverified state,
- provider failures,
- conflicts,
- blocked missions.

## 15.7 P11 gate

The polished UI must render real tested state and remain consistent with backend state under:

- normal operation,
- restart,
- recovery,
- conflict,
- provider failure,
- human escalation.

---

# 16. Cross-Phase Vertical Proof Slices

The phase roadmap is mandatory, but the Build Protocol also requires vertical proof slices.

## VS0 — Minimal deterministic mission

Prove:

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

Then inject:

- timeout,
- process crash,
- persistence interruption,
- duplicate provider response,
- checkpoint interruption.

No major later slice should be treated as proven until this foundation works.

## VS1 — Partial change + repair

Prove:

```text
partial edit
→ reconciliation
→ targeted repair
→ verification
→ checkpoint
```

## VS2 — Real Aider

Prove:

```text
worktree
→ Aider
→ evidence
→ verification
→ accepted merge
```

## VS3 — Real provider

Prove real Gemini/OmniRoute behavior and capture real failure cases.

## VS4 — Handoff

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

## VS5 — gstack

Prove:

```text
Engineering Review
→ Build
→ Review
→ QA
```

with capability enforcement.

---

# 17. Global Test Strategy

Every phase must use the cheapest trustworthy proof first.

```text
LEVEL 1 — Deterministic
        ↓
LEVEL 2 — Simulation
        ↓
LEVEL 3 — Real Integration
```

## Level 1

Use for:

- state transitions,
- failure classification,
- reconciliation,
- acceptance evaluation,
- command generation,
- resource scoring,
- capability decisions.

No network, real clock, or real provider.

## Level 2

Use:

- FakeProvider,
- VirtualClock,
- VirtualFilesystem,
- VirtualGit,
- VirtualProcess,
- VirtualUser,
- Scenario DSL,
- seeded chaos.

## Level 3

Use only where real integration is necessary:

- real Aider,
- real Git,
- real OmniRoute,
- real Gemini/Mistral,
- real OS process behavior.

Never use real provider quota to discover logic that deterministic testing could have caught.

---

# 18. Global Failure-Corpus Rules

Every meaningful bug becomes permanent memory.

Example structure:

```text
failure_cases/
    F-001-timeout-after-write/
    F-002-duplicate-provider-response/
    F-003-user-edit-conflict/
    F-004-checkpoint-crash/
    F-005-stale-capability/
```

Each case must retain:

```text
SCENARIO
INITIAL STATE
TRIGGER
OBSERVED FAILURE
EXPECTED BEHAVIOR
REGRESSION TEST
```

Before removing or weakening a safeguard, the agent must inspect the failure cases that motivated it.

---

# 19. Global Architecture Integrity Audit

Before every phase gate, verify:

```text
[ ] no invariant was weakened
[ ] no state transition was silently changed
[ ] AttemptStatus remains distinct from ReconciliationOutcome
[ ] Succeeded remains Verification-Engine-only
[ ] Conflict still halts rather than silently overwriting
[ ] two-phase merge remains intact
[ ] event chain remains tamper-evident
[ ] evidence provenance/freshness remains intact
[ ] cancellation remains separately represented
[ ] external-side-effect idempotency semantics remain intact
[ ] correlation IDs remain canonical
[ ] Clock abstraction remains respected
[ ] capability enforcement remains at the tool boundary
[ ] persistence guarantees remain intact
[ ] credentials remain outside logs/events/diagnostics
[ ] recovery budget remains monotonic
[ ] multi-agent parallelism has not been introduced early
```

A phase that passes functional tests but fails this audit has **failed the phase gate**.

---

# 20. Global Scope Rules

During any phase:

### Allowed

- work explicitly listed for the active phase,
- tests required to prove that work,
- narrowly necessary implementation support directly required by the active task,
- permanent regression tests for discovered bugs,
- build-state and verification artifacts.

### Not automatically allowed

- unrelated refactors,
- dependency upgrades,
- speculative abstractions,
- UI polish,
- learned routing,
- multi-agent parallelism,
- architecture redesign,
- production configuration changes.

If another change is required:

```text
STOP
→ document
→ request scope/architecture decision
→ resume only when authorized
```

---

# 21. Phase Entry Gate

Before beginning any phase, OX Alpha must produce the following internal/session record:

```text
PHASE ENTRY REPORT

Blueprint:
2.4

Protocol:
1.0

Manifest:
1.0

Phase:
P__

Phase name:
...

Previous phase:
...

Previous phase gate:
PASS / NOT PASS / UNKNOWN

Current repository checkpoint:
...

Current branch/worktree:
...

Known failures:
...

Known unknowns:
...

Relevant Blueprint sections:
...

Authorized work:
...

Explicit non-goals:
...

Required tests:
...

Required evidence:
...

Phase exit criteria:
...

Next authorized action:
...
```

If the previous phase gate cannot be established, stop.

---

# 22. Task Contract Generation Rules

The Phase Manifest does not replace task contracts.

For every bounded task, create a Task Contract containing at minimum:

```yaml
version: 1
id: P?-W??-T??
phase: P?
section_refs:
  - "Blueprint ..."
objective: "..."
allowed_files: []
forbidden_files: []
allowed_commands: []
forbidden_actions: []
dependencies: []
implementation_requirements: []
required_tests: []
acceptance: []
non_goals: []
risk_level: low | medium | high | critical
done_when: []
```

The task must be small enough to:

- understand,
- implement,
- test,
- review,
- checkpoint,
- resume after interruption,
- localize if failed.

---

# 23. Stop / Resume Rules

A phase or task may stop because of:

- context limit,
- network failure,
- provider failure,
- process interruption,
- time/resource exhaustion,
- test failure,
- architecture contradiction,
- scope conflict,
- dependency problem,
- human decision requirement.

Stopping is not failure if state is safely preserved.

Before stopping:

```text
update BUILD_STATE
write STOP_REPORT
record exact unfinished work
record verified checkpoint
record changed files
record tests
record failures
record unknowns
record next exact action
record what must not be repeated
```

When the human later says:

> Continue according to the blueprint.

the agent resumes from recorded evidence rather than conversation memory.

---

# 24. Phase Advancement Rule

The only valid advancement is:

```text
CURRENT PHASE
    ↓
ALL TASKS ACCEPTED
    ↓
ALL REQUIRED TESTS PASS
    ↓
ARCHITECTURE INTEGRITY AUDIT PASS
    ↓
PHASE GATE PASS
    ↓
CHECKPOINT
    ↓
BUILD_STATE UPDATE
    ↓
NEXT PHASE AUTHORIZED
```

No phase may advance on model confidence alone.

---

# 25. Phase Status Vocabulary

Use only controlled status values.

### Phase status

```text
not_started
in_progress
blocked
failed
gate_pending
accepted
```

### Task status

```text
not_started
in_progress
blocked
failed
implemented
verified
accepted
```

### Evidence status

```text
unknown
observed
documented
inferred
verified
```

Never silently promote:

```text
inferred → verified
implemented → accepted
```

---

# 26. Human Decision Boundaries

The following are not silently automated:

- overlapping user/AI changes,
- production-affecting changes,
- security-sensitive exceptions,
- irreversible actions,
- exhausted recovery budget,
- unresolved architecture contradictions,
- unresolved dependency/security concerns.

The system must use the Blueprint's human-decision state where applicable.

---

# 27. Resource-Efficiency Rules

The implementation process must optimize useful progress, not raw token or request count.

Prefer:

```text
small context
+ exact files
+ exact failing criterion
+ exact test
+ targeted repair
```

over:

```text
whole repository
+ whole conversation
+ vague "fix everything"
```

Use dry-run and deterministic simulation before expensive real provider execution where applicable.

Do not spend provider quota to answer questions the repository, deterministic tests, or simulator can answer.

---

# 28. Dry-Run and Shadow Requirements

Where the Blueprint supports dry-run:

```text
compute intended commands
→ show/audit them
→ execute nothing
```

Where new intelligence changes routing/recovery/context selection:

```text
live policy
+
shadow candidate
→ compare verdicts
→ record divergence
→ review
→ promote only after evidence
```

Shadow code must not control active missions.

---

# 29. Security Boundary

At every phase, credentials must remain outside:

- logs,
- events,
- diagnostic bundles,
- mission ledgers,
- model prompts,
- ordinary UI state.

Verification commands must remain inside the defined containment boundary.

Security claims must never exceed the documented threat model.

---

# 30. Phase Manifest as an Authorization Boundary

This file is not merely a roadmap.

It is an authorization boundary.

If the current phase is `P3`, the agent is not authorized to start implementing:

```text
P6 OmniRoute integration
P7 routing intelligence
P8 provider handoff
P9 gstack runtime
P11 polished frontend
```

unless a formal phase change is made.

The agent may read later-phase material when necessary to understand an interface, but reading a later phase does not authorize implementing it.

---

# 31. Phase Dependency Graph

The intended dependency direction is:

```text
P0
 │
 ▼
P1
 │
 ▼
P2
 │
 ▼
P3
 │
 ▼
P4
 │
 ▼
P5
 │
 ▼
P6
 │
 ▼
P7
 │
 ▼
P8
 │
 ▼
P9
 │
 ▼
P10
 │
 ▼
P11
```

Cross-cutting proof slices may span the phases, but they must not violate phase authorization.

---

# 32. First Build Session — Required Order

The first OX Alpha build session must not begin by asking the agent to "build AI Conductor."

It must begin with:

```text
1. verify Blueprint v2.4 exists
2. verify Build Protocol v1.1 exists
3. verify this Phase Manifest exists
4. inspect actual repository
5. create/verify .ai-conductor-build/
6. create BUILD_STATE.json
7. create Phase 0 task contracts
8. create Phase 0 verification gates
9. record repository baseline/checkpoint
10. execute Phase 0 only
11. stop at the Phase 0 gate
```

Phase 1 is not authorized until Phase 0 passes.

---

# 33. First Phase 0 Task Contract Set

The following are the recommended initial bounded contracts derived directly from the Phase 0 checklist.

These are planning identifiers, not permission to implement beyond Phase 0.

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

Each must remain independently reviewable and must not silently turn into implementation.

---

# 34. Final Acceptance Criteria for This Manifest

This Master Phase Manifest itself is accepted only when:

```text
[ ] It identifies Blueprint v2.4 as architecture authority.
[ ] It identifies Build Protocol v1.1 as implementation authority.
[ ] It defines Phase 0 through Phase 11.
[ ] Each phase has a bounded objective.
[ ] Each phase has explicit non-goals.
[ ] Each phase has required work/proof.
[ ] Each phase has a gate.
[ ] Phase advancement is evidence-based.
[ ] Phase 0 is explicitly no-code.
[ ] Multi-agent parallelism remains blocked until the defined point.
[ ] Vertical proof slices are preserved.
[ ] Failure-corpus rules are preserved.
[ ] Architecture-integrity auditing is mandatory.
[ ] Task contracts remain separate from phase manifests.
[ ] Build state remains the persistent implementation memory.
[ ] Stop/resume behavior is defined.
[ ] Later phases are not implicitly authorized early.
[ ] No architecture not present in Blueprint v2.4 is invented.
```

---

# 35. OX Alpha Master Instruction

When using this manifest, the coding agent must behave as follows:

> You are an implementation agent operating under `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`, `AI_CONDUCTOR_BUILD_PROTOCOL_v1.1_HARNESS_NEUTRAL.md`, and this Phase Manifest.
>
> The Blueprint defines the architecture.
>
> The Build Protocol defines how you work.
>
> This Manifest defines what phase is currently authorized.
>
> Do not invent architecture.
>
> Do not silently expand scope.
>
> Do not skip phases.
>
> Do not claim completion without executed evidence.
>
> Do not treat model confidence as proof.
>
> Do not bypass verification.
>
> Do not let reconciliation declare success.
>
> Do not let UI, provider output, Aider output, or skills declare success.
>
> Do not add a dependency merely for convenience.
>
> Do not introduce multi-agent parallelism before its explicit authorization point.
>
> When uncertain, mark `UNKNOWN` or `BLOCKED`.
>
> When the current task is complete, verify it, checkpoint it, update `BUILD_STATE`, and stop at the next safe boundary.
>
> When told "Continue according to the blueprint," reconstruct the exact unfinished state from the repository and recorded evidence, then continue only the currently authorized task.
>
> If architecture, scope, state, or evidence cannot be reconciled safely, stop and escalate rather than guessing.

---

# 36. Authority and Change Control

Changes to this Manifest must identify:

```text
manifest version changed
blueprint version
protocol version
reason
affected phases
affected task contracts
affected verification gates
affected invariants
migration impact
human approval
```

A Blueprint change supersedes this Manifest where the architecture changes.

A Build Protocol change supersedes procedural instructions where the procedure changes.

Neither change silently rewrites already accepted historical evidence.

---

# 37. Closing Principle

The implementation is not considered successful because an AI agent wrote a large amount of code.

It is successful when each phase proves its required contracts, preserves the Blueprint's invariants, survives the defined failures, records evidence, and leaves a trustworthy checkpoint from which another agent can continue without guessing.

The build process therefore follows:

```text
SPECIFY
   ↓
BOUND
   ↓
IMPLEMENT
   ↓
TEST
   ↓
BREAK
   ↓
RECOVER
   ↓
VERIFY
   ↓
CHECKPOINT
   ↓
ADVANCE
```

The project should move quickly because each unit is small and deterministic—not because correctness checks are skipped.

---

**END — AI CONDUCTOR MASTER PHASE MANIFEST v1.1**


---

## v1.1 Harness-Neutral Roadmap Amendment

Phase 5 is deliberately an **evaluation-and-conformance phase**, not a bet on one coding agent.

The permanent roadmap principle is:

```text
Aider / Jcode / future executor
            ↓
      same Adapter Contract
            ↓
      same Conformance Suite
            ↓
    same Conductor authority
```

This preserves future optionality and prevents execution-harness lock-in.
