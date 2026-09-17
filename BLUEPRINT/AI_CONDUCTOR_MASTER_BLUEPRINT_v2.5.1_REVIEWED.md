# AI CONDUCTOR
## Master Reliability & Architecture Blueprint — v2.5.1

*Supersedes v2.4. v2.5 preserves the sixteen reliability invariants, the Mission/Step/Attempt state model, evidence provenance, two-phase merge safety, event-chain anchoring, external-side-effect idempotency, the Verification Sandbox, Provider Health Ledger, deterministic Clock boundary, and implementation-reliability layer established by v2.4. The sole architectural expansion in v2.5 is the execution boundary: the Conductor is now explicitly harness-neutral, with a first-class Execution Adapter Contract and a controlled evaluation path for Jcode, Aider, and future execution harnesses. Jcode is a candidate executor, not a replacement for the reliability kernel.

---

## 0. How to Use This Document

This is not a feature list. It is a **reliability-first engineering specification**. Every section exists to answer one question: *"If something goes wrong here, how does the system know, and how does it recover without losing, duplicating, or silently overwriting work?"*

Read it in order once. After that, use it as a reference: before building any module, re-read its section, write the acceptance criteria first, then implement.

---

## 1. Vision & Positioning

**What this is NOT:** "a nicer interface for Aider," "another AI coding agent," "a GUI wrapper."

**What this IS:**

> **Reliable orchestration for unreliable AI resources.**

You are not competing with Cursor, Copilot, or Windsurf on raw model quality — you don't control the models. You are competing on **what happens when the model fails**, because with free/limited providers, failure is not an edge case — it is a routine, expected event.

```
gstack        →  the engineering PROCESS layer (roles, review gates, skills)
Execution     →  the EXECUTION-HARNESS layer (Jcode, Aider, or another conforming executor)
OmniRoute     →  the provider CONNECTIVITY layer (keys, routing, rate limits)
AI Conductor  →  the CONTROL, AUTHORITY, and RELIABILITY layer that governs all of the above

Execution Harnesses are replaceable implementation components.
Jcode and Aider are candidates, not authorities and not architectural dependencies.
```

AI Conductor's job is not to be smarter than the models. Its job is to guarantee that no matter how the models or execution harnesses fail, **no work is silently lost, no work is silently duplicated, no work is silently overwritten, and the person never has to personally track what happened.**

The execution layer is deliberately adapter-based:

```text
                    AI CONDUCTOR
                         │
              Execution Adapter Contract
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
        Jcode           Aider       Future Harness
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                model / provider resources
```

The Conductor owns authority over mission, step, attempt, capability, workspace, verification, recovery, cancellation, persistence, and merge. An execution harness owns only the execution work explicitly delegated to it.

A harness may have its own planning, memory, swarm, session, provider, or UI features. Those features may be used when beneficial, but they **must not become a second source of truth for Conductor state or a second authority for acceptance, merge, recovery, or security policy**.

The Conductor must therefore be able to replace one harness with another without changing the reliability kernel.


---

## 2. Non-Negotiable Principles

1. **Risk-first, not feature-first.** A feature untested against failure is a liability, not an asset.
2. **The model is advisory, not authoritative.** Nothing a model claims is true until the Conductor observes evidence of it.
3. **External truth, not self-certification.** Never let an AI design a recovery system and then ask an AI whether that design is correct. Correctness comes from observable behavior, deterministic tests, deliberate failure injection, and independent review — never from a model's confidence, including this document's own first draft.
4. **Unknown and Conflict are real outcomes**, not edge cases to special-case later. A system with only `success`/`failure` cannot survive contact with real, ambiguous failures.
5. **Efficiency is part of reliability.** A recovery that "works" but re-reads the whole project and burns 40,000 tokens has failed the actual mission.
6. **Build the smallest scientifically testable core first**, against a fake, deterministic provider, before touching a real one.
7. **Never let the system silently choose between two truths.** If both the user and the AI touched the same file, the system stops and asks — it does not guess which version is "right."

---

## 3. Core Invariants

| # | Invariant |
|---|---|
| 1 | Accepted work is never silently lost |
| 2 | Unaccepted work is never presented as completed |
| 3 | Resume never blindly repeats an accepted side effect |
| 4 | Every mutation has an attributable attempt |
| 5 | Recovery is deterministic given the same recorded state |
| 6 | User changes are protected — and **attributable**, not merely "detected as some change" |
| 7 | Model output is advisory, not authoritative |
| 8 | Every state transition is observable — logged as an event, or it didn't happen |
| 9 | State schema is versioned and migratable |
| 10 | Capability boundaries are structurally enforced, not requested via prompt |
| 11 | **(new in v2.1)** A detected conflict between two sources of change halts automatic recovery — it is never silently auto-resolved |
| 12 | **(new in v2.1)** Credentials never appear in plaintext in any log, event, ledger, crash report, or model prompt |
| 13 | **(new in v2.2)** Before merging an isolated attempt's result back into the live workspace, the live workspace is re-checked against its original baseline — a change in the meantime is treated as a Conflict, never merged blind |
| 14 | **(new in v2.2)** No executor, provider, UI action, or model response may directly set an Attempt to `Succeeded`. Only the Verification Engine (Section 7), after its full pipeline passes, may cause that transition |
| 15 | **(revised in v2.4)** Every multi-record logical state transition is durable and recoverable: it either reaches a known committed state, or remains represented by a durable intent that can be safely completed or reconciled on restart. This is a **local persistence + recovery** guarantee, enforced via the outbox pattern (Section 20) until true transactional storage (Section 4.8) exists — it does **not**, on its own, make an external side effect (a provider call, a shell command) atomic; see Invariant 16 |
| 16 | **(new in v2.4)** No external side effect (a provider API call, a verification command, any process the Conductor didn't fully control) is ever assumed idempotent merely because a local idempotency key was generated for it. Safety on retry depends on what the external system actually guarantees (Section 10.5) — when that is unknown, the correct response to an ambiguous outcome is reconciliation or explicit surfaced uncertainty, never a silent assumed-safe retry |

---

## 4. State & Data Model

### 4.1 The Hierarchy

```
Mission
  └── Step (a phase: Plan / Build / Review / QA)
        └── Attempt (one concrete try, on one provider/combo)
              └── Evidence (what was actually observed)
```

### 4.1.1 Correlation ID Hierarchy (New in v2.4 — Was Implicit)

Phase 1 already commits to threading correlation IDs across mission/step/attempt/event/provider-request/tool-call. Made explicit as its own contract so every module generates and propagates IDs against one canonical shape, not six ad hoc conventions:

```
mission_id
  └── step_id
        └── attempt_id
              ├── provider_request_id
              ├── tool_call_id
              ├── verification_execution_id
              └── event_id  (one or more per attempt, each carrying the full chain above)
```

Every event, evidence record, and diagnostic export carries the full chain, not just its own ID. This turns "why did attempt-4821 do that" from grepping several files into one query: find every record whose `attempt_id == attempt-4821`, ordered by `sequence_number` (Section 4.9). The Developer Reliability Console (Section 21) and the Diagnostic Bundle (Section 13) both key off this same chain.

### 4.2 Attempt Status — Six States (Lifecycle Only)

**Fix applied here:** v2.0 conflated an attempt's *lifecycle* status with the *result of reconciling evidence*. Those are two different concerns and must be two different enums. This was the internal contradiction the second review correctly caught — `PartiallyApplied` was used in the recovery logic but never existed in the state enum it was supposed to belong to.

```rust
enum AttemptStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,   // resolved ONLY by reconciliation, never by assumption
}
```

Valid transitions, enforced in code, never by bare field assignment:

```
Pending   → Running
Running   → Succeeded | Failed | Unknown | Cancelled
Unknown   → Succeeded | Failed             (only via reconciliation)
Failed / Succeeded / Cancelled → terminal (a retry creates a NEW Attempt)
```

### 4.2.1 Mission and Step State Machines (New in v2.4 — Were Underspecified)

v2.3 gave `Attempt` a rigorous transition table but left `Mission` and `Step` defined only by their hierarchy, not their own lifecycle. That's a real gap: a system can be perfectly precise at the Attempt level while the levels above it stay ambiguous about, e.g., what it means for a Mission to be "paused" versus "blocked," or whether a Step can be skipped. Both get the same treatment as `AttemptStatus` — an explicit enum, an explicit transition table, and a single owner authorized to mutate it.

```rust
enum MissionStatus {
    Draft,       // contract being defined, nothing executing yet
    Planning,     // decomposing into Steps
    Running,      // at least one Step is Running
    Paused,       // person-initiated pause; resumable
    Blocked,      // system-initiated halt — Conflict, budget exhaustion, escalation (Section 28.13)
    Succeeded,    // every required Step Succeeded AND the Mission Acceptance Contract passed as a whole
    Failed,       // could not complete within Recovery Budget (Section 28.13)
    Cancelled,    // person-initiated termination
}

enum StepStatus {
    Pending,
    Running,
    Blocked,       // waiting on a Mission-level pause/block, not a failure of its own
    Succeeded,
    Failed,
    Skipped,       // explicitly not required for this Mission's contract
}
```

```
Mission: Draft → Planning → Running → { Succeeded | Failed | Cancelled }
         Running ⇄ Paused                         (person-initiated, reversible)
         Running → Blocked → { Running | Failed | Cancelled }   (system-initiated)

Step:    Pending → Running → { Succeeded | Failed | Skipped }
         Running → Blocked → Running              (mirrors an owning Mission pause/block)
```

**Transition authority, matching the pattern already established for Attempt:** only the single authoritative executor (Section 17) may mutate `MissionStatus`/`StepStatus`, and each transition is itself an event (Section 4.9). `MissionStatus::Succeeded` specifically requires every blocking criterion in the Mission Acceptance Contract (Section 8) to have passed — a Mission is not "done" merely because its last Step's Attempt succeeded; the contract is evaluated once more at the Mission level, since some criteria (e.g. `auth-07`, "only expected files changed") are naturally mission-scoped rather than per-step.

### 4.3 Reconciliation Outcome — A Separate Enum (New)

This is what actually answers "what did the evidence show," independent of the attempt's lifecycle bookkeeping:

```rust
enum ReconciliationOutcome {
    NoChange,             // filesystem matches the pre-attempt baseline exactly
    ExpectedChange,        // actual diff matches the intended operation
    UnexpectedChange,      // something changed, but not what was requested
    PartialChange,         // some but not all of the intended change landed
    UserChangeDetected,    // a change is attributable to the user, not the attempt
    Conflict,              // the attempt's change and a user change overlap on the same region
}
```

An `Attempt` in `Unknown` is resolved by running reconciliation, which produces one of the above. **Critical correction (v2.3): reconciliation never sets `AttemptStatus::Succeeded` itself — that would violate Invariant 14.** Its outcome instead determines what happens next, and only the Verification Engine (Section 7) can ultimately produce `Succeeded`:

```
NoChange / nothing happened   → AttemptStatus::Failed (clean retry is safe)
ExpectedChange                → hand off to the Verification Engine (Section 7) —
                                  its full pipeline must still pass before this can
                                  ever become AttemptStatus::Succeeded
UnexpectedChange / Partial    → AttemptStatus::Failed, but flagged for a targeted repair,
                                  never a full redo
UserChangeDetected            → AttemptStatus::Failed for the attempt (nothing of ours landed),
                                  no action taken on the user's own change
Conflict                      → AttemptStatus::Failed, mission PAUSED, person must choose
                                  [Keep user version] [Keep AI version] [Merge] [Review diff]
```

The distinction matters architecturally, not just semantically: `ExpectedChange` means *"the diff we intended appears to be present"* — it is still only advisory-grade evidence at that point. Whether it actually *counts* as success is a separate question the Verification Engine answers by running build/tests/lint/acceptance-contract checks against it. Reconciliation observes; Verification Engine decides. Conflating the two was exactly the mistake this correction fixes.

**Conflict is never resolved automatically.** This is Invariant 11. The moment two independent sources of change overlap on the same lines, the only safe action is to stop and ask.

### 4.4 Evidence Model — Diff-Aware, Not Just Hash-Aware (Expanded)

v2.0's evidence model compared before/after hashes. That answers *"did something change,"* but not *"did the change we intended actually happen."* Fix: record the expected diff up front, and compare it to the actual diff after.

```json
{
  "attempt_id": "attempt-4821",
  "mission_id": "mission-102",
  "step": "build",
  "provider": "gemini",
  "combo": "Gemini Power",
  "started_at": 1755000000,
  "baseline": {
    "App.tsx": "sha256:abc...",
    "auth.ts": "sha256:111..."
  },
  "expected_operation": {
    "files": ["App.tsx"],
    "intent": "wire the new settings route into the router"
  },
  "result": {
    "process_outcome": "disconnected_mid_response",
    "files_after": {
      "App.tsx": "sha256:def...",
      "auth.ts": "sha256:111..."
    },
    "actual_diff": "…unified diff text or structured hunks…"
  },
  "mutation_attribution": {
    "App.tsx": "conductor_attempt_4821",
    "auth.ts": "unchanged"
  },
  "reconciliation_outcome": "expected_change",
  "verification": {
    "status": "passed",
    "execution_id": "verification-4821-01",
    "pipeline_stages_passed": ["diff", "build", "tests", "lint", "requirements", "acceptance_contract"]
  },
  "final_status": "succeeded"
}
```

**(Fixed in v2.4)** v2.3's version of this example showed `reconciliation_outcome` and `final_status: succeeded` side by side with nothing in between, which reads as reconciliation causing success — exactly the conflation Invariant 14 exists to prevent. The explicit `verification` block above makes the causal chain visible in the data itself, not just in the surrounding prose: reconciliation produced `expected_change`; that was then handed to the Verification Engine (Section 7), whose full pipeline passed, and *that* is what set `final_status: succeeded`.

### 4.5 Evidence — Structured Provenance, Not a Simple Ladder (Revised)

Not all evidence is equally trustworthy — v2.2's `EvidenceTrust` enum captured that correctly, but as a single linear ranking it implies something not always true: that, say, a CI result is automatically "more trustworthy" than a fresh local deterministic test. In reality these are different **provenance classes**, not rungs on one universal ladder — a stale, days-old CI result and a test that ran ninety seconds ago against the exact current attempt are not comparable just because both are "tests."

**Correction:** model evidence as a structured record, of which the trust category is only one field:

```rust
struct Evidence {
    provenance: EvidenceProvenance,   // where this evidence came from
    observation_type: ObservationType, // what kind of check it was
    freshness: Duration,               // how long ago it was produced
    execution_id: String,              // ties it to a specific run, not just "a run once"
    source: String,                    // e.g. "npm test -- auth.test.ts", "ci:build-4821"
    reproducibility: Reproducibility,  // Deterministic | Flaky | Unknown
    integrity: IntegrityLevel,         // was this verified independently, or only reported?
}

enum EvidenceProvenance {
    ModelClaim, ProcessObservation, FilesystemObservation,
    LocalTestRunner, CiSystem, ExternalSystemObservation,
}
```

A Mission Acceptance Contract criterion (Section 8) can now require a **specific combination**, not just a minimum rung:

```
must be:
  provenance    = local_test_runner
  freshness     <= 5 minutes
  execution_id  = current_attempt   (not a stale prior run)
  result        = pass
```

This directly prevents a subtle but real failure mode: a CI badge or an old test summary satisfying a criterion that was actually meant to require fresh, current-attempt evidence.

### 4.6 Mutation Attribution

Hash comparison alone cannot answer *"which part of this file did the AI change versus the user."* Fix: capture the actual Git diff (not just a hash) at both the start and end of every attempt, and attribute each changed region:

```
Attempt begins → capture baseline diff-state
Execution Harness operates → capture resulting diff
Compare baseline → result, line-region by line-region
Classify each changed region as:
   expected_conductor_change | unexpected_ai_change | user_change | conflict_region
```

This is what makes Invariant 6 ("user changes are protected") actually true rather than aspirational — protection requires *attribution*, not just detection that "a file changed."

### 4.7 Checkpoint Semantics

A checkpoint is a known-good state boundary with four parts — still true from v2.0, unchanged:

```
CHECKPOINT
  ├── workspace snapshot reference   (Git commit hash)
  ├── mission state at that point
  ├── verification evidence
  └── resource/attempt metadata
```

### 4.8 Storage Design

**Now:** JSON files written atomically (temp file + rename). **Later:** SQLite once the schema stabilizes, for real ACID guarantees across multi-field updates. **Every record carries `schema_version` from day one.** Credentials are **never** stored in these files at all — see Section 12, Credential Vault.

**Invariant 15 in practice:** until SQLite provides real multi-record transactions, every operation touching more than one record (e.g. "mark attempt Succeeded" + "write the corresponding event" + "update the checkpoint") is implemented via the outbox pattern (Section 20): write an intent record first, perform each side effect, mark each complete, and have startup recovery (Section 15) replay any intent left unfinished by a crash. This is not optional cleanup — it is the mechanism that makes Invariant 15's *local, recoverable* durability guarantee actually true rather than aspirational on the current JSON-file storage. It says nothing about external side effects — that is Invariant 16 and Section 10.5.

### 4.8.1 Schema Migration Protocol (New in v2.4 — Was a Gap)

Every record already carries `schema_version` (v2.0). What was missing is what happens when that version needs to change, which is inevitable given this system persists mission/event history indefinitely:

```rust
struct MigrationPolicy {
    current_version: u32,
    supported_versions: RangeInclusive<u32>,   // oldest still readable without migration
    migration_path: Vec<MigrationStep>,         // ordered, one version at a time — no version-skipping
}

struct MigrationStep {
    from: u32,
    to: u32,
    migrate: fn(serde_json::Value) -> Result<serde_json::Value, MigrationError>,
    reversible: bool,
}
```

**Rules:**
- Migrations run **one version at a time** (`v3 → v4 → v5`, never `v3 → v5` directly) — this keeps each migration function small, independently testable, and composable.
- **Backup before migration is mandatory, not optional** — copy the pre-migration store to a versioned backup path before touching anything, so a failed migration is always recoverable by restoring the backup.
- **Migration failure halts startup with a clear message**, never a silent partial migration. The app should tell the person exactly which records failed and offer to restore the pre-migration backup.
- A record older than `supported_versions` is not silently dropped or force-migrated blind — it's surfaced for explicit person confirmation before any destructive step.
- Every migration function gets a fixture-based regression test (old-format input → expected new-format output), living permanently alongside the `failure_cases/` corpus (Section 11), since a migration bug is exactly as costly as a reliability bug — it can silently corrupt years of mission history in one bad startup.

### 4.9 Tamper-Evident Event Chain (New)

Invariant 8 says every state transition is observable via the event log. That guarantee is weaker than it sounds if the log itself could later be silently edited. Fix: a lightweight hash chain, not a blockchain — just enough to make tampering detectable:

```json
{
  "event_id": "evt-88213",
  "sequence_number": 4821,
  "previous_event_hash": "sha256:aaa...",
  "event_hash": "sha256:bbb...",
  "type": "attempt_status_changed",
  "payload": { "attempt_id": "attempt-4821", "to": "Succeeded" }
}
```

Each event's hash is computed over its own payload plus the previous event's hash, exactly like a hash chain. Diagnostics can then prove — not just claim — that "this mission's history hasn't been silently rewritten."

**Gap this doesn't cover on its own: truncation.** A chain proves the *surviving* events weren't modified, but deleting the first N events and rebuilding a valid-looking chain starting from event N+1 would pass this check undetected. Fix: periodically persist an **anchor** — a small, separately-stored record that independently pins the chain's shape:

```json
{
  "anchor_id": "anchor-2026-08-22T09:00Z",
  "genesis_hash": "sha256:...",
  "event_count": 4821,
  "last_event_hash": "sha256:bbb...",
  "checkpoint_hash": "sha256:..."
}
```

Written to a separate file/location from the event log itself, on a regular cadence (e.g., after every checkpoint). Diagnostics then check the *live* chain against the *nearest anchor*, not just against itself — so a truncation shows up as a mismatch against the last known-good anchor, not as a clean-looking (but shortened) chain.

**Threat model, stated explicitly (new in v2.4 — was previously implied, not scoped):** this mechanism protects against different threats to different degrees, and the document should not overclaim which:

| Threat | Protected? |
|---|---|
| Accidental corruption (crash mid-write, disk error, a bug that mutates a past event) | **Yes** — the hash chain plus anchor makes this reliably detectable |
| A compromised dependency/process silently editing history at runtime | **Partially** — detectable *if* the anchor's storage location is genuinely separate (different file, ideally a different write path) and checked on load; not detectable if the same compromised process can rewrite both the chain and the anchor together |
| A user or administrator with full filesystem access on the machine, intentionally rewriting both the event log and the anchor | **No** — two local files under the same account's write access is not a defense against that account. This is a tamper-*evident* mechanism against accidental and software-level corruption, not a cryptographic forensic system that survives a fully compromised or fully trusted-but-hostile local administrator |

The correct framing for documentation, support material, and any future security review: *"the event chain is tamper-evident against accidental and software-level corruption, not a forensic-grade guarantee against someone who controls the machine."* Overstating this would be worse than not having it at all, since it would create false confidence in exactly the scenario (a compromised or malicious local actor) where the mechanism provides the least protection.

---

## 5. Workspace Isolation & Git Worktree Safety (New Major Section)

This is the single most important structural addition from review. A Git commit alone is a good *snapshot* mechanism, but it does not give an execution harness a *safe place to operate*. The fix: isolate every attempt in its own workspace.

```
User's Real Workspace
        │
        ▼
Conductor Safety Boundary
        │
        ▼
Attempt Workspace  (a Git worktree, or an isolated working copy)
        │
        ▼
      Selected Execution Harness operates here — never directly in the user's live folder
        │
        ▼
Verification Engine (Section 7)
        │
   ┌────┴────┐
   ▼         ▼
Accepted   Rejected
   │         │
   ▼         │
Two-phase merge check   │
(Invariant 13: re-verify   │
live workspace still     │
matches baseline)        │
   │                     │
   ▼                     ▼
Merged into            Discarded — the user's real
user workspace           workspace was never touched
```

**Why this matters concretely:** a failed, partial, or rejected attempt currently has to be *undone* from the user's real files (via `git reset` or similar). With worktree isolation, a failed attempt simply **never touched the real workspace in the first place** — there is nothing to undo, because nothing real was ever at risk. This also makes retries, provider handoffs, and repair attempts dramatically safer: each one gets a clean, disposable copy to work in, and only a verified, accepted result ever gets merged back.

**The merge step itself is two-phase, not one (Invariant 13):** isolation alone still leaves a gap between when an attempt's baseline was captured and when its result is finally merged back — the person could have edited the real workspace themselves in that window.

**Correction (v2.3): "check, then merge" alone still has a race.** Nothing stops the person from editing the file in the instant *between* the re-check passing and the merge actually writing — a classic check-then-act (TOCTOU) gap. The fix is to make the whole sequence a single locked transaction, not two separate steps:

```
Acquire workspace mutation lock (single authoritative executor, Section 17,
      already gives us one writer — this reuses that same serialization point)
        ↓
Re-check live workspace against original baseline
        ↓
   Changed?  → release lock → Conflict (Section 4.3), not merged blind
   Unchanged? → perform the merge, still holding the lock
        ↓
Verify the resulting workspace matches what was intended
        ↓
Release lock
```

Because Section 17 already mandates one authoritative executor per mission owning all mission-state mutation, this lock is not new infrastructure — it's routing the merge operation through the same single-writer boundary that already exists, closing the race for free rather than needing a separate locking mechanism.

`git worktree add` is the concrete mechanism on the Git side — cheap, fast, and already part of Git itself, requiring no new dependency.

### 5.1 The Conductor's Lock Only Controls the Conductor (New in v2.4 — Closes an Overclaim)

**Correction:** the locked check-then-merge sequence above prevents a *second Conductor-initiated* mutation from racing the merge. It does **not**, and structurally cannot, prevent a completely separate OS process — VS Code, another terminal, Git itself run manually, another application — from writing to the same files during that same window. A single-writer lock inside one application's process space is not a filesystem-level lock.

The merge is therefore correctly understood as an **expected-state conditional operation** (comparable to optimistic-concurrency-control / compare-and-swap), not a true mutual-exclusion guarantee against arbitrary external actors:

```
Acquire Conductor mutation boundary (Section 17's single-writer lock)
        ↓
Verify live workspace against the recorded baseline, file by file
        ↓
Apply merge, expressed as: "only proceed if the live tree still equals the expected baseline"
        ↓
Verify the resulting tree matches what the merge was supposed to produce
        ↓
   Mismatch at either check → release lock → Conflict (Section 4.3), never merged blind
   Match at both checks     → commit / checkpoint (Section 4.7), still holding the lock
        ↓
Release lock
```

This is the same guarantee a well-built merge tool gives you against a concurrent external editor: it cannot stop the other process from writing, but it can guarantee it never silently merges *over* a write it didn't expect, because every merge is conditioned on the baseline still matching, checked immediately before the write and re-verified immediately after.

**Git edge cases the merge contract must explicitly define behavior for (new in v2.4 — was previously undocumented, meaning implementation would have had to invent answers ad hoc):**

| Workspace state at merge time | Required behavior |
|---|---|
| File modified (tracked) since baseline | Conflict (Section 4.3) — never merged blind |
| File untracked, newly created since baseline | Not part of the baseline comparison; left alone; merge proceeds on the files it actually owns |
| File deleted since baseline | Conflict — a deletion is a change like any other |
| File renamed since baseline | Treated as delete + create for baseline-comparison purposes unless rename-detection explicitly confirms it's the same logical file with no content change, in which case treat as unmodified |
| File matches `.gitignore` | Excluded from baseline comparison and from the merge entirely — the Conductor never manages ignored files |
| Merge would produce a Git conflict marker state | Halt before writing; surface as Conflict, never write conflict markers into the person's files silently |
| Submodule present | Out of scope for automatic merge in the current phase — treated as an unsupported repository state (see below), surfaced explicitly rather than silently mismanaged |
| Git LFS-tracked file | Treated as an opaque blob for diffing purposes; hash-based baseline comparison still applies, content-diff display does not |
| Detached HEAD | Unsupported repository state (see below) |
| Branch deleted or changed externally mid-mission | Detected at the next baseline re-check; surfaced to the person rather than assumed |
| Repository itself missing or not a Git repo (e.g. deleted externally) | Fails closed — Mission moves to `Blocked` (Section 4.2.1), never proceeds as if nothing changed |

**Supported repository state, defined explicitly rather than left to be discovered during implementation:** a single local Git repository, on a named branch (not detached HEAD), with a clean or normally-dirty working tree, no in-progress rebase/merge/cherry-pick, and no unresolved conflict markers already present when a Mission starts. Anything outside this — mid-rebase, mid-merge, submodules the Mission needs to touch, detached HEAD — causes the Mission to refuse to start (or, if discovered mid-mission, to move to `Blocked`) with a specific, human-readable reason, rather than attempting best-effort handling of a state nobody designed for.

**Post-merge verification and checkpoint ordering, made explicit (new in v2.4):** the "verify the resulting tree" step above is not optional and not reorderable — it happens *after* the write and *before* the checkpoint is recorded, so a checkpoint is never written against a merge whose actual result wasn't itself confirmed. If post-merge verification fails (the resulting tree doesn't match what the merge should have produced), that is treated as its own `FailedInfrastructure` case (Section 9) — the merge is not retried blindly, and no checkpoint is written for it.

---

## 6. The Reconciliation Engine (Expanded)

```
Mission state (what we think happened)
        +
Baseline diff-state (what existed before the attempt)
        +
Actual diff (what genuinely changed, region by region)
        +
Git log
        +
Execution Harness's last output (advisory only — never trusted alone)
        +
OS process state
        ↓
  RECONCILIATION
        ↓
ReconciliationOutcome (Section 4.3)
```

**Algorithm, updated for diff-awareness:**
1. On app start, or whenever an Attempt is `Unknown`, list all non-terminal attempts.
2. Re-diff the attempt's workspace (Section 5) against its recorded baseline.
3. If the diff is empty → `NoChange` → safe, clean retry.
4. If the diff exists and matches the `expected_operation` → `ExpectedChange` → **handed to the Verification Engine (Section 7), not marked `Succeeded` here.** Reconciliation's job ends at producing the outcome; acceptance is a separate decision made against real evidence.
5. If the diff exists but doesn't match what was requested → `UnexpectedChange` or `PartialChange` → flag for a **targeted repair request**, never a blind full redo.
6. If a changed region is attributable to the user (Section 4.6) and does not overlap the AI's intended region → `UserChangeDetected` → leave it alone entirely.
7. If a changed region overlaps **both** the AI's intended region and a user edit → `Conflict` → **halt**, surface the four-way choice from Section 4.3.

**Restart-safety (new):** reconciliation itself must be safe to interrupt — the exact scenario of "Conductor crashes while figuring out what happened" must not leave the system worse off than before reconciliation started. Concretely: each step above writes its own intermediate result as an event (Section 4.9) *before* moving to the next step, so if the process dies mid-reconciliation, restarting resumes from the last completed step rather than re-deriving everything from scratch or, worse, half-applying a conclusion.

This must be a real, independently callable, unit-tested function — never inline logic sprinkled into `run_phase`.

---

## 7. The Verification Engine (New — Elevated to Its Own Component)

v2.0 mentioned verification in passing. Any Execution Harness or model claiming "done" must never be treated as done. This deserves to be a first-class pipeline, not a scattered assumption:

```
AI proposes/executes
        ↓
Filesystem evidence (Section 4.4)
        ↓
Diff verification (does the actual diff match the expected operation?)
        ↓
Build (does the project still compile/run?)
        ↓
Tests (do existing tests still pass? do new ones, if any, pass?)
        ↓
Lint / typecheck
        ↓
Requirement verification (does this satisfy what was actually asked?)
        ↓
Mission Acceptance Contract check (Section 8)
        ↓
Checkpoint accepted — ONLY now
```

Each stage can short-circuit the rest: a failed build means there is no point running tests. Every stage's result becomes part of the Attempt's evidence record (Section 4.4), tagged with its structured Evidence provenance (Section 4.5), so a later audit can see exactly which check passed or failed, not just a final yes/no.

**"Requirement verification," precisely defined (new in v2.4 — was previously a single vague stage name):** the stage above labeled "does this satisfy what was actually asked" can silently become "the model says it does," which is exactly the failure mode this entire document exists to prevent. Every requirement in a Mission Acceptance Contract (Section 8) must be classified as one of three kinds, and each kind has a different authority to grant a PASS:

```rust
enum RequirementVerificationClass {
    MachineVerifiable,   // e.g. "route exists," "test passes," "file present," "API returns X" —
                          // verified by running the declared check; result is authoritative
    HumanVerifiable,      // e.g. "UI feels polished," "matches intended style" — requires an
                          // explicit person confirmation step; never auto-passed
    ModelAssisted,         // a model's assessment is used as one input (e.g. summarizing a diff
                            // for a human reviewer) but is always advisory evidence, never itself
                            // sufficient to satisfy the criterion (Principle 2, Invariant 7)
}
```

A criterion tagged `ModelAssisted` can *contribute* evidence to a Mission Acceptance Contract, but it can never, by itself, be the thing that makes `blocking: true` pass — it must resolve into either a `MachineVerifiable` check or an explicit `HumanVerifiable` confirmation before the Verification Engine treats it as satisfied. This closes the gap where a criterion looks rigorous ("verification: requirement_check") but is actually just asking a model whether it thinks it succeeded.

**Hard enforcement (Invariant 14):** this pipeline is the *only* legal path to `AttemptStatus::Succeeded`. No executor, no provider response, no UI button, and no model output may set that status directly. In code terms: the field is not publicly settable at all — only a function owned by the Verification Engine can perform that specific transition, after every required stage above has genuinely passed.

### 7.1 The Verification Execution Sandbox (New — Closes a Real Security Gap)

Acceptance criteria run real commands (`npm test -- auth.test.ts`, and whatever similar commands future criteria specify). This is a genuine execution path into the person's machine, and the blueprint had strong capability enforcement everywhere *except* here. A malformed or malicious acceptance contract could in principle request `powershell`, `rm`, `curl`, or anything else a shell can run — an unguarded verification step would quietly become a new execution escape hatch, undermining every other safeguard in this document.

**Every verification command runs inside a sandbox with:**
- a **command allowlist** — only recognized test/build/lint runners for the project's detected stack, nothing arbitrary
- **working-directory restriction** — confined to the attempt's isolated worktree (Section 5), never the live workspace, never outside the project entirely
- a **timeout**, so a hung verification command can't stall a mission indefinitely
- **resource limits** where practically enforceable (CPU/memory)
- **network permission** — off by default; a criterion must explicitly declare if it genuinely needs network access, and that declaration is itself auditable
- **filesystem permission** — read/write scoped to exactly the worktree, nothing else
- **environment-variable filtering** — the sandbox process never inherits the app's full environment, specifically excluding anything the Credential Vault (Section 12) manages
- **no credential access** — structurally impossible, not merely "not passed in this case"
- **mutation detection** — if a "verification" command somehow modifies files outside what it was supposed to (i.e., it wasn't read-only when it claimed to be), that's itself logged as suspicious and blocks acceptance rather than being silently allowed

This makes the Verification Engine trustworthy in both directions: it neither accepts fabricated success (the original Section 7 goal) nor becomes a new way for something to reach further into the system than intended.

**Concrete OS-level containment model (new in v2.4 — the allowlist alone is not a sandbox):** a command allowlist restricts which *top-level* command can be launched (`npm test`, `cargo test`, `pytest`, etc.), but any of those can spawn arbitrary descendant processes through the project's own tooling — `npm test` running a `package.json` script that shells out, a test runner invoking a build tool that invokes a compiler plugin, and so on. An allowlist of command *names* is not equivalent to a secure sandbox unless the containment boundary applies to the whole process tree, not just the first process:

```
Verification Sandbox (process-tree scoped, not single-process scoped)
  ├── launched via OS-level process isolation appropriate to the platform
  │     (job objects + restricted token on Windows; namespaces/cgroups or an
  │     equivalent sandboxing primitive on Unix-likes) — not a bare child_process spawn
  ├── working directory: hard-bound to the attempt's isolated worktree (Section 5)
  ├── filesystem: read/write scoped to that worktree only, enforced at the OS
  │     permission/namespace level, not merely "the command shouldn't need more"
  ├── environment: an explicit allow-listed subset passed in; everything else,
  │     including anything Section 12's Credential Vault manages, is absent —
  │     not filtered after inheriting, never inherited at all
  ├── network: off by default at the OS/process level (not just "we didn't tell
  │     it a proxy") — a criterion must explicitly declare network need, which
  │     is itself audited
  ├── timeout and resource limits: enforced against the whole process tree total,
  │     not just the top-level process, so a descendant can't outlive or out-consume
  │     the parent's budget
  └── all descendant processes inherit every restriction above — the boundary is
        the process tree, not the process
```

The concrete requirement for implementation: verification commands must be launched through whatever OS-native containment primitive actually constrains a full process tree (not `std::process::Command` alone), and that containment must be validated by a dedicated test that deliberately tries to escape it — e.g. a `FakeProvider`-style verification command that attempts to spawn a shell, read an environment variable that should be absent, or write outside the worktree — and asserts the attempt is blocked and logged, not merely "usually doesn't happen to occur."

---

## 8. Mission Acceptance Contracts (Expanded)

Every mission gets machine-readable, checkable success criteria — decided **before** execution starts, not inferred afterward from whether the model sounds confident.

**Hard rule (new):** a criterion is not "Authentication works" as free text. Every criterion must declare *how* it is verified and *what evidence provenance/freshness* (Section 4.5) it requires — otherwise it's just a nicer-looking version of trusting the model's word:

```json
{
  "id": "auth-03",
  "description": "Invalid credentials are rejected",
  "verification": {
    "type": "test_command",
    "command": "npm test -- auth.test.ts",
    "expected_result": "exit_code_0"
  },
  "required_evidence": {
    "provenance": "local_test_runner",
    "max_freshness_seconds": 300,
    "execution_id": "current_attempt"
  },
  "blocking": true
}
```

A full contract is a list of these, not prose:

```
MISSION: Add authentication
  auth-01  Login page renders           → verification: dom_check         → provenance: filesystem_observation
  auth-02  Valid credentials succeed     → verification: test_command      → provenance: local_test_runner
  auth-03  Invalid credentials rejected  → verification: test_command      → provenance: local_test_runner
  auth-04  Session persists on reload    → verification: integration_test  → provenance: local_test_runner
  auth-05  Logout clears session         → verification: test_command      → provenance: local_test_runner
  auth-06  No pre-existing tests broke   → verification: test_suite_diff   → provenance: local_test_runner
  auth-07  Only expected files changed   → verification: diff_scope_check  → provenance: filesystem_observation
```

```
Model says "completed"
        ↓
Verification Engine (Section 7) runs
        ↓
Each criterion's declared verification method actually executes, right now
        ↓
Result's Evidence record (Section 4.5) compared against the criterion's required_evidence
        ↓
PASS  → Attempt = Succeeded, Checkpoint accepted   (Verification Engine only — Invariant 14)
FAIL  → specific unmet criteria become the next repair request — never a vague "try again"
```

This is the concrete mechanism that makes Principle 2 ("the model is advisory, not authoritative") enforceable rather than aspirational — and it specifically closes the gap between "the AI says it passed," "a stale summary from an old run says it passed," and "a test actually just ran and passed," which look identical in a transcript but must never be treated as equivalent evidence.

---

## 9. Recovery & Handoff Protocol (Expanded)

```
Failed              → new Attempt, different (or recovered) combo, with the expanded
                       handoff package below.
Unknown             → NEVER auto-retry. Reconciliation runs first, always.
PartialChange        → targeted repair request describing exactly the gap, not a full redo.
Conflict             → mission PAUSED. No automatic action. Person chooses.
```

### Structured Failure Classification (New)

`Failed` alone is too coarse a bucket for recovery decisions — "the build failed with zero file changes" and "the build failed after seven files were partially modified" call for completely different next steps. Rather than inventing more lifecycle states (which would re-create the exact `AttemptStatus`/`ReconciliationOutcome` conflation problem fixed in v2.1), attach structured **failure metadata** to every `Failed` attempt:

```rust
enum FailureCategory {
    FailedClean,          // failed before any change was made — safe, simple retry
    FailedWithChanges,     // failed after partial changes landed — needs targeted repair
    FailedVerification,     // changes landed but didn't pass the Acceptance Contract
    FailedInfrastructure,   // the failure was ours (process/network/persistence), not the model's
}
```

This reuses the Failure Taxonomy classes from Section 11 as the underlying detail, while `FailureCategory` is the coarser bucket the Recovery Protocol actually branches on.

### The Expanded Handoff Package (fixes v2.0's thin version)

```json
{
  "objective": "add authentication",
  "requirements": ["email/password login", "session persistence"],
  "acceptance_criteria": ["see Mission Acceptance Contract"],
  "completed_steps": ["plan", "settings-ui"],
  "completed_files": ["Settings.tsx", "settingsStore.ts"],
  "modified_files": ["App.tsx"],
  "current_operation": "wire the settings route into App.tsx",
  "remaining_work": ["API client", "tests"],
  "failed_attempt": "attempt-4821",
  "failure_reason": "disconnected mid-response; reconciliation found ExpectedChange partially applied",
  "known_constraints": ["do not touch payment code"],
  "expected_changes": ["App.tsx: add /settings route"],
  "actual_changes": ["App.tsx: import added, route registration missing"],
  "verification_status": "build passes, route not yet reachable",
  "last_checkpoint": "commit 6a41517",
  "do_not_repeat": ["Settings.tsx", "settingsStore.ts"],
  "tests_already_run": ["unit:settingsStore"],
  "tests_failed": [],
  "provider_context": {"previous_provider": "gemini", "previous_combo": "Gemini Power"}
}
```

This is machine-readable, structured state — dramatically cheaper in tokens and dramatically more reliable than "hope the next model remembers from the conversation."

---

## 10. Provider Abstraction, Fake Provider & Capability Registry (Expanded)

### 10.1 Provider Adapter (Thickened — Was Too Thin)

The v2.2 interface was correct in shape but too minimal for what this system actually needs to reason about — especially given how much of AI Conductor's value depends on OmniRoute/provider fallback behavior. `send()` staying a single simple call is fine; what needs expanding is what flows through `ProviderRequest`/`ProviderResult`:

```rust
trait ProviderAdapter {
    async fn send(&self, request: ProviderRequest) -> ProviderResult;
}

struct ProviderRequest {
    request_id: String,        // unique per call, for idempotency (Section 20)
    attempt_id: String,        // ties this call back to the owning Attempt
    idempotency_key: String,   // lets a retry safely detect "already sent this exact request"
    provider: String,
    model: String,
    streaming: bool,
    tool_calls_allowed: bool,
    timeout: Duration,
    capability_snapshot: CapabilitySnapshot, // what we believed the provider could do
                                               // at request time (Section 10.3), so a later
                                               // mismatch is detectable, not just assumed
}

struct ProviderResult {
    provider_response_id: Option<String>,  // the provider's own ID for this call, if any
    usage: Option<UsageInfo>,               // tokens/cost, for the Mission Ledger (Section 18)
    quota_info: Option<QuotaInfo>,           // remaining quota, if the provider reports it
    rate_limit_info: Option<RateLimitInfo>,   // including retry_after, when present
    raw_metadata: RawMetadata,                 // full response metadata, kept separate from...
    sanitized_metadata: SanitizedMetadata,      // ...a redacted version safe for logs/diagnostics
                                                  // (never mixed — see Credential Vault, Section 12)
}
```

Not every field is populated by every provider — OmniRoute's own combos won't always surface all of this — but the contract exists so the reliability core can consume whatever *is* available in a structured way, rather than each new integration inventing its own ad hoc shape.

### 10.2 Fake Provider — Now With a Deterministic Virtual Clock

```rust
enum FakeBehavior {
    Success, Timeout, RateLimited, MalformedOutput, PartialOutput,
    Disconnect, SlowResponse(Duration), Crash, DuplicateResponse, InvalidToolRequest,
}
```

**The addition from review that matters most for test speed:** don't make tests actually wait 15, 30, or 120 real seconds to exercise cooldowns, quota resets, retry windows, and backoff. Give the test environment a **virtual clock** — an injectable time source that the reliability core reads instead of the real system clock. Advancing it instantly lets you test "quota resets after 60 seconds" in milliseconds of real wall-clock time.

**Hard architecture rule (new):** no file in the reliability core may call the real system clock directly (`SystemTime::now()` or equivalent) for any time-dependent logic. All such reads go through an injectable `Clock` trait, with a `RealClock` implementation in production and a `VirtualClock` in tests. This isn't a style preference — a single stray direct clock call anywhere in cooldown, quota, retry, or backoff logic silently reintroduces slow, flaky, real-time-dependent tests exactly where they're most expensive to have.

### 10.3 Provider / Model Capability Registry (Expanded, With Staleness Policy)

The best fallback isn't simply "the next provider in the list" — it's **the next provider actually capable of continuing this specific attempt**. And critically, capabilities are not permanent facts — a provider can silently change its model, free-tier terms, or routing behind the scenes, so the registry must record *when* and *how confidently* each capability was last confirmed, and — new in v2.3 — **when that confirmation stops being trustworthy**:

```json
{
  "provider": "mistral", "model": "codestral-2508",
  "context_window": 256000,
  "max_output_tokens": 8192,
  "streaming": true,
  "tool_calling": true,
  "structured_output": false,
  "vision": false,
  "supports_handoff": true,
  "rate_limits": {"rpm": 5, "rpd": 2000},
  "free_tier_status": "active",
  "typical_latency_ms": 1800,
  "capability_verified_at": 1755000000,
  "capability_source": "provider_docs_v2508",
  "capability_confidence": "high",
  "provider_version": "codestral-2508",
  "verification_method": "live_probe",
  "verification_version": "probe-v3",
  "expires_at": 1755600000,
  "max_age_seconds": 604800,
  "actual_observation": "tool_call succeeded in probe request #4821"
}
```

`supports_handoff` in particular matters: if an attempt needs tool-calling and the next candidate combo's model doesn't support it, that combo is not a valid fallback for *this* attempt, regardless of how healthy it otherwise is.

**Formal staleness rule (new):** a capability record past its `expires_at` is not silently trusted as-is — the router treats it as `capability_confidence: unknown` until re-verified, rather than continuing to route on an assumption nobody has checked recently. `tool_calling = true, verified 23 days ago via provider docs` and `tool_calling = true, verified 30 seconds ago via a live probe` are explicitly different confidence levels, not interchangeable facts — `verification_method` and `verification_version` make that distinction machine-readable instead of implicit.

### 10.4 Chaos Mode — With Reproducible Seeds (Expanded)

```
CHAOS MODE
☑ Random provider failure       [ 10% ]
☑ Random network timeout        [ 10% ]
☑ Process termination           [  5% ]
☑ Delayed response               [ 15% ]
☑ Malformed model output         [  5% ]
☑ Checkpoint interruption        [  5% ]
☑ State persistence interruption [  5% ]
☑ Simulated user edit mid-attempt [ 5% ]   ← exercises Conflict detection

Seed:            [ 893442 ]
Scenario ID:     [ chaos-4812 ]
Virtual time seed: [ 0 ]

[ Start Reliability Test ]
```

**Hard rule (new):** every chaos run records its `chaos_seed`, `scenario_id`, and `virtual_time_seed`. "It failed once, randomly" is not an actionable bug report; "replay chaos scenario 4812 with seed 893442" is. Random chaos without a recorded seed is chaos you cannot debug.

### 10.5 External Side-Effect Idempotency (New in v2.4 — Closes Invariant 16's Gap)

`ProviderRequest.idempotency_key` (Section 10.1) is necessary but not sufficient. Generating a key is a *local* signal of intent — it says nothing about whether the *provider* actually treats repeated requests bearing that key as a no-op. Assuming otherwise is exactly the gap Invariant 16 exists to close: a dropped connection after the provider already processed a request, followed by a naive retry, can execute the same side effect twice even though the Conductor's own bookkeeping looks perfectly idempotent.

**Formal rule:** every provider/capability in the Capability Registry (Section 10.3) carries an explicit idempotency classification, and retry behavior branches on it — never assumed:

```rust
enum IdempotencySupport {
    ServerSideGuaranteed,   // provider documents/confirms it deduplicates by idempotency key —
                             // safe to retry automatically
    Unknown,                 // provider behavior on retry-with-same-key is not confirmed —
                              // treat as NOT safe to blind-retry
    NotIdempotent,             // provider explicitly does not deduplicate — a retry after an
                                // ambiguous outcome is a genuinely new side effect
}
```

```
Request outcome ambiguous (timeout, disconnect, no confirmed response)
        ↓
IdempotencySupport::ServerSideGuaranteed → safe to retry with the same idempotency_key
IdempotencySupport::Unknown / NotIdempotent → do NOT blind-retry:
        ↓
    Can the provider be queried for the original request's actual outcome
    (e.g. "did request abc-123 complete")?
        Yes → query, reconcile against that ground truth, proceed from the real state
        No  → surface as Unknown (Section 4.3) to the person — an explicit
              "I'm not sure whether that request went through" beats a silent
              possible double-execution every time
```

`IdempotencySupport` defaults to `Unknown` for any provider/combo not explicitly verified otherwise — the safe default is "don't assume," matching the same philosophy as the staleness policy in Section 10.3.

### 10.6 Provider Health Ledger (New in v2.4 — Was Referenced, Never Specified)

`provider_health.rs` appears in the module layout (Section 27) and Section 26.1 mentions confidence-decayed scoring, but v2.3 never gave Provider Health its own formal model — a real gap, since routing decisions (Section 28.8, 28.9) depend on it directly.

```rust
struct HealthObservation {
    provider: String,
    combo: String,
    timestamp: Timestamp,          // via the injectable Clock (Section 10.2), never SystemTime::now()
    outcome: HealthOutcomeKind,    // Success | Failure(FailureCategory) | RateLimited | Timeout
}

struct HealthScore {
    provider: String,
    combo: String,
    score: f64,                     // 0.0–1.0, confidence-decayed per Section 26.1's formula
    sample_count: u32,
    minimum_sample_size: u32,        // below this, score is reported alongside `confidence: low`,
                                       // never treated as equal to a well-sampled score
    last_success: Option<Timestamp>,
    last_failure: Option<Timestamp>,
    cooldown_state: CooldownState,     // None | Cooling(until: Timestamp) | Exhausted
    quota_state: QuotaState,            // remaining, resets_at — sourced from ProviderResult.quota_info
    rate_limit_state: RateLimitState,    // sourced from ProviderResult.rate_limit_info
    stale: bool,                          // true if no observation within a configured freshness window
}
```

**Scoring rule (formalizing Section 26.1):** `score = clamp01(base_success_rate - Σ(failure_weight_i))`, where each failure's weight decays exponentially with age: `failure_weight = 1.0 × 0.5^(hours_since_failure / half_life_hours)`. `half_life_hours` is a tunable, not hardcoded — logged alongside the score (see Policy Versioning, Section 28.9) so a scoring-formula change is itself auditable.

**Recovery and cooldown:** a combo enters `Cooling(until)` after a rate-limit or quota signal, computed from the provider's own `retry_after` when available, or a conservative default backoff when not. It never requires a manual reset to leave `Cooling` — cooldown expiry alone returns it to normal scoring, consistent with Section 26.1's "heals gradually" goal, but stays out of the routing candidate pool entirely while `Exhausted` (hard quota exhaustion until the provider's own reported reset time).

**Minimum sample size and staleness both explicitly guard against overconfidence:** a combo with two data points and a combo with two hundred should never be presented as equally trustworthy just because both compute to the same numeric score, and a score with no observation in the last N hours is flagged `stale: true` rather than silently trusted as current — mirroring the Capability Registry's staleness policy (Section 10.3) so the two systems don't diverge in philosophy.

---

## 10.7 Execution Harness Abstraction & Execution Adapter Contract (New in v2.5)

The Conductor must not be architecturally coupled to one coding agent. Aider was the original reference executor in earlier revisions; Jcode is now an additional serious candidate. The correct architectural response is **not** to choose one by assumption. It is to define the boundary every executor must satisfy and evaluate implementations against that boundary.

### 10.7.1 Executor Role

An Execution Harness is a component that can:

- receive an authorized execution request,
- inspect an assigned workspace,
- invoke model/provider resources,
- use permitted tools,
- create code or configuration changes,
- run permitted commands,
- report observations/results,
- terminate or complete an execution.

The harness is **not** allowed to decide that an Attempt is `Succeeded`, that an Acceptance Contract passed, that a merge is safe, that a Conflict can be ignored, that an Unknown outcome may be retried blindly, that a capability restriction may be bypassed, or that a Conductor policy can be replaced by its own policy.

Those decisions remain Conductor authority.

### 10.7.2 Execution Adapter Contract

Every supported harness is wrapped by one common semantic contract:

```rust
trait ExecutionAdapter {
    fn identify(&self) -> ExecutorIdentity;
    fn capabilities(&self) -> ExecutorCapabilitySnapshot;
    fn start(&self, request: ExecutionRequest) -> ExecutionHandle;
    fn observe(&self, handle: &ExecutionHandle) -> ExecutionObservation;
    fn cancel(&self, handle: &ExecutionHandle) -> CancellationObservation;
    fn collect_evidence(&self, handle: &ExecutionHandle) -> ExecutionEvidence;
    fn finalize(&self, handle: &ExecutionHandle) -> ExecutionFinalization;
}
```

The exact implementation may differ, but the semantic contract must not.

Every execution request carries, at minimum:

```text
mission_id
step_id
attempt_id
provider/model/combo identity where applicable
workspace identity
baseline identity
allowed capabilities
expected operation
timeout/budget policy
policy_version
correlation IDs
```

Every adapter must return enough observable information to distinguish:

```text
started | running | completed | failed | cancelled | timed_out | crashed | unknown
```

The adapter must never collapse an ambiguous process/provider outcome into `completed-successfully`.

### 10.7.3 Hard Boundary

The only legal path from Conductor policy to an executor is:

```text
Command
  ↓
Capability / Policy Check
  ↓
Execution Adapter
  ↓
Selected Harness
  ↓
Observed Evidence
  ↓
Verification Engine
```

No UI, model response, skill, provider callback, or harness-internal coordinator may bypass this path.

### 10.7.4 Jcode's Architectural Position

**Jcode is an optional Execution Harness candidate. It is not part of the Conductor kernel.**

Current upstream Jcode documentation describes a terminal coding-agent harness with non-interactive execution, session resume, provider integrations, MCP/tooling, persistent server/client operation, and a swarm/task-DAG system. Its swarm documentation also describes optional worktrees, lifecycle persistence, and optimistic conflict handling. These are useful executor capabilities, but they do not replace the Conductor's State Engine, Verification Engine, event chain, two-phase merge authority, or reconciliation model.

These external facts must be versioned and recorded in the external-components knowledge base before they become implementation assumptions.

Therefore:

```text
Jcode
  = potentially excellent execution surface
  ≠ Conductor reliability kernel
  ≠ Conductor verification authority
  ≠ Conductor merge authority
  ≠ Conductor persistent source of truth
```

Jcode may use its own swarm, memory, session, provider, or planning features only as delegated execution capabilities. The Conductor must remain able to disable, constrain, observe, or replace those capabilities.

### 10.7.5 Jcode-Specific Evaluation Requirements

Before Jcode is promoted from candidate to supported executor, prove—not assume—the following against the actual installed/version-pinned implementation:

```text
[ ] non-interactive execution can be invoked reliably
[ ] working directory can be constrained to the authorized attempt workspace
[ ] all file mutations remain inside the authorized boundary
[ ] subprocess descendants can be observed/terminated where required
[ ] stdout/stderr and exit information can be captured
[ ] ambiguous termination can be classified as Unknown
[ ] cancellation can be reconciled
[ ] timeout can be distinguished from successful completion
[ ] crash/restart behavior is observable
[ ] session resume does not bypass Conductor reconciliation
[ ] internal swarm cannot bypass Conductor merge/verification authority
[ ] internal worktree behavior does not replace Conductor isolation guarantees
[ ] provider selection does not bypass Conductor capability/policy decisions
[ ] credentials do not enter Conductor event/diagnostic records
[ ] tool execution remains subject to the Conductor capability boundary
[ ] deterministic adapter tests pass
[ ] real-world failure injection passes
```

A documented capability that has not been independently verified is `UNVERIFIED`, not `PASS`.

### 10.7.6 Common Executor Conformance Tests

Every adapter, including Aider and Jcode, must pass the same conformance suite:

```text
EC-01 identity/capability discovery
EC-02 authorized workspace execution
EC-03 forbidden-path rejection
EC-04 successful execution evidence
EC-05 partial execution evidence
EC-06 non-zero exit
EC-07 timeout
EC-08 cancellation
EC-09 crash / abrupt termination
EC-10 ambiguous outcome → Unknown
EC-11 restart/reconciliation
EC-12 provider failure propagation
EC-13 credential-redaction boundary
EC-14 capability denial at tool boundary
EC-15 no direct Succeeded transition
EC-16 no direct merge authority
```

This makes executor replacement a measured engineering decision rather than an architectural rewrite.

### 10.7.7 Executor Selection

The Decision Engine may select an executor based on:

```text
executor eligibility
× executor capability fit
× provider/model compatibility
× workspace compatibility
× observed reliability
× resource/budget cost
× recovery cost
× policy constraints
```

The score must be explainable and recorded.

A new executor may be added without changing the State Engine or Verification Engine.


## 11. Failure Taxonomy & Regression Corpus

| Class | Examples |
|---|---|
| PROCESS | app restart, Execution Harness crash, Conductor crash |
| NETWORK | timeout, disconnect, delayed response |
| PROVIDER | 429, quota exhausted, 5xx, malformed response |
| EXECUTION | partial file edit, tool failure, unexpected exit |
| PERSISTENCE | checkpoint write failure, state write interrupted |
| RECOVERY | retry after partial success, handoff after partial success, resume after restart |
| **CONFLICT** *(new)* | user edit overlapping an in-flight attempt, simultaneous external Git operation |

Every real bug becomes a permanent numbered entry in `failure_cases/`, each with its own regression test that runs forever.

---

## 12. Credential Vault & Security Architecture (New — Was Entirely Missing)

This was a genuine gap. The system holds Gemini keys, Mistral keys, and an OmniRoute API key — treating that as an afterthought is not acceptable for something meant to be trusted with a person's real project.

**Requirements:**
- Encrypted at rest (Windows Credential Manager / DPAPI, not a plaintext JSON field)
- **Never** appear in: event logs, the mission ledger, crash reports, diagnostic bundles, or model prompts
- Masked in the UI (`Gemini key ****A82F`, never the real value)
- Safe export/import that explicitly excludes secrets unless the person deliberately opts in
- Rotation supported without losing combo/health history tied to the old key

**Enforcement point:** every logging/serialization path that could touch a struct containing a credential must go through a redaction step — ideally enforced at the type level (a `Secret<String>` wrapper whose `Debug`/`Serialize` implementations always redact, so it is structurally difficult to accidentally leak one into a log line).

---

## 13. Observability & Diagnostics (Expanded)

The Developer Reliability Console (Section 15) already gives live visibility. Add:

**Diagnostic Bundle Export** — a single button that produces a zip for debugging. **Hard rule (new): this is an allowlist, not a denylist.** Only fields explicitly approved to leave the app ever enter the bundle — mission state, the event log, failure classification, provider metadata (never raw keys — see Section 12), attempt timeline, reconciliation results, Git metadata, test results, and version info, each individually whitelisted. The alternative ("include everything except known secrets") is fragile: a future developer adding a new field to some struct can accidentally leak it into every diagnostic bundle from that point on, with nothing forcing them to think about it. An allowlist requires a deliberate decision to include anything new, which is the safer default direction for a system that explicitly promises never to leak credentials or private source.

---

## 14. Cancellation Semantics (Precisely Defined)

"The user clicked Stop" must never simply mean `→ Cancelled`, because The selected Execution Harness might still be mid-write to disk at that exact moment.

```
Cancel requested
      ↓
Stop accepting new actions for this attempt
      ↓
Attempt graceful process termination (SIGTERM-equivalent)
      ↓
Wait, with a bounded timeout
      ↓
Force-kill if it hasn't exited
      ↓
Run Reconciliation on the (isolated, per Section 5) attempt workspace
      ↓
Determine final CancellationOutcome
```

A cancellation that skips reconciliation can silently leave a half-written file mistaken for "nothing happened."

**Correction (v2.3):** `Cancelled-clean` and `Cancelled-with-partial-change` are not values of `AttemptStatus` (Section 4.2) — putting them there would recreate the exact category error that `ReconciliationOutcome` was split out to fix. `AttemptStatus` gets exactly one terminal value, `Cancelled`; the detail lives in its own enum, mirroring the existing state/outcome separation pattern:

```rust
enum CancellationOutcome {
    CancelledClean,        // nothing landed; a clean retry later is safe
    CancelledWithChanges,   // some partial change landed; needs the same repair
                             // path as PartialChange, not a blind clean retry
    CancellationUnknown,     // reconciliation itself couldn't determine the result —
                             // surfaced to the person, never guessed
}
```

**Auditability:** every cancellation carries its own `request_id` and `requested_at` timestamp, and each stage of the sequence above is logged as its own event against that ID — `cancel_requested`, `cancel_acknowledged`, `process_exited`, `reconciliation_completed`, `final_cancellation_outcome_determined`. This makes "what actually happened when I clicked Stop" fully reconstructable after the fact, not just "eventually it stopped."

---

## 15. Crash Recovery / Startup Watchdog (New)

The entire value proposition is "you don't have to personally track what happened" — which means the app must recover gracefully from its own crashes, not just the AI's.

```
App launch
     ↓
Load persisted mission/attempt state
     ↓
Find any Attempts still marked Running
     ↓
Check: is the OS process that was running it still alive?
     ↓
   No  → treat as Unknown, run Reconciliation immediately
   Yes → re-attach monitoring to it (rare, but possible after a UI-only crash)
     ↓
Resume, repair, or surface for a decision — never leave it silently stuck
```

---

## 16. Capability & Permission Model (Strengthened)

The direction from v2.0 was already right; this revision makes the enforcement point precise rather than general:

```
Skill: Engineering Review
  Capabilities: READ_PROJECT, RUN_SAFE_INSPECTION, WRITE_REVIEW
  (no WRITE_CODE — structurally cannot edit files)
```

**Hard rule (new):** enforcement does not live at the "skill level" as a soft check somewhere upstream — it lives at the **tool-execution boundary itself**:

```
ExecutionEngine.execute(tool_call)
        ↓
  capability_check(active_skill, tool_call.required_capability)
        ↓
   authorized?  → proceed
   denied?      → reject, log, surface — the call never reaches Aider or the provider
```

The distinction matters: "the model says it won't write files" is a promise the model can break; "the skill says no WRITE_CODE" checked somewhere vague in the pipeline can be accidentally bypassed by a new code path later. A check that every single tool call must pass through, with no alternate route to execution, makes it **structurally impossible** for a Review-phase skill to write code — regardless of what the model asks for, and regardless of what future code is added elsewhere in the system.

---

## 17. Concurrency Model

**One authoritative executor per mission.** A single long-lived task owns mission state; everything else sends messages into a channel it alone reads. No multi-agent parallelism until the single-mission case is provably solid (Phase 10+ at the earliest — pushed back further in the revised roadmap below, deliberately).

**New requirement:** every concurrency-sensitive operation (the health monitor updating state, a checkpoint write racing a cancellation, the UI reading state mid-mutation) must have a dedicated deterministic test that exercises the race directly — not just hope that normal testing happens to trigger it. Concurrency bugs are exactly the class of problem that "it worked when I tried it" cannot rule out.

---

## 18. Efficiency & Cost Intelligence

Unchanged in substance — still central to the actual goal:

```
MISSION LEDGER — "Add authentication"
Steps completed:      7/11
Resources used:       Gemini ×6, Mistral ×2
Failures:             Gemini 429 ×1, Timeout ×1, Conflict ×1
Recoveries:           2
Files changed:        12
Recovery overhead:    4.2%
```

---

## 19. Testing Strategy

**Level 1 — Deterministic unit tests.** State transitions, reconciliation outcome classification, checkpoint logic, handoff package generation. Property-based testing (`proptest`) generates hundreds of transition sequences automatically.

**Level 2 — Simulation tests.** `FakeProvider` with the full fault menu (Section 10.2), run against the **virtual clock**, so time-dependent scenarios (cooldowns, quota resets, backoff) run in milliseconds, not real minutes.

**Level 3 — Real-world integration tests.** Actual Gemini, Mistral, OmniRoute, Aider, Git — fewer, run before releases, catching the gap between "logic is correct" and "the real world matches our assumptions."

All three run permanently, not as a one-time milestone.

---

## 20. Engineering Tricks & Best Practices

- **Idempotency keys** on every mutating operation, so a retry can check "have I already applied this" before acting.
- **Atomic file writes** — temp file + rename, never a direct write to the target path.
- **Append-only JSON Lines event log** — a crash mid-write corrupts at most the last line, never the whole file.
- **Git commit trailers for traceability** (`AI-Conductor-Mission-Id: ...`) instead of parsing commit message text — more robust than the string-prefix approach the current codebase actually uses today.
- **The outbox pattern** for multi-system side effects (write intent → perform effect → mark complete), so a crash between steps is recoverable by replaying the pending intent.
- **Structured error enums, not strings** — directly relevant given a real bug already occurred in this project from string-based heuristics (the false-positive auto-file-chaining detection).
- **Golden-file/snapshot tests** for the Reconciliation Engine specifically, since its logic will keep evolving.
- **Declarative chaos scenario files** rather than hand-written test code per scenario.
- **Budget guardrails** — a hard per-mission ceiling on tokens/requests, pausing to ask rather than silently burning the day's quota.
- **`Secret<T>` wrapper type** for anything credential-shaped (Section 12), so redaction is structural, not a discipline someone has to remember.
- **Fail loud on invariant violation in debug builds** — panic immediately with context rather than limping along silently.

---

## 21. The Developer Reliability Console

```
MISSION #41
State:               RUNNING
Step:                BUILD / attempt #3
Attempt:             provider=gemini, status=UNKNOWN
Workspace:           isolated worktree #7
Checkpoint:          #17
Files:               3 expected · 2 modified · 1 uncertain
Reconciliation:      auth.ts — Conflict (user edit overlaps AI change)
Recovery:            PAUSED — awaiting person's decision
Acceptance contract: 4/7 criteria met
Events:              [live scrolling list]
```

Built plain and functional in Phase 1; stays until the reliability core is proven; only then replaced by the polished Mission Mode UI.

---

## 22. gstack Skill Contracts

Unchanged: define the input/output/permission contract early; defer the elaborate visual workflow stepper until the reliability substrate is proven.

---

## 23. Phased Roadmap (Reordered Per Review)

Each phase has an explicit Definition of Done. Do not start a phase until the previous one's checklist is fully checked.

### Phase 0 — Architecture Contracts (no code)
- [ ] AttemptStatus and ReconciliationOutcome defined as separate enums (Sections 4.2–4.3)
- [ ] Evidence provenance/freshness model defined (Section 4.5)
- [ ] Checkpoint semantics agreed (4.7)
- [ ] Tamper-evident event chain format agreed (4.9)
- [ ] Failure taxonomy classes AND structured FailureCategory listed, including Conflict (Sections 9, 11)
- [ ] Skill contract format defined (Section 22)
- [ ] Capability table drafted, with tool-execution-boundary enforcement point identified (Section 16)
- [ ] Mission Acceptance Contract format defined, including required verification method + required_evidence (provenance/freshness) per criterion (Section 8)
- [ ] Two-phase merge protocol agreed, including the Git edge-case contract (Invariant 13, Section 5 / 5.1; rationale in Section 26.2)
- [ ] Mission and Step state machines agreed, not just Attempt's (Section 4.2.1)
- [ ] External side-effect idempotency classification model agreed (Invariant 16, Section 10.5)
- [ ] Clock trait interface agreed — no direct system-clock calls permitted in the reliability core (Section 10.2)

### Phase 1 — State + Event + Persistence Core
- [ ] State Engine with enforced transition table
- [ ] `CancellationOutcome` implemented as its own enum from the start (Section 14) — do not let cancellation detail leak into `AttemptStatus`
- [ ] Append-only JSON Lines event log, hash-chained (Section 4.9)
- [ ] Event-chain anchoring, persisted separately, checked against on load (Section 4.9)
- [ ] Atomic file writes everywhere state is persisted
- [ ] Outbox/intent mechanism for any multi-record write (Invariant 15, Section 20)
- [ ] Idempotency-key store, so a request/attempt retry can detect "already applied" (Section 20)
- [ ] Monotonic sequence-number allocator for the event log (feeds `sequence_number` in Section 4.9)
- [ ] Correlation IDs threaded consistently across mission/step/attempt/event/provider-request/tool-call — added now, not bolted on later once dozens of call sites already lack them
- [ ] Schema versioning on every record
- [ ] Developer Reliability Console (Section 21)

*These are foundational specifically because retrofitting idempotency keys, correlation IDs, or a sequence allocator after dozens of call sites already exist without them is materially more expensive than building them in from the start — this is why they're pulled into Phase 1 rather than left implicit.*

### Phase 2 — Workspace Safety, Git Worktree Isolation, Reconciliation
- [ ] Every attempt runs in an isolated worktree (Section 5)
- [ ] Diff-aware Reconciliation Engine (Section 6), unit-tested
- [ ] Reconciliation Engine never sets `Succeeded` — only produces `ReconciliationOutcome`, verified by a test that asserts this directly
- [ ] Mutation attribution (Section 4.6)
- [ ] Conflict detection and the four-way person decision UI

### Phase 3 — Fake Provider + Deterministic Failure Simulation
- [ ] `FakeProvider` with full fault menu
- [ ] Virtual clock for instant time-dependent testing (Section 10.2)
- [ ] Chaos Mode engine, including simulated mid-attempt user edits

### Phase 4 — Verification Engine + Acceptance Contracts
- [ ] Full pipeline from Section 7 implemented and tested against `FakeProvider`
- [ ] Mission Acceptance Contracts enforced before any Attempt is marked Succeeded

### Phase 5 — Execution Harness Adapter + Real Executor Validation
- [ ] Execution Adapter Contract implemented as the only Conductor-to-harness boundary
- [ ] Common executor conformance suite implemented (Section 10.7.6)
- [ ] One reference executor validated end-to-end inside isolated worktrees
- [ ] Aider adapter validated if retained as a supported executor
- [ ] Jcode adapter evaluated against the same contract if the installed/version-pinned Jcode build is available
- [ ] Normal execution, interruption, restart, cancellation, ambiguous outcome, and evidence collection proven
- [ ] No executor can directly set `Succeeded` or perform a Conductor-authorized merge
- [ ] Executor-specific capabilities are represented as observations/claims with provenance and freshness
- [ ] Executor choice remains reversible: changing the executor does not require changing the reliability kernel

### Phase 6 — OmniRoute + One Real Provider (Gemini)
- [ ] Real quota/rate limits observed and correctly classified
- [ ] Real failure modes captured as new `failure_cases` entries

### Phase 7 — Resource Intelligence + Capability Registry (moved earlier per review)
- [ ] Provider/Model Capability Registry (Section 10.3) populated for Gemini and Mistral
- [ ] Provider Health Ledger implemented per Section 10.6, feeding cost/usage into the Mission Ledger (Section 18)
- [ ] Basic quota/cooldown-aware routing live

### Phase 8 — Multi-Provider Handoff
- [ ] Gemini → Mistral handoff using the expanded package (Section 9)
- [ ] Recovery overhead measured and logged

### Phase 9 — gstack Skill Runtime
- [ ] Office Hours, Engineering Review, Build, Review, QA, Debug as backend-level skills
- [ ] Capability enforcement verified with a test proving a Review-phase skill cannot write files

### Phase 10 — Full Mission Orchestration
- [ ] Mission → gstack workflow → resource scheduler → execution → verification → recovery, end to end
- [ ] Crash recovery/watchdog (Section 15) proven via a real forced-restart test

### Phase 11 — Polished Frontend
- [ ] Mission Mode UI, rendering real, tested state

*(Multi-agent parallelism remains explicitly out of scope for all listed phases — reconsidered only once Phase 10 is bulletproof.)*

---

## 24. Risk Register

| Risk | Mitigation |
|---|---|
| Self-referential confidence loop | External truth only: tests, evidence, chaos runs, independent review |
| Scope creep back to feature-first habits | Every new feature must map to a Section 3 invariant or be deferred |
| Silent conflict overwrite | Invariant 11 — Conflict always halts, never auto-resolves |
| Credential leakage into logs/diagnostics | `Secret<T>` wrapper type, redaction enforced structurally (Section 12) |
| Reliability core becomes untestable | Three-level test strategy enforced from Phase 1, not retrofitted |
| Forgetting *why* a safeguard exists | `failure_cases/` corpus is permanent memory — read before removing any safeguard |
| Premature parallelism | Explicitly blocked until Phase 10+ |
| Executor/harness lock-in | Execution Adapter Contract + common conformance suite; no harness owns Conductor authority |

---

## 25. Glossary

- **Mission / Step / Attempt** — the three-level hierarchy of a user request.
- **AttemptStatus** — an attempt's lifecycle state (Pending/Running/Succeeded/Failed/Cancelled/Unknown).
- **ReconciliationOutcome** — the result of comparing expected vs. actual evidence (NoChange/ExpectedChange/UnexpectedChange/PartialChange/UserChangeDetected/Conflict) — deliberately a separate concept from AttemptStatus.
- **Mutation Attribution** — determining which specific changed region of a file came from the AI versus the user.
- **Workspace Isolation** — running each attempt in its own disposable Git worktree so a failed/rejected attempt never touches the user's real files.
- **Verification Engine** — the pipeline (diff → build → tests → lint → requirements → acceptance) that must pass before an attempt is accepted.
- **Mission Acceptance Contract** — the machine-readable, pre-defined success criteria for a mission.
- **Provider Capability Registry** — structured data about what each provider/model can actually do, used to pick a *valid* fallback, not just the next one in line.
- **Credential Vault** — the encrypted-at-rest, redaction-enforced storage for API keys.
- **Handoff Package** — the compact, structured state passed to a new Attempt after a failure.

---

## 26. Additional Recommendations — Beyond Both Reviews

Four techniques neither review raised, worth building in from the start because retrofitting them later is much more expensive:

**26.1 — Confidence-Decayed Health Scoring.** The current health model (Section on Provider Health) treats "3 consecutive failures" as a hard cutoff, which never recovers on its own until a fresh success resets it. In practice, a combo that failed three times an hour ago is not the same risk as one that failed three times a minute ago. Fix: weight each failure by recency using simple exponential decay (`weight = failure_count × 0.5^(hours_since / half_life)`), so health scores heal gradually over time instead of staying "stuck red" until manually cleared or luckily retried back to green. This avoids two real failure modes: permanently avoiding a combo that's actually recovered, and thrashing back to a combo that failed moments ago.

**26.2 — Two-Phase Merge on Accepting a Worktree.** *(Promoted to Invariant 13 in v2.2 — kept here for the full reasoning.)* Section 5's worktree isolation solves "Aider never touches the live workspace directly," but there's a residual gap: *between* when an attempt started (baseline captured) and when its result is accepted, the person could have manually edited the real workspace themselves. Merging the worktree's result back in at that point risks silently clobbering that manual edit. Fix, mirroring how real merge tools stay safe: before merging, re-check the live workspace against the original baseline. If it's unchanged, merge freely. If it changed, treat it exactly like the Conflict path in Section 4.3 — halt and ask, rather than merging blind.

**26.3 — Time-Travel Replay in the Developer Console.** Because the event log (Section 20) is already append-only and ordered, add one small but disproportionately useful capability: a "replay" mode in the Developer Reliability Console that steps through a past mission's exact event sequence, one event at a time, showing the full state at each point. This turns "why did this go wrong three days ago" from an exercise in re-reading raw JSON into an actual debugging tool — and it's nearly free to build once the event log exists, since it's just re-applying the same events the state engine already knows how to process.

**26.4 — Shadow-Run New Reconciliation Logic Before Trusting It.** The Reconciliation Engine (Section 6) will keep evolving as new edge cases are discovered. Before trusting a change to it on live missions, run it in "shadow mode" against the recorded evidence from every past mission in the `failure_cases/` corpus, compare its new verdicts to the old ones, and require a human to review every case where the verdict changed before shipping the update. This directly operationalizes Principle 3 (external truth, not self-certification) for the one component most likely to silently regress in subtle ways.

## 27. Appendix — Proposed Module Layout (Updated for v2.5.1)

```
src-tauri/src/
  kernel/                        // Section 28.1 — pure Conductor Kernel: NO filesystem/network/
                                  // model calls/direct clock reads. reduce(state, event, evidence,
                                  // policy) -> (new_state, commands). This is where most invariant
                                  // logic should end up living.
    reducer.rs                   // state transition validator — Attempt/Mission/Step tables
    decisions.rs                 // recovery / capability / acceptance decisions (pure)
    failure_classification.rs
    resource_selection.rs
  decision_engine/                // Section 28.8 — the single canonical decision authority
    engine.rs                     // (state, evidence, capabilities, health, budget, policy) ->
                                   // { continue | retry | handoff | pause | recover | verify | ask_user }
  commands/                        // Section 28.7 — canonical Command model
    model.rs                       // StartMission, StartAttempt, CancelAttempt, RetryAttempt, etc.
  mission/
    state.rs               // AttemptStatus + transition table
    mission_step_state.rs    // MissionStatus/StepStatus + transition tables (Section 4.2.1)
    cancellation.rs           // CancellationOutcome, separate from AttemptStatus (Section 14)
    reconciliation_outcome.rs   // separate enum, per Section 4.3
    evidence.rs                // Evidence struct + provenance model (Section 4.5)
    events.rs                  // append-only JSON Lines event log, hash-chained (Section 4.9)
    event_anchor.rs              // truncation-detection anchoring (Section 4.9), scoped threat model
    checkpoint.rs                  // four-part checkpoint model
    reconciliation.rs                // diff-aware, restart-safe Reconciliation Engine (Section 6);
                                      // NEVER sets Succeeded directly — Invariant 14
    mutation_attribution.rs            // Section 4.6
    handoff.rs                          // expanded handoff package (Section 9), minimum-context
                                         // continuation (Section 28.10)
    acceptance.rs                         // Mission Acceptance Contracts, per-criterion
                                           // verification method + required_evidence (Section 8),
                                           // RequirementVerificationClass (Section 7)
    failure_category.rs                     // structured failure classification (Section 9)
    recovery_budget.rs                        // Section 28.13 — attempt/handoff/escalation ceilings,
                                                // monotonic-retry + semantic-fingerprint enforcement
    schema_migration.rs                         // Section 4.8.1
  verification/
    engine.rs             // Verification Engine — sole owner of the Succeeded transition (Section 7)
    sandbox.rs            // Verification Execution Sandbox, process-tree scoped (Section 7.1)
    preflight.rs           // Section 28.11 — gate before any expensive attempt/verification
  workspace/
    isolation.rs           // Git worktree management (Section 5)
    two_phase_merge.rs       // locked, expected-state-conditional merge (Invariant 13, Section 5.1)
    git_contract.rs           // supported-repository-state + edge-case table (Section 5.1)
  providers/
    adapter.rs             // thickened ProviderRequest/ProviderResult (Section 10.1)
    fake.rs               // FakeProvider (Section 10.2)
    capability_registry.rs  // versioned, staleness-aware capability data (Section 10.3)
    idempotency.rs            // IdempotencySupport classification (Invariant 16, Section 10.5)
    provider_health.rs          // Provider Health Ledger (Section 10.6)
    omniroute.rs
  time/
    clock.rs               // Clock trait: RealClock + VirtualClock (Section 10.2 hard rule)
  chaos/
    engine.rs              // seeded, reproducible chaos runs (Section 10.4)
    scenarios/
  scenario_dsl/                // Section 28.4 — declarative scenario format, parsed + executed
    parser.rs
    runner.rs
  simulator/                    // Section 28.5 — full virtual-world simulator
    virtual_provider.rs
    virtual_filesystem.rs
    virtual_git.rs
    virtual_process.rs
    virtual_user.rs
  fuzz/                          // Section 28.6 — event-sequence property/fuzz testing
    event_sequence.rs
  execution/
    contract.rs             // Execution Adapter Contract (Section 10.7)
    adapter.rs              // common adapter interface and lifecycle semantics
    capabilities.rs         // executor capability snapshot / provenance / freshness
    tool_boundary.rs        // capability_check() gate — the only path to any Execution Harness
    conformance.rs          // EC-01..EC-16 common executor contract tests
    aider.rs                // Aider adapter, if retained as a supported executor
    jcode.rs                // Jcode adapter, if promoted after conformance/evidence gates
    process_tree.rs         // executor/subprocess lifecycle containment and observation
  persistence/
    outbox.rs              // intent → effect → complete pattern (Invariant 15, Section 20)
    idempotency_store.rs     // local retry-detection store (distinct from provider-side
                              // IdempotencySupport in providers/idempotency.rs)
  security/
    vault.rs              // Credential Vault, Secret<T> wrapper (Section 12)
  diagnostics/
    bundle.rs              // allowlist-based Diagnostic Bundle export (Section 13)
  policy/
    version.rs               // Section 28.9 — policy_version stamped on every major decision
    budget_planner.rs          // Section 28.12 — request budget + resource reservation
    safe_mode.rs                 // Section 28.18 — global automation kill switches
  skills/
    contract.rs
    capabilities.rs
  task_router.rs               // starts as a transparent scoring formula (Section 28.8's router example)
  context_engine.rs             // context fingerprinting/caching (Section 28.10)
  failure_cases/                // permanent regression corpus

conductor-sim/                  // Section 28.16 — separate CLI crate, no desktop UI dependency:
  main.rs                       // `conductor-sim scenario <name>` runs the kernel + simulator
                                 // + scenario DSL directly, printing PASS/FAIL per invariant
```

---

## 28. Implementation Reliability & Acceleration Layer (New in v2.4)

Everything in Sections 3–24 defines *what must be true*. This section is different in kind: it doesn't add invariants and doesn't change the state model or the core pipeline (Section 5's isolation-and-merge flow, Section 7's verification pipeline). It defines *how to build Sections 3–24 so that they end up actually true* — with less code that has to be trusted, more failures caught automatically before they reach a person, and smaller, harder-to-derail units of work for AI coding agents. Nothing here is user-facing. That's deliberate.

**If reduced to three things, these are the three that matter most, and each attacks the problem from a different angle:**

1. **28.1 — Functional Core + deterministic Conductor Kernel.** Less code has to be trusted, because most of the hardest correctness logic becomes pure, exhaustively-testable functions with no I/O.
2. **28.5 — Full virtual-world simulator + 28.4 Scenario DSL.** Far more failure modes get tested automatically, in milliseconds, before a real provider or a real person is ever involved.
3. **28.21 — Contract-first, invariant-driven AI implementation protocol.** AI coding agents get smaller, unambiguous slices of work, which sharply reduces the chance they introduce a subtle architectural violation while implementing this document.

### 28.1 Functional Core, Imperative Shell — the Conductor Kernel

The biggest single addition. Split the system into two halves with a hard boundary between them:

```
┌──────────────────────────────────────────────┐
│             IMPURE OUTER SHELL                │
│  Git • Execution Harnesses • Filesystem •     │
│  OmniRoute • OS processes • UI • real Clock    │
└─────────────────────┬──────────────────────────┘
                       │  state + event + evidence + policy
                       ▼
┌──────────────────────────────────────────────┐
│              PURE CONDUCTOR KERNEL             │
│  State reducer (Attempt/Mission/Step tables)   │
│  Transition validator                          │
│  Recovery decisions                            │
│  Capability decisions                          │
│  Acceptance decisions                          │
│  Failure classification                        │
│  Resource-selection rules                       │
│                                                 │
│  NO filesystem · NO network · NO model calls ·  │
│  NO direct clock reads (Section 10.2's rule     │
│  already requires this — the Kernel is where    │
│  that rule structurally lives)                  │
└──────────────────────────────────────────────┘
        │
        ▼  new_state + commands (Section 28.7)
```

The Kernel is a pure function: `reduce(current_state, event, observed_evidence, policy) -> (new_state, commands)`. This is not a new invariant — every rule it enforces already exists in Sections 3–9 — it's a placement decision: put the logic that decides transitions, recovery, and acceptance in one place with no side effects, so it can be tested as "given this exact state and event, was this transition allowed?" instead of "did the whole app recover correctly?" The impure shell (Aider, Git, the filesystem, OmniRoute) becomes a set of adapters that only *execute* the Commands the Kernel emits — if Aider breaks, the state machine's correctness doesn't become uncertain, because the state machine never touched Aider directly. This also directly serves Section 28.21: an AI coding agent implementing one Kernel function has no filesystem, network, or timing nondeterminism to accidentally get wrong.

### 28.2 Formal State-Model Checking for the Critical State Machine

Not formal verification of the whole application — that's the wrong scope for a desktop tool. But `AttemptStatus`/`MissionStatus`/`StepStatus` (Sections 4.2, 4.2.1) and their safety properties are a small, bounded state space that's a genuinely good fit for a lightweight model checker (a small TLA+/PlusCal model, or an equivalent state-space explorer):

```
Properties to check:
  Succeeded cannot return to Running
  Unknown cannot directly become Succeeded          (Invariant 14)
  Conflict cannot auto-resolve                      (Invariant 11)
  Succeeded only follows a passed Verification pipeline
  Cancelled must not itself cause a mutation
```

```
formal model  →  Rust implementation  →  property tests (28.3)  →  simulation (28.5)
```

Three independent checks of the same logic, each catching a different class of mistake — a much better use of formal methods here than attempting to verify the entire desktop application, which would be disproportionate effort for the actual risk surface.

### 28.3 Invariant-Generated Test Families

Section 3's sixteen invariants should not live only as documentation and a handful of hand-written tests. Each one becomes a generated *family* of tests, so coverage is systematic rather than dependent on remembering to test each rule by hand:

```
Invariant 14 — "Only the Verification Engine may produce Succeeded" generates:
  provider_result       → cannot set Succeeded
  UI command            → cannot set Succeeded
  Execution Harness result           → cannot set Succeeded
  reconciliation          → cannot set Succeeded
  skill                    → cannot set Succeeded

Invariant 11 — "Conflict never auto-resolves" generates:
  conflict + retry
  conflict + provider failure
  conflict + restart
  conflict + cancellation
  conflict + timeout
```

This is materially stronger than manually maintained test files, because adding a new attempted-violation case to an invariant's family is a one-line addition rather than a new test file someone has to remember to write.

### 28.4 Scenario DSL

Writing every chaos/reconciliation test by hand doesn't scale. Define a declarative scenario format instead, and let one engine execute all of them:

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

One format → hundreds of scenarios → readable, reproducible, easy for an AI coding agent to both generate and verify against, and a permanent, growing regression corpus that reads like documentation. This directly operationalizes Section 20's "declarative chaos scenario files" recommendation, generalized to reconciliation and recovery scenarios too, not chaos alone.

### 28.5 Full Virtual-World Simulator

`FakeProvider` (Section 10.2) fakes the provider. Go further and fake the whole environment the Kernel's shell talks to, so entire multi-part failures run deterministically and fast:

```
VirtualProvider · VirtualFilesystem · VirtualGit · VirtualClock (already required, Section 10.2)
· VirtualProcess · VirtualUser
```

```
Example composite scenario, entirely deterministic:
  User edits App.tsx
  + Aider edits auth.ts
  + provider times out
  + checkpoint write crashes
  + clock advances 30s
  + process restarts
```

Thousands of small simulations can run in the time real integration tests would take for a handful — this is one of the biggest available reductions in both development time and blind spots, since it tests interaction failures (not just single-fault scenarios) before any real provider or real person is involved.

### 28.6 Event-Sequence Property Testing

Section 19 already calls for property-based testing of state transitions. Extend the same idea to *sequences* of events, not just individual transitions — fuzz the event stream itself:

```
AttemptStarted → ProviderSelected → ToolStarted → Timeout → CancelRequested →
ProcessExited → Restart → ReconcileStarted → UserEditedFile → ...
```

Generate large numbers of syntactically valid but semantically unusual orderings of these, and assert the one property that must always hold regardless of ordering: **`Succeeded` never appears unless the exact evidence path required by Invariant 14 occurred.** This is a strong, cheap way to catch a Kernel bug that only manifests under an event ordering nobody thought to write a scenario for by hand.

### 28.7 Canonical Command Model

Replace ad hoc functions (`run_phase()`, `retry()`, `continue()`, `recover()`) with explicit, named Commands the Kernel emits and the shell executes:

```
StartMission · StartAttempt · CancelAttempt · ReconcileAttempt · VerifyAttempt ·
RetryAttempt · HandoffAttempt · AcceptAttempt · PauseMission · ResumeMission ·
RestoreCheckpoint · MergeAttempt
```

```
Command → validate → execute → events (Section 4.9)
```

This is also what makes Dry-Run Mode (28.14) close to free: if the Kernel already communicates intent as explicit Commands rather than performing side effects inline, "compute the Commands but don't execute them" is a mode switch, not new architecture.

### 28.8 Single Decision Engine

Decisions currently risk being scattered across `task_router.rs`, `provider_health.rs`, `recovery.rs`, `run_phase()`, the UI, and the Aider bridge — a real source of contradictory logic if left implicit. Consolidate into one component with one job:

```
DecisionEngine(state, evidence, capabilities, health, budget, policy)
    -> { continue | retry | handoff | pause | recover | verify | ask_user }
```

**Router intelligence starts as a transparent formula, not a model.** Explicitly deferred: reinforcement learning, a learned router, bandits, neural scoring. Start with:

```
eligibility × executor_fit × capability_fit × health × quota_headroom × context_fit × reliability
  ÷ expected_recovery_cost
```

Log every component of every decision. When a decision looks wrong, the formula shows exactly why — something a learned scorer cannot give you early on, when you most need to trust or debug it. Learning-based routing is a legitimate *later* upgrade once the transparent version's blind spots are actually known from real data, not a Phase 1–10 concern.

**Recovery cost belongs in the routing formula, not just quality.** The question routing should actually answer is not "which provider is better" but "which is likely to finish this mission with the least total wasted quota" — a slightly-lower-quality, cheaper-to-recover-from combo can be the correct choice for a small task even when a stronger combo would be correct for a critical one. `expected_recovery_cost` above is where this lives.

### 28.9 Policy Versioning

Every major Decision Engine output stamps the `policy_version` that produced it:

```json
{ "decision": "provider_selected", "provider": "mistral", "policy_version": "0.8.3",
  "components": { "health_score": 0.93, "context_fit": 0.91, "quota_headroom": 0.81 } }
```

Six months later, "why did it choose Mistral" has an exact, reproducible answer instead of a guess — and a scoring-formula or health-decay-half-life change (Section 10.6) becomes something you can diff behavior across, not just something you hope didn't regress anything.

### 28.10 Context Fingerprints, Caching, and Minimum-Context Continuation

Two related quota-saving techniques, both about not re-sending or re-deriving what's already known:

**Caching read-only analysis:** compute `project_snapshot_hash + file_hashes + prompt_hash + model` and cache architecture summaries, file summaries, dependency maps, symbol maps, and test discovery keyed on that fingerprint. Reuse when nothing relevant changed. **Hard rule: cache observations and analysis, never mutation decisions** — a cached "this file does X" is safe to reuse; a cached "therefore do Y to it" is not, since it can silently go stale in a way that produces a wrong action rather than a wrong description.

**Minimum-context continuation on handoff:** Section 9's expanded handoff package is already a strong version of this. Push further — compute context from the *unresolved* problem specifically, not the whole mission history. If the remaining gap is "route registration missing," the handoff sends the mission contract, the current step, the accepted changes, the specific unresolved change, and the relevant files — not the entire prior conversation, not every file touched three steps ago, unless the unresolved problem actually still depends on them.

An **incremental project index** (file changed → re-index only that file → update the dependency graph, never a full re-index) is the mechanical complement to both of these, saving CPU as well as tokens.

### 28.11 Preflight Gate and Cheap-Before-Expensive Ordering

Two ordering disciplines that directly reduce wasted spend:

```
PRE-FLIGHT (before any expensive attempt or verification run)
  ✓ provider capable            ✓ enough quota
  ✓ enough context               ✓ workspace clean/safe (per Section 5.1's contract)
  ✓ required files known          ✓ skill has permission
  ✓ acceptance criteria known      ✓ checkpoint exists
If any condition fails → don't spend the request.
```

```
Verification ordering, cheapest-disproof-first (Section 7's pipeline, made explicit about order):
  file existence → diff scope → syntax → typecheck → build → unit tests →
  integration tests → runtime/browser checks → expensive external checks
Stop at the first cheap check that already disproves success.
```

### 28.12 Request Budget Planner and Resource Reservation

Before a Mission starts, estimate its call cost per Step (`planning: 2, build: 5, review: 2, QA: 3 ≈ 12 total`) and compare against currently available headroom per provider from the Health Ledger (Section 10.6). This prevents starting a Mission whose resource pool will predictably run dry partway through — a Mission-level companion to the per-mission ceiling Section 20 already requires. **Resource reservation** is the same idea applied to contention rather than exhaustion: a *logical*, not necessarily provider-enforced, reservation (e.g. "Gemini key A reserved for the current high-value Mission; Mistral pool available for background/review work") so concurrent Conductor activity doesn't compete with itself for the same scarce combo later.

### 28.13 Recovery Budget, Monotonic Retry, and Semantic Retry Fingerprint

A budget distinct from the token/request budget above — a ceiling on *recovery actions themselves*, closing off the failure mode of an automated recovery loop quietly burning a day's quota:

```
Recovery Budget (per Mission, policy-configurable):
  3 repair attempts · 1 handoff · 1 human escalation
Once exhausted → MissionStatus::Blocked (Section 4.2.1) with:
  "I couldn't safely continue automatically. Your checkpoint is safe."
```

**Monotonic retry rule:** every recovery action must leave the system more informed than the last, never merely repeat it. Enforced via a `attempt_fingerprint` — a hash over `(prompt, files, model, failure_signature)` — computed for each Attempt; a new Attempt whose fingerprint exactly matches the immediately prior failed Attempt's is rejected unless a person explicitly requests the identical retry. Concretely, every new Attempt should be able to state **why this retry is different**:

```
Previous attempt: FailedVerification
New strategy:
  - provider changed
  - context reduced to the isolated failing criterion
  - repair scope limited to the files the failure actually touched
```

If the recovery logic can't populate that "what changed" statement, it should not retry automatically — a strong, cheap guard against an autonomous loop of `retry → retry → different model → repair → repair → loop`. **One repair request beats a full redo:** Section 9's handoff package already models this correctly ("specific unmet criteria become the next repair request, never a vague 'try again'") — treat it as a hard optimization principle, not just a nicety, since a scoped repair request (failing criterion, observed symptom, relevant files, already-verified criteria, explicit do-not-modify scope, required re-verification) is both cheaper and more likely to succeed than re-explaining the whole mission.

### 28.14 Dry-Run Mode

Because the Kernel already emits explicit Commands (28.7) rather than performing side effects inline, computing the Command sequence without executing it is close to free:

```
DRY RUN — Mission: "Add authentication"
Would:
  1. Create checkpoint          4. Run gstack Engineering Review
  2. Select provider/model/combo 5. Give selected executor: App.tsx, auth.ts, routes.ts
  3. Start isolated worktree     6. Run tests
Nothing actually changes.
```

Valuable for debugging routing decisions, building person trust before a Mission runs, and diagnosing why a Mission would behave a particular way, before spending any real quota.

### 28.15 Shadow Mode, Extended Beyond Reconciliation

Section 26.4 already specifies shadow-running new Reconciliation logic against the `failure_cases/` corpus before trusting it live. Extend the same discipline to every other place intelligence evolves over time: run a candidate router, a candidate recovery policy, or a candidate context-selection strategy alongside the live one, with **no action taken by the shadow path**, and compare verdicts:

```
Router:            live=Gemini,   shadow=Mistral           → compare outcome, no action from shadow
Recovery policy:    live=retry,    shadow=handoff            → record the divergence
Context engine:      live=12 files, shadow=5 files            → compare verification outcomes
```

This is how the system's intelligence improves without ever risking an active Mission on an unproven change — Principle 3 ("external truth, not self-certification") applied to every component likely to keep evolving, not only Reconciliation.

### 28.16 Reference Simulator CLI

Before the desktop UI exists at all, a CLI crate (`conductor-sim`, see Section 27's module layout) that runs the Kernel plus the Simulator (28.5) plus the Scenario DSL (28.4) directly:

```
$ conductor-sim scenario timeout_after_partial_edit
PASS
  Invariant 1  ✓   Invariant 3  ✓   Invariant 6  ✓   Invariant 8  ✓   Invariant 14 ✓
```

Most of the reliability engine can be developed and proven against this alone, with a feedback loop measured in seconds rather than requiring the desktop app, Tauri, or a real provider — dramatically shortening the loop for the highest-risk part of the system.

### 28.17 Benchmark Suite

Beyond correctness, track cost so the system stays lightweight by measurement, not assumption:

```
core state operation        < 1 ms (target)
reconciliation                < 500 ms for a normal-sized repo (target)
handoff package size            < target KB
mission startup                   < target
UI idle memory                      < target
worktree creation time                < target
recovery time                           < target
```

Exact numbers are measured against the real system once it exists, not invented up front — the point is establishing the *habit* of tracking these from early on, since a system that becomes correct-but-heavy has still failed the "lightweight, free-tier-friendly" goal in Section 1.

### 28.18 Safe Mode and Per-Automation Kill Switches

Two related escape hatches, most valuable while building the Conductor itself, and useful to a person indefinitely after:

```
SAFE MODE
  ✓ no automatic handoffs      ✓ no autonomous repair
  ✓ no automatic merges         ✓ inspection only
  ✓ no automatic destructive commands
```

```
Per-layer kill switches, each independently toggleable:
  Automatic retry    [ON]   Automatic handoff  [ON]   Automatic repair [ON]
  Automatic merge     [ON]   Automatic QA fix    [ON]
```

With any switch off, the corresponding action becomes `inspect → ask → execute` instead of automatic — directly reusing the `AwaitingHumanDecision` state from 28.19 as the mechanism.

### 28.19 Human Escalation as a First-Class State

The design already says "pause and ask" in several places (Conflict, budget exhaustion). Formalize it as one real state rather than an implicit pattern repeated ad hoc:

```rust
struct AwaitingHumanDecision {
    reason: EscalationReason,     // Conflict | RecoveryBudgetExhausted | SafeModeBlocked | ...
    options: Vec<Decision>,
    evidence: Evidence,             // Section 4.5 — what's actually known, not a summary
    recommended_option: Option<Decision>,
    risk: RiskLevel,
}
```

A `Mission` in `Blocked` (Section 4.2.1) with this attached gives the UI one consistent shape to render regardless of *why* execution stopped, instead of a different ad hoc dialog per escalation source.

### 28.20 Three Kinds of Automation, Named Explicitly

To prevent the system from becoming accidentally over-autonomous as features accumulate, every automated action falls into exactly one of three classes, and the class determines whether it may run without a person:

```
Deterministic automation   — safe to run unattended: state transitions, health calculations,
                              preflight, reconciliation, simple retries within the Recovery Budget
AI-assisted automation      — model proposes, Conductor verifies before acting: planning, repair,
                              code review, context selection, mission decomposition
Human-required decisions     — never automated: conflicting edits, production-affecting changes,
                                security-sensitive exceptions, irreversible actions, budget exhaustion
```

### 28.21 Contract-First AI Implementation Protocol

The most important practical technique for building this specific document with AI coding agents, since it's the difference between an agent implementing exactly one invariant correctly and an agent quietly reinterpreting scope. Never hand an agent an open-ended instruction ("implement Phase 1," "build reconciliation"). Instead, one invariant → one implementation slice → one test gate, every time:

```
TARGET:    Invariant 14
INPUT:     State + event
EXPECTED:  <exact expected behavior>
FORBIDDEN: <what must not change>
FILES:     <exact file list>
TESTS:     <exact test list, generated per Section 28.3 where applicable>
DONE WHEN: <explicit, checkable completion condition>
```

```
Example, concretely:
  Implement ReconciliationOutcome::Conflict.
  Do not modify: AttemptStatus, Verification Engine, UI.
  Required: enum variant, classification behavior, five deterministic tests,
            serialization, a regression fixture in failure_cases/.
  Acceptance: all existing tests pass, new tests pass, no new dependency added.
```

Development itself should follow the AI Conductor's own safety philosophy: an AI coding agent implementing this document does its work in a disposable branch/worktree, gated by tests, reviewed before merge — the project dogfoods its own worktree-isolation principle (Section 5) to build itself, once the initial reliability layer exists to make that safe.

### 28.22 Dependency Footprint Discipline for the Kernel

The Kernel (28.1) specifically should depend on very little — `std`, `serde`/`serde_json`, and small error/type utility crates, deliberately avoiding heavier dependencies inside that boundary even when they'd be convenient. `core = extremely stable; shell/UI = more flexible` — this keeps the part of the system every invariant ultimately depends on the least likely part to be destabilized by an unrelated dependency upgrade.

### 28.23 The Nested Development Loop

Replaces "build eleven phases sequentially" as the actual moment-to-moment working rhythm *within* each phase in Section 23's roadmap — Section 23's phases still define *what* gets built and in what order; this defines the loop used *while* building each one:

```
ARCHITECTURE → ONE INVARIANT → PURE IMPLEMENTATION (28.1) → DETERMINISTIC TESTS (28.3) →
SIMULATOR (28.5) → CHAOS (10.4) → REAL INTEGRATION → REGRESSION → NEXT INVARIANT
```

Slower per individual feature, faster overall, because it prevents the specific failure mode of catastrophic rework: discovering an architectural violation only after several features were built on top of it.

### 28.24 The Governing Rule: Never Solve the Same Uncertainty Twice

Section 2's Principle 6 says *"build the smallest scientifically testable core first."* This section's one philosophical addition sharpens that into an operating discipline for the entire project, not just its first phase:

```
discover → reproduce → encode as a scenario (28.4) → encode as a permanent regression
(Section 11's failure_cases/) → prevent recurrence → move on
```

This is the concrete mechanism that keeps the project from becoming a cycle of *fix → break something else → rediscover an old bug → fix it again* — every failure, once found, becomes a permanent, automatically-checked fact about the system rather than a fact someone has to remember.

---

*This document remains a reliability-first architecture. v2.5 introduced the harness-neutral execution boundary so Aider, Jcode, and future executors can be evaluated under one Conductor-controlled contract instead of becoming architectural authorities. v2.5.1 is a consistency and wording correction pass only: it removes stale Aider-specific phrasing where the architecture is executor-neutral, fixes the implementation-layout and dry-run wording, and avoids claiming that architectural consistency has been independently proven merely by document review.*

*The correct next step remains implementation and empirical verification: turn the sixteen invariants into deterministic test families, continue the Conductor Kernel and simulator work, and validate each execution harness against the common adapter/conformance contract using actual source inspection, controlled probes, failure injection, and verification evidence.*


---

## v2.5 Change-Control Summary — Jcode / Harness-Neutral Execution

This revision changes the architecture in these precise ways:

1. **Execution is now an explicit replaceable layer.**
2. **Aider is no longer a privileged architectural dependency.**
3. **Jcode is introduced as a candidate Execution Harness, not as a kernel component.**
4. **A common Execution Adapter Contract is mandatory before supporting multiple harnesses.**
5. **A common conformance suite is mandatory so executor choice is evidence-driven.**
6. **Conductor remains the sole authority for state transitions, verification, recovery, capability enforcement, and merge.**
7. **Harness-internal swarm, memory, planning, provider routing, and session features remain subordinate to Conductor policy.**
8. **Jcode-specific behavior must be versioned and independently verified; documentation alone is not acceptance evidence.**
9. **The roadmap changes from "integrate Aider" to "validate an execution adapter and evaluate Aider/Jcode under the same contract."**
10. **No Phase 1 kernel work is invalidated by this revision.**

### What this revision intentionally does NOT do

- It does not declare Jcode superior to Aider.
- It does not require Jcode to be installed.
- It does not replace OmniRoute.
- It does not replace gstack.
- It does not make Jcode's swarm the Conductor's multi-agent system.
- It does not weaken isolation, verification, reconciliation, event-chain, or two-phase-merge invariants.
- It does not treat Jcode documentation or model output as proof of correctness.

The correct engineering question is:

> **Which execution harness gives the Conductor the best observable, controllable, recoverable execution surface under the same contract?**

That question is answered by conformance tests, failure injection, real execution evidence, recovery measurements, and resource-cost measurements—not by brand preference.


---

## v2.5.1 Review Corrections

This revision does not change the reliability model or add a new product subsystem. It corrects consistency issues found during a fresh review of v2.5:

1. Removed a duplicated `OS processes` line in the functional-core diagram.
2. Replaced stale Aider-specific execution wording with the selected Execution Harness where the rule is executor-independent.
3. Replaced the dry-run example's hard-coded Gemini/Aider choices with provider/model/combo + selected executor language.
4. Corrected the module-layout appendix version label.
5. Removed an unsupported claim that the document had already been proven internally consistent by a fixed number of independent passes.
6. Reaffirmed that Jcode-specific behavior belongs in the versioned external-components knowledge base and must be independently verified before becoming a supported capability.
7. Preserved all sixteen existing reliability invariants and the Phase 5 harness-neutral execution strategy.
