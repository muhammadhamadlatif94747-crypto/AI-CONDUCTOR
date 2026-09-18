# P5 Conformance Foundation — Verification / Readiness Report

**Scope of this session:** build the minimum reusable, executor-neutral
conformance harness needed to safely begin P5-W03 once a real executor is
available. No attempt at real execution was made. No `FakeExecutionAdapter`
evidence is represented anywhere as real-executor proof. P5-W01/W02
accepted work and all prior accepted checkpoints are untouched.

---

## 1. Documents read before implementation

Master Blueprint v2.5.1 (§10.7, §10.7.6), Build Protocol v1.1, Phase
Manifest v1.1 (§9), Task Contracts v1.1 (§39 P5 registry, §1.3), Verification
Gates v1.1 (§66/§67), Execution Adapter Contract v1.0 (§14), External
Components Knowledge Base v1.1, current `BUILD_STATE.json`,
`P5-REAL-EXECUTOR-READINESS.md`, and the P5-W01-T01/P5-W02-T01 verification
reports already in the repository.

## 2. Contractual requirements reconstructed for P5-W03/W04/W05

- **P5-W03 (Reference Executor Isolation):** validate one real executor
  inside a Conductor-created isolated worktree; prove the live workspace
  remains untouched during execution (Phase Manifest §9.3).
- **P5-W04 (Real Execution Lifecycle):** observe successful execution,
  timeout, cancellation, crash/abrupt termination, and ambiguous
  (`Unknown`) outcome handling against a real executor.
- **P5-W05 (Restart and Reconciliation):** resume safely after a real
  executor interruption, reconciling actual workspace state before any
  continuation, without blindly repeating accepted side effects.
- All three explicitly require a **real** executor per the human's prior
  directive and this repository's own P5-W01/W02 module docs — none of
  them can be satisfied, even partially, by `FakeExecutionAdapter`.

## 3. Genuine conflict found and reported before implementation

Three governing documents define non-identical "EC-xx" conformance
enumerations (16 items, 17 items, and a 12-item differently-named scheme).
Full comparison, resolution rationale, and provisional-status flag:
`.ai-conductor-build/architecture_changes/AC-13-execution-conformance-enumeration-conflict.md`.
Per Task Contracts §1.3, this blocked only the *labeling* question
(which numbering to print), not the underlying mechanical checks
themselves, which are identical in substance under any of the three
schemes. Implementation proceeded using the Execution Adapter Contract's
17-item list as canonical, explicitly marked provisional pending human
confirmation.

## 4. What was built

`crates/conductor-kernel/src/conformance.rs` (new module, registered in
`lib.rs`):

- `ConformanceItem` — the 17-item EC-01..EC-17 enumeration, each variant
  documenting its mapping to the Blueprint's 16-item list and the
  Verification Gates' `VG-P5-EXEC-*` gates.
- `EvidenceStatus` — Observed / Verified / Unknown / Blocked /
  NotApplicable, matching the project-wide evidence vocabulary rather
  than inventing a new one.
- `ConformanceResult` — one check's outcome, carrying a mandatory
  `synthetic: bool` field every producer in the module must set honestly.
- `ExecutorProvenance` — `Synthetic { double_kind }` vs.
  `Real { name, version }`, supplied by the *caller* (never inferred from
  an adapter's self-reported identity, which could be wrong or dishonest).
- Seventeen check functions (`check_identity`, `check_capabilities`,
  `check_workspace_confinement`, `check_lifecycle_outcome`,
  `check_restart_reconciliation`, `check_evidence_collection`,
  `check_credential_redaction`, `check_capability_denial_not_applicable`,
  `check_no_direct_succeeded_or_merge_authority`,
  `check_executor_replacement`), each taking real data and returning one
  (or, for EC-15/16, two) `ConformanceResult`(s). None of them fabricate a
  scenario (timeout, crash, restart) — they classify what a caller
  genuinely observed, honestly reporting `Blocked` when nothing was
  actually exercised.
- `ConformanceSuiteReport` with a pure `disposition()` function that
  **cannot** produce `SuiteDisposition::RealConformancePassed` (or
  `RealConformancePartial`) from any result carrying `synthetic: true`,
  regardless of how many items "passed" — enforced by a dedicated test
  (`suite_disposition_never_reaches_real_conformance_passed_from_synthetic_evidence`)
  using a deliberately all-green synthetic run as the adversarial case.
- `ExternalEnvironmentFacts` / `assess_phase5_readiness()` — answers
  questions A through J exactly as posed by the human directive, from
  caller-supplied facts only; defaults to `Unknown` for C/D/E
  (`ExternalEnvironmentFacts::unconfirmed()`), never favorable, enforced
  by `readiness_assessment_defaults_never_fabricate_environment_facts`.

### Genuinely substantive checks (not just "compiles" checks)

- **EC-03 (workspace confinement):** captures the live repository's own
  `Baseline` (P2-W02, reused directly) before and after the adapter runs;
  a real, mechanically-detected violation if anything outside the
  isolated `AttemptWorkspace` changed. Proven against **real Git** in
  both directions: a test that writes only inside the workspace (passes)
  and a test that deliberately writes into the live repo (fails, and the
  failure is asserted, not just logged).
- **EC-13 (credential redaction):** scans every text-bearing field an
  adapter surfaces (`Observation`, `EvidenceRefs`) for caller-supplied
  secret markers. Proven to actually detect a planted (synthetic, clearly
  fake-shaped) marker, and to correctly report `Unknown` rather than a
  false "clean" pass when no markers were supplied to scan for.
- **EC-15/EC-16 (no direct Succeeded / no direct merge):** turns
  `execution_adapter.rs`'s own prose claim ("grep-audited... zero
  references to `crate::state` or `crate::conflict`") into an actual,
  automatically re-run regression test via `include_str!`. **This caught
  a real false positive during development**: the module's own doc
  comments *name* `crate::state`/`crate::conflict` in backticks while
  explaining the rule, and a naive whole-file substring scan flagged that
  prose as a violation. Fixed by filtering to non-comment lines before
  scanning (this crate has zero block comments crate-wide, confirmed by
  grep, so a `//`-prefix line filter is sound); a dedicated regression
  test (`ec15_check_is_not_fooled_by_the_forbidden_symbols_appearing_in_doc_comment_prose`)
  pins the fix so it cannot silently regress.
- **EC-17 (executor replacement):** the one item that genuinely does not
  need a real executor to prove, since replaceability is a property of
  the calling code/trait design, not of any executor's real-world
  behavior — proven by driving two differently-identified adapters
  through identical generic code with zero branching on which one it is.

### Deliberately NOT built

- No `AiderAdapter`/`JcodeAdapter` — no executor was prematurely selected
  (Manifest §9.4 non-goal; also grep-audited: zero `aider`/`jcode` string
  references anywhere in `conformance.rs`).
- No attempt to simulate timeout/crash/interruption for a real process —
  those require an actual process and are explicitly the caller's job to
  orchestrate; this harness only classifies what actually happened.
- No workspace/baseline bridging into `ExecutionRequest.workspace_id`/
  `baseline_id` (the gap `P5-REAL-EXECUTOR-READINESS.md` flagged) — out of
  this session's scope; it belongs to P5-W03 itself once a real executor
  exists to bridge toward, not to the conformance harness that tests it.

## 5. Test results

```
cargo test --workspace --locked
→ 286 passed; 0 failed; 0 ignored   (268 pre-existing + 18 new conformance tests)

cargo test --workspace --locked conformance::
→ 18 passed; 0 failed; 268 filtered out
```

`cargo build --workspace --locked`: 0 warnings.

`cargo test --doc`: same pre-existing sandbox tooling quirk documented in
`P5-REAL-EXECUTOR-READINESS.md` §1 (cargo/rustdoc version-pairing artifact
of this sandbox's apt packaging, unrelated to any code in this crate,
including the new module).

`rustfmt --edition 2021 --check` on the new file: clean after one
formatting pass, applied **only** to `conformance.rs` (no unrelated
reformatting of other files — confirmed via `git status --porcelain
crates/` showing only `lib.rs` (+1 line) and the new `conformance.rs`
before and after the format pass).

`Cargo.lock`: byte-identical (`4a231757bb9efd4e50201e2184c250bc`) before
and after this session's entire body of work. No dependency was added —
the module deliberately reuses the existing `WorktreeManager`/`Baseline`
primitives and the same manual `std::env::temp_dir()` test-fixture pattern
`worktree.rs` already established, rather than introducing `tempfile` or
any other new crate.

## 6. Final readiness determination (A–J)

| # | Question | Status | Detail |
|---|---|---|---|
| A | Is the conformance infrastructure implemented? | **Verified** | All 16 applicable `ConformanceItem`s (EC-14 correctly `NotApplicable` pre-P9) have a real, tested check function. |
| B | Can it exercise a real `ExecutionAdapter`? | **Observed (structurally)** | Every check function takes `&dyn`/owned data from the trait's real return types, not `FakeExecutionAdapter`-specific types — a real adapter's `Observation`/`EvidenceRefs`/`ExecutorIdentity` would flow through the identical functions unchanged. Only actually exercised against `FakeExecutionAdapter` so far (`SuiteDisposition` for any such run is capped at `SyntheticOnly`, never higher — mechanically enforced, not asserted in prose). |
| C | Is Aider actually available? | **Unknown** | Not reconfirmed this session (see U-0004). Last observation: `aider 0.86.2` on the human's machine, 2026-08-26. |
| D | Is Jcode actually available? | **Unknown** | Never observed installed in this project's history. |
| E | Is at least one usable provider/model credential available? | **Unknown** | Never observed available in any session to date. |
| F | Can the real executor perform a controlled test task? | **Blocked** | Requires B=Observed-for-real (not yet true) AND (C or D)=Observed AND E=Observed. None of these hold. |
| G | Can the workspace isolation requirement be observed? | **Observed (mechanism only)** | `check_workspace_confinement` is proven against real Git with both a clean and a violating scenario. Not yet run against a real executor's actual behavior. |
| H | Can execution interruption be observed? | **Unknown** | `check_lifecycle_outcome` can classify a reported `TimedOut`/`Cancelled`/`Crashed` state once one is genuinely produced; no real interruption has been produced yet (aggregated weakest of EC-07/08/09, all currently only synthetic-tested). |
| I | Can post-interruption reconciliation be observed? | **Blocked** | `check_restart_reconciliation` requires the caller to have actually run a restart; none has occurred against a real executor. |
| J | Which exact P5 gate can now be attempted? | **None** | F is not `Observed`. P5-W03/W04/W05 remain `BLOCKED`. When F becomes `Observed`, `VG-P5-EXEC-01` (Adapter Contract) followed by `VG-P5-EXEC-02` (Workspace Confinement) are the first gates to attempt, per this module's own `assess_phase5_readiness()` logic (exercised in test, not just asserted in this table). |

**A and B pass. C through I cannot yet be proven.** Per the human directive,
P5-W03/W04/W05 remain `BLOCKED`, unchanged from the prior session's
determination — this session narrows *why* (the harness itself is no
longer the blocker; external prerequisites are) but does not remove the
block.

## 7. Exact missing external prerequisites (unchanged from U-0004, now more precisely framed)

1. Reconfirm Aider 0.86.2 (or a current version) is still present and
   reachable from wherever the P5-W03 implementation session actually
   runs — OR substitute Jcode.
2. A valid provider/model credential that executor can use to perform a
   real, non-trivial edit.
3. A disposable Git repository for the controlled test task (any small
   real repo; does not need to be `AI-CONDUCTOR` itself).

None of the three exist in this sandbox and this sandbox's restricted
network egress allowlist would prevent acquiring (1) or (2) from here even
if authorized to try, which it was not.

## 8. Disposition

- P5-W03-T01 / P5-W04-T01 / P5-W05-T01: remain `BLOCKED` (unchanged
  classification; readiness narrowed, not resolved).
- No verification gate weakened. No synthetic evidence represented as
  real. No executor prematurely selected.
- `.ai-conductor-build/architecture_changes/AC-13-...md`: **updated
  since this report was first written** — a human ruling has since
  formally adopted the 17-item enumeration as the active provisional
  standard (status: `PROVISIONAL_17_ITEM_EXECUTION_ADAPTER_ENUMERATION`,
  recorded in `BUILD_STATE.json`'s `decisions.accepted`). This does not
  resolve the underlying Blueprint/Verification Gates/Phase Manifest
  numbering disagreement, which remains open; it only authorizes
  continued implementation under the existing provisional choice. See
  `GOVERNANCE/EC_ENUMERATION_MAPPING.md` for the canonical mapping.
- **Certification boundary, stated explicitly:** structural/synthetic
  conformance work (this report's own scope) is allowed and has
  proceeded. Real-executor certification remains blocked until a real
  executor and required credentials/environment actually exist — P5-W03
  is blocked outright; P5-W04 is blocked pending the real-execution
  boundary that P5-W03 would establish; P5-W05 is blocked pending both
  real execution and the actual reconciliation/idempotency integration
  documented as deferred in `conformance.rs`'s EC-11
  `TODO(P5-W05)` comment. No synthetic evidence may be treated as real
  evidence at any point in this chain.
