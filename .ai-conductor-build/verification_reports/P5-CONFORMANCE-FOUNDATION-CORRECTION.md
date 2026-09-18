# P5 Conformance Foundation — Correction Pass Verification Report

Scope: correct the four categories of issue the forensic audit found in
`crates/conductor-kernel/src/conformance.rs`, without implementing
P5-W03/W04/W05, without adding an Aider/Jcode adapter, and without
weakening any gate. Governing documents and the audit were re-read in
full before any code was touched.

---

## A. Governance / documentation changes

- **New standalone canonical artifact:** `GOVERNANCE/EC_ENUMERATION_MAPPING.md`.
  Contains the full 17-row mapping table (Adapter Contract §14 axis →
  Blueprint §10.7.6 → Verification Gates §66/67 → Phase Manifest P5 →
  Task Contracts P5 §39), explicit one-to-many/many-to-one call-outs, and
  dedicated sections for the two genuine mismatches (EC-12 vs. Blueprint's
  differently-scoped EC-12; EC-17's total absence from Blueprint). Marked
  **PROVISIONAL** at the top and in its closing section; states plainly
  that real-executor certification must not cite it as a permanent
  resolution.
- **No governing document was edited.** Blueprint, Build Protocol, Phase
  Manifest, Task Contracts, Verification Gates, Adapter Contract, and the
  External Components Knowledge Base are byte-identical to before this
  session (none were even opened for writing).
- **AC-13 amended non-destructively:** a four-line supersession-style note
  added at the top (matching the existing convention used elsewhere in
  this repository, e.g. the 48-Hour Campaign document's own supersession
  note), pointing to the new mapping document. Nothing below that note in
  AC-13 was altered — its original STATUS line and reasoning remain the
  historical record.
- **`conformance.rs`'s module doc comment** now links to
  `GOVERNANCE/EC_ENUMERATION_MAPPING.md` as the authoritative
  cross-reference, rather than containing its own inline comparison
  (which is now kept in exactly one place).
- **Recommended AC-13 resolution restated:** a C+B hybrid (keep the
  provisional choice in active use; formalize the comparison as a
  standalone document; block only real-executor certification on the
  eventual human ruling) — this was the audit's recommendation, now
  acted on for the documentation half; the actual ruling remains pending.

## B. Type-safety changes

- `ConformanceResult`'s five fields (`item`, `status`, `passed`,
  `synthetic`, `detail`) are now **private**. Public access is via
  `.item()`, `.status()`, `.passed()`, `.synthetic()`, `.detail()`
  accessors only.
- Construction is now exclusively through five named constructors:
  `ConformanceResult::observed(item, passed: bool, synthetic: bool, detail)`,
  `::verified(...)` (same shape), `::blocked(item, synthetic: bool, detail)`,
  `::unknown(item, synthetic: bool, detail)`, `::not_applicable(item, detail)`.
  `blocked`/`unknown` have **no `passed` parameter at all** — the invalid
  combination the audit found (`status: Blocked, passed: Some(true)`) is
  not merely undocumented now, it has no parameter list that could
  express it.
- `not_applicable` deliberately omits a `synthetic` parameter (documented
  reasoning in its own doc comment: the only current use, EC-14, never
  depends on an adapter, so synthesizing a value for a meaningless
  question was avoided rather than guessed).
- **Rust module-privacy honesty note, stated in the module doc rather than
  overclaimed:** this module's own `tests` submodule retains access to
  the private fields (Rust privacy is scoped to a module and its
  descendants, not per-file). The tests were written to use *only* the
  public constructor/accessor surface anyway, by choice, to prove that
  surface is sufficient — not because the compiler forced it. The actual
  hardened boundary is against every other module in the crate (a future
  `aider.rs`/`jcode.rs`), which is the boundary that matters for the
  audit's concern.
- `ConformanceSuiteReport.results` was also made private (was `pub`),
  with `.result_for(item)`, `.len()`, `.is_empty()` accessors added —
  the same class of hardening applied consistently, not just to
  `ConformanceResult`.
- **Adversarial regression tests added** (5 new, all passing):
  `blocked_can_never_carry_a_passed_value`,
  `unknown_can_never_carry_a_favorable_passed_value`,
  `not_applicable_can_never_masquerade_as_passed`,
  `observed_and_verified_still_work_for_every_existing_valid_call_site_shape`,
  `missing_applicable_evidence_remains_blocked_after_hardening`.
- `disposition_impl`'s (now `disposition`'s) actual matching logic is
  **unchanged in semantics** — same conservative rules, same
  `SuiteDisposition` outcomes for the same inputs — only its field access
  now goes through accessor methods. Every pre-existing valid call site
  (all thirteen check functions) was updated to the new constructors with
  no change to what `status`/`passed`/`synthetic` value each one
  produces.

## C. EC-03 documentation changes

- The doc comment on `check_workspace_confinement` was rewritten with six
  explicitly numbered points, matching the forensic audit's Section 4
  findings verbatim in substance: gitignored/untracked files ARE
  included (stricter than Blueprint §5.1); new symlinks at
  previously-nonexistent paths are invisible; transient writes between
  snapshots are invisible; only the given root is observed; the
  "repository-tree snapshot integrity" vs. "absolute machine-wide
  filesystem confinement" distinction is stated as its own point;
  explicit statement that none of this is being fixed in this pass
  (`Baseline` not redesigned, no filesystem watcher added — both
  explicitly out of this task's scope).
- The function's own runtime `detail` strings (both the pass and fail
  paths) now say "repository-tree snapshot integrity" rather than the
  more sweeping "workspace confinement" language, and the failing-path
  detail explicitly notes it is not a claim about activity outside the
  given root.
- New regression test:
  `ec03_gitignored_style_paths_are_still_captured_stricter_than_blueprint`
  — writes to a `.gitignore`d path inside a live repo and asserts EC-03
  reports it as a violation, pinning the documented (not fixed)
  over-strictness as permanent, checked behavior rather than a claim that
  could silently drift.
- `Baseline` itself (`baseline.rs`) was **not modified**. No filesystem
  watcher was added. No machine-wide surveillance was attempted. All
  three explicitly forbidden by this task's Part 3.

## D. EC-12 / EC-13 changes

**EC-12:** Determined the Adapter Contract's own literal §14/§9 scope for
EC-12 is the evidence-reference-field list (attempt/workspace/executor/
version/process/commands/outputs/artifacts/exit-result) — it does **not**
itself mention "failure classification" as part of its text. Blueprint's
differently-numbered EC-12 ("provider failure propagation") is the
concern that actually names failure classification, and per AC-13/the
mapping document, that concern was only ever "folded in" as a prior
session's documentation claim with no backing check. Resolution taken:
added the smallest appropriate structural check, using the taxonomy the
crate already defines (`FailureCategory`, Adapter Contract §13) rather
than inventing new architecture — `check_evidence_collection` now also
asserts that whenever `Observation.final_executor_state` is
failure-shaped (`Failed`/`Crashed`/`TimedOut`/`Cancelled`/`Unknown`),
`failure_classification` is `Some(_)`. Explicitly documented, in both the
function's doc comment and the new mapping document, that this does
**not** prove a real provider failure was correctly classified and
propagated end-to-end — that remains P6+ scope. Two new tests:
`ec12_requires_failure_classification_when_final_state_is_failure_shaped`,
`ec12_does_not_require_failure_classification_on_a_non_failure_outcome`.

**EC-13:** `EvidenceRefs.process_ref` and `.exit_result` are now included
in the credential-scan haystack (`process_ref` was the specific gap the
audit found; `exit_result` was added at the same time as a consistent,
equally-cheap completion of the same field list, since it was the other
`Option<String>` on `EvidenceRefs` not yet scanned). New regression test:
`ec13_detects_a_marker_planted_only_in_process_ref`, which plants a
marker exclusively in `process_ref` and asserts it is caught — proving
the fix, not just asserting it. The `stderr` limitation (no dedicated
field anywhere upstream of this function, in either the Adapter Contract
or this crate's `Observation` type) is now recorded explicitly in the
function's doc comment, per this task's instruction not to invent a
dedicated stderr field inside `conformance.rs` itself.

## E. What remains intentionally deferred

- **EC-11's real bridge** (`reconciliation::reconcile()` and
  `IdempotencyStore::has_already_applied()`) — documented as a
  `TODO(P5-W05)` block directly above `check_restart_reconciliation`,
  including the exact required chain diagram from this task's
  instructions, reproduced in the code comment. **Not implemented.** The
  function's behavior and detail-string wording are unchanged from before
  this pass except for an added note pointing at the TODO.
- **EC-04 through EC-10** — explicitly not upgraded. `check_lifecycle_outcome`'s
  doc comment now states, as a permanent constraint rather than a
  one-time note, that any future change adding subprocess-spawning,
  timing, or OS-level observation belongs in P5-W03/W04, not as a quiet
  expansion of this function. No fake subprocesses or simulated
  timeouts/crashes were added.
- **AC-13's actual ruling** — still pending. The mapping document exists
  and is provisional; no human decision was made or assumed on this
  agent's behalf.
- **The `ExecutorProvenance` caller-honesty limitation** (a caller could
  mislabel a real adapter as synthetic or vice versa) — stated explicitly
  in the module docs as irreducible; no fix attempted or claimed.
- **No Aider adapter, no Jcode adapter, no real execution orchestration**
  — none attempted, per this task's hard boundary.

## F. Tests / evidence

```
cargo build --workspace --locked   -> 0 warnings
cargo test --workspace --locked    -> 295 passed; 0 failed
                                       (268 baseline + 27 conformance;
                                        was 18 conformance tests before
                                        this pass, +9 new)
cargo test --workspace --locked conformance::
                                    -> 27 passed; 0 failed; 268 filtered out
cargo test --doc                   -> same pre-existing sandbox tooling
                                       quirk documented in
                                       P5-REAL-EXECUTOR-READINESS.md SS1
                                       (cargo/rustdoc version-pairing
                                       artifact of this sandbox's apt
                                       packaging), unrelated to any code
                                       touched this session
rustfmt --edition 2021 --check conformance.rs  -> clean (applied once,
                                                    scoped to this file only)
Cargo.lock                          -> byte-identical throughout:
                                        4a231757bb9efd4e50201e2184c250bc
```

`git diff --stat` against the prior commit shows exactly two modified
files (`conformance.rs`, `AC-13-...md`) and this report plus the mapping
document as new files — no other file in the repository was touched, in
particular nothing under `state.rs`, `verification_engine.rs`,
`reconciliation.rs`, `baseline.rs`, `worktree.rs`, `event_log.rs`, or any
other previously-accepted P1/P2/P4 module.

## G. Commit hash

`ed35983`

## H. Confirmation: P5-W03/W04/W05 status

**Unchanged. Still `BLOCKED`.** Nothing in this correction pass claims,
implies, or moves toward claiming real-executor conformance. `BUILD_STATE.json`'s
`task_history` entries for `P5-W03-T01`, `P5-W04-T01`, `P5-W05-T01` were
inspected before this pass began and confirmed already `BLOCKED`; this
pass added one new sibling entry
(`P5-CONFORMANCE-FOUNDATION-CORRECTION`) recording its own scope and did
not modify those three existing entries at all.
