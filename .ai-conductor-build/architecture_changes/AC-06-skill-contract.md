# Architecture Contract AC-06 — Backend Skill Contract
# Task: P0-W06-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-06
task_id: P0-W06-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §22 (gstack Skill Contracts)"
  - "Build Protocol §28 (gstack Skill Implementation Protocol)"
  - "Knowledge Base §35–§39 (pinning, prompt-vs-security, artifacts, state, layer separation)"
  - "Phase Manifest P0-C06"
```

## 1. Skill definition contract (required)

Every backend skill MUST be defined with exactly these seven fields
(Build Protocol §28):

```text
skill_id               stable identifier (e.g. "backend.build")
INPUT                  what the skill receives (typed, from handoff data)
OUTPUT                 what the skill produces (SkillResult, §4)
CAPABILITIES           capability identities requested (must exist in AC-07 table)
FORBIDDEN_CAPABILITIES explicitly denied capability identities
ALLOWED_TOOLS          tools the skill may invoke — each gated by the capability boundary
VERIFICATION           how the Conductor verifies this skill's output before acceptance
HANDOFF_DATA           structured package passed onward on completion/failure
```

A skill definition missing any field is invalid and cannot be registered.
The elaborate visual workflow stepper is explicitly deferred (Blueprint §22:
define the input/output/permission contract first).

## 2. Initial backend skill set

Build first, exactly per Build Protocol §28:

| skill_id | Role |
|---|---|
| backend.office-hours | requirements clarification |
| backend.engineering-review | design/process review |
| backend.build | implementation |
| backend.review | code review |
| backend.qa | quality assurance |
| backend.debug | diagnosis |

No other skills are defined until the reliability substrate is proven.

## 3. Enforcement rule (required)

**A skill must never rely on a prompt alone to enforce its permissions.**
Tool execution must pass through the actual capability boundary (Blueprint Invariant 10;
KB §36). gstack skill prose is process guidance, not security. The Conductor's
capability table (AC-07) is the sole enforcement point.

## 4. SkillResult structured artifact

Skill outputs that matter are preserved as structured evidence (KB §37):

```text
SkillResult
  artifact              produced plan/report/findings reference
  decisions             choices made with rationale
  risks                 identified risks
  findings              review/QA/security findings
  acceptance_criteria   proposed or consumed criteria
  unresolved_questions  open items surfaced to person/engine
```

This makes skill state transferable between models and consumable as evidence
(provenance rules of AC-02 apply — a skill result is ModelClaim-class evidence
until independently verified).

## 5. Layer separation and state authority

```text
gstack:            "What engineering role/process should happen now?"
AI Conductor:      "Can, how, and under what safety/evidence/resource conditions?"
```

- gstack context-save/restore (`/context-save`, `/context-restore`) is useful
  supplementary context, NEVER authoritative mission state (KB §38). Authoritative
  continuation remains: Build State + Task Contract + Verification evidence + Repository.
- Integration pins a specific gstack version/commit and records `gstack_skill_manifest`;
  compatibility checks re-run after upgrades (KB §35). Until availability is confirmed
  (BUILD_STATE unknown U-0002), skills are defined against this contract so any
  conforming process source can fill the role.
- A failed skill is NOT automatically a provider failure; classify the layer
  before retrying (KB failure-layer rule; AC-05 classes apply).

## 6. Non-goals

Visual workflow stepper UI (deferred per Blueprint §22), Skill Runtime mechanics
(campaign item 11 / Phase 5+), capability table contents themselves (AC-07),
gstack installation/discovery (blocked on U-0002).

## 7. Invariants preserved

7 (model output advisory), 8 (every transition observable as events), 10 (capability
boundaries structurally enforced). None weakened.
