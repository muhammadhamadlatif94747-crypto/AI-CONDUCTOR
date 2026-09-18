# Phase 5 — Real-Executor Readiness Audit

**Session type:** Independent takeover audit + BUILD_STATE reconciliation only.
**No production code was changed in this session.** P5-W01-T01 and P5-W02-T01
remain exactly as accepted (commits `8cf503a`, `f961159`, checkpoints `bdc01cb`,
`fa564e2`) — untouched.

**Trigger:** Human directive (verbatim, summarized): record P5-W03/W04/W05 as
BLOCKED/DEFERRED rather than FAILED; do not start P5-W08; do not fabricate
partial/fake evidence for the blocked tasks; do not prematurely select Aider
over Jcode or vice versa; prepare the project for the real-executor stage and
report exactly what is/isn't ready; then stop.

---

## 1. Independent re-verification (fresh environment, real toolchain)

This session's sandbox had no Rust toolchain installed. Installed
`rustc 1.91.1` / `cargo 1.91.1` from scratch (the sandbox's default
`cargo 1.75` could not even parse the committed `Cargo.lock`, which is lock
format v4 — this by itself is a useful fact: the project's `Cargo.lock`
requires cargo ≥ ~1.78 to read at all, not just to build).

```
cargo test --workspace --locked
→ 268 passed; 0 failed; 0 ignored
```

`Cargo.lock` md5sum identical before and after (`--locked` enforces this
mechanically as well). Working tree was clean before and after. This is a
genuinely independent confirmation of the 268/268 claim, in a different
machine/toolchain/OS (Linux container vs. the Windows/cargo 1.97.1 environment
BUILD_STATE.json's `dependencies.environment` records) — the kind of
cross-environment evidence the External Components KB's evidence policy
(§1) asks for rather than trusting one environment's result forever.

One unrelated tooling artifact, **not a project defect**: `cargo test --doc`
fails with `-Z unstable-options` / `check-cfg` errors. This is a
`cargo`/`rustdoc` version-pairing quirk of this sandbox's specific apt
packaging (a `cargo-1.91` binary paired against a mismatched default
`rustdoc`), reproducible in isolation, orthogonal to `conductor-kernel`'s own
267 unit tests + 1 console-bin test, all of which passed. Recorded here per
Build Protocol §12 (evidence must be classified honestly) rather than
silently ignored or silently folded into the 268 count.

## 2. Environment discrepancy — flagged, not resolved

`BUILD_STATE.json.dependencies.environment.aider` currently reads `"0.86.2"`,
sourced from `.ai-conductor-build/diagnostics/environment_baseline.md`
(2026-08-26T05:27:15+05:00 inspection): `aider 0.86.2` was found on PATH at
`%LOCALAPPDATA%\...\Scripts\aider.exe` on the human's actual machine.

`current_phase.known_environment_constraint` (as written before this session)
says *"Neither Aider nor Jcode is installed in this sandbox"* — that statement
is about the P5-W01/W02 implementation sandbox, not the same claim as "Aider
is absent from the human's machine." Those are two different facts and this
document was at risk of conflating them. Precisely:

| Fact | Status |
|---|---|
| Aider binary present on the human's machine at 2026-08-26 inspection | OBSERVED (aider --version → 0.86.2) |
| Aider binary present in *this* (or any prior) implementation sandbox | Not observed / not installed |
| Provider API keys (Gemini/Mistral/OpenAI-compatible) available to let Aider actually call a model | Not observed anywhere in any session to date |
| Jcode installed anywhere | Not observed anywhere in any session to date |

**The actual current blocker for P5-W03 is therefore more specific than "no
executor installed": it is "no provider credentials exist anywhere this
agent has run, so even the one executor binary known to exist (Aider
0.86.2, last confirmed 2026-08-26) cannot be driven end-to-end."** This
sandbox additionally has no Aider/Jcode binary and a restricted egress
allowlist (crates.io/npm/pypi/github-style hosts only — no arbitrary package
installers), so even if credentials existed, this specific session could not
install/run Aider from here either.

This is recorded as `U-0004` below rather than silently re-asserting the
older, less precise sentence.

## 3. Execution Adapter — can it accept a real executor?

**Yes, structurally.** `execution_adapter.rs`'s `ExecutionAdapter` trait:

```rust
pub trait ExecutionAdapter {
    fn identity(&self) -> ExecutorIdentity;
    fn capabilities(&self) -> CapabilitySnapshot;
    fn start(&mut self, request: &ExecutionRequest) -> Result<(), AdapterError>;
    fn observe(&self) -> Observation;
    fn cancel(&mut self) -> CancellationReport;
    fn collect_evidence(&self) -> EvidenceRefs;
    fn finalize(&self) -> FinalizationClaim;
}
```

Findings:

- **No executor-specific coupling.** Crate-wide case-insensitive grep for
  `aider|jcode` across all of `crates/conductor-kernel/src` returns matches
  **only** in doc-comments/module docs explaining the environment constraint
  and one illustrative test comment (`worktree.rs:262`, `271` — "simulate
  what Aider would do"); zero occurrences in any type name, trait, `impl`
  block, or field. Nothing needs to change in this file to plug in a real
  `AiderAdapter` or `JcodeAdapter` — both would simply be new `struct`s
  implementing the same trait, exactly as `FakeExecutionAdapter` already does.
- **Sync, not async.** `start`/`observe`/`cancel`/`finalize` are synchronous.
  `provider.rs` documents the identical deliberate deviation from the
  Blueprint's `async fn send()` (no async runtime exists anywhere in the
  crate yet). A real Aider/Jcode adapter driven via subprocess + polling can
  satisfy this synchronous trait directly (spawn, poll `try_wait()`,
  translate to `Observation`/`ExecutionState`) without requiring an async
  runtime to be pulled into the kernel. If a real adapter genuinely needs
  async I/O, that is itself an architecture decision for whoever builds
  P5-W03, not something this audit should decide by default.
- **`&mut self` on `start`/`cancel`, `&self` on the rest** — consistent with
  a real adapter holding a `Child` process handle as internal mutable state.
- **Object-safety:** every method takes `&self`/`&mut self` and returns
  owned/concrete types — no generics, no `Self` return — so `&mut dyn
  ExecutionAdapter` works, matching the Blueprint's `Box<dyn
  ExecutionAdapter>` intent and P5-W08's reversibility requirement (swap
  adapters behind a trait object with zero kernel changes).

**Gap:** `start()`'s `Result<(), AdapterError>` only distinguishes
`StartRejected`/`CapabilityNotAllowed` — a real adapter will need a richer
error path for "the executor binary was not found on PATH" /
"credentials rejected by provider" distinctly from a policy-level rejection.
Not fixed here (would be silent scope expansion outside this audit); flagged
for whoever picks up P5-W03.

## 4. Workspace contract — is it ready?

**Yes**, and re-proven live in this session (not merely re-read):
`worktree.rs`'s `WorktreeManager`/`AttemptWorkspace` tests were part of the
268/268 that ran against a real Git repository in this sandbox
(`worktree::tests::*`, 10 tests, all passing) — isolation was proven both
directions (live-repo changes don't leak into the attempt workspace;
attempt-workspace writes never appear in the live repo), the exact contract
a real executor will run inside. `create()`/`remove()` wrap real
`git worktree add`/`remove` calls; nothing here is mocked.

**Gap:** nothing in `worktree.rs` yet computes a `baseline_id` string in the
shape `ExecutionRequest.baseline_id` (`String`) expects, or wires
`AttemptWorkspace`'s path into `ExecutionRequest.workspace_id`. `baseline.rs`
(P2-W02) produces the actual `Baseline` content-hash structure separately.
Someone building P5-W03 needs to bridge `WorktreeManager::create()` →
`Baseline::capture()` → populate `ExecutionRequest` — straightforward given
both pieces exist and are tested, but not yet wired together.

## 5. Conformance test harness — is it ready?

**No.** Grep for `conformance|EC-0` across the crate returns **zero**
matches. None of the following exist yet, anywhere in the repository:

- an `execution/conformance.rs` module (or equivalent) implementing the
  Execution Adapter Contract v1.0 §14 suite (EC-01 through EC-17) / Phase
  Manifest §9.3's common conformance suite (EC-01..EC-16 per the
  Verification Gates amendment);
- any test that drives a `&mut dyn ExecutionAdapter` through
  `start → observe → cancel/finalize` and asserts against the required
  lifecycle semantics (successful execution, partial execution, non-zero
  exit, timeout, cancellation, crash, Unknown outcome, restart/reconciliation,
  credential redaction, capability denial, no-direct-Succeeded,
  no-direct-merge);
- a fixture executor deliberately built to fail each of those ways on
  purpose (distinct from `FakeExecutionAdapter`, which is scripted to
  succeed/return whatever it's told — useful for proving the trait is
  implementable, not for proving a conformance suite actually catches a
  bad implementation).

This is real, necessary work for whoever starts P5-W03 — building the
conformance suite is arguably a precondition for validating *any* real
executor, not an optional nicety, since VG-P5-EXEC-01 through
VG-P5-EXEC-12 (Verification Gates §66) require it.

## 6. P5-W03/W04/W05 fixtures/scenarios — are they prepared?

**No**, beyond what already exists for other purposes. `scenario.rs`
(P3-W04) drives `FakeProvider`/`ProviderBehavior`, not
`dyn ExecutionAdapter` — different trait, different lifecycle shape. No
scenario file currently constructs an `ExecutionRequest`, calls a real or
fake adapter's `start()`, and asserts on `Observation`/`FinalizationClaim`.
Building this is naturally part of P5-W03/W04 (the tasks that need it), not
separable prep work that could be done without knowing which real executor
is being validated first.

## 7. Unnecessary assumptions preventing either Aider or Jcode from plugging in

Audited (grep, read through `execution_adapter.rs` in full): **none found.**
The trait, its associated types, and `FakeExecutionAdapter` make no
executor-specific assumption (no Aider-specific flags, no Jcode-specific
session/swarm concepts, no hardcoded model/provider names). `CapabilitySnapshot`'s
13 fields are the Adapter Contract's generic capability list, not tailored to
either tool. This satisfies Manifest §9.4's non-goal ("do not prematurely
select Jcode over Aider or Aider over Jcode") structurally, not just by
policy.

## 8. What is required, concretely, to unblock P5-W03

1. A real executor reachable from wherever the implementing session actually
   runs — either Aider (confirmed present at 0.86.2 on the human's machine as
   of 2026-08-26; presence should be reconfirmed, not assumed still true) or
   Jcode (never observed installed anywhere in this project's history).
2. Valid provider credentials that executor can use to make a real model
   call (Gemini/Mistral/OpenAI-compatible key, or whatever OmniRoute would
   otherwise broker — OmniRoute's own status is separately `UNKNOWN`/U-0001).
3. A disposable Git repository the executor is authorized to mutate inside a
   `WorktreeManager`-created `AttemptWorkspace` (any small real repo works;
   does not need to be `AI-CONDUCTOR` itself).
4. The EC-01..EC-16/17 conformance suite built first (§5 above), so the real
   executor is validated against the same mechanical suite any future
   executor will be, rather than an ad hoc one-off test.

None of the above exist in this sandbox and this sandbox cannot acquire (1)
or (2) itself (no interactive credential flow, restricted egress). This is
therefore accurately a **human-dependency block**, not an agent capability
gap.

## 9. Disposition

P5-W03-T01, P5-W04-T01, P5-W05-T01 → `BLOCKED` (dependency: real executor +
credentials), not `FAILED`. P5-W06-T01/P5-W07-T01 remain conditional per
their own Manifest wording ("if Aider is retained" / "if the pinned/installed
Jcode build is available") and are recorded as `NOT_APPLICABLE_YET` rather
than `BLOCKED` — they may turn out to be legitimately skippable depending on
which executor is eventually selected, which is itself a decision this audit
does not make. P5-W08-T01 remains `NOT_STARTED` by explicit human instruction
(not attempted this session, per directive).

No verification gate was weakened. No `FakeExecutionAdapter` test is
represented anywhere as real-executor evidence. See `BUILD_STATE.json` for
the corresponding ledger entries.
