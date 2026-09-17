# AI CONDUCTOR
# Execution Adapter Contract — v1.0
## Common Boundary for Aider, Jcode, and Future Execution Harnesses

**Authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`
**Procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.1_HARNESS_NEUTRAL.md`
**Phase:** `P5`
**Status:** Normative execution-boundary contract

---

# 0. Purpose

This contract prevents AI Conductor from becoming coupled to Aider, Jcode, or any future coding harness.

It defines the **minimum semantic interface** an execution harness must provide to become a Conductor-managed executor.

The contract is intentionally about observable behavior and authority boundaries, not a demand for one internal implementation.

---

# 1. Ownership Boundary

```text
AI CONDUCTOR
  owns:
  mission
  step
  attempt
  policy
  capability authority
  workspace authority
  persistence
  evidence authority
  reconciliation
  verification
  recovery
  merge

EXECUTION HARNESS
  performs:
  delegated execution
  repository inspection
  authorized tool use
  authorized mutations
  command execution
  result reporting
```

The harness may not own Conductor truth.

---

# 2. Required Identity

Every executor must expose:

```text
executor_id
name
version
commit/build identity where available
adapter_version
runtime/environment identity
```

Version-sensitive behavior must be attributable to this identity.

---

# 3. Required Capabilities

The adapter must expose a capability snapshot describing:

```text
interactive execution
non-interactive execution
workspace/workdir control
file mutation
shell/process execution
Git interaction
streaming
cancellation
timeout
session persistence
provider selection
MCP/tooling
browser capability
parallel execution
```

Unknown capabilities remain unknown.

The adapter must not fabricate capability values merely to satisfy a task.

---

# 4. Execution Request

Conceptual request:

```yaml
execution_request:
  mission_id: ...
  step_id: ...
  attempt_id: ...
  executor_id: ...
  workspace_id: ...
  baseline_id: ...
  allowed_capabilities: []
  requested_operation: ...
  provider_resource: ...
  policy_version: ...
  timeout_policy: ...
  correlation_ids: ...
```

The exact serialization is implementation-defined.

---

# 5. Execution Lifecycle

Required semantic lifecycle:

```text
Pending
  ↓
Starting
  ↓
Running
  ├── Completed
  ├── Failed
  ├── Cancelled
  ├── TimedOut
  ├── Crashed
  └── Unknown
```

`Unknown` is mandatory.

If the adapter cannot determine whether an externally visible action occurred, it must report `Unknown` rather than invent `Failed` or `Completed`.

---

# 6. Start

`start()` must:

- execute only after Conductor policy/capability checks;
- use the assigned workspace;
- preserve attempt/correlation identity;
- capture the executor identity;
- avoid unauthorized live-workspace mutation.

---

# 7. Observe

The adapter must provide enough information to reconstruct:

```text
started
process identity if available
stdout/stderr or structured events
tool activity where observable
exit information
changed-workspace observations
failure classification
final executor state
```

No claim from the harness is automatically authoritative.

---

# 8. Cancel

Cancellation must report:

```text
requested
accepted/rejected
observed termination state
remaining ambiguity
cleanup status
```

Cancellation does not automatically mean `Failed`.

If side effects may have occurred, reconciliation remains required.

---

# 9. Evidence Collection

The adapter must make available the evidence needed by Conductor to perform independent verification.

Evidence should include references to:

```text
attempt
workspace
executor
version
process
commands
outputs
artifacts
exit result
```

Secrets must be redacted.

---

# 10. Finalization

Finalization reports the executor's observed outcome.

It does NOT authorize acceptance.

The semantic result may be:

```text
CompletedClaim
FailedClaim
CancelledClaim
UnknownClaim
```

The Verification Engine determines whether the accepted task outcome is actually valid.

---

# 11. Hard Authority Rules

An executor adapter MUST NOT:

- mutate Conductor `AttemptStatus::Succeeded` directly;
- bypass Verification Engine;
- authorize merge;
- bypass two-phase merge;
- silently resolve Conflict;
- downgrade Unknown to Failed for convenience;
- expose credentials;
- write outside authorized workspace;
- disable required verification;
- rewrite Build State as its own authority.

---

# 12. Workspace Rule

Every mutation-capable execution must run through a Conductor-authorized workspace boundary.

Preferred:

```text
live user workspace
        │
        X  direct executor mutation forbidden
        │
        ▼
Conductor-created isolated workspace
        │
        ▼
executor
```

The exact mechanism is governed by the Workspace Safety phase.

---

# 13. Failure Contract

Adapter failures must be mapped into the Conductor's structured taxonomy.

Examples:

```text
provider failure
network failure
process failure
executor failure
workspace failure
filesystem failure
Git failure
permission failure
capability mismatch
unknown
```

Do not map everything to generic `ERROR`.

---

# 14. Conformance Suite

Every supported adapter must pass:

```text
EC-01 identity
EC-02 capabilities
EC-03 workspace confinement
EC-04 successful execution
EC-05 partial execution
EC-06 non-zero exit
EC-07 timeout
EC-08 cancellation
EC-09 crash
EC-10 Unknown outcome
EC-11 restart/reconciliation
EC-12 evidence collection
EC-13 credential redaction
EC-14 capability denial
EC-15 no direct Succeeded
EC-16 no direct merge
EC-17 executor replacement
```

---

# 15. Aider Adapter

Aider is a supported-candidate executor only after it passes this contract.

Aider-specific behavior—including Git automation, model metadata handling, context limits, commands, and interactive warnings—belongs behind the adapter and in the External Components Knowledge Base.

---

# 16. Jcode Adapter

Jcode is a supported-candidate executor only after it passes this contract.

Jcode-specific capabilities—including its session, provider, MCP, skill, memory, browser, server/client, swarm, and worktree behavior—must be verified against the pinned installation/version.

Do not promote a documented or claimed feature to a Conductor capability until the relevant conformance/evidence test passes.

---

# 17. Selection

Executor selection must consider:

```text
eligibility
capability fit
workspace compatibility
provider/model compatibility
observed reliability
latency
resource cost
recovery cost
policy
```

The selected executor and reason must be attributable.

---

# 18. Replacement Principle

A conforming executor can be replaced without rewriting:

```text
State Engine
Event Log
Persistence
Verification Engine
Reconciliation Engine
Two-Phase Merge
Failure Corpus
Build State
```

This is a mandatory architectural property.

---

# 19. Promotion States

Every new executor progresses through:

```text
DISCOVERED
  ↓
ADAPTER-BUILT
  ↓
CONFORMANCE-PASSING
  ↓
REAL-INTEGRATION-PASSING
  ↓
FAILURE-TESTED
  ↓
SUPPORTED
```

It may instead become:

```text
PARTIAL
REJECTED
QUARANTINED
```

No executor is promoted merely because it is popular or feature-rich.

---

# 20. Final Principle

The Conductor should be **thick where cross-harness reliability is weak** and **thin where the harness already provides a strong execution capability**.

Use external harness strength.
Do not inherit external authority.

**Executor does work. Conductor decides what the work means.**
