# Architecture Contract AC-07 — Capability Table & Tool Boundary
# Task: P0-W07-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-07
task_id: P0-W07-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §16 (Capability & Permission Model)"
  - "Blueprint §7.1 (Verification Execution Sandbox)"
  - "Blueprint module map (tool_boundary.rs)"
  - "Build Protocol §28 (skill rule)"
  - "AC-06 §3"
  - "Phase Manifest P0-C07"
```

## 1. Capability identities (required proof #1)

The Blueprint defines exactly four capability identities (§16). They form the
**initial closed set**:

```text
READ_PROJECT          read files/metadata within the mission workspace
RUN_SAFE_INSPECTION   run non-mutating inspection commands (linters, read-only queries)
WRITE_REVIEW          produce review/report artifacts (not source changes)
WRITE_CODE            create/modify/delete project source files
```

Extension rule (P0-C07 anti-invention constraint): a new capability identity may be
added ONLY via an architecture-change record citing the Blueprint section that defines
it. A manifest, skill, or model request can never invent one at runtime or build time.

## 2. Skill → capability assignment

Assignments of the AC-06 initial skills within the closed set (design discretion;
least privilege):

| skill_id | CAPABILITIES | Explicitly forbidden |
|---|---|---|
| backend.office-hours | READ_PROJECT | RUN_SAFE_INSPECTION, WRITE_REVIEW, WRITE_CODE |
| backend.engineering-review | READ_PROJECT, RUN_SAFE_INSPECTION, WRITE_REVIEW | WRITE_CODE |
| backend.build | READ_PROJECT, RUN_SAFE_INSPECTION, WRITE_CODE | WRITE_REVIEW |
| backend.review | READ_PROJECT, RUN_SAFE_INSPECTION, WRITE_REVIEW | WRITE_CODE |
| backend.qa | READ_PROJECT, RUN_SAFE_INSPECTION, WRITE_REVIEW | WRITE_CODE |
| backend.debug | READ_PROJECT, RUN_SAFE_INSPECTION | WRITE_CODE, WRITE_REVIEW |

`engineering-review` matches the Blueprint §16 worked example verbatim.
A Review-phase skill structurally cannot edit files — the Phase 7 checklist proof
("a Review-phase skill cannot write files") tests this exact row pair.
Debug diagnoses only; repairs route through new build attempts (AC-05 targeted-repair path).

## 3. Enforcement boundary (required)

Hard rule (Blueprint §16): enforcement does NOT live at the skill level as a soft
check somewhere upstream — it lives at the **tool-execution boundary itself**, and
every single tool call passes through it with **no alternate route to execution**:

```text
ExecutionEngine.execute(tool_call)
        ↓
  capability_check(active_skill, tool_call.required_capability)
        ↓
   authorized?  → proceed
   denied?      → reject, log, surface — the call never reaches Aider or the provider
```

"The model says it won't write files" is a promise the model can break; this check
makes bypass structurally impossible regardless of model output or future code paths
(Invariant 10).

## 4. Tool-execution-boundary location (required)

- Primary gate: the `capability_check()` function in the tool-boundary module —
  per the Blueprint module map, `tool_boundary.rs` is "**the only path to Aider**".
- Second gated execution path: verification commands run inside the §7.1 sandbox
  (command allowlist, working-directory confined to the attempt worktree, timeout,
  resource limits, network off by default with auditable opt-in). The sandbox is
  downstream of the same capability discipline — an acceptance criterion cannot
  become an execution escape hatch.
- Denials are events (Invariant 8) and must never log credential material (Invariant 12).

## 5. Disambiguation

"Capability" appears in the Blueprint with two distinct meanings. This contract
covers ONLY skill-level permission capabilities (§16). The Provider/Model Capability
Registry (§10.3 — provider competence/freshness data used for fallback selection)
is a different mechanism, owned by the resource-intelligence phase, and MUST NOT be
conflated with this permission table.

## 6. Non-goals

Provider/model capability registry internals (§10.3), verification pipeline logic (§7),
Skill Runtime mechanics, UI permission presentation.

## 7. Invariants preserved

8 (denials observable as events), 10 (structural enforcement — core subject),
12 (redaction at logging boundaries). None weakened.
