# Architecture Contract AC-09 — Two-Phase Merge & Git Edge Cases
# Task: P0-W09-T01 (Phase 0 — no production code; this document IS the deliverable)
# Provenance: content pre-staged during P0-W05 order reconciliation; formalized at P0-W09-T01.

```yaml
contract_id: AC-09
task_id: P0-W09-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §5 (Workspace Isolation & Git Worktree Safety), §5.1 (lock-scope correction)"
  - "Blueprint §4.6 (Mutation Attribution)"
  - "Blueprint §3 Invariant 13"
  - "Phase Manifest P0-C09"
```

## 1. Isolation rule

Every attempt executes in an **isolated attempt workspace** — a `git worktree` (or
equivalent isolated working copy). Aider/agents/providers NEVER operate directly in
the user's live folder.

```text
User's real workspace → Conductor safety boundary → Attempt workspace → execution
→ Verification Engine → accepted? two-phase conditional merge : discard
```

Consequences that are contractual, not incidental:
- A failed/partial/rejected attempt leaves the user's files untouched — nothing to undo.
- Every retry, provider handoff, or repair gets a fresh disposable copy.
- Only verified, accepted results are eligible for merge-back.

## 2. Merge is an expected-state conditional operation

Per Blueprint v2.4 §5.1 correction: the Conductor's single-writer mutation lock
(§17) does NOT and cannot stop external OS processes from writing during the merge
window. The merge is therefore specified as compare-and-swap semantics, not mutual
exclusion:

```text
Acquire Conductor mutation boundary (single-writer lock, §17)
→ re-check live workspace against recorded baseline, file-by-file
→ apply merge ONLY IF live tree still equals expected baseline
→ verify resulting tree matches what the merge should produce
→ mismatch at either check → release lock → Conflict; NEVER merge blind
→ match at both checks → commit/checkpoint while still holding the lock
→ release lock
```

Ordering rule: post-merge verification happens after the write and before any
checkpoint; a checkpoint is never written for an unconfirmed merge. Post-merge
verification failure = FailedInfrastructure case — not blindly retried.

Invariant 13 binding form: a baseline change detected at re-check is a Conflict,
never merged blind; per Invariant 11 it halts automatic recovery — the person chooses.

## 3. Baseline-comparison edge-case table

Binding behavior per workspace state at merge time (from Blueprint §5):

| Workspace state | Required behavior |
|---|---|
| Tracked file modified since baseline | Conflict — never merged blind |
| Untracked new file | Outside baseline comparison; left alone; merge proceeds on owned files |
| File deleted since baseline | Conflict |
| File renamed | Delete+create unless rename-detection confirms same logical file unchanged → unmodified |
| .gitignore'd file | Excluded from comparison AND merge — never managed by Conductor |
| Merge would produce conflict markers | Halt before writing; surface Conflict; never write markers silently |
| Submodule | Unsupported state — surface explicitly |
| LFS-tracked file | Opaque blob for diffing; hash comparison yes, content-diff display no |
| Detached HEAD | Unsupported state |
| Branch deleted/changed externally mid-mission | Detected at next baseline re-check; surfaced to person |
| Repository missing/not a repo | Fail closed → Mission Blocked |

## 4. Supported repository state

A Mission may start only on: single local git repository, named branch (no detached
HEAD), clean or normally-dirty tree, no in-progress rebase/merge/cherry-pick, no
unresolved conflict markers. Anything else → refuse to start (or move to Blocked if
discovered mid-mission) with a specific human-readable reason.

## 5. Mutation attribution feeds reconciliation

Baseline diff-state captured at attempt start; resulting diff at end; region-by-region
classification (expected_conductor_change | unexpected_ai_change | user_change |
conflict_region) drives ReconciliationOutcome (AC-01) and targeted repair flags.
Hash comparison alone is insufficient — actual diffs are required evidence (AC-02).

## 6. Non-goals

Worktree lifecycle mechanics/cleanup policy internals (P2-W02), reconciliation engine
algorithms (P2-W03), verification pipeline (P4), failure taxonomy itself (AC-05).

## 7. Invariants preserved

5, 6, 7, 11, 12, 13, 14. None weakened.
