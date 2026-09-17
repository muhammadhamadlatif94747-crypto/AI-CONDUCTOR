# AI CONDUCTOR
# OX Alpha 48-Hour Execution Campaign — v1.0
## Two-Day High-Intensity Build Directive

> **Supersession note (added on governance update to v2.5.1/v1.1, historical record — not altered above):** this document records what the original campaign was authorized under at the time it was written (Blueprint v2.4, Build Protocol/Phase Manifest/Task Contracts/Verification Gates v1.0, External Components KB v1.0). Current authority is Blueprint v2.5.1 and Build Protocol/Phase Manifest/Task Contracts/Verification Gates v1.1 (Harness-Neutral), plus the new Execution Adapter Contract v1.0 and External Components KB v1.1 — see `.ai-conductor-build/BUILD_STATE.json`'s `authority` block for the live version of record. The campaign's actual accepted task history (P1, P2-W01…W04) is unaffected by this update and remains valid under the new authority chain, per the harness-neutral amendment's own scope (execution-boundary only).

**Primary implementation agent:** OX Alpha in Trae  
**Campaign purpose:** Maximize the remaining OX Alpha preview window by building the strongest verified and reusable AI Conductor foundation possible in two days.  
**Architecture authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.4.md`  
**Permanent procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.0.md`  
**Phase authority:** `AI_CONDUCTOR_PHASE_MANIFEST_v1.0.md`  
**Task authority:** `AI_CONDUCTOR_TASK_CONTRACTS_v1.0.md`  
**Verification authority:** `AI_CONDUCTOR_VERIFICATION_GATES_v1.0.md`  
**Continuity/state authority:** `AI_CONDUCTOR_BUILD_STATE_SPECIFICATION_v1.0.md`  
**Failure-memory authority:** `AI_CONDUCTOR_FAILURE_CORPUS_SPECIFICATION_v1.0.md`  
**External-component knowledge:** `AI_CONDUCTOR_EXTERNAL_COMPONENTS_KNOWLEDGE_BASE_v1.0.md`  
**Human authority:** Project owner  
**Campaign duration:** approximately 48 hours of available OX Alpha preview time  
**Status:** Temporary execution directive; it does not replace the permanent architecture/governance documents.

---

# 0. Executive Directive

OX Alpha:

You are the primary high-capability implementation engineer for this 48-hour campaign.

Your job is **not** to produce the largest amount of code.

Your job is to create the **largest amount of verified, reusable engineering value** before the preview window ends.

The target is:

> **A real, runnable vertical slice of AI Conductor with the reliability foundation strong enough that development can continue safely after the OX Alpha preview ends.**

Use the permanent project documents as the engineering system around you.

Do not treat them as a reason to become passive, bureaucratic, or mechanical.

They exist to prevent:

- guessing,
- hallucinated integrations,
- fake completion,
- scope drift,
- silent architectural changes,
- wasted AI/provider quota,
- repeated work,
- unsafe recovery.

Inside an authorized boundary, you are explicitly expected to use your strongest engineering judgment.

You may:

- choose a better implementation,
- simplify an implementation,
- improve algorithms,
- add useful tests,
- improve error handling,
- discover better internal abstractions,
- optimize performance,
- reduce dependencies,
- improve maintainability,
- improve robustness,
- propose better engineering approaches.

You must not silently change:

- the architecture,
- invariants,
- mission/step/attempt semantics,
- security boundaries,
- acceptance semantics,
- phase authorization,
- external contracts.

If a change to one of those is genuinely required, use the project's change-control process.

**The documents constrain authority and proof, not engineering intelligence.**

---

# 1. Product Mission

The product is:

> **Reliable orchestration for unreliable AI resources.**

It is not:

- a new foundation model,
- a clone of Aider,
- a GUI wrapper around OmniRoute,
- a generic chatbot,
- a flashy dashboard.

The intended architecture is:

```text
gstack
    ↓
ENGINEERING PROCESS / SKILL LAYER

Aider + tools
    ↓
EXECUTION LAYER

OmniRoute + providers
    ↓
CONNECTIVITY / RESOURCE LAYER

AI CONDUCTOR
    ↓
CONTROL / STATE / SAFETY / RECOVERY /
VERIFICATION / HANDOFF / RESOURCE-EFFICIENCY LAYER
```

The central promise is continuity:

```text
provider can fail
model can fail
Aider can fail
network can fail
process can fail
machine can restart

BUT

the mission must not silently disappear.
```

The Blueprint explicitly makes mission continuity, failure recovery, verification, and safe provider substitution the primary product value.

---

# 2. Campaign Objective

The preferred result is **not** superficial completion of all permanent phases.

The preferred result is:

```text
A SMALL REAL SYSTEM THAT WORKS
        +
A TRUSTWORTHY CORE
        +
A REPRODUCIBLE TEST ENVIRONMENT
        +
REAL AIDER INTEGRATION
        +
REAL OMNIROUTE/PROVIDER ADAPTER
        +
VERIFICATION
        +
RECOVERY
        +
PERSISTENT BUILD STATE
        +
CONTINUATION AFTER INTERRUPTION
```

The Blueprint's own strategy is to establish a scientifically testable reliability core before making polished UI, broad provider coverage, or dozens of skills dominant priorities.

---

# 3. Role of This Campaign File

The permanent Phase Manifest remains authoritative.

This file is a **48-hour execution optimizer**.

It compresses the implementation into high-value vertical slices while preserving:

- architecture,
- invariants,
- verification,
- phase authorization,
- security,
- task contracts,
- evidence.

A vertical slice may touch implementation concepts associated with several permanent phases in order to prove an end-to-end behavior. That does **not** mean later phase gates are automatically passed.

---

# 4. Non-Negotiable Campaign Rules

## 4.1 No fake completion

Never claim:

> "Done"

unless the required evidence exists.

Compilation is not enough.

A model response is not enough.

A file existing is not enough.

A test file existing is not enough.

A provider returning HTTP 200 is not enough.

## 4.2 No architecture invention

If the Blueprint defines the architecture, use it.

If the Blueprint leaves an implementation detail open, choose the strongest reasonable implementation.

If there is a genuine contradiction:

```text
STOP AFFECTED PATH
→ record contradiction
→ explain
→ use change-control process
```

Do not silently choose a competing architecture.

## 4.3 No blind guessing about external tools

Use the External Components Knowledge Base, then verify the actual installed/pinned version.

If the local version differs:

```text
knowledge base
+
actual version
+
source/docs
+
safe local probe
```

determine the actual behavior.

## 4.4 No unnecessary blocking

Do not stop merely because an implementation detail is not predetermined.

Use engineering discretion inside the authorized task boundary.

Stop only when the missing information affects:

- architecture,
- correctness,
- safety,
- security,
- required external behavior,
- acceptance,
- or credentials/configuration necessary for live testing.

## 4.5 Do not waste OX Alpha time

Do not spend the preview window on:

- elaborate animations,
- branding,
- broad provider coverage,
- every gstack skill,
- speculative features,
- trivial cleanup,
- unnecessary dependency upgrades,
- ML routing before transparent routing works.

---

# 5. Permanent Files — How to Use Them During This Campaign

### Master Blueprint
Use for architecture, invariants, module ownership, state, recovery, UX target and tool boundaries. Read relevant sections rather than the full document for every task.

### Build Protocol
Use for agent behavior, evidence, scope, stop/resume, testing, dependencies, Git/worktree rules and architecture changes.

### Phase Manifest
Use for official phase objectives, non-goals and permanent gates.

### Task Contracts
Use one bounded contract per real implementation slice. Keep contracts small enough to implement, test, review, checkpoint and resume.

### Verification Gates
Create or instantiate proof criteria with the implementation. Do not postpone verification until the end.

### Build State
Update continuously. It must always tell a fresh session where the project is, what is accepted, what failed, what is unknown, what checkpoint is valid, and what happens next.

### Failure Corpus
Capture serious failures, classify them, reproduce where practical, fix the root cause, and add regression protection.

### External Component Knowledge Base
Use before touching Aider, OmniRoute or gstack. It is a verified knowledge accelerator, not an architecture authority.

---

# 6. First 60 Minutes — Mandatory Boot

Do **not** begin by writing a large product feature.

### Step 1 — Inspect the environment

Determine:

```text
OS
CPU architecture
Node
npm/pnpm/bun
Rust/cargo
Git
Aider version
OmniRoute availability/version
gstack availability/commit
repository state
```

### Step 2 — Read the control files

Read:

```text
Master Blueprint v2.4
Build Protocol v1.0
Phase Manifest v1.0
Task Contracts v1.0
Verification Gates v1.0
Build State Specification v1.0
Failure Corpus Specification v1.0
External Components Knowledge Base v1.0
this 48-hour Campaign
```

### Step 3 — Inspect the actual repository

Do not trust document descriptions over repository reality.

Determine:

```text
what exists
what compiles
what runs
what is missing
what is broken
what is safely reusable
```

### Step 4 — Establish `.ai-conductor-build/`

Create/validate:

```text
BUILD_STATE.json
task_contracts/
verification_gates/
verification_reports/
checkpoints/
failure_cases/
architecture_changes/
dependency_requests/
handoffs/
diagnostics/
```

### Step 5 — Record baseline

Record:

```text
Git branch
HEAD
dirty state
tool versions
current build result
current test result
```

### Step 6 — Produce an Implementation Readiness Report

Include:

```text
actual repository state
actual tool versions
working capabilities
broken capabilities
missing capabilities
first vertical proof slice
highest-priority task
required dependencies
information still needed from human
what can proceed without waiting
```

Do not make large speculative changes during this inspection.

---

# 7. First Product Target — Minimal Real Conductor

Use a tiny test mission.

For example:

> Create a small known file/change in an isolated worktree, verify it, capture evidence, create a checkpoint, simulate interruption, reconstruct state, and safely continue.

The exact test task may be chosen by OX Alpha if it is smaller and stronger.

Required path:

```text
Mission
  ↓
Step
  ↓
Attempt
  ↓
Tool
  ↓
Observation
  ↓
Evidence
  ↓
Verification
  ↓
Checkpoint
  ↓
Restart
  ↓
Reconstruction
  ↓
Resume
```

This is the first behavior that deserves to become real.

---

# 8. DAY 1 — Block A: Deterministic Kernel

Build the pure/deterministic Conductor Kernel first.

It should contain, as applicable:

```text
state transitions
failure classification
reconciliation decisions
acceptance decisions
capability decisions
retry decisions
handoff package construction
resource eligibility
command creation
policy evaluation
```

Keep external effects outside the kernel:

```text
network
filesystem
Git
Aider
OmniRoute
OS process
UI
real clock
```

Target:

```text
PURE KERNEL
    ↓
COMMANDS
    ↓
IMPERATIVE ADAPTERS
```

### Required result

State/reducer behavior testable without external services.

---

# 9. DAY 1 — Block B: Persistence + Events + Build State

Implement enough persistent state for:

```text
Mission
Step
Attempt
Event
Evidence
Checkpoint
Build State
```

Keep the initial implementation lightweight.

Prove:

```text
write
→ restart
→ reload
→ state preserved
```

Then inject:

```text
interrupted write
checkpoint interruption
process crash
```

---

# 10. DAY 1 — Block C: Workspace / Worktree Safety

Implement:

```text
create isolated worktree
capture baseline
run attempt in worktree
capture diff
reconcile
```

Do not let Aider operate directly on the live user workspace.

The Conductor must own the safety boundary.

---

# 11. DAY 1 — Block D: FakeProvider + Virtual Testing

Build the smallest simulator that can break the kernel.

Support enough to test:

```text
success
timeout
disconnect
429
malformed output
partial edit
process crash
duplicate response
checkpoint crash
user edit
conflict
```

Use where practical:

```text
VirtualClock
VirtualFilesystem
VirtualGit
VirtualProcess
FakeProvider
```

Do not overbuild the simulator before the first vertical slice works.

---

# 12. DAY 1 — Block E: Verification Engine

Implement:

```text
Evidence
    ↓
Verification Runner
    ↓
Acceptance Criteria
    ↓
PASS / FAIL
    ↓
authorized state transition
```

Critical rule:

```text
NO MODEL
NO PROVIDER
NO AIDER
NO RECONCILIATION
NO UI

may directly declare AttemptStatus::Succeeded.
```

Only verification may produce the accepted success transition.

---

# 13. DAY 1 — Block F: First Failure Cycle

Deliberately break the system:

```text
timeout after partial edit
process crash during attempt
restart before verification
checkpoint interruption
user edit during recovery
duplicate response
```

For each:

```text
does the system know what happened?
does it avoid false success?
does it avoid blind retry?
does it preserve user changes?
does it create recoverable state?
does it produce evidence?
```

Fix root causes, not symptoms.

---

# 14. DAY 1 — Block G: Aider Adapter

Only after the deterministic substrate is working.

Inspect the actual Aider version.

Probe:

```text
startup
model selection
model metadata
unknown-model warning behavior
non-interactive startup
worktree operation
file edit
diff
process termination
test execution
```

Create an adapter that isolates Aider-specific behavior.

The Conductor needs facts such as:

```text
started
finished
failed
unknown outcome
files changed
```

without letting Aider own mission truth.

---

# 15. DAY 1 — Block H: End-of-Day Vertical Slice

Before ending Day 1, force:

```text
Mission
↓
Attempt
↓
isolated worktree
↓
FakeProvider or Aider
↓
change
↓
evidence
↓
verification
↓
checkpoint
↓
process stop/restart
↓
state reconstruction
↓
resume
↓
verification
```

### Day 1 success

The system demonstrates **real continuity**.

That is more valuable than superficial feature breadth.

---

# 16. DAY 2 — Block A: Real OmniRoute Integration

Inspect the actual pinned/installed OmniRoute.

Discover:

```text
base URL
management API
inference endpoint
providers
connections
models
combos
health state
```

Run a minimal compatibility probe.

Record:

```text
actual invocation syntax
actual response shape
actual selected model/provider
actual combo behavior
```

Do not guess combo syntax from examples.

---

# 17. Combo/Model Information — Do Not Block Early Work

Do not wait for combo names during kernel work.

Initially use placeholders/interfaces such as:

```text
PRIMARY_COMBO
CODING_COMBO
REVIEW_COMBO
FALLBACK_COMBO
MODEL_A
MODEL_B
```

When the live OmniRoute boundary is reached:

1. discover what can be discovered automatically;
2. ask the human only for missing information;
3. never request information the environment can safely reveal.

---

# 18. When to Ask the Human for Combos/Models

Ask only once live testing needs them.

Request:

```text
Primary coding combo:
Fallback combo:
Review combo:
Optional specialized combo:

Available provider/model mappings:
...

Preferred free-first order:
...
```

If the user does not know exact IDs, use the local OmniRoute discovery/management interface where possible.

Never put credentials in source.

---

# 19. Credential Handling

When credentials are actually required:

Ask for:

```text
provider
credential type
intended use
secure storage mechanism
```

Never ask the human to put a secret into:

```text
source
Git
Task Contract
Build State
Failure Corpus
Verification report
prompt text
```

Do not echo credentials into logs.

---

# 20. DAY 2 — Block B: Provider Capability Registry

Implement the minimal registry:

```text
provider
connection/account
credential identity
model
combo
capabilities
context limits
health
quota
cooldown
availability
```

Unknown capability remains:

```text
UNKNOWN
```

until verified.

---

# 21. DAY 2 — Block C: First Useful Resource Scheduler

Do not build ML routing.

Use transparent:

```text
eligibility
+
capability fit
+
context fit
+
health
+
quota headroom
+
cooldown
+
reliability
+
risk
+
recovery cost
```

Every important choice should be explainable.

Example:

```text
Mistral selected because:
✓ capability match
✓ healthy
✓ enough context
✓ sufficient quota
✓ lower recovery risk
```

---

# 22. DAY 2 — Block D: Real Provider Failure

Prove one real recovery route:

```text
Gemini
  ↓
failure
  ↓
classification
  ↓
reconciliation
  ↓
resume package
  ↓
Mistral
  ↓
continue
  ↓
verification
```

Do not attempt broad provider coverage.

Prove one real handoff.

---

# 23. Handoff Package

Minimum package:

```text
mission
current skill
current step
status
completed work
unfinished work
relevant files
important decisions
constraints
failures already attempted
evidence
last safe checkpoint
next safe action
do-not-repeat information
```

Never transfer the full raw transcript just because it is available.

---

# 24. DAY 2 — Block E: Thin gstack Runtime

Implement only the core loop first:

```text
Office Hours / Planning
Engineering Review
Build
Review
QA
Debug
```

If the reliable core is already strong and time remains:

```text
Design Review
Security
Ship
Retro
```

Each skill requires:

```text
input contract
output artifact
capabilities
permissions
forbidden actions
state interaction
verification path
```

Do not let gstack become the owner of mission state.

---

# 25. DAY 2 — Block F: Capability Enforcement

Prove a negative case:

```text
Engineering Review
        ↓
unauthorized code mutation
        ↓
tool boundary
        ↓
DENIED
```

Then:

```text
Build
        ↓
authorized code mutation
        ↓
ALLOWED
```

Prompt instructions alone are not enough.

---

# 26. DAY 2 — Block G: Minimal Mission UI

Only after core behavior is real.

Create a functional mission screen:

```text
MISSION
Objective

Workflow
✓ Planning
✓ Engineering Review
◉ Build
○ Review
○ QA

Current Work
...

Resource
Automatic

Checkpoint
...

Evidence
...

[Pause] [Stop]
```

Do not spend the OX Alpha preview window on final polish.

---

# 27. Parallel Tool Strategy

## OX Alpha — primary

Use OX Alpha for:

```text
state engine
recovery
reconciliation
persistence
verification
worktrees
Aider adapter
OmniRoute adapter
resource scheduler
handoff
capability enforcement
failure simulation
tests
hardening
```

## Replit — secondary

Use Replit for:

```text
UI
API shell
dashboard
visualization
web preview
evidence viewer
settings presentation
```

Never allow it to silently change:

```text
mission semantics
verification authority
provider contracts
security invariants
state model
```

## Google AI Studio — experimentation

Use AI Studio for:

```text
UI ideas
visual variants
small prototypes
prompt experiments
structured-output experiments
```

Promote only useful work into the canonical repository.

---

# 28. One Canonical Repository

Use one canonical repository.

```text
                 CANONICAL GIT REPO
                        ↑
        ┌───────────────┼───────────────┐
        │               │               │
      OX Alpha        Replit        AI Studio
      core work       UI/API        experiments
```

No competing permanent codebases.

Accepted work returns to the canonical repository through normal verification.

---

# 29. Context and Quota Economics

Before real model execution:

```text
preflight
↓
minimum sufficient context
↓
execute
```

Do not send the whole project and full conversation by default.

Prefer:

```text
task contract
+
mission state
+
relevant files
+
recent diff
+
acceptance criteria
+
relevant failure cases
```

Measure where practical:

```text
tokens sent
duplicate context
handoff size
retry waste
recovery overhead
```

---

# 30. Recovery Economics

Never do:

```text
failure
→ full restart
→ full context
→ same model
→ same prompt
```

Prefer:

```text
failure
→ classify
→ reconcile
→ isolate failed criterion
→ targeted repair
→ minimal context
→ appropriate resource
```

A repeated retry should be meaningfully different.

---

# 31. Safe Mode

During unstable integration development, use Safe Mode to disable or require approval for:

```text
automatic handoff
automatic repair
automatic merge
destructive operations
```

Prove the path manually, then enable automation one boundary at a time.

---

# 32. Dry Run

Use dry-run before high-impact real actions when practical:

```text
compute intended commands
→ inspect
→ execute only if safe
```

Especially useful for:

```text
new OmniRoute combos
new merge behavior
new recovery behavior
new provider routing
```

---

# 33. Engineering Discretion Zone

This is a direct instruction to OX Alpha.

Within the authorized Task Contract:

```text
IF a better implementation is discovered
AND it preserves the Blueprint
AND preserves invariants
AND remains within scope
AND can be verified
THEN USE THE BETTER IMPLEMENTATION.
```

You are free to improve:

```text
algorithms
internal data structures
error handling
testing
performance
dependency footprint
maintainability
developer ergonomics
```

You are not required to copy illustrative code from the Blueprint or this campaign.

If the improvement changes architecture, invariants, security boundaries or acceptance semantics, use the formal change-control process.

---

# 34. When to Escalate to the Human

Do not ask the human for routine engineering choices.

Ask only when genuinely required for:

```text
credentials
combo/model mapping needed for live tests
irreversible product decisions
architecture change
security decisions
scope change
human-only acceptance
```

When asking, state:

```text
WHAT I NEED
WHY
WHAT CAN CONTINUE WITHOUT IT
EXACT FORMAT
```

Do not unnecessarily stop unrelated work.

---

# 35. Failure Handling During the Campaign

When a serious failure occurs:

```text
classify
↓
preserve evidence
↓
reproduce when practical
↓
fix root cause
↓
create/update regression case
↓
verify
```

If ambiguous:

```text
UNKNOWN
↓
reconcile
```

Do not blind retry.

---

# 36. End-of-Day and End-of-Preview Reports

At the end of Day 1, write:

```text
DAY1_STATUS_REPORT.md
```

At the end of the two-day preview campaign, write:

```text
PREVIEW_END_STATE_REPORT.md
```

The final report must contain:

```text
WHAT ACTUALLY WORKS
WHAT IS VERIFIED
WHAT IS PARTIAL
WHAT IS NOT BUILT
WHAT IS UNKNOWN
KNOWN FAILURES
KNOWN LIMITATIONS
LAST VERIFIED CHECKPOINT
LAST VERIFIED COMMIT
CURRENT BUILD STATE
EXACT NEXT TASK
EXACT NEXT SAFE ACTION
```

No marketing language.

No invented percentages.

No unverified "production ready" claim.

---

# 37. Two-Day Final Definition of Success

Prefer a narrow verified success over broad superficial completion.

Minimum preferred result:

```text
[ ] Conductor core runs
[ ] Mission/Step/Attempt exists
[ ] State persists across restart
[ ] Isolated worktree execution works
[ ] Evidence is captured
[ ] Verification controls completion
[ ] Fake failure injection works
[ ] At least one meaningful recovery path works
[ ] Aider adapter is functional
[ ] OmniRoute adapter is functional or precisely bounded with probe evidence
[ ] One real provider path works
[ ] One real provider failure path is handled
[ ] Handoff structure is implemented or strongly wired
[ ] Thin gstack Skill Runtime foundation exists
[ ] Capability enforcement has a negative test
[ ] Build State can resume a later session
[ ] Failure Corpus contains real regression cases
[ ] Repository has a known verified checkpoint
[ ] No secrets are committed
[ ] No fake completion claims exist
```

---

# 38. If Time Runs Out

If OX Alpha's preview expires before everything is complete:

Do not rush into unverified work.

Leave:

```text
verified checkpoint
BUILD_STATE
STOP_REPORT
verification reports
failure cases
task contracts
current repository
PREVIEW_END_STATE_REPORT
```

The final report must identify the exact next task and why it is next.

The system is designed for another agent to continue.

---

# 39. Continuation After OX Alpha

A later agent should only need:

> **Continue according to the blueprint and current build state.**

It must reconstruct:

```text
phase
task
checkpoint
verification
failures
unknowns
next action
```

from durable artifacts.

Do not rely on conversational memory.

Do not repeat accepted work unless evidence invalidates it.

---

# 40. The First Prompt to Send OX Alpha

Copy/paste this as the first user message after placing the documents in the workspace:

```text
You are the primary engineering agent for AI Conductor.

You have a limited implementation window. Your objective is NOT to write the most code possible. Your objective is to create the maximum amount of VERIFIED, REUSABLE engineering value before this window ends.

Read these files before making substantial changes:

1. AI_CONDUCTOR_MASTER_BLUEPRINT_v2.4.md
2. AI_CONDUCTOR_BUILD_PROTOCOL_v1.0.md
3. AI_CONDUCTOR_PHASE_MANIFEST_v1.0.md
4. AI_CONDUCTOR_TASK_CONTRACTS_v1.0.md
5. AI_CONDUCTOR_VERIFICATION_GATES_v1.0.md
6. AI_CONDUCTOR_BUILD_STATE_SPECIFICATION_v1.0.md
7. AI_CONDUCTOR_FAILURE_CORPUS_SPECIFICATION_v1.0.md
8. AI_CONDUCTOR_EXTERNAL_COMPONENTS_KNOWLEDGE_BASE_v1.0.md
9. AI_CONDUCTOR_OX_ALPHA_48_HOUR_EXECUTION_CAMPAIGN_v1.0.md

Authority order:

actual verified repository/evidence
>
human-approved architecture changes
>
Master Blueprint
>
Build Protocol
>
Phase Manifest
>
Task Contract
>
Verification Gate
>
previous agent statements/chat
>
assumptions

Do not invent facts.

Do not claim completion without actual evidence.

Do not silently change architecture, invariants, security boundaries, mission semantics, acceptance rules, or phase authorization.

However, do NOT treat these documents as a restriction on your engineering intelligence.

Inside an authorized Task Contract you have full engineering discretion to choose a better implementation, improve robustness, simplify internals, write stronger tests, reduce dependencies, improve performance, or create better error handling, provided the architectural contracts, invariants, scope, security rules and acceptance semantics remain intact.

The documents define boundaries and proof requirements. They do not dictate every implementation detail.

FIRST, DO NOT BUILD A LARGE FEATURE.

Perform a complete implementation-readiness inspection:

1. inspect the actual repository;
2. inspect the actual environment;
3. determine installed versions of Git, Aider, OmniRoute if available, gstack if available, Node/Bun/Rust and project tooling;
4. read the governing documents;
5. inspect/create .ai-conductor-build;
6. create or validate BUILD_STATE.json;
7. record the repository baseline;
8. inspect what currently works;
9. inspect what currently fails;
10. inspect what can be safely reused;
11. identify the first end-to-end reliability vertical slice.

Then give me a concise readiness report containing:
- repository reality;
- actual tool versions;
- working capabilities;
- broken capabilities;
- missing pieces;
- first vertical slice;
- current highest-priority task;
- dependencies;
- information that genuinely requires me;
- what you can proceed with without waiting.

Do NOT wait for combo names or model mappings if they are not yet required.

When the real OmniRoute/provider integration boundary is reached, discover what can be discovered locally and ask me only for the missing information you truly need.

Do NOT ask me to paste secrets into source code, Git, documents, logs or chat.

Do NOT ask me for something the environment can safely discover itself.

After the readiness report, proceed with the authorized campaign.

Priority:

1. deterministic Conductor Kernel
2. persistent state/event/checkpoint foundation
3. isolated worktree/reconciliation safety
4. FakeProvider/virtual failure testing
5. Verification Engine
6. Aider adapter and real execution
7. OmniRoute/provider adapter
8. provider capability/resource selection
9. one real provider path
10. one real failure/recovery/handoff path
11. thin gstack Skill Runtime
12. minimal functional mission UI
13. hardening/regression

Do not spend this window on visual polish or broad provider coverage before the reliability vertical slice works.

For every implementation slice:

- create/use a bounded Task Contract;
- run preflight;
- implement the smallest safe change;
- use engineering discretion inside that boundary;
- test;
- verify;
- inspect the diff;
- record evidence;
- checkpoint;
- update BUILD_STATE.

If a failure occurs:
classify it, preserve it, reproduce it where practical, fix the root cause, add regression protection, and verify again.

If an outcome is ambiguous:
DO NOT blindly retry.
Reconcile first.

If architecture is genuinely contradictory:
stop only the affected path, record the contradiction, and use the architecture-change process.

If you discover a better implementation inside the current architecture:
USE IT.

If the idea requires changing an invariant, security boundary, state model, acceptance rule or architecture:
DO NOT silently implement it; record the change and ask for the appropriate decision.

If the context/network/process window stops you:
leave BUILD_STATE + STOP_REPORT + exact next action.

When I later say:
"Continue according to the blueprint."

reconstruct the state from BUILD_STATE + repository + task contracts + verification evidence + checkpoints and continue only the authorized unfinished work.

Do not restart blindly.
Do not rely on conversational memory.
Do not repeat verified work unless evidence invalidates it.

Begin with the implementation-readiness inspection now.
```

---

# 41. What You Should Say About Combos/API Keys

You **do not need to give the combo names or all model/API details in the first prompt**.

That would unnecessarily block OX Alpha.

Start with the readiness inspection.

When it reaches OmniRoute live integration, let it tell you exactly what it needs.

Then give:

```text
Primary coding combo:
...

Fallback combo:
...

Review combo:
...

Available models/providers:
...

Preferred free-first resource:
...
```

For credentials, use the secure mechanism it identifies rather than putting them into the prompt or source.

---

# 42. Final Campaign Rule

The two-day OX Alpha window must buy something that survives beyond the two days:

```text
real code
+
real tests
+
real failure cases
+
real evidence
+
real checkpoints
+
real Build State
+
real integrations
+
real continuation path
```

Not merely:

```text
many files
+
many features
+
a beautiful demo
```

The project should finish this window **harder to break, easier to continue, and closer to the actual product**.

That is the correct use of the remaining OX Alpha capacity.
