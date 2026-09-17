# AI CONDUCTOR
# Verification Gates — v1.1 (Harness-Neutral)
## Evidence-Driven Completion, Acceptance, and Advancement System

**Authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`  
**Build procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.1_HARNESS_NEUTRAL.md`  
**Phase authorization:** `AI_CONDUCTOR_PHASE_MANIFEST_v1.1_HARNESS_NEUTRAL.md`  
**Task definition:** `AI_CONDUCTOR_TASK_CONTRACTS_v1.1_HARNESS_NEUTRAL.md`  
**Human authority:** Project owner  
**Status:** Operational verification specification

---

# 0. Purpose

This document defines the **Verification Gate layer** of AI Conductor.

Verification Gates are the objective barrier between:

```text
"the agent implemented something"
```

and:

```text
"the system has evidence that the authorized work is correct enough to accept."
```

The purpose is to prevent:

- model self-certification,
- false completion,
- tests that were written but not executed,
- tests that passed against stale code,
- unrelated changes being accepted,
- weak evidence being mistaken for strong evidence,
- requirements being accepted without a verification method,
- a successful model response being mistaken for a successful software outcome,
- a phase advancing while its foundations remain unproven.

The governing principle is:

> **A claim is not a result. An executed, attributable, relevant, fresh, and sufficient verification result is evidence of a result.**

This file is deliberately separate from Task Contracts.

A Task Contract says:

> **What must be built and what must be proven.**

This document says:

> **How the proof is evaluated and what gate result is authoritative.**

---

# 1. Source-of-Truth Stack

```text
MASTER BLUEPRINT v2.4
        ↓
BUILD PROTOCOL v1.1
        ↓
PHASE MANIFEST v1.1
        ↓
TASK CONTRACT
        ↓
VERIFICATION GATE
        ↓
BUILD STATE
        ↓
ACTUAL REPOSITORY + EXECUTED EVIDENCE
```

The final authoritative fact is always the actual repository state plus real executed evidence.

Conversation text is never sufficient proof.

A model's statement is never sufficient proof.

A previous unverified report is never sufficient proof.

---

# 2. Verification Vocabulary

The verification system uses controlled terms.

## 2.1 Implemented

The required code/artifact exists within task scope.

```text
IMPLEMENTED
```

does not imply correctness.

---

## 2.2 Tested

One or more specified verification commands/tests were actually executed.

---

## 2.3 Verified

The required technical evidence passed according to the gate.

---

## 2.4 Accepted

All mandatory acceptance criteria passed, evidence is valid, scope is clean, architecture integrity is preserved, and the required checkpoint/build-state update has been completed.

---

## 2.5 Rejected

Required proof failed.

---

## 2.6 Blocked

Proof cannot safely proceed because of:

- missing dependency,
- unresolved ambiguity,
- unavailable environment,
- architecture decision,
- required human decision,
- unsupported repository condition,
- security boundary,
- insufficient evidence.

Blocked is not equivalent to Failed.

---

## 2.7 Invalidated

Evidence that was once valid is no longer trustworthy.

Examples:

- source changed after a test,
- configuration changed,
- test ran against a different commit,
- provider capability became stale,
- evidence refers to an old attempt,
- environment changed materially,
- checkpoint no longer matches repository state.

Invalidated evidence must not be reused as if it were current.

---

# 3. Gate Hierarchy

Verification occurs at four levels.

```text
LEVEL A — Acceptance Criterion Gate
        ↓
LEVEL B — Task Gate
        ↓
LEVEL C — Vertical Proof Slice Gate
        ↓
LEVEL D — Phase Gate
```

A higher gate may require all lower gates relevant to its scope to pass.

No higher gate may override a failed lower gate.

---

# 4. Verification Gate State Machine

Each gate has a lifecycle:

```text
DRAFT
  ↓
READY
  ↓
RUNNING
  ↓
PASSED
```

Exceptional paths:

```text
RUNNING → FAILED
RUNNING → BLOCKED
RUNNING → INVALIDATED
PASSED → INVALIDATED
DRAFT/READY → CANCELLED
```

A gate may not become `PASSED` merely because the expected artifact exists.

---

# 5. Gate Contract Schema

Every executable verification gate must be represented in machine-readable form.

Canonical YAML:

```yaml
gate_version: 1

id: VG-P?-W??-T??-01

scope:
  type: acceptance | task | vertical_slice | phase | release
  id: "..."

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"
  task_contract: "..."

objective: >
  ...

preconditions: []

required_evidence: []

verification_steps: []

acceptance_criteria: []

scope_checks: []

integrity_checks: []

failure_cases: []

evidence_policy: {}

environment: {}

outputs: []

pass_conditions: []

fail_conditions: []

block_conditions: []

invalid_conditions: []

risk_level: low | medium | high | critical

status: DRAFT
```

---

# 6. Gate Preconditions

A verification gate may run only when its preconditions are satisfied.

Typical preconditions:

```text
[ ] referenced Task Contract exists
[ ] Task Contract version matches
[ ] task is in a verifiable state
[ ] repository is available
[ ] checkpoint/baseline is known
[ ] required commands are available
[ ] required dependencies are available
[ ] evidence source is available
[ ] test environment is identified
[ ] no unresolved architecture decision blocks the test
```

If a precondition is not established:

```text
BLOCKED
```

Do not convert uncertainty into a pass.

---

# 7. Evidence Model

Every verification result must identify the evidence it used.

A useful evidence record:

```yaml
evidence:
  id: EV-...
  type: test_output | diff | filesystem_observation | git_observation |
        runtime_observation | provider_observation | security_check |
        browser_check | manual_review | requirement_check

  source: "..."
  execution_id: "..."
  task_id: "..."
  attempt_id: "..."
  commit_id: "..."
  worktree_id: "..."
  timestamp: "..."
  clock_source: real | virtual
  freshness: "..."
  provenance: "..."
  integrity: verified | unverified | invalid
  reproducibility: deterministic | repeatable | observational
  result: pass | fail | unknown
  artifact_reference: "..."
```

Evidence must be attributable.

---

# 8. Evidence Trust Rules

The verification system must preserve the project's evidence classes:

```text
OBSERVED
VERIFIED
DOCUMENTED
INFERRED
UNKNOWN
```

## 8.1 OBSERVED

Directly observed in an actual environment.

Examples:

- command output,
- filesystem state,
- Git diff,
- process result,
- provider response.

Observation is useful evidence but is not automatically sufficient for acceptance.

---

## 8.2 VERIFIED

Confirmed through reproducible testing or controlled experiment.

This is the preferred class for deterministic behavior.

---

## 8.3 DOCUMENTED

Confirmed from an authoritative external/project source.

Documentation can establish an expected contract but does not automatically prove runtime behavior.

---

## 8.4 INFERRED

Reasoned from evidence.

Inference must not silently satisfy an acceptance criterion that requires executable proof.

---

## 8.5 UNKNOWN

Insufficient evidence.

Unknown causes:

```text
BLOCKED
```

or:

```text
FAIL
```

depending on whether the evidence can be obtained within the gate.

It never becomes PASS merely because continuing would be convenient.

---

# 9. Evidence Freshness

Evidence is valid only for the state it actually tested.

The gate must invalidate evidence when relevant inputs have changed.

Examples:

```text
source changed after build
→ build evidence invalid

test configuration changed
→ test evidence invalid

provider capability changed
→ capability evidence may be stale

worktree changed after diff inspection
→ diff evidence invalid
```

Where possible, evidence should identify:

```text
commit SHA
worktree ID
file/content fingerprints
configuration fingerprint
dependency lock state
task/attempt ID
```

---

# 10. Verification Environment Identity

Every meaningful executable verification should record the environment sufficiently to reproduce the result.

At minimum where applicable:

```text
OS
architecture
toolchain version
runtime version
package/dependency lock state
environment mode
worktree identity
commit/revision
provider/model identity
test configuration
```

Never claim universal correctness from a result produced in a materially different environment.

---

# 11. Verification Command Policy

Verification commands are an execution boundary.

They must be:

- explicitly authorized by the Task Contract or gate,
- constrained to the correct worktree/environment,
- prevented from reading credentials,
- prevented from writing outside allowed surfaces,
- subject to timeout/resource controls,
- observed for mutation.

An executable test such as:

```text
npm test
```

must not be treated as harmless merely because its name contains "test"; child processes and package scripts can execute arbitrary operations.

The actual verification sandbox defined by the Blueprint controls execution.

---

# 12. Cheapest Reliable Proof First

Default verification ordering:

```text
1. file existence / artifact existence
2. scope / changed-file inspection
3. syntax
4. static validation
5. typecheck
6. build
7. deterministic unit tests
8. simulation
9. integration tests
10. runtime/browser checks
11. requirement checks
12. expensive external checks
13. human review where required
```

Stop at the earliest authoritative failure.

Do not waste expensive provider calls or browser runs if a cheap prerequisite already disproves success.

---

# 13. Acceptance Criterion Gate

Each acceptance criterion must have:

```text
criterion
verification method
evidence requirement
pass condition
fail condition
```

Example:

```yaml
- id: AC-01
  criterion: "Illegal AttemptStatus transition is rejected."
  verification_method: "deterministic unit test"
  evidence_required: "test output + commit identity"
  pass_condition: "specified test passes"
  fail_condition: "test fails or does not execute"
```

A criterion without a verification method is incomplete and cannot be accepted.

---

# 14. Machine-Verifiable Requirements

These should be verified by deterministic or executable methods wherever practical.

Examples:

```text
enum transition exists
file exists
schema matches
test passes
API returns expected response
build succeeds
lint succeeds
typecheck succeeds
diff scope matches
migration applies
migration rollback works
```

Do not use model opinion as the primary proof.

---

# 15. Human-Verifiable Requirements

Some requirements cannot be fully reduced to deterministic tests.

Examples:

```text
visual hierarchy
design quality
whether a workflow feels understandable
product wording
complex UX judgment
certain architectural trade-offs
```

For these:

```text
machine checks
+
explicit human review
```

The gate must state that human acceptance is required.

---

# 16. Model-Assisted Requirements

Models may be used as reviewers or inspectors, but their result remains advisory unless independently verified.

Example:

```text
model review:
"Possible architectural issue in module X."
```

This becomes:

```text
review finding
```

not:

```text
accepted failure
```

A model-assisted finding should lead to:

```text
investigate
→ evidence
→ deterministic/integration proof
→ decision
```

---

# 17. Task Gate

A Task Gate evaluates the entire Task Contract.

It must confirm:

```text
[ ] objective satisfied
[ ] allowed scope only
[ ] forbidden surface untouched
[ ] required implementation obligations satisfied
[ ] all required tests executed
[ ] all required tests passed
[ ] required failure scenarios passed
[ ] acceptance criteria passed
[ ] evidence is current
[ ] evidence is attributable
[ ] no critical unknown remains
[ ] architecture integrity preserved
[ ] checkpoint requirement satisfied
[ ] BUILD_STATE can be updated consistently
```

If any mandatory item fails:

```text
TASK GATE = FAILED
```

---

# 18. Scope Verification Gate

Before acceptance, compare:

```text
Task Contract allowed_files
        vs
Actual changed files
```

Also compare:

```text
allowed_commands
        vs
actual commands executed
```

and:

```text
dependencies
        vs
actual dependency changes
```

Any unapproved material deviation is:

```text
SCOPE VIOLATION
```

unless formally amended.

---

# 19. Diff Verification

For implementation tasks, inspect:

```text
git diff
```

or equivalent repository/worktree diff.

Verify:

- intended files changed;
- unrelated files did not;
- no secrets entered source;
- no accidental generated artifacts appeared;
- no debug-only bypass remains;
- no test disabled without authorization;
- no TODO substituted for required behavior;
- no fake implementation was inserted.

A clean compile does not substitute for a clean diff.

---

# 20. Architecture Integrity Gate

Any task touching core architecture must run an integrity checklist.

At minimum:

```text
[ ] AttemptStatus remains distinct from ReconciliationOutcome
[ ] Succeeded remains Verification-Engine-only
[ ] Conflict remains explicit
[ ] Two-phase merge remains protected
[ ] Event chain remains intact
[ ] Evidence provenance/freshness remains intact
[ ] Cancellation semantics remain intact
[ ] External-side-effect idempotency semantics remain intact
[ ] Mission/Step/Attempt state semantics remain intact
[ ] Clock abstraction remains respected
[ ] Capability enforcement remains at the tool boundary
[ ] Persistence semantics remain intact
[ ] Credentials remain outside evidence/logs
[ ] Recovery budget remains bounded
[ ] No unauthorized parallel-agent model introduced
```

A phase gate may not pass if its architecture-integrity audit fails.

---

# 21. Security Verification Gate

Security-sensitive tasks require explicit security evidence.

Examples:

```text
credential not present in logs
credential not included in diagnostics
verification cannot reach secrets
capability boundary blocks forbidden tool
production path requires authorization
sandbox restricts descendants
```

Security claims must remain within the Blueprint's defined threat model.

---

# 22. Persistence Verification Gate

For persistence/state tasks, verification must include applicable fault injection:

```text
normal write
write interruption
restart after interruption
partial state recovery
event-chain verification
outbox recovery
schema-version handling
```

The test should establish:

```text
known committed state
OR
known recoverable intent
```

not an ambiguous silently corrupted state.

---

# 23. State Machine Verification Gate

For State Engine changes, use a transition matrix.

Example:

```text
             Pending Running Succeeded Failed Unknown Cancelled
Pending        ✓       ✓        ✕       ✕       ✕       ✕
Running        ✕       ✕        ✕       ✓       ✓       ✓
Unknown        ✕       ✕        ✕       ✓       ✕       ✕
Succeeded      ✕       ✕        ✕       ✕       ✕       ✕
Failed         ✕       ✕        ✕       ✕       ✕       ✕
Cancelled      ✕       ✕        ✕       ✕       ✕       ✕
```

The exact matrix is governed by the Blueprint's authoritative state model; this example is illustrative and must not override it.

Tests must cover:

- each permitted edge;
- each prohibited edge;
- terminal-state immutability;
- protected `Succeeded` transition.

---

# 24. Reconciliation Verification Gate

For reconciliation tasks, prove at least:

```text
NoChange
ExpectedChange
UnexpectedChange
PartialChange
UserChange
Conflict
Ambiguous/Unknown
```

The exact enum values must match the Blueprint implementation.

Required rule:

```text
Reconciliation
    ↓
classifies reality
    ↓
does NOT declare Succeeded
```

Verification then decides whether the accepted outcome is valid.

---

# 25. Merge Verification Gate

For two-phase merge tasks:

```text
isolated worktree result
        ↓
live baseline check
        ↓
expected-state conditional merge
        ↓
conflict detection
        ↓
post-merge diff verification
        ↓
post-merge verification
        ↓
checkpoint acceptance
```

Required scenarios:

- clean merge,
- user modification,
- external Git mutation,
- deleted file,
- new file,
- rename,
- conflict,
- merge interruption,
- post-merge verification failure.

Do not mark the merge accepted before post-merge verification.

---

# 26. Provider Verification Gate

For provider/resource work:

Verify:

```text
provider identity
model identity
capabilities
health
quota/rate state
cooldown
response handling
structured errors
streaming behavior where applicable
timeouts
cancellation
```

Provider claims must be based on:

```text
documentation
AND/OR
controlled live observation
```

depending on the specific acceptance requirement.

Never fabricate provider behavior to complete a gate.

---

# 27. Handoff Verification Gate

For provider handoff:

```text
source attempt
↓
failure classification
↓
reconciliation
↓
resume package generation
↓
destination resource
↓
context restoration
↓
no-repeat constraints honored
↓
continuation
↓
verification
```

Verify:

- completed work preserved;
- relevant context transferred;
- unresolved ambiguity explicitly transferred;
- failed strategy is not blindly repeated;
- destination provider has required capabilities;
- recovery budget is respected;
- verification still controls success.

---

# 28. gstack Skill Verification Gate

For skill runtime tasks:

Every skill must be verified for:

```text
contract input
output artifact
capabilities
permissions
forbidden actions
state interaction
handoff compatibility
verification behavior
```

Example:

```text
Engineering Review
→ proposes architecture findings
→ may inspect
→ may not mutate production code

Build
→ may modify authorized code
→ remains bounded by task contract

QA
→ runs authorized verification
→ cannot declare Succeeded directly
```

The tool-execution boundary must enforce these rules mechanically wherever possible.

---

# 29. Resource Scheduler Verification Gate

For routing/scheduler tasks, test:

```text
capability eligibility
health eligibility
quota eligibility
cooldown exclusion
context fit
risk policy
resource availability
explainability
policy versioning
```

Required properties:

```text
unavailable resource is not selected
incompatible resource is not selected
exhausted resource is not selected
policy forbidden resource is not selected
selection is reproducible for the same inputs
```

Do not introduce opaque learned behavior until the Blueprint explicitly authorizes it.

---

# 30. Context Engine Verification Gate

For context-related tasks:

Verify:

```text
minimal sufficient context
relevant-file selection
scope constraints
context budget
fingerprint/caching correctness
cache invalidation
no stale code included as current truth
handoff context compactness
```

Where possible, compare:

```text
needed evidence
vs
context transmitted
```

The goal is to reduce unnecessary provider usage without omitting required information.

---

# 31. Verification of the "Continue" Path

This is a critical product behavior.

A Continue Gate must prove:

```text
saved BUILD_STATE
+
repository state
+
checkpoint
+
Task Contract
+
STOP_REPORT
        ↓
reconstruct exact unfinished state
        ↓
resume only authorized work
        ↓
do not repeat accepted work
        ↓
verify
        ↓
checkpoint
```

Required scenarios:

- context limit,
- network failure,
- provider failure,
- process crash,
- Trae restart,
- task complete but BUILD_STATE stale,
- repository changed unexpectedly,
- active process unknown,
- conflict.

The system must prove that "Continue according to the blueprint" is a recoverable procedure, not a natural-language hope.

---

# 32. Failure-Injection Gate

Every reliability subsystem should have at least one intentional failure scenario.

At phase level, the gate should verify applicable classes:

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

For each applicable scenario:

```text
trigger
→ observed evidence
→ classification
→ expected transition
→ expected recovery
→ final outcome
```

---

# 33. Chaos Verification Gate

A chaos run is not accepted merely because the runner completed.

It must prove:

```text
scenario reproducible
seed recorded
expected outcome known
actual outcome captured
invariants preserved
unexpected divergence recorded
```

A failed chaos scenario becomes:

```text
REGRESSION CASE
```

before the broader gate can pass.

---

# 34. Concurrency Verification Gate

For concurrency-sensitive tasks:

Use deterministic scheduling or controlled interleavings.

Test:

```text
event ordering
duplicate commands
simultaneous cancellation
checkpoint race
reconciliation race
state write interleaving
provider callback race
UI command race
```

The goal is not merely "no crash."

It is:

```text
same controlled inputs
+
same interleaving
=
same valid state
```

where deterministic behavior is required.

---

# 35. Performance / Lightweight Gate

The tool has an explicit requirement to remain lightweight.

Where a phase affects performance, verify:

```text
startup time
memory footprint
core operation latency
context size
serialization size
worktree overhead
test time
provider calls
unnecessary network activity
```

Do not invent benchmark numbers.

Record actual measured values.

Use the project benchmark targets only once they are measured and approved.

---

# 36. Regression Gate

Before task acceptance:

```text
existing deterministic suite
+
relevant simulation suite
+
task-specific tests
+
relevant historical failure cases
```

must pass.

A historical failure case that becomes invalid because architecture changed must be explicitly reviewed and superseded; it must not simply disappear.

---

# 37. Phase Gate

A Phase Gate is the highest routine implementation gate.

It requires:

```text
ALL REQUIRED TASKS = ACCEPTED
        +
ALL PHASE-LEVEL TESTS = PASS
        +
RELEVANT FAILURE CORPUS = PASS
        +
VERTICAL PROOF SLICE = PASS
        +
ARCHITECTURE INTEGRITY = PASS
        +
SECURITY CHECKS = PASS
        +
BUILD STATE = CONSISTENT
        +
CHECKPOINT = VALID
```

No phase advances if any mandatory requirement is:

```text
FAILED
BLOCKED
UNKNOWN
INVALIDATED
```

unless the Blueprint explicitly identifies it as non-blocking and the gate records why.

---

# 38. Vertical Proof Slice Gate

A vertical slice must prove the architecture end-to-end for its intended scope.

Examples from the Build Protocol:

### VS0

```text
mission
→ attempt
→ FakeProvider
→ virtual workspace
→ evidence
→ verification
→ checkpoint
→ persisted state
→ restart
→ recovery
```

### VS1

```text
partial change
→ reconciliation
→ targeted repair
→ verification
→ checkpoint
```

### VS2

```text
worktree
→ real Aider
→ evidence
→ verification
→ merge
```

### VS3

```text
real provider
→ real observation
→ failure capture
```

### VS4

```text
Gemini
→ failure
→ reconciliation
→ handoff
→ Mistral
→ verification
```

### VS5

```text
Engineering Review
→ Build
→ Review
→ QA
```

with capability enforcement.

---

# 39. Release Gate

A release candidate is not simply a Phase 11 pass.

Before a production release, the system must demonstrate:

```text
[ ] all required phases accepted
[ ] critical regression corpus clean
[ ] real-provider integrations pass
[ ] recovery paths tested
[ ] crash recovery tested
[ ] conflict paths tested
[ ] credential leakage checks pass
[ ] verification sandbox tests pass
[ ] capability boundary tests pass
[ ] database/persistence migrations verified
[ ] release artifact reproducible
[ ] build metadata recorded
[ ] diagnostic bundle redaction verified
[ ] accessibility checks pass
[ ] performance baseline recorded
[ ] no unresolved critical issue
[ ] no fabricated benchmark/security claim
```

---

# 40. Gate Failure Protocol

When a gate fails:

```text
GATE = FAILED
```

Then:

```text
1. Preserve evidence.
2. Record exact failure.
3. Do not hide/overwrite the failing result.
4. Classify the failure.
5. Determine whether it is:
   - implementation,
   - test,
   - environment,
   - provider,
   - architecture,
   - scope,
   - dependency,
   - security.
6. Create a failure-corpus case if meaningful.
7. Return to the smallest responsible task.
8. Fix.
9. Re-run the gate.
```

Do not "skip the gate to continue."

---

# 41. Gate Blocked Protocol

Use `BLOCKED` when verification cannot fairly be judged.

Examples:

```text
required provider unavailable
missing test dependency
unresolved architecture decision
unsupported environment
required human review unavailable
insufficient evidence due to external outage
```

A blocked gate remains blocked until the blocking condition is resolved.

Do not convert it to PASS simply because the code looks correct.

---

# 42. Gate Invalidation Protocol

A previously passed gate must be invalidated if:

- accepted code changes,
- relevant configuration changes,
- dependency lock changes,
- verification environment changes materially,
- evidence source changes,
- related architecture change alters the contract,
- a later discovery proves the evidence was stale or incorrect.

On invalidation:

```text
PASSED → INVALIDATED
```

The task/phase must not continue as though the evidence remained valid.

---

# 43. Gate Evidence Report

Every completed gate should leave a compact report:

```text
AI CONDUCTOR VERIFICATION REPORT

Gate ID:
Scope:
Contract:
Repository:
Worktree:
Commit:
Environment:

PRECONDITIONS:
PASS / ...

VERIFICATION STEPS:
1. ...
2. ...

TESTS EXECUTED:
- ...

RESULTS:
- PASS
- PASS

ACCEPTANCE:
- AC-01 PASS
- AC-02 PASS

SCOPE:
PASS

ARCHITECTURE INTEGRITY:
PASS

FAILURE SCENARIOS:
PASS

SECURITY:
PASS / N/A

EVIDENCE:
- EV-...
- EV-...

CHECKPOINT:
...

FINAL GATE STATUS:
PASSED
```

It must include actual command/output references where practical.

---

# 44. Build-State Integration

After a gate passes, update `BUILD_STATE.json` with:

```yaml
accepted:
  task_id:
  task_contract_version:
  gate_id:
  gate_version:
  accepted_at:
  checkpoint:
  commit:
  evidence_references:
  tests_passed:
```

After failure:

```yaml
failed:
  task_id:
  gate_id:
  reason:
  evidence_references:
  next_action:
```

After block:

```yaml
blocked:
  task_id:
  gate_id:
  reason:
  unblock_condition:
```

Do not overwrite historical results.

---

# 45. Verification Gate Security Rules

A verification system must never become a hidden bypass around the security model.

Therefore:

```text
verification command
    ↓
capability check
    ↓
sandbox
    ↓
execution
    ↓
mutation detection
    ↓
evidence capture
```

The verification engine may prove that an application works; it may not gain unrestricted authority merely because it is called "verification."

---

# 46. Anti-Hallucination Gate

For every gate result, verify that:

```text
[ ] commands actually executed
[ ] outputs actually observed
[ ] test result belongs to current repository/task
[ ] evidence references are valid
[ ] no unsupported provider claim was substituted
[ ] no benchmark invented
[ ] no security claim exceeds evidence
[ ] no "expected" result is presented as "actual"
```

A gate report that contains fabricated evidence is invalid even if the code happens to be correct.

---

# 47. No "Green by Silence"

These are not PASS:

```text
no errors shown
no failing logs found
agent did not report a problem
model stopped responding
test command returned no visible output
```

PASS requires positive evidence appropriate to the criterion.

---

# 48. No "Green by Compilation"

Compilation/build success is one signal.

It does not prove:

- state semantics,
- recovery,
- merge safety,
- user-change preservation,
- provider behavior,
- acceptance criteria,
- security,
- UX,
- correctness of runtime behavior.

A build may be green while the gate remains FAILED.

---

# 49. Verification of Documentation-Only Tasks

Even documentation tasks need evidence.

Typical:

```text
diff check
required section presence
link validation where applicable
schema/example consistency
version/reference consistency
```

A documentation task may use a lighter gate than a state-machine task, but it still needs an explicit proof.

---

# 50. Verification of Frontend Tasks

For UI tasks, use the smallest sufficient combination:

```text
diff/scope
→ typecheck
→ build
→ component/unit tests where available
→ runtime/browser smoke
→ accessibility smoke
→ requirement check
→ human review when visual/UX criteria require it
```

Do not accept a screenshot alone as proof of the underlying behavior.

The frontend must remain a renderer/controller over backend-authoritative state.

---

# 51. Verification of Backend Tasks

Typical:

```text
compile
→ typecheck/lint
→ deterministic tests
→ integration tests
→ persistence/failure tests
→ acceptance
```

For reliability-core tasks, failure injection is mandatory.

---

# 52. Verification of Database/Data Tasks

Required evidence may include:

```text
migration applies
schema correct
existing data preserved where required
rollback/restore behavior
constraint validation
test fixture
```

Production data must not be modified merely to test a migration.

Use isolated test databases/workspaces.

---

# 53. Verification of Security-Sensitive Tasks

Required:

```text
negative tests
permission-denied behavior
secret-leak checks
capability-boundary checks
sandbox checks
audit/log checks
```

A security-positive test alone is insufficient.

Always test that forbidden behavior is rejected.

---

# 54. Verification of Resource Intelligence

For routing/resource changes, test both positive and negative eligibility:

```text
eligible resource → can be selected
incompatible resource → cannot be selected
unhealthy resource → excluded/penalized
quota-exhausted resource → excluded
cooldown resource → excluded
stale capability → not trusted beyond policy
```

Also verify the decision is explainable and records its policy version.

---

# 55. Verification of Context Caching

For cache-related tasks:

```text
same snapshot + same relevant inputs
→ cache may be reused

relevant file changed
→ cache invalidated

configuration changed materially
→ cache invalidated

provider/model capability changed where relevant
→ policy/evidence recalculated

stale cache used as current truth
→ test must fail
```

Caching must never silently weaken correctness.

---

# 56. Verification of Recovery Budget

Test:

```text
retry 1
retry 2
retry 3
handoff
human escalation
```

and prove the system does not loop indefinitely.

Also test that a materially different strategy is required for autonomous retry where the Blueprint specifies the monotonic-retry rule.

---

# 57. Verification of Safe Mode

Safe Mode must be tested as an actual reduction in autonomous authority.

When active:

```text
automatic handoff → disabled or approval-required
automatic repair → disabled or approval-required
automatic merge → disabled or approval-required
destructive operation → blocked/approval-required
```

Safe Mode must not merely change a UI label.

---

# 58. Verification of Human Escalation

When automation reaches an explicit human boundary:

```text
awaiting human decision
```

must persist across restart.

The decision context must include:

```text
problem
evidence
risk
available options
recommended option
what will happen for each option
```

After the human decision, the resulting command must re-enter normal verification.

---

# 59. Verification of "No Repeat" Recovery

When a recovery package says:

```text
DO NOT REPEAT:
- provider X
- edit Y
- context Z
```

the destination attempt must be tested for compliance.

This is part of recovery correctness, not merely a prompt feature.

---

# 60. Gate Versioning

Every gate is versioned:

```yaml
gate_version: 1
```

If the acceptance logic materially changes:

```text
gate_version increments
```

Historical reports continue to point to the exact version used.

Do not rewrite past evidence under a new gate definition.

---

# 61. Gate Reproducibility

For deterministic gates, record:

```text
test inputs
scenario ID
seed
virtual clock state
repository/fixture identity
expected output
actual output
```

A deterministic gate should be replayable.

For real integrations, record sufficient environment/provider metadata to reproduce the observation as closely as practical.

---

# 62. Gate-to-Task Traceability

Every gate must trace back:

```text
gate
→ acceptance criterion
→ task contract
→ phase
→ Blueprint requirement
→ invariant where applicable
```

Example:

```text
VG-P1-W03-T04-01
        ↓
AC-02
        ↓
P1-W03-T04
        ↓
P1
        ↓
Blueprint §...
        ↓
Invariant 14
```

This is how the system avoids "we tested something, but don't know why."

---

# 63. Gate-to-Evidence Traceability

Every accepted gate should allow the reviewer to trace:

```text
criterion
→ command/test
→ execution
→ output
→ evidence ID
→ commit/worktree
```

A green checkbox without traceable evidence is not sufficient.

---

# 64. Gate-to-Failure-Corpus Traceability

If a task was created because of a historical bug, the gate must reference the relevant failure case.

Example:

```text
failure_case:
F-004-checkpoint-crash
```

Then:

```text
historical failure
→ regression test
→ task fix
→ gate
```

This closes the loop between learning and verification.

---

# 65. Gate Completion Example

```yaml
gate_version: 1

id: VG-P1-W03-T04-01

scope:
  type: task
  id: P1-W03-T04

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"
  task_contract: "P1-W03-T04@1"

objective: >
  Prove that the implemented transition validator accepts all authorized
  transitions and rejects all forbidden transitions without allowing an
  unauthorized Succeeded path.

preconditions:
  - "Task contract status is IN_PROGRESS."
  - "Repository baseline is known."

required_evidence:
  - "deterministic test output"
  - "changed-file diff"
  - "commit/checkpoint"

verification_steps:
  - "Run transition matrix tests."
  - "Run invalid-transition tests."
  - "Run protected-Succeeded tests."
  - "Inspect diff scope."

acceptance_criteria:
  - id: AC-01
    criterion: "All legal transitions pass."
    verification_method: "deterministic_test"
  - id: AC-02
    criterion: "Forbidden transitions fail."
    verification_method: "deterministic_test"
  - id: AC-03
    criterion: "No non-verification path can create Succeeded."
    verification_method: "deterministic/property_test"

scope_checks:
  - "Only allowed files changed."

integrity_checks:
  - "Invariant 14 preserved."

failure_cases:
  - "invalid transition"
  - "direct Succeeded attempt"

evidence_policy:
  freshness: "current commit"
  reproducibility: "deterministic"

environment:
  network_required: false

pass_conditions:
  - "All acceptance criteria PASS."
  - "Scope PASS."
  - "Integrity PASS."

fail_conditions:
  - "Any mandatory test fails."
  - "Unauthorized transition is accepted."
  - "Unauthorized file changed."

block_conditions:
  - "Required test runner unavailable."

invalid_conditions:
  - "Repository changed after test without re-running gate."

risk_level: high

status: PASSED
```

---

# 66. Final Verification Rules

The following rules are absolute:

```text
1. No executed evidence → no PASS.
2. No current evidence → no PASS.
3. No verification method → no acceptance.
4. Model claim ≠ evidence.
5. Compilation ≠ correctness.
6. Reconciliation ≠ success.
7. HTTP 200 ≠ successful task.
8. A test file existing ≠ test passed.
9. Historical evidence ≠ current evidence.
10. Unknown ≠ pass.
11. Blocked ≠ pass.
12. Out-of-scope change ≠ accepted change.
13. Failed gate cannot be silently skipped.
14. A phase cannot advance through a failed mandatory gate.
15. Verification Engine remains the authority for AttemptStatus::Succeeded.
```

---

# 67. Role in the Full Control Pack

The completed control system now has six distinct layers:

```text
01 MASTER BLUEPRINT
    What the product MUST be.

02 BUILD PROTOCOL
    How OX Alpha MUST work.

03 PHASE MANIFEST
    What phase is authorized and what it must prove.

04 TASK CONTRACTS
    What exact bounded change is authorized now.

05 VERIFICATION GATES
    What evidence proves that bounded change is correct.

06 BUILD STATE
    What has actually happened and where to continue.
```

Supporting operational records are produced underneath these authorities:

```text
failure_cases/
verification_reports/
checkpoints/
STOP_REPORT.md
architecture_changes/
dependency_requests/
```

These are evidence/state artifacts, not competing specifications.

---

# 68. Closing Principle

The Verification Gate exists to make one statement trustworthy:

> **This part is actually done.**

Not:

> "The agent says it is done."

Not:

> "The code looks finished."

Not:

> "The build passed."

Not:

> "The model had high confidence."

The gate passes only when:

```text
CORRECT SCOPE
+
REAL EXECUTION
+
RELEVANT EVIDENCE
+
FRESH EVIDENCE
+
SUFFICIENT PROOF
+
ARCHITECTURE INTEGRITY
+
SECURITY INTEGRITY
+
ACCEPTANCE CRITERIA
+
CHECKPOINT
```

are all satisfied.

That is what makes the build process trustworthy enough for an autonomous AI implementation agent.

---

**END — AI CONDUCTOR VERIFICATION GATES v1.1**


---

# 66. Execution Harness Conformance Gates — v1.1 Amendment

The following gates are mandatory for any supported executor, including Aider or Jcode.

## VG-P5-EXEC-01 — Adapter Contract

PASS only if the adapter exposes the required lifecycle and preserves Conductor authority.

## VG-P5-EXEC-02 — Workspace Confinement

PASS only if all file mutations remain inside the authorized attempt workspace and live-workspace protection is independently verified.

## VG-P5-EXEC-03 — Lifecycle Observation

PASS only if the executor's success, failure, cancellation, timeout, crash, and Unknown outcomes can be distinguished.

## VG-P5-EXEC-04 — Unknown Safety

PASS only if ambiguous termination becomes `Unknown` and is reconciled before continuation.

## VG-P5-EXEC-05 — No Direct Succeeded

PASS only if no adapter/harness/provider/UI/model path can directly create `Succeeded`.

## VG-P5-EXEC-06 — No Direct Merge

PASS only if no adapter/harness internal merge operation can bypass the Conductor's two-phase merge and verification boundaries.

## VG-P5-EXEC-07 — Capability Enforcement

PASS only if forbidden capabilities are rejected at the execution/tool boundary, not merely by prompt.

## VG-P5-EXEC-08 — Restart/Reconciliation

PASS only if executor interruption followed by restart safely reconstructs and reconciles workspace state.

## VG-P5-EXEC-09 — Evidence Provenance

PASS only if executor results are attributable to executor/version/attempt/workspace and are not treated as stronger than their provenance permits.

## VG-P5-EXEC-10 — Executor Reversibility

PASS only if an equivalent bounded task can be passed to another conforming executor without altering the reliability kernel.

## VG-P5-EXEC-11 — Jcode Candidate Gate

If Jcode is evaluated, it may be marked `CANDIDATE-VERIFIED`, `CANDIDATE-PARTIAL`, or `CANDIDATE-REJECTED`. Only `CANDIDATE-VERIFIED` with the required evidence permits promotion to supported executor.

## VG-P5-EXEC-12 — Aider Candidate Gate

Apply the same classification to Aider. No executor receives preferential verification criteria.

---

## 67. Executor Comparison Rule

Aider and Jcode must be compared on the same evidence dimensions:

```text
correctness
workspace safety
failure observability
recovery
verification compatibility
structured control
headless reliability
latency
resource cost
context efficiency
maintenance burden
```

Do not select an executor based solely on:

- benchmark marketing;
- feature count;
- popularity;
- model brand;
- subjective preference.

Evidence determines the decision.
