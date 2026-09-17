# AI CONDUCTOR
# External Components Integration & Operational Knowledge Base — v1.1
## Verified Operating Knowledge for Aider, OmniRoute, gstack, Jcode, and Their Boundaries

**Purpose:** Give OX Alpha the verified, version-aware knowledge it needs to integrate the external components correctly without making assumptions that create avoidable implementation pain.

**Architecture authority:** `AI_CONDUCTOR_MASTER_BLUEPRINT_v2.5.1_REVIEWED.md`  
**Build procedure:** `AI_CONDUCTOR_BUILD_PROTOCOL_v1.1_HARNESS_NEUTRAL.md`  
**Phase authorization:** `AI_CONDUCTOR_PHASE_MANIFEST_v1.1_HARNESS_NEUTRAL.md`  
**Task contracts:** `AI_CONDUCTOR_TASK_CONTRACTS_v1.1_HARNESS_NEUTRAL.md`  
**Verification gates:** `AI_CONDUCTOR_VERIFICATION_GATES_v1.1_HARNESS_NEUTRAL.md`  
**Build state:** `AI_CONDUCTOR_BUILD_STATE_SPECIFICATION_v1.0.md`  
**Failure memory:** `AI_CONDUCTOR_FAILURE_CORPUS_SPECIFICATION_v1.0.md`

**Research date:** 2026-08-28

---

# 0. Why This File Exists

AI Conductor deliberately leverages strong external systems instead of rebuilding capabilities that can be safely delegated. The current primary external set is Aider, Jcode, OmniRoute, and gstack:

```text
gstack
    ↓
ENGINEERING PROCESS / SKILLS

Execution Harnesses (Jcode / Aider / future adapters)
    ↓
CODE EXECUTION / REPOSITORY EDITING

OmniRoute
    ↓
PROVIDER CONNECTIVITY / ROUTING / COMBOS

AI Conductor
    ↓
CONTROL / STATE / SAFETY / RECOVERY / VERIFICATION /
RESOURCE EFFICIENCY / CONTINUITY
```

The Blueprint explicitly establishes this separation. AI Conductor is not intended to become a replacement implementation of Aider or OmniRoute. Its job is to coordinate them safely and intelligently. The Blueprint's core product promise is reliable orchestration for unreliable AI resources.

This file exists because an AI coding agent should **not have to guess how those external systems behave**.

It records:

- what the component actually does;
- what it does not do;
- relevant commands/interfaces;
- known limitations;
- integration hazards;
- observed failure patterns;
- version sensitivity;
- what must be probed rather than assumed;
- what AI Conductor should own instead of delegating;
- how to use the component efficiently;
- how to recognize and recover from common failures.

---

# 1. Evidence Policy

This document intentionally separates knowledge by evidence strength.

## A — Direct authoritative documentation/source

Examples:

- official Aider documentation;
- official Aider source;
- official OmniRoute repository/docs for the pinned reference line;
- official gstack repository/docs/source.

Use this for implementation assumptions.

## B — Reproducible issue/report evidence

Examples:

- official GitHub issue with exact reproduction;
- official discussion containing a maintainer response;
- reproducible project behavior.

Use this to populate the known-failure/edge-case registry.

## C — Project-observed behavior

Examples:

- behavior observed in the user's current AI Conductor integration;
- logs/errors from the actual environment;
- command behavior already reproduced during project development.

This is valuable but is **version/environment specific**.

It must be labeled as observed rather than generalized as universal behavior.

## D — Community experience

Examples:

- Reddit discussions,
- community posts,
- third-party examples.

Useful for detecting recurring pain points and practical workarounds, but never sufficient by itself for a critical architectural assumption.

## E — Model inference

Inference is not implementation truth.

If a fact cannot be verified:

```text
UNKNOWN
    ↓
PROBE / DOCUMENT / TEST
```

not:

```text
UNKNOWN
    ↓
ASSUME
```

---

# 2. Version Discipline — Mandatory

These external projects move quickly.

OX Alpha must therefore record exact versions/commits used in the implementation environment.

Required runtime record:

```text
Aider version
OmniRoute version + commit/branch
gstack commit
Node version
Bun version
Rust version where relevant
OS
```

Do not implement against:

```text
"latest"
```

as though it were a stable API contract.

### Current research reference points

- **Aider:** GitHub currently lists **v0.86.0** as the latest tagged release reviewed here (released 2026-08-09). 
- **Jcode:** GitHub Releases currently lists **v0.58.0** as the latest tagged stable release visible in the release page. The Jcode `master` README also contains benchmarks for a separate development build (`v0.9.1888-dev`, commit `be386f2`); do not confuse that development benchmark with the latest stable release. 
- **OmniRoute:** the v3.8.50 release line is active in current documentation, but the repository also has a `release/v3.8.51` branch already cut. v3.8.50 was published as a package release on 2026-08-26 while some release artifacts/workflows were still catching up. Treat the exact installed build/commit as authoritative. 
- **gstack:** the current `main` repository is the relevant moving reference; the project documentation describes installation via Claude Code/skills and an expanding skill set. Do not pin feature claims to an old blog post or fork. 

### Integration rule

For a released build of AI Conductor:

```text
pin exact external component versions/commits
+
record them in Build State
+
run compatibility tests
```

---

# 3. Aider — What It Is

Aider is an open-source terminal-native AI pair-programming/coding tool that works directly with a local repository and model APIs.

Its strengths relevant to AI Conductor are:

```text
repository-aware coding
file editing
diff generation
Git integration
repo map
chat modes
model switching
shell/test execution
code review/asking
```

Aider operates on real repository files and understands the repository through its repo map. Official documentation recommends giving it only the files relevant to the current edit while relying on the repository map for surrounding code awareness. citeturn720478search1turn720478search8

---

# 4. Aider — What AI Conductor Should Delegate to It

AI Conductor should normally delegate:

```text
reading source
understanding source
editing source
generating diffs
applying code changes
running normal project commands when explicitly authorized
```

AI Conductor should NOT delegate ownership of:

```text
mission state
phase state
acceptance
verification authority
recovery truth
provider resource selection
user-workspace safety
architecture authority
checkpoint semantics
```

Aider is an execution engine, not the Conductor's source of truth.

---

# 5. Aider — Git Behavior Is Important

Aider is tightly integrated with Git.

Official documentation states that by default it:

- expects/wants a Git repository;
- can create one if none exists;
- commits its edits;
- can commit pre-existing dirty work before making its own changes;
- supports `/diff`, `/undo`, `/commit`, and `/git`;
- can disable these behaviors with flags including `--no-auto-commits`, `--no-dirty-commits`, or `--no-git`. citeturn720478search2turn720478search5

### AI Conductor rule

AI Conductor should **not rely on Aider's default Git ownership**.

The Blueprint explicitly places Aider in the execution layer and gives Conductor its own worktree, checkpoint, merge, reconciliation, and verification responsibilities.

Therefore the preferred architecture is:

```text
Conductor creates/controls isolated worktree
        ↓
Aider operates there
        ↓
Conductor captures actual diff/evidence
        ↓
Verification
        ↓
Conductor-controlled acceptance/merge
```

Where Aider auto-commit behavior would interfere with Conductor's transaction semantics, use the supported Aider configuration that disables auto-commits.

The current project has already been designed around this distinction.

---

# 6. Aider — Dirty Files Are a Real Integration Hazard

Aider deliberately has behavior around dirty/uncommitted files: it can commit pre-existing dirty work before editing so the AI changes remain separately reversible. citeturn720478search2

This is useful for standalone Aider.

It is dangerous if AI Conductor accidentally lets Aider see the user's live dirty workspace.

### Rule

Aider should operate in:

```text
AI Conductor isolated worktree
```

not:

```text
user's live workspace
```

This is why the Blueprint's worktree isolation is not optional decoration.

---

# 7. Aider — File Context

Official guidance strongly favors adding only the files needed for the current task, rather than dumping many files into the chat. Aider uses the repository map to retain structural awareness elsewhere in the repo. citeturn720478search8

Useful commands:

```text
/add       add editable files
/read      add read-only reference files
/drop      remove files from active context
/clear     clear chat history
/reset     drop files and clear history
/ls        list known files
/map       display repository map
/map-refresh refresh repository map
/tokens    inspect token usage
```

These are documented by Aider's current in-chat command reference. citeturn720478search0

### AI Conductor integration rule

The Conductor's Context Engine should select the minimum sufficient context and use Aider's native context mechanisms rather than dumping the repository into every request.

---

# 8. Aider — Token/Context Limits Are Real

Aider's documentation identifies context overflow as one of the common failure modes.

Important points:

- input can exceed the model context window;
- input + output can exceed the available context;
- output limits can also stop large edits;
- removing unnecessary files/context is a recommended mitigation;
- `/tokens`, `/drop`, and `/clear` help manage the session. citeturn720478search13

### AI Conductor rule

Never assume:

```text
model supports long context
```

from a model name alone.

The Conductor must have:

```text
verified context capability
+
current task context size
+
Aider overhead estimate
+
output headroom
```

before an expensive attempt.

---

# 9. Aider — Unknown Model Metadata Is a Real, Documented Problem

Aider explicitly warns when a model is unfamiliar and it does not know the model's context window and token costs.

The documented behavior is essentially:

```text
Unknown context window size and costs
→ use sane defaults
```

Aider provides:

- `--model-settings-file`
- `--model-metadata-file`
- `--alias`
- model-listing and metadata controls

for dealing with unfamiliar models. citeturn281259search0turn281259search1

This matches the behavior observed in the user's project.

### Important observed behavior

Aider can also offer to open the documentation URL when a warning occurs. This has been visible in real Aider issue transcripts, and the current Aider source contains an `offer_url(...)` path for model warnings. citeturn599026search0turn599026search1

Therefore the user's observation that Aider may produce a browser/documentation prompt when it does not recognize model metadata is **consistent with actual Aider behavior**.

### AI Conductor rule

Do not make browser interaction part of the normal recovery path.

Instead:

```text
Conductor detects unknown model metadata
       ↓
use its own verified capability registry
       ↓
prepare Aider model metadata/settings if required
       ↓
launch Aider non-interactively
```

The agent should not be left waiting on:

```text
Open documentation url?
```

or other interactive prompts.

---

# 10. Aider — Configure Unknown Models Rather Than Guess

Current Aider options include:

```text
--model-metadata-file
--model-settings-file
--alias
--show-model-warnings
--check-model-accepts-settings
--reasoning-effort
--thinking-tokens
--timeout
```

and chat/runtime model controls such as:

```text
/model
/models
/weak-model
/editor-model
```

The current options reference documents these capabilities. citeturn281259search1

### Conductor strategy

For every externally routed model:

```text
provider/model identity
+
capability metadata
+
context window
+
output limits
+
supported settings
```

should be available before launching Aider.

Where a model cannot be confidently characterized:

```text
status = UNKNOWN
```

The Conductor may run a controlled capability probe instead of guessing.

---

# 11. Aider — Chat Modes

Current Aider modes include:

```text
code
ask
architect
help
```

and individual messages can be sent with `/code`, `/ask`, `/architect`, `/help`. citeturn720478search11

### Why this matters

AI Conductor can map gstack roles to Aider execution modes.

Example:

```text
gstack Engineering Review
→ Aider /ask or read-only task

gstack Build
→ Aider /code

gstack architecture planning
→ Aider /architect where beneficial

diagnostic question
→ Aider /help only for Aider-specific troubleshooting
```

Do not confuse Aider's `architect` mode with AI Conductor's architecture authority.

Aider can propose an implementation.

The Blueprint still owns the architecture.

---

# 12. Aider — Useful Command Set for Conductor

The most relevant commands for automation are:

```text
/add
/read
/drop
/clear
/reset
/diff
/ls
/map
/map-refresh
/model
/models
/weak-model
/editor-model
/ask
/code
/architect
/lint
/test
/run
/git
/commit
/undo
/tokens
/settings
/help
```

Official command documentation confirms these roles. citeturn720478search0

### Important distinction

Some commands are **informational**.

Some commands **modify state**.

Some commands execute arbitrary shell commands.

The Conductor must never treat the names as a safety boundary.

For example:

```text
/run
/git
```

are execution surfaces and must be controlled by Conductor policy.

---

# 13. Aider — `/help` Is Not the Same as AI Conductor Help

Aider's `/help` is an Aider-specific troubleshooting/usage mechanism. Aider itself retrieves its indexed documentation to answer questions. citeturn720478search12

Conductor should use:

```text
Aider /help
```

only when the problem concerns:

```text
Aider configuration
Aider command semantics
Aider supported behavior
Aider-specific failures
```

Conductor should NOT ask Aider's `/help` to define:

```text
project architecture
mission completion
provider routing
recovery truth
Conductor state
security policy
```

---

# 14. Aider — Browser/Web Capability

Aider can work with web pages and provides mechanisms for bringing context between browser/web chats and Aider. Aider also has experimental browser functionality in its history. citeturn720478search14turn281259search5

### Conductor rule

Do not depend on Aider's browser UX for core operation.

If web research is needed:

```text
Conductor-controlled web research
or
gstack /browse
```

should be used according to the applicable skill.

Aider's browser/help URLs are fallback/debugging behavior, not part of Conductor's core workflow.

---

# 15. Aider — Known Practical Weaknesses

Verified/documented weaknesses include:

### Model metadata mismatch

Unknown models can trigger warnings and use incomplete metadata. citeturn281259search0

### Context overflow

Too much input or output can exceed limits. citeturn720478search13

### Weaker model edit reliability

Aider's own troubleshooting documentation warns that weaker/local models are more prone to editing errors and may struggle with its system prompts. citeturn720478search6

### Git side effects

Aider's Git automation is useful standalone but must be deliberately controlled in Conductor. citeturn720478search2

### Interactive warnings/prompts

Unknown model metadata and similar warnings can interrupt automation if not handled. citeturn599026search0turn599026search1

### Long/large edits

Large changes can hit output limits and become unreliable. citeturn720478search13

### Practical mitigation

AI Conductor should solve these through:

```text
model capability registry
context budgeting
small task contracts
isolated worktrees
non-interactive startup
Aider metadata configuration
verification
targeted repair
handoff
```

---

# 15A. Jcode — What It Is

Jcode is an open-source terminal coding-agent **harness**. The current upstream project emphasizes low runtime overhead, multiple sessions, memory, many provider integrations, MCP, remote/server operation, and multi-agent swarm workflows. Its current README documents a small stable release line while the `master` branch is moving rapidly; therefore version/commit pinning is mandatory.

Official references:
- Repository: https://github.com/1jehuang/jcode
- Docs: https://jcode.sh/docs
- Releases: https://github.com/1jehuang/jcode/releases

The current README documents built-in provider flows for Claude, OpenAI/Codex, Gemini, GitHub Copilot, Azure, Alibaba Coding Plan, Fireworks, MiniMax, Meta Model API/Muse, LM Studio, Ollama, and a custom OpenAI-compatible endpoint. It also documents OpenAI-compatible profiles such as OpenRouter, DeepSeek, Z.ai, Kimi/Moonshot and others. 

Jcode therefore overlaps with parts of what we originally expected OmniRoute + Aider to provide, but overlap does not mean replacement.

# 15B. Jcode — Strengths Relevant to AI Conductor

Current upstream material documents:

```text
fast / lightweight harness
multiple concurrent sessions
session resume / fork / transfer
memory and semantic recall
MCP
custom OpenAI-compatible endpoints
provider/account switching
server/client mode
TypeScript SDK / API bridge
background tasks
swarm / task-DAG coordination
TUI with rich side panels and diagnostics
```

Jcode also documents remote execution using a server-side working directory and a socket/client architecture; this is useful for future remote-execution experiments, but it must not replace Conductor workspace identity and evidence semantics.

# 15C. Jcode — Stable vs Development Version Warning

Jcode's public release page currently shows **v0.58.0** as the latest tagged release, while the current `master` README cites performance measurements from **v0.9.1888-dev (be386f2)**. Those are different channels.

Therefore the Conductor knowledge model must distinguish:

```text
stable_release
source_revision
benchmark_revision
installed_revision
```

Never write:

```text
Jcode supports X
```

without identifying which version/revision was inspected.

# 15D. Jcode — CLI / Session Interfaces

Current documentation lists commands/functions including:

```text
/model
/effort
/login
/account
/resume
/fork
/transfer
/compact
/clear
/rewind
/todos
```

Current docs also describe programmatic control through the TypeScript SDK and `jcode api-bridge`, plus local/remote socket operation.

For Conductor integration, prefer a structured, non-interactive surface when available. Do not scrape TUI output as the primary protocol.

# 15E. Jcode — Provider Integration

Jcode currently has substantial built-in provider support and a shared OpenAI-compatible path. This means Jcode can potentially consume:

```text
OpenRouter
OmniRoute-compatible endpoint
local Ollama/LM Studio
other OpenAI-compatible gateways
```

However, a provider being supported by Jcode does not prove that:

```text
its capabilities are identical across providers
its quota semantics are identical
its tool-calling behavior is identical
its error classes are identical
its retry semantics are identical
```

Conductor must normalize provider observations at its own adapter boundary.

# 15F. Jcode + OmniRoute — Important Current Status

A current Jcode issue proposes/records adding OmniRoute as a built-in OpenAI-compatible provider profile. That issue is evidence of an integration effort, **not proof that every current release has the profile**.

Therefore do not assume:

```text
jcode login --provider omniroute
```

exists in the installed version unless the installed version/source confirms it.

For the first integration, use the generic OpenAI-compatible endpoint if necessary and probe the actual configuration surface.

Reference: https://github.com/1jehuang/jcode/issues/704

# 15G. Jcode — Swarm

Jcode's current swarm documentation says the swarm is largely implemented, supports agent messaging/coordination, can spawn workers, preserves daemon state through reload/crash recovery, and uses optional Git worktrees when isolation is helpful. It also says integration is handled by worktree managers rather than the coordinator.

This is useful capability, but it is **not Conductor's reliability authority**.

The Conductor must not delegate:

```text
Succeeded authority
merge authority
Conflict resolution authority
workspace safety authority
verification authority
mission state authority
```

to Jcode's swarm.

If Conductor later uses multiple Jcode workers, they must operate under Conductor-owned:

```text
workspace identity
baseline
mutation attribution
merge lock
verification
acceptance
```

A swarm's internal conflict messaging or automatic integration must never be treated as sufficient proof of safe merge.

# 15H. Jcode — Worktrees

Jcode documents optional worktree usage. That is useful, but optional worktree behavior is not equivalent to the Conductor's mandatory isolation contract.

Conductor must independently verify:

```text
attempt workspace identity
baseline
live workspace protection
changed-file scope
user-change detection
merge precondition
post-merge verification
```

If Jcode is configured to operate in a worktree, Conductor still owns the worktree identity and acceptance boundary.

# 15I. Jcode — Session Resume vs Conductor Recovery

Jcode's `/resume`, `/transfer`, and related session features preserve conversational/session continuity. They are valuable but are **not equivalent** to Conductor recovery.

Conductor recovery must additionally reason about:

```text
filesystem state
Git state
attempt state
persistent intent
verification evidence
mutation attribution
unknown outcome
external side effects
checkpoint
```

A Jcode session can resume while the corresponding external file mutation remains ambiguous. Therefore:

```text
Jcode resume
    ≠
Conductor recovery
```

Jcode resume can be used as an input to reconciliation, not as a substitute for it.

# 15J. Jcode — Memory

Jcode documents semantic memory extraction, retrieval, consolidation, and session search.

This can reduce repeated conversational context and may be useful for human-facing continuity.

It does not replace:

```text
Conductor event log
Build State
Evidence provenance
Checkpoint state
Failure Corpus
Mission/Step/Attempt state
```

The Conductor should treat Jcode memory as advisory/contextual data with source and freshness, never as operational truth.

# 15K. Jcode — Safety System

Jcode's current `SAFETY_SYSTEM.md` is explicitly labeled **Design** in the source reviewed here and describes an auto-allowed vs requires-permission model for unmonitored actions.

That is useful for human approval workflows but is orthogonal to Conductor's correctness model.

Conductor must not assume Jcode's safety system proves:

```text
atomicity
recovery correctness
mutation attribution
false-success prevention
verification integrity
```

Those remain Conductor responsibilities.

# 15L. Jcode — Harness API Maturity

Jcode's current documentation contains a design for a stable, versioned Harness API. The design notes that the existing internal socket protocol is unversioned, has many variants, and is coupled to client rendering assumptions, and proposes a versioned external API boundary.

This is important for Conductor:

```text
Do not integrate against an undocumented internal socket schema
```

Prefer:

```text
documented CLI
stable SDK/API bridge
versioned public protocol when available
```

If only an internal interface is available, isolate it behind the Jcode adapter and mark the integration as version-sensitive.

# 15M. Jcode — Real Current Failure Signals

The public issue tracker is active. Examples reviewed include:

- a resumed session showing an empty transcript after live sessions were orphaned across a reload;
- shortened Bash command visibility making audit/debugging harder;
- installation/runtime issues on particular platforms.

These are not reasons to reject Jcode. They prove why the adapter must treat Jcode as an evolving external dependency and why Conductor needs its own evidence and observability.

Relevant references:
https://github.com/1jehuang/jcode/issues/753
https://github.com/1jehuang/jcode/issues/850
https://github.com/1jehuang/jcode/pulls

# 15N. Jcode — Resource-Efficiency Claims

Jcode publishes its own memory/startup benchmark results. These are useful directional measurements, but they are project-owned benchmarks on specified versions/environments.

Do not copy benchmark numbers into Conductor product claims without:

```text
same version
same hardware
same workload
same measurement method
same concurrency
```

The Conductor should eventually measure its own end-to-end overhead separately.

# 15O. Jcode — Adapter Conformance Requirement

Before Jcode is accepted as a supported executor, it must pass the common Execution Adapter Conformance suite defined in Blueprint v2.5.1:

```text
identity/capability discovery
authorized workspace execution
forbidden-path rejection
success evidence
partial execution
non-zero exit
timeout
cancellation
crash/abrupt termination
Unknown outcome classification
restart/reconciliation
provider failure propagation
credential redaction
capability denial
no direct Succeeded transition
no direct merge authority
```

Do not lower the Conductor contract to fit a harness.

# 15P. Jcode — Practical Position in the Final Architecture

The intended relationship is:

```text
                    AI CONDUCTOR
                         │
                Execution Adapter
                         │
                     Jcode
                         │
              Model / provider resource
```

Jcode may become the preferred execution harness if empirical evaluation shows it is better than Aider for the relevant tasks.

It may also coexist with Aider:

```text
Build/large multi-session work → Jcode
simple deterministic repo edit → Aider
specialized task → another executor
```

The Decision Engine may select the executor based on evidence.

The kernel must not care which executor was chosen.

---

# 15Q. Jcode — What We Must NOT Claim Yet

Until tested against the exact installed build, do not claim that Jcode:

```text
is crash-safe for arbitrary file mutations
is transaction-safe
makes every swarm conflict safe
provides Conductor-grade verification
provides Conductor-grade merge safety
provides deterministic recovery
provides idempotent external side effects
provides a stable public harness API in every release
contains OmniRoute as a built-in provider in every release
```

These are evaluation questions.

That distinction is intentionally strict.

---

# 16. OmniRoute — What It Is

For the current reference line, OmniRoute is an open-source local AI gateway/router that exposes compatible model APIs and provides provider connections, routing, combos, resilience, quota handling, and related tooling.

The reviewed `release/v3.8.50` documentation describes a move toward a modular architecture while maintaining the 3.8.x stabilization line. citeturn216020search2

The current repository exposes:

```text
one local gateway
provider connections
model routing
combos
auto-combos
provider health
cooldowns
model lockouts
API keys
MCP/A2A
CLI
```

The current project should treat the exact pinned OmniRoute version/commit as authoritative because the project is evolving quickly.

---

# 17. OmniRoute — The Important Boundary

OmniRoute handles:

```text
provider connectivity
authentication/connection state
model/provider mapping
combo routing
rate/quota-related routing mechanisms
provider fallback
```

AI Conductor handles:

```text
mission state
task state
workflow
gstack
context
Aider execution
verification
safe worktrees
checkpointing
cross-provider mission continuity
human escalation
resource economics at mission level
```

Do not duplicate OmniRoute's lower-level router unnecessarily.

---

# 18. OmniRoute — Combos

Current OmniRoute documentation defines a combo as a named group/chain of provider/models with a routing strategy. The caller can use a single model identifier representing that combo, and OmniRoute performs the routing/fallback internally. citeturn524701search8

A representative documented creation structure is:

```json
{
  "name": "my-combo",
  "strategy": "priority",
  "targets": [
    {
      "provider": "anthropic",
      "model": "claude-opus-4-7",
      "weight": 1
    },
    {
      "provider": "openai",
      "model": "gpt-4o",
      "weight": 1
    }
  ]
}
```

The current v3.8.50 source also exposes combo creation through its management/MCP surfaces and validates combo names, model chains, and related structures. citeturn524701search5

---

# 19. OmniRoute — Multiple Combos

Multiple combos can coexist.

A combo is a named routing policy, not a single provider.

AI Conductor can therefore maintain multiple logical resources such as:

```text
coding-free
architecture-free
fast-free
review-free
gemini-primary
mistral-repair
high-context
```

However:

> A single ordinary model request should be treated as selecting one model/endpoint representation at a time.

If multiple different combos need to participate in a mission, AI Conductor should orchestrate that at the **mission/step/attempt level**, not assume one HTTP request can magically invoke multiple independent combos.

This distinction avoids accidental coupling between OmniRoute routing and Conductor orchestration.

---

# 20. Important Combo-Naming Warning

Current OmniRoute versions have had issues around combo/model naming collisions.

For example, current issue tracking documents a bug where a combo name matching a model ID can shadow that model in some paths. citeturn524701search3

There are also documented route-specific combo-resolution bugs.

### Conductor rule

Treat combo identity as:

```text
combo_id
combo_name
provider/model targets
strategy
version/observed schema
```

Do not identify a combo solely by display name when a stable ID is available.

---

# 21. Important OmniRoute Combo-Syntax Rule

The current OmniRoute documentation for direct OpenAI-compatible use describes combo consumption as:

```text
model = "<combo-name>"
```

and current OpenCode documentation describes using the combo name directly as a model identifier. citeturn524701search0turn524701search1

The user's existing AI Conductor prototype has used an adapter-style model string resembling:

```text
openai/combo/<combo-name>
```

That may be a client/provider adapter representation rather than OmniRoute's canonical combo identifier.

### Therefore:

**Do not hardcode either syntax as a universal fact.**

The integration layer must determine:

```text
client adapter syntax
vs
actual OmniRoute model selector
```

using the pinned OmniRoute version and a live local probe.

Required compatibility test:

```text
list combo
→ resolve exact combo
→ send chat completion
→ record provider/model used
```

This is exactly the kind of detail that should be verified rather than guessed.

---

# 22. OmniRoute — Auto Routing

Current reference documentation provides built-in `auto` variants including examples such as:

```text
auto
auto/coding
auto/fast
auto/cheap
auto/offline
auto/smart
```

with different routing goals. citeturn333166search2

The current documentation also describes a live candidate scoring engine using factors such as health, quota, cost, latency, success rate, and freshness.

### AI Conductor rule

Do not blindly duplicate OmniRoute's auto-routing logic.

AI Conductor should use OmniRoute as a connectivity/resource layer and choose when mission-level logic should:

```text
select combo
select model family
choose role
handoff
reserve resource
escalate
```

OmniRoute can make the low-level provider choice inside the selected combo.

---

# 23. OmniRoute — Resilience Layers

Current OmniRoute documentation describes several resilience mechanisms, including:

```text
provider circuit breaker
key/account cooldown
model lockout/quarantine
```

and current v3.8.50 sources/issues show ongoing work around health checks, token states, cooldowns, and provider-specific failures. citeturn333166search0turn216020search2

### Important implication

AI Conductor should NOT assume:

```text
OmniRoute failure
=
provider is dead
```

The failure may be:

```text
one key exhausted
one model locked out
one provider circuit open
one connection auth state stale
one route unsupported
```

Therefore the ProviderResult adapter should preserve as much structured failure classification as OmniRoute can provide.

---

# 24. OmniRoute — Health Checks Can Have False/Overly Aggressive States

This is especially important because the user has already encountered health-check behavior.

A current OmniRoute issue documents a health-check bug in which a connection with an access token but no refresh token was incorrectly marked expired, causing it to disappear from the routing pool; the issue specifically notes the same class of behavior around GitHub-access-token sessions with no refresh token. citeturn211377search0turn211377search4

The current 3.8.50 changelog also shows continued fixes around token health, refresh-token handling, and provider-specific authentication states. citeturn211377search5

### Conductor rule

Never treat a dashboard health state as absolute truth without considering:

```text
health state
+
recent actual request outcome
+
cooldown state
+
provider error
+
credential state
```

AI Conductor should observe real request results and maintain its own higher-level reputation/evidence model.

---

# 25. OmniRoute — Provider-Specific Bugs Are Normal

Current issue/changelog history includes examples involving:

- token-health semantics;
- provider-specific validation;
- combo cleanup;
- auto-combo model-family parsing;
- response route differences;
- provider-specific API behavior;
- stale connection states. citeturn211377search5turn524701search3turn524701search4

### Conductor rule

Never build an assumption like:

```text
all providers behave like OpenAI
```

Use:

```text
ProviderCapability
ProviderBehaviorProfile
ProviderHealthObservation
```

for provider-specific facts.

---

# 26. OmniRoute — API Management vs Inference Endpoint

Important distinction:

```text
Inference:
POST /v1/...
```

versus:

```text
Management:
POST /api/...
```

Current documentation shows combo-management routes under `/api/`, while OpenAI-compatible inference uses `/v1/`. citeturn333166search3

Management endpoints may require an admin/management credential distinct from the inference API key.

### AI Conductor rule

Never assume:

```text
chat API key
=
admin API key
=
MCP token
```

Treat them as distinct credential types.

---

# 27. OmniRoute — Version Churn Warning

The current v3.8.50 roadmap explicitly describes ongoing stabilization and a future modular 4.0 direction. citeturn216020search2

This means:

```text
interface currently observed
```

is not necessarily:

```text
permanent public contract
```

### Conductor strategy

Build an `OmniRouteAdapter` that isolates:

```text
base URL
authentication
model selection
combo discovery
combo invocation
health observations
error mapping
management API
```

Only this adapter should know OmniRoute-specific quirks.

---

# 28. OmniRoute — Never Parse UI When an API Exists

The Conductor should prefer:

```text
documented API
local API
CLI
structured output
```

over:

```text
scrape dashboard HTML
screen-scrape browser
infer combo state from UI colors
```

This keeps the integration robust.

---

# 29. gstack — What It Is

gstack is an engineering workflow/skill system rather than a model provider.

Its current README describes a broad collection of skills covering:

```text
office hours
CEO/product planning
engineering planning
design planning/review
build/review
QA
debug/investigate
browser
ship/deploy
retro
security
DevEx
context save/restore
and more
```

Current installation documentation requires:

```text
Claude Code
Git
Bun v1.0+
Node.js on Windows
```

and installs skills into a user-level skills directory by default. citeturn137449search0

### AI Conductor architecture rule

gstack is the **engineering process/skill source**, not the Conductor's reliability engine.

---

# 30. gstack — Skills Are Valuable, But Runtime Coupling Must Be Controlled

gstack's current installation and routing are heavily oriented around Claude Code and its skills.

Therefore AI Conductor should not make the entire application depend on:

```text
Claude Code must exist
```

especially because the product is supposed to be free-first and multi-resource.

Instead:

```text
gstack skill contract
        ↓
AI Conductor Skill Runtime
        ↓
resource scheduler
        ↓
Aider/provider/tool
```

Use gstack's methodology/skill definitions as the process layer while keeping the Conductor runtime independent where practical.

---

# 31. gstack — Current Skill Routing

The current gstack design explicitly routes request types to specialized skills.

Examples documented in its current skill routing include:

```text
product ideas → /office-hours
strategy/scope → /plan-ceo-review
architecture → /plan-eng-review
design → /plan-design-review / design-consultation
bugs → /investigate
QA → /qa /qa-only
code review → /review
visual polish → /design-review
ship → /ship /land-and-deploy
save progress → /context-save
resume → /context-restore
```

citeturn137449search10

### Conductor mapping

AI Conductor should model these as:

```text
Skill
  input contract
  output artifact
  capabilities
  allowed tools
  required evidence
  next-state rules
```

---

# 32. gstack — Browse Is a Special Dependency

gstack's current browser tooling uses Playwright/Chromium and a browser server/extension architecture. The browser implementation uses a browser binary and a local browser control server. citeturn137449search3turn137449search6

Therefore:

```text
gstack installed
≠
browser QA automatically healthy
```

Browser infrastructure is a separate health domain.

---

# 33. gstack — Real Windows Browser Issues Exist

Current gstack issue tracking documents a Windows/Bun/Chromium problem where `browse.exe` can fail to connect to Chromium due to `--remote-debugging-pipe` behavior under Bun. The issue identifies Node.js as a working workaround. citeturn137449search4

Another current setup issue documents Playwright/Chromium installation problems in certain environments. citeturn137449search2turn137449search8

### Conductor rule

Browser health must be independently probed:

```text
gstack installed
+
browse executable exists
+
Chromium available
+
browser launch works
+
control channel works
+
page navigation works
```

Only then is the browser capability considered healthy.

---

# 34. gstack — Browser Failure Must Not Break Core Coding

The core tool should remain functional if:

```text
Chromium unavailable
Playwright broken
browser skill unavailable
extension unavailable
```

Then:

```text
QA browser capability = unavailable
```

rather than:

```text
AI Conductor = unusable
```

A mission should use a non-browser verification path where appropriate or escalate only when browser verification is genuinely required.

---

# 35. gstack — gstack Has Its Own Moving Surface

The repository has no conventional GitHub releases in the reviewed interface, while the changelog continues to evolve. citeturn216020search5turn216020search7

### Rule

Pin a specific gstack commit for production integration.

Record:

```text
gstack_commit
gstack_skill_manifest
```

and re-run compatibility checks after upgrades.

---

# 36. gstack — Do Not Treat Skill Prompts as Hard Security

gstack skills are process guidance.

AI Conductor must enforce actual permissions.

Example:

```text
Engineering Review
→ model proposes code change
→ Conductor capability boundary blocks mutation
```

This is stronger than:

```text
prompt says "don't edit"
```

The Blueprint and Task Contract system already require this philosophy.

---

# 37. gstack — Skill Artifacts

Where a skill creates:

```text
plan
review
QA findings
security findings
ship report
retro
```

AI Conductor should preserve the important outputs as structured evidence:

```text
SkillResult
  artifact
  decisions
  risks
  findings
  acceptance criteria
  unresolved questions
```

This makes skill state transferable between models.

---

# 38. gstack — Context Save/Restore Is Useful but Not the Conductor State

gstack provides context-preservation skills such as `/context-save` and `/context-restore`. citeturn137449search10

AI Conductor should use them as useful supplementary context, not as its authoritative mission state.

The authoritative continuation state remains:

```text
Build State
+
Task Contract
+
Verification evidence
+
Repository
```

---

# 39. gstack — Engineering Process vs AI Conductor Mission Engine

The correct separation is:

```text
gstack:
"What engineering role/process should happen now?"

AI Conductor:
"Can, how, and under what safety/evidence/resource conditions should that role happen?"
```

This allows the system to use gstack without becoming gstack-dependent.

---

# 40. Component Interaction Model

The intended real system is:

```text
                    USER
                     │
                     ▼
              AI CONDUCTOR
              Mission Engine
                     │
                     ▼
                gstack Skill
                     │
                     ▼
               Skill Contract
                     │
             ┌───────┴────────┐
             ▼                ▼
       Capability        Resource Scheduler
       enforcement              │
             │                  ▼
             └──────────► OmniRoute
                              │
                     ┌────────┼────────┐
                     ▼        ▼        ▼
                   Gemini   Mistral   Other
                     │
                     ▼
                    Aider
                     │
              isolated worktree
                     │
                     ▼
                  evidence
                     │
              reconciliation
                     │
                 verification
                     │
                  checkpoint
                     │
                    merge
```

The actual project may place Aider before/after a provider adapter depending on how the CLI is invoked, but the **ownership boundaries** remain the same.

---

# 41. Who Owns What

| Responsibility | gstack | Aider | OmniRoute | AI Conductor |
|---|---:|---:|---:|---:|
| Product planning | ✓ | advisory | — | orchestrates |
| Engineering review | ✓ | — | — | persists/enforces |
| Code editing | — | ✓ | — | controls |
| Repo map | — | ✓ | — | complements |
| Git editing | — | ✓ | — | owns safety boundary |
| Provider connection | — | consumes | ✓ | observes |
| Combo routing | — | consumes | ✓ | selects policy |
| Provider health | — | — | ✓ | higher-level reputation |
| Mission state | — | — | — | ✓ |
| Checkpoint semantics | — | — | — | ✓ |
| Reconciliation | — | — | — | ✓ |
| Verification authority | — | tests/helpers | — | ✓ |
| Handoff | skill output | execution | route fallback | ✓ |
| Human escalation | skill may ask | — | — | ✓ |
| Resource economics | — | — | low-level | ✓ |
| Failure corpus | — | — | — | ✓ |

---

# 42. What AI Conductor Must NOT Duplicate

Do not rebuild:

```text
Aider:
repository editing engine
repo map implementation
Git diff/editor machinery

OmniRoute:
provider connection manager
low-level combo router
provider authentication implementation

gstack:
full proprietary-style process recreation
skill prose when the existing skill is already adequate
```

Instead:

```text
adapter
observe
constrain
orchestrate
verify
recover
```

---

# 43. What AI Conductor Must Add Because These Systems Don't Solve It

AI Conductor owns the difficult cross-system layer:

```text
mission continuity
cross-model handoff
state reconstruction
safe retries
unknown outcomes
workspace ownership
two-phase merge
verification authority
acceptance contracts
failure memory
resource economics
context budgeting
human escalation
capability enforcement
task lifecycle
```

This is the actual differentiation.

---

# 44. Integration Failure Taxonomy

When an external component fails, classify before reacting.

```text
AIDER_STARTUP
AIDER_MODEL_METADATA
AIDER_CONTEXT
AIDER_EDIT
AIDER_PROCESS
AIDER_GIT

OMNIROUTE_AUTH
OMNIROUTE_COMBO
OMNIROUTE_ROUTING
OMNIROUTE_HEALTH
OMNIROUTE_PROVIDER
OMNIROUTE_API
OMNIROUTE_SCHEMA

GSTACK_INSTALL
GSTACK_SKILL
GSTACK_BROWSER
GSTACK_PLAYWRIGHT
GSTACK_NODE/BUN

NETWORK
FILESYSTEM
GIT
VERIFICATION
SECURITY
```

Do not call everything:

```text
provider error
```

because recovery policy depends on the exact layer.

---

# 45. Aider Startup Strategy for AI Conductor

Before launching Aider:

```text
1. determine exact Aider version
2. determine current model identifier
3. resolve model metadata
4. prepare known settings
5. prepare allowed files
6. prepare isolated worktree
7. prepare environment
8. disable/handle unwanted interactive prompts
9. set timeout policy
10. launch
```

After launch:

```text
capture startup banner
capture model identity
capture repo/worktree identity
capture repo-map metadata
capture warning state
```

This makes Aider behavior auditable.

---

# 46. Aider Model Metadata Strategy

AI Conductor should maintain:

```text
Conductor Capability Registry
```

and, when needed, generate/use Aider-compatible metadata configuration.

But do not silently inject false metadata.

If:

```text
actual context = UNKNOWN
```

then:

```text
unknown
```

until verified.

A wrong context value is worse than a warning because it can cause incorrect context planning.

---

# 47. OmniRoute Connection Strategy

For a new OmniRoute installation:

```text
discover local endpoint
→ verify management access
→ discover configured providers/connections
→ discover combos
→ test one known-good model
→ test one known-good combo
→ record actual accepted model syntax
→ record health behavior
```

Do not assume the dashboard state is enough.

---

# 48. OmniRoute Combo Compatibility Probe

Before using a combo in production:

```text
1. discover combo name/id
2. resolve target list
3. send a minimal chat request
4. observe routing result
5. verify response format
6. verify error classification
7. verify fallback behavior
8. record the exact invocation syntax
```

This is especially important because current OmniRoute versions have had combo-resolution bugs affecting different endpoints. citeturn524701search6turn524701search7

---

# 49. Multiple Keys / Multiple Accounts

OmniRoute can maintain multiple provider credentials/accounts and uses connection/key health mechanisms to avoid repeatedly sending traffic through exhausted/unhealthy resources. Current repository documentation and changelog show per-key health, cooldown, quota-related behavior, and provider-specific recovery work. citeturn333166search0turn211377search3

AI Conductor should therefore distinguish:

```text
provider
account/connection
credential/key
model
combo
```

These are not interchangeable identities.

---

# 50. Multiple Combos in AI Conductor

The Conductor should support:

```text
combo registry
    ├── planning combo
    ├── coding combo
    ├── review combo
    ├── QA combo
    └── emergency combo
```

A mission can move between them according to gstack role.

This is different from asking OmniRoute to run several combos inside one ordinary request.

---

# 51. Free-First Strategy

The product's "free" goal should not be implemented as:

```text
use random free model
```

It should be:

```text
prefer free resources
+
verify capability
+
preserve quota
+
avoid waste
+
route to best fit
+
fallback safely
```

Aider and OmniRoute both have useful native mechanisms, but AI Conductor must manage mission-level economics.

---

# 52. Context Economics

Aider itself recommends small, relevant file context and has explicit context/token tools. citeturn720478search8turn720478search13

Therefore AI Conductor should prefer:

```text
relevant files
+
repo map
+
mission state
+
recent diff
+
acceptance criteria
```

instead of:

```text
whole repository
+
full transcript
```

The system should measure:

```text
tokens sent
tokens repeated
handoff overhead
unnecessary context
```

---

# 53. Aider Retry Rule

Never interpret Aider errors alone.

Aider may fail because:

```text
provider
model
context
edit format
tool
filesystem
Git
weak model
interactive prompt
```

Therefore:

```text
Aider error
    ↓
classify layer
    ↓
reconcile workspace
    ↓
then decide retry/handoff/repair
```

---

# 54. OmniRoute Retry Rule

Never blindly retry a route just because HTTP failed.

For example:

```text
429
→ cooldown / next target

quota exhausted
→ quarantine target

auth failure
→ connection problem

unsupported model
→ capability/model mapping problem

5xx
→ provider reliability problem

malformed response
→ integration/parser problem
```

AI Conductor must preserve the distinction.

---

# 55. gstack Retry Rule

A failed gstack skill is not necessarily a provider failure.

Possible causes:

```text
skill logic
input contract
missing prerequisite artifact
browser failure
tool capability
model failure
context issue
```

Classify the layer before repeating the skill.

---

# 56. External Component Failure Does Not Own Mission State

Aider can stop.

OmniRoute can go down.

gstack can fail.

The mission remains:

```text
AI Conductor state
```

That is the core design.

---

# 57. What to Log for Each External Attempt

Record:

```text
component
component_version
adapter_version
mission_id
step_id
attempt_id
request_id
model/resource identifier
capability snapshot
task role
input context fingerprint
worktree
start time
end time
result
failure classification
sanitized metadata
```

Never record secrets.

---

# 58. What Not to Trust From External Components

Never treat these as authoritative on their own:

```text
Aider: "Applied edit"
Aider: "Done"
Aider: "Git commit succeeded"
OmniRoute: "connection healthy"
OmniRoute: "combo healthy"
gstack: "review complete"
model: "tests pass"
HTTP 200
```

Instead:

```text
external statement
+
actual evidence
+
Conductor verification
```

---

# 59. What OX Alpha May Rely On

OX Alpha may use external component documentation/source to choose implementation details.

However:

```text
documentation claim
→ implementation assumption
```

must be appropriate to the exact version.

For critical behavior:

```text
documentation
+
source/issue evidence
+
controlled local probe
```

is preferred.

---

# 60. When OX Alpha Should Stop Instead of Guessing

Stop and mark `UNKNOWN` when:

- OmniRoute version differs materially from the reference;
- combo invocation syntax is unclear;
- Aider model metadata is unknown and affects safety;
- Aider's edit/commit behavior differs from the expected version;
- gstack skill name/contract differs from the reference;
- provider capability cannot be verified;
- browser dependencies are missing;
- an error may represent an ambiguous external side effect.

The right action is:

```text
record uncertainty
+
probe
+
update evidence
```

not improvisation.

---

# 61. Integration Knowledge Update Policy

This file itself is versioned.

When a new external behavior is discovered:

```text
observe
→ classify evidence
→ update entry
→ link source/issue
→ add regression/compatibility test
→ update adapter if necessary
```

Do not silently replace old behavior notes.

---

# 62. Component Compatibility Matrix

The implementation should maintain a machine-readable compatibility record similar to:

```yaml
components:

  aider:
    version: "0.86.0"
    integration_status: "verified"
    adapter_version: "1.0"
    capabilities:
      git: true
      streaming: true
      repo_map: true
      command_interface: true
    known_risks:
      - unknown_model_metadata
      - context_overflow
      - git_side_effects
      - interactive_warnings

  omniroute:
    version: "3.8.50"
    commit: "PIN_REQUIRED"
    integration_status: "probe_required"
    capabilities:
      chat_completions: true
      combos: true
      auto_routing: true
    known_risks:
      - fast_moving_api
      - provider_specific_health
      - combo_endpoint_differences
      - token_health_edge_cases

  gstack:
    commit: "PIN_REQUIRED"
    integration_status: "adapter_required"
    capabilities:
      skill_runtime: true
      browser: true
      qa: true
      review: true
    known_risks:
      - claude_code_orientation
      - browser_runtime_dependencies
      - platform_specific_browser_issues
```

Do not put `verified` in the actual implementation until the local compatibility suite passes.

---

# 63. Mandatory Compatibility Test Suite

Before an external component is accepted into a release:

## Aider

```text
launch
model selection
unknown-model handling
metadata configuration
file add/read/drop
edit
diff
process interruption
non-interactive startup
worktree execution
Git behavior
verification interaction
```

## OmniRoute

```text
health
provider connection
one concrete model
one combo
combo discovery
combo invocation
fallback
rate-limit behavior
quota behavior
error classification
management vs inference auth
```

## gstack

```text
skill discovery
skill invocation
skill output
capability boundary
browser health
QA health
failure handling
upgrade/version detection
```

---

# 64. Known External-Component Facts vs Project-Specific Facts

This distinction is mandatory.

### Publicly verified:

Aider documents unknown-model warnings, metadata files, Git behavior, context handling, slash commands, and supported modes. citeturn281259search0turn281259search1turn720478search2turn720478search0turn720478search11

OmniRoute currently documents combos, auto routing, routing strategies, health/resilience features, and management/inference API separation. citeturn333166search0turn524701search8turn333166search2turn333166search3

gstack currently documents its skill set, installation requirements, browser architecture, and skill-routing model. citeturn137449search0turn137449search10

### Project-observed:

The user's current AI Conductor/Aider integration has observed behaviors including:

```text
unknown model warnings
documentation-browser prompts
release-notes prompts
OmniRoute connection health behavior
combo selection
provider/model routing
Aider process and edit-output behavior
```

These should be preserved as project observations and regression tests, but not generalized beyond the version/environment in which they were observed.

---

# 65. Sources and Verification Register

Primary sources used for this knowledge base include:

### Aider

- Aider documentation: usage, commands, Git integration, configuration/options, token limits, model warnings, modes, troubleshooting. citeturn720478search1turn720478search0turn720478search2turn281259search1turn720478search13turn281259search0turn720478search11
- Aider current source code and issue reports for interactive documentation URL behavior and real-world warning prompts. citeturn599026search1turn599026search0
- Aider release history/current releases. citeturn216020search0turn281259search5

### OmniRoute

- Current OmniRoute repository/reference line. citeturn524701search12turn216020search2
- Combo/strategy documentation. citeturn524701search8turn333166search2
- Management/API behavior. citeturn333166search3
- Current issues/changelog for real routing, combo, authentication, and health-check edge cases. citeturn211377search0turn211377search5turn524701search3turn524701search7
- Community discussion as practical supporting evidence, not sole authority. citeturn211377reddit33turn211377reddit34

### gstack

- Current README/install/skill set. citeturn137449search0
- Skill-routing behavior. citeturn137449search10
- Browser implementation and browser-side issues. citeturn137449search3turn137449search4turn137449search2turn137449search8
- Current changelog and repository state. citeturn216020search7turn216020search5

---

# 66. Final Integration Principle

The external components are powerful.

Do not make AI Conductor weaker by pretending they are simple.

Do not make AI Conductor fragile by pretending they are perfect.

The correct posture is:

```text
LEVERAGE THEIR STRENGTHS
        +
UNDERSTAND THEIR REAL LIMITS
        +
OBSERVE THEIR ACTUAL BEHAVIOR
        +
ISOLATE THEIR QUIRKS BEHIND ADAPTERS
        +
VERIFY CRITICAL ASSUMPTIONS
        +
PRESERVE OX ALPHA'S ENGINEERING JUDGMENT
```

The goal is not to force OX Alpha to follow every line of this file mechanically.

The goal is to prevent it from wasting time rediscovering known facts, guessing APIs, or repeating known integration mistakes.

---

# 67. OX Alpha Integration Instruction

When implementing any external component integration:

> Read the relevant section of this Knowledge Base first.
>
> Identify which statements are verified documentation, which are reproduced issue behavior, and which are project-observed behavior.
>
> Check the actual installed/pinned component version.
>
> If the local version differs materially from the documented reference, inspect the actual source/docs and run a compatibility probe.
>
> Do not assume a model ID, combo syntax, capability, health state, or command behavior from memory.
>
> Use adapters so external quirks remain outside the Conductor Kernel.
>
> Preserve the external component's strengths instead of unnecessarily reimplementing them.
>
> Do not treat an external tool's statement of success as authoritative completion.
>
> Give the agent room to choose a better implementation when the contract is not normative.
>
> If a better solution is discovered that stays inside the architecture, use it and record the meaningful decision.
>
> If the better solution would change an invariant, architecture boundary, security boundary, or acceptance rule, stop and use the formal change process.

---

# 68. The Strategic Rule

AI Conductor should be:

```text
THIN WHERE EXTERNAL TOOLS ARE STRONG
THICK WHERE CROSS-TOOL RELIABILITY IS WEAK
```

Therefore:

```text
Aider editing
→ delegate

OmniRoute low-level provider routing
→ delegate

gstack specialist workflow ideas
→ leverage

Mission state
→ own

Reconciliation
→ own

Verification authority
→ own

User-workspace safety
→ own

Cross-model continuity
→ own

Failure memory
→ own

Resource economics
→ own
```

That is how the final system stays lightweight **without being shallow**.

---

# 69. Final Knowledge-Base Acceptance Rules

This document is considered usable when:

```text
[ ] Aider integration facts are version-aware.
[ ] Aider Git behavior is documented.
[ ] Aider context/model-warning behavior is documented.
[ ] Aider command surfaces are documented.
[ ] Aider limitations are tied to mitigation.
[ ] OmniRoute combo behavior is documented.
[ ] OmniRoute management vs inference APIs are distinguished.
[ ] OmniRoute version churn is acknowledged.
[ ] OmniRoute health-check quirks are documented.
[ ] Combo syntax is explicitly treated as version/client sensitive.
[ ] Multiple combo use is distinguished from multiple targets in one combo.
[ ] gstack skill architecture is documented.
[ ] gstack runtime dependencies are documented.
[ ] gstack browser dependency is documented.
[ ] Windows/browser failure modes are documented.
[ ] External ownership boundaries are explicit.
[ ] Unknown facts must be probed rather than guessed.
[ ] Project-observed facts remain distinguished from public facts.
[ ] Version/commit pinning is mandatory.
[ ] Compatibility probes are defined.
[ ] OX Alpha engineering discretion is explicitly preserved.
[ ] External quirks are kept behind adapters.
[ ] The information is intended to save implementation time, not restrict reasoning.
```

---

# 70. Closing Principle

This file exists for one reason:

> **The next coding agent should start with knowledge, not rediscovery.**

The ideal behavior is:

```text
KNOWN FACT
   ↓
USE IT

KNOWN LIMIT
   ↓
DESIGN AROUND IT

KNOWN FAILURE
   ↓
REGRESSION-PROTECT IT

UNKNOWN
   ↓
VERIFY IT

BETTER IDEA INSIDE AUTHORIZED BOUNDARY
   ↓
USE IT

ARCHITECTURE-CHANGING IDEA
   ↓
ESCALATE IT
```

This gives OX Alpha the thing you specifically asked for:

**not a cage, not a giant pile of commands, and not another hallucination-prone specification — but a reliable body of verified knowledge that lets it spend its intelligence solving the actual engineering problem.**

---

**END — AI CONDUCTOR EXTERNAL COMPONENTS INTEGRATION & OPERATIONAL KNOWLEDGE BASE v1.1**


---

# 71. Current Source Register — 2026-08-28

Primary references used for this v1.1 update:

## Jcode

- Repository: https://github.com/1jehuang/jcode
- Documentation: https://jcode.sh/docs
- Releases: https://github.com/1jehuang/jcode/releases
- Swarm Architecture: https://github.com/1jehuang/jcode/blob/master/docs/SWARM_ARCHITECTURE.md
- Safety System: https://github.com/1jehuang/jcode/blob/master/docs/SAFETY_SYSTEM.md
- Harness API design: https://github.com/1jehuang/jcode/blob/master/docs/HARNESS_API_AND_DESKTOP_REWRITE.md
- OmniRoute provider request: https://github.com/1jehuang/jcode/issues/704
- Session-resume issue: https://github.com/1jehuang/jcode/issues/753
- Bash visibility issue: https://github.com/1jehuang/jcode/issues/850

## Aider

- Git integration: https://aider.chat/docs/git.html
- In-chat commands: https://aider.chat/docs/usage/commands.html
- Models/API keys: https://aider.chat/docs/troubleshooting/models-and-keys.html
- Connecting to LLMs: https://aider.chat/docs/llms.html
- Releases: https://github.com/Aider-AI/aider/releases

## OmniRoute

- v3.8.50 README: https://github.com/diegosouzapw/OmniRoute/blob/release/v3.8.50/README.md
- Auto-Combo guide: https://github.com/diegosouzapw/OmniRoute/blob/release/v3.8.50/docs/getting-started/AUTO-COMBO-GUIDE.md
- Combo routing skill: https://github.com/diegosouzapw/OmniRoute/blob/release/v3.8.50/skills/omni-combos-routing/SKILL.md
- Troubleshooting: https://github.com/diegosouzapw/OmniRoute/blob/release/v3.8.50/docs/getting-started/TROUBLESHOOTING.md
- Setup guide: https://github.com/diegosouzapw/OmniRoute/blob/release/v3.8.50/docs/guides/SETUP_GUIDE.md
- Release/branch discussion: https://github.com/diegosouzapw/OmniRoute/discussions/10439
- Release branch: https://github.com/diegosouzapw/OmniRoute/tree/release/v3.8.50
- Active issue tracker: https://github.com/diegosouzapw/OmniRoute/issues

## gstack

- Repository/README: https://github.com/garrytan/gstack/blob/main/README.md
- Browser skills design: https://github.com/garrytan/gstack/blob/main/docs/designs/BROWSER_SKILLS_V1.md
- Browser launcher: https://github.com/garrytan/gstack/blob/main/scripts/app/gstack-browser

---

# 72. v1.1 Update Policy

The knowledge base must be refreshed whenever one of these occurs:

```text
Aider version changes materially
Jcode version/revision changes materially
OmniRoute version/branch changes materially
gstack skill/runtime changes materially
A new integration is promoted
A known failure is fixed upstream
A project-observed behavior is disproven
```

Each update must preserve the evidence distinction:

```text
DOCUMENTED
OBSERVED
REPRODUCED
INFERRED
UNKNOWN
```

No volatile external fact should silently become an architectural invariant.

---

**END — AI CONDUCTOR EXTERNAL COMPONENTS INTEGRATION & OPERATIONAL KNOWLEDGE BASE v1.1**
