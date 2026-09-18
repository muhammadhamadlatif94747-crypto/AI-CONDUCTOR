# P5 Conformance Foundation — Forensic Audit

**Not committed.** This is an audit/decision-support artifact only, per this
round's instructions. No code was changed. Every empirical claim below was
produced by real, executed Rust tests written as scratch additions to
`conformance.rs`, run with `cargo test ... -- --nocapture`, then reverted
with `git checkout --` before this report was written — confirmed clean
(`git status --porcelain` empty, 286/286 passing, `Cargo.lock` unchanged:
`4a231757bb9efd4e50201e2184c250bc`) immediately before this report's text
was finalized. Where a claim is a straightforward reading of code rather
than something worth executing (e.g. "capture() takes exactly one root
path"), it is marked `(by construction)` rather than `(executed)`.

Your message was cut off mid-sentence at the end of Section 7 ("stdout /
stderr / structured adapter output / error"). Sections 1–6 and the visible
part of 7 are answered in full below. Send the rest when ready — nothing
past what you actually wrote has been guessed at or invented.

---

## Recommended resolution for AC-13 (answered first, as requested)

**C, with a document produced as part of C — effectively C+B, not a bare
"leave it provisional forever."**

Reasoning:
- **A (edit the governing documents)** is not this agent's authority to
  do unilaterally — Build Protocol §14 and Task Contracts §1.3 both
  require a human decision before an architecture-level document is
  changed, and three documents disagreeing about a numbering scheme is
  exactly that class of decision.
- **B alone (a canonical mapping document with no provisional code)**
  would mean holding all P5 conformance work until the human rules —
  unnecessary, since none of the three candidate resolutions changes what
  the mechanical checks themselves verify, only what they're labeled.
- **D** — no better option was found.
- **C** is close, but "keep the provisional mapping and block only the
  affected certification" undersells what's actually needed: the mapping
  itself should be a first-class, explicit artifact (not just a paragraph
  inside AC-13), because it's the thing a future Aider/Jcode adapter
  author will actually consult. **Recommendation: keep AC-13's provisional
  17-item choice in active use (already true), but promote its comparison
  table into a standalone canonical-mapping document
  (`EC_ENUMERATION_MAPPING.md` or similar) that `conformance.rs`'s module
  doc links to, and block only real-executor certification (never
  structural/synthetic work) on the human's eventual ruling.** This is a
  documentation reorganization, not a new decision — proposed here, not
  yet done, pending your go-ahead since you asked to review the
  recommendation before any action.

---

## 1. Contract Enumeration Audit

Full item-by-item comparison across all five sources you named. "≈" means
conceptually related but not textually identical; "—" means no
corresponding entry exists in that document.

| # (Adapter Contract, canonical per AC-13) | Adapter Contract §14 | Blueprint §10.7.6 | Verification Gates §66/67 | Phase Manifest P5 | Task Contracts P5 §39 |
|---|---|---|---|---|---|
| EC-01 | "identity" | ≈ "identity/capability discovery" (**combined** with EC-02's concern) | ≈ VG-P5-EXEC-01 "Adapter Contract" | P5-W02 (Capability Snapshot) | P5-W01-T01 "identity" |
| EC-02 | "capabilities" | ≈ (combined into Blueprint EC-01, above) | ≈ VG-P5-EXEC-01, VG-P5-EXEC-07 | P5-W02 | P5-W01-T01 "capabilities" |
| EC-03 | "workspace confinement" | "authorized workspace execution" (EC-02) **+** "forbidden-path rejection" (EC-03) — **two Blueprint items map to one Adapter Contract item** | VG-P5-EXEC-02 "Workspace Confinement" | P5-W03 | P5-W01-T01 "start" (partially) |
| EC-04 | "successful execution" | "successful execution evidence" (EC-04) | VG-P5-EXEC-03 "Lifecycle Observation" (**groups EC-04..EC-10 together — one VG gate for seven EC items**) | P5-W04 | P5-W01-T01 "observe" |
| EC-05 | "partial execution" | "partial execution evidence" (EC-05) | VG-P5-EXEC-03 | P5-W04 | — |
| EC-06 | "non-zero exit" | "non-zero exit" (EC-06) | VG-P5-EXEC-03 | P5-W04 | — |
| EC-07 | "timeout" | "timeout" (EC-07) | VG-P5-EXEC-03 | P5-W04 | — |
| EC-08 | "cancellation" | "cancellation" (EC-08) | VG-P5-EXEC-03 | P5-W04 | P5-W01-T01 "cancel" |
| EC-09 | "crash" | "crash / abrupt termination" (EC-09) | VG-P5-EXEC-03 | P5-W04 | — |
| EC-10 | "Unknown outcome" | "ambiguous outcome → Unknown" (EC-10) | VG-P5-EXEC-03, **also** VG-P5-EXEC-04 "Unknown Safety" (**one EC item feeds two VG gates**) | P5-W04 | — |
| EC-11 | "restart/reconciliation" | "restart/reconciliation" (EC-11) | VG-P5-EXEC-08 "Restart/Reconciliation" | P5-W05 | — |
| EC-12 | "evidence collection" | **—** (Blueprint's EC-12 is "provider failure propagation" instead — genuinely different requirement at the same number) | VG-P5-EXEC-09 "Evidence Provenance" (name suggests provenance specifically, narrower than "collection" generally) | P5-W02 (Capability Snapshot's provenance/freshness fields) | P5-W01-T01 "collect_evidence" |
| EC-13 | "credential redaction" | "credential-redaction boundary" (EC-13) | VG-P5-EXEC-09 (**shared with EC-12** — same VG gate covers two distinct EC items) | — (Blueprint §12 Credential Vault, cross-cutting) | — |
| EC-14 | "capability denial" | "capability denial at tool boundary" (EC-14) | VG-P5-EXEC-07 "Capability Enforcement" | — (P9 scope, not P5) | — |
| EC-15 | "no direct Succeeded" | "no direct Succeeded transition" (EC-15) | VG-P5-EXEC-05 "No Direct Succeeded" | Invariant 14 (cross-cutting) | — |
| EC-16 | "no direct merge" | "no direct merge authority" (EC-16) | VG-P5-EXEC-06 "No Direct Merge" | Invariant 13 (cross-cutting) | — |
| EC-17 | "executor replacement" | **—** (no corresponding Blueprint item at all) | VG-P5-EXEC-10 "Executor Reversibility" | P5-W08 | — |

### Findings, stated precisely as requested (not assuming same-numbered = equivalent)

1. **Blueprint EC-01 and EC-02 do not correspond 1:1 to Adapter Contract
   EC-01 and EC-02.** Blueprint's EC-01 already combines identity +
   capability discovery; Adapter Contract splits these into two items.
   This means "Blueprint's EC-01" and "Adapter Contract's EC-01" are
   **not the same requirement** despite the identical label — Blueprint's
   EC-01 is actually the union of Adapter Contract's EC-01 and EC-02.
2. **Blueprint EC-02+EC-03 collapse into Adapter Contract's single EC-03.**
   "Authorized workspace execution" and "forbidden-path rejection" are
   presented as two distinct Blueprint requirements; the Adapter Contract
   treats them as one ("workspace confinement"). AC-13's own claim that
   "confinement failing *is* a forbidden-path write" is a reasonable
   argument for why they can be merged, but it is an argument, not a
   textual equivalence — flagging this as an interpretive choice made in
   AC-13, not a fact independently verifiable from the documents alone.
3. **Adapter Contract EC-12 ("evidence collection") and Blueprint EC-12
   ("provider failure propagation") are genuinely different
   requirements**, not a numbering coincidence with the same content.
   AC-13's resolution ("folded into this item's failure-classification
   checks") is a real scope decision: it means **provider-failure
   propagation, specifically, is not independently checked by any
   currently-implemented function** — `check_evidence_collection` checks
   that evidence *fields are populated*, it does not check that a
   provider failure is *correctly classified and propagated* through
   those fields. **This is a real, currently-unfilled gap**, not merely a
   labeling difference — see the completeness table in Section 2, row
   EC-12.
4. **Verification Gates' VG-P5-EXEC-03 groups seven distinct EC items
   (EC-04 through EC-10) into one gate.** This means "VG-P5-EXEC-03
   passes" is a much coarser claim than "EC-07 (timeout) passes" — a
   report that says only "VG-P5-EXEC-03: PASS" without also reporting
   each of EC-04..EC-10 individually would hide exactly which lifecycle
   behaviors were actually proven. `ConformanceSuiteReport` in the current
   module does retain per-EC-item granularity (`BTreeMap<ConformanceItem,
   ConformanceResult>`), so this granularity is *not* lost in the current
   design — worth stating explicitly as a design property that must be
   preserved going forward, not discarded when a future summary view is
   built.
5. **VG-P5-EXEC-09 covers both EC-12 and EC-13**, which are legitimately
   related (both about evidence integrity) but still two separable
   requirements a single gate result could mask if reported only at the
   VG level.
6. **EC-17 has no Blueprint equivalent at all.** This is the strongest
   piece of evidence for AC-13's recommendation to adopt the 17-item list:
   dropping EC-17 would leave Phase Manifest P5-W08 and Adapter Contract
   §18's Replacement Principle with no corresponding conformance item.
7. **Task Contracts §39's P5-W01-T01 registry only explicitly names five
   of the seventeen items** (identity, capabilities, start, observe,
   cancel, collect_evidence — really six, matching P5-W01's "Must prove"
   list) — it does not enumerate all 17 by number at all. This is not a
   conflict so much as a **lower resolution**: Task Contracts describes P5
   work by adapter *method* (`start`, `observe`, `cancel`, ...), not by EC
   number, so it cannot be checked for numbering agreement in the same
   way the other four documents can. Recorded here so it isn't silently
   treated as "silent about a conflict" versus "uses a different
   organizing axis entirely."

---

## 2. Conformance Module Completeness Audit

Strict version, as instructed: "adapter reports timeout" ≠ "the executor
actually timed out and the Conductor correctly observed and handled it."
The **"What it does NOT prove"** column is the load-bearing one.

| EC | Requirement | Check fn | Input evidence | What it actually proves | What it does NOT prove | Synthetic possible? | Real executor required? | Current status | Gate |
|---|---|---|---|---|---|---|---|---|---|
| 01 | Identity discovery | `check_identity` | `ExecutorIdentity` struct | The adapter's `identity()` method returns a value with all required fields non-empty | That any field is *truthful* (a hostile/buggy adapter could report `version: "999.0"` and this passes); that the identity is stable across calls | Yes | No | Structural only | VG-P5-EXEC-01 |
| 02 | Capability discovery | `check_capabilities` | `CapabilitySnapshot` | `capabilities()` returns without panicking | Truthfulness of any individual capability value; that `Supported`/`Unsupported`/`Unknown` correctly reflects real behavior | Yes | No | Structural only (explicitly documented as such in the function's own `detail` string) | VG-P5-EXEC-01, 07 |
| 03 | Workspace confinement | `check_workspace_confinement` | Two `Baseline`s of the **live repo root** | The live repo's Git-tracked-and-untracked-**and**-gitignored regular-file tree is byte-identical before/after — a genuine, mechanically detected property | **Four specific gaps, each empirically confirmed this session (see §4):** (a) new symlinks created at previously-nonexistent paths are completely invisible; (b) writes that happen then are deleted before the "after" snapshot are invisible; (c) writes anywhere outside the given root path are structurally unobservable (by construction — not a bug, but must not be assumed covered); (d) `.gitignore`d paths ARE included, which is *stricter* than Blueprint §5.1's stated intent ("Conductor never manages ignored files") — meaning a real executor's legitimate write to e.g. a gitignored build-cache directory would currently register as a confinement **violation**, a false positive relative to Blueprint's own stated tolerance | Yes (proven both ways with real Git) | **No** — this check's core mechanism needs no real executor; it only needs *something* to run between two Baseline captures | Real Git, synthetic trigger | VG-P5-EXEC-02 |
| 04–10 | Lifecycle outcomes | `check_lifecycle_outcome` | Caller-supplied `Observation` + expected `ExecutionState` | That `observation.final_executor_state` matches (or mismatches) a caller-asserted expectation | **Everything about whether the underlying real-world event actually occurred.** This function cannot distinguish "the executor genuinely timed out and the adapter correctly reported `TimedOut`" from "the caller manually constructed an `Observation` claiming `TimedOut` and nothing real happened at all." See full breakdown in §5 | Yes, and **only exercised synthetically so far** | **Yes**, for any result that should count toward real conformance | Classification-only; zero real orchestration exists | VG-P5-EXEC-03, 04 |
| 11 | Restart/reconciliation | `check_restart_reconciliation` | Two `Baseline`s + a caller-supplied `bool` | That *some* diff exists (or doesn't) between two baselines, given a caller's honesty-based assertion that a restart genuinely happened | **Does not call `reconciliation::reconcile()` at all.** Produces no `ReconciliationOutcome`. Does not distinguish `ExpectedChange` from `UnexpectedChange` from `PartialChange`. Does not consult `IdempotencyStore` to verify a side effect wasn't blindly repeated. This is the single largest gap found in this audit — see §6 | Yes | Yes, for real meaning | Shallow diff-count only; **not integrated with P2's actual Reconciliation Engine** | VG-P5-EXEC-08 |
| 12 | Evidence collection | `check_evidence_collection` | `EvidenceRefs` | Four specific string fields are non-empty | Does not check `failure_classification` propagation at all (see AC-13 finding #3 above — this is where Blueprint's differently-scoped EC-12 "provider failure propagation" would need to live, and currently doesn't); does not check `commands`/`artifacts_ref` are non-empty (only the four identity fields) | Yes | No (structural check) | Partial — identity fields only | VG-P5-EXEC-09 |
| 13 | Credential redaction | `check_credential_redaction` | `Observation` + `EvidenceRefs` + caller-supplied marker list | That none of the **specific strings the caller already knows to look for** appear unredacted in the specific fields scanned | A credential the caller didn't think to list as a marker; a credential in a field not scanned (`EvidenceRefs.process_ref` is **not currently scanned** — a real gap, small but real); that redaction happened *inside* the adapter rather than merely "didn't happen to leak this time"; this is fundamentally a **known-marker scanner**, not a **secret-shape detector** (no entropy/pattern heuristics) | Yes | No for the mechanism; yes for a credential that would only exist with a real provider | Real mechanism, only exercised with synthetic markers | VG-P5-EXEC-09 |
| 14 | Capability denial | `check_capability_denial_not_applicable` | none | That no `capability_check`/`tool_boundary` module exists yet in this crate (confirmed by grep, current session) | Nothing about actual enforcement — correctly out of scope, not a gap | N/A | N/A | `NotApplicable` (correctly) | VG-P5-EXEC-07 |
| 15 | No direct Succeeded | `check_no_direct_succeeded_or_merge_authority` (first of pair) | `execution_adapter.rs`'s own source text | That the module's compiled-in source, filtered to non-comment lines, contains no reference to `crate::state::`/`AttemptStatus::Succeeded`/`use crate::state` | Nothing about `conformance.rs` itself, or any future `aider.rs`/`jcode.rs` adapter module, making the same violation — **this check only ever protects `execution_adapter.rs`**, not the crate as a whole | No (genuinely non-synthetic; checks source text, not adapter behavior) | No | Verified, narrow scope | VG-P5-EXEC-05 |
| 16 | No direct merge | (second of pair) | same | Same mechanism, for `crate::conflict::` | Same scope limitation as EC-15 | No | No | Verified, narrow scope | VG-P5-EXEC-06 |
| 17 | Executor replacement | `check_executor_replacement` | Two `FinalizationClaim`s + two `ExecutorIdentity`s | That the exact same generic calling code drives two differently-identified adapter instances to completion | That a **real** Aider adapter and a **real** Jcode adapter are interchangeable in practice (only ever tested with two synthetic instances so far); resource/performance/reliability differences between real executors, which is explicitly out of this item's scope per Adapter Contract §17 (separate from §18) | Yes, and this is the one item where synthetic testing is not just "the best available" but **structurally sufficient** for what EC-17 actually claims (a kernel-code property, not an executor-behavior property) | No | Fully proven for its actual scope | VG-P5-EXEC-10 |

### Cross-cutting completeness finding

Of 16 applicable items (EC-14 correctly excluded pre-P9), **only EC-15,
EC-16, and EC-17 are proven to a standard that does not need to change
when a real executor arrives** (EC-15/16 check source text; EC-17's claim
is inherently about calling code, not executor behavior). **The other 13
items are honestly self-labeled as classification/structural checks that
will require real orchestration machinery neither requested nor built this
session** — this matches what `P5-CONFORMANCE-FOUNDATION.md` already
says, and this audit did not find that report to have overstated anything.
If anything, this audit surfaces that EC-12 and EC-13 are *less* complete
than that report's framing suggested (EC-12 misses failure-classification
propagation entirely; EC-13 has an unscanned field).

---

## 3. Real vs. Synthetic Evidence Audit

Claims proven this session (all four via executed tests, then reverted):

- ✅ **Synthetic evidence can never become real conformance.**
  `disposition_impl()` caps at `SyntheticOnly` the moment any applicable
  result has `synthetic: true`, proven adversarially (all-green synthetic
  run still yields `SyntheticOnly`, never `RealConformancePassed` or
  `RealConformancePartial`) — this was already tested in the original
  session and re-confirmed, unchanged, this session.
- ✅ **Unknown/Blocked/NotApplicable cannot silently become Passed** — by
  construction, `disposition_impl()` treats any applicable item in
  `Blocked`/`Unknown` status as forcing the whole suite to `Blocked`,
  regardless of `passed`'s value on *other* items. `NotApplicable` items
  are correctly skipped rather than counted either way.
- ✅ **A missing observation cannot receive a favorable default** — a
  `ConformanceItem` with no entry in the `BTreeMap` is treated identically
  to `Blocked` in `disposition_impl()` (the `any_missing` branch), proven
  by the existing `suite_disposition_is_blocked_when_an_applicable_item_is_missing`
  test.
- ✅ **`ExternalEnvironmentFacts::unconfirmed()` defaults never fabricate**
  — all three environment fields default to `Unknown`, proven by the
  existing `readiness_assessment_defaults_never_fabricate_environment_facts`
  test.

### ⚠️ Found: a dangerous representable-but-invalid state (empirically confirmed)

**`ConformanceResult`'s fields are all `pub`, with no validating
constructor.** This makes the following compile and construct without
error or panic — proven this session:

```rust
ConformanceResult {
    item: ConformanceItem::Ec03WorkspaceConfinement,
    status: EvidenceStatus::Blocked,   // "cannot be checked yet"
    passed: Some(true),                //  ...but claims it passed
    synthetic: false,                  //  ...and claims real evidence
    detail: "...".to_string(),
}
```

`disposition_impl()` happens to still classify a suite containing this
value as `Blocked` (because its match arm checks `result.status` before
`result.passed`) — **but this is safe only because of the current
implementation's arm ordering, not because the type system forbids the
invalid value from existing.** A future refactor of `disposition_impl()`
that checked `passed` first, or a future function added elsewhere in this
module or a downstream consumer that reads `.passed` directly without
also checking `.status`, would be silently fooled. This was proven by
constructing exactly such a value and confirming the *downstream*
disposition logic depends on a specific field-check order to stay safe.

**Recommendation:** make `ConformanceResult`'s fields private, add a
`ConformanceResult::observed(item, passed, synthetic, detail)` /
`::blocked(item, detail)` / `::not_applicable(item, detail)` /
`::verified(item, passed, detail)` constructor set that makes the invalid
`Blocked`+`Some(_)` combination (and equivalently `Unknown`+`Some(_)`,
`NotApplicable`+`Some(_)`) unrepresentable — e.g. by having `passed` only
settable via the `observed`/`verified` constructors, which unconditionally
set `status` themselves rather than accepting it as a separate parameter.
This is a **type-design hardening**, not a behavior change (every existing
call site already only produces valid combinations) — flagged, not fixed,
per this round's audit-only scope; would be a small, mechanical, low-risk
follow-up.

### Irreducible limitation (not a bug, stated for the record)

**Nothing can prevent a caller from asserting `ExecutorProvenance::Real`
for what is actually a `FakeExecutionAdapter`, or vice versa.** The trait
object `&mut dyn ExecutionAdapter` is structurally identical whether it's
real or fake — `ExecutorProvenance` is caller-supplied precisely *because*
the module correctly refuses to infer this from self-reported identity
(which a hostile or buggy adapter could lie about equally well). This
means the entire real/synthetic distinction rests on **caller honesty**,
which is an operational/process control (code review of whatever P5-W03
code eventually constructs a `Real` provenance value), not something this
module's types can enforce by themselves. This is stated as an accepted,
irreducible limitation — flagging it does not imply a fix exists.

---

## 4. EC-03 Workspace Confinement Audit

Answering each sub-question directly, with `(executed)` or `(by
construction)` markers:

- **What baseline is captured?** A full content-hash map of every regular
  file under the given root, via `Baseline::capture()`. *(by
  construction — `baseline.rs:81-145`)*
- **When?** Exactly when the caller calls `check_workspace_confinement`
  with `before`/`after` — the function itself does not decide timing; a
  real P5-W03 orchestration would need to capture `before` immediately
  prior to `adapter.start()` and `after` immediately after
  `adapter.finalize()`/`observe()` settles. *(by construction)*
- **What paths are included?** Every regular file under the root except
  `.git` (directory or gitlink file) at the root. *(by construction, and
  the existing `dot_git_directory_is_excluded` /
  `dot_git_as_a_gitlink_file_is_also_excluded` tests in `baseline.rs`
  already prove the `.git` exclusion specifically)*
- **Are untracked files detected?** **Yes** — `Baseline` walks the
  filesystem directly, not `git status`; it has no concept of "tracked"
  vs. "untracked" at all. *(by construction)*
- **Are deleted files detected?** Yes — surfaces as `PathChange::Removed`.
  *(executed, via the pre-existing `diff_reports_modified_added_and_removed_paths`
  test, re-confirmed passing this session)*
- **Are modified files detected?** Yes — `PathChange::Modified`. *(same
  test)*
- **Do ignored (`.gitignore`) files matter?** **Yes, and this is a real
  finding.** `Baseline` has zero `.gitignore` awareness. A file under a
  path like `node_modules/` is captured identically to any other file.
  *(executed this session:
  `scratch_gitignored_style_path_is_still_captured_by_baseline` — wrote a
  file under `node_modules/` with a `.gitignore` excluding it, confirmed
  `Baseline` captured it anyway)*. This means EC-03 is **stricter** than
  Blueprint §5.1's stated intent ("File matches `.gitignore`" → "Excluded
  from baseline comparison and from the merge entirely — the Conductor
  never manages ignored files"). A real executor's legitimate write to a
  gitignored path (e.g., a build tool regenerating `dist/` or
  `target/`) would currently be flagged as an EC-03 **violation** — a
  false positive relative to Blueprint's own tolerance. **This is a
  genuine terminology/scope mismatch, not a security bug** (over-strict
  is safer than under-strict), but it will produce confusing false
  failures against a real executor unless corrected before real
  certification.
- **Are files outside the repository detectable?** **No, by
  construction** — `Baseline::capture` takes exactly one root path
  argument and only walks beneath it; a write anywhere else is
  structurally invisible to this mechanism. *(confirmed this session via
  `scratch_writes_entirely_outside_the_captured_root_are_structurally_unobservable`,
  included for an executed assertion rather than resting on reading the
  function signature alone)*. **EC-03's current name, "workspace
  confinement," should therefore be understood/documented precisely as
  "the live repository tree is unchanged," not "the executor made no
  filesystem writes anywhere on the machine."** The distinction matters:
  a real executor that (for example) wrote a stray file to `/tmp` or
  modified something in its own working directory unrelated to any
  repository would pass EC-03 while still having done something a
  stricter reading of "confinement" might want caught. Recommend a
  documentation correction (not a mechanism change — that would require
  deciding what "the whole machine" even means, out of scope here) to
  make this precise.
- **Symlinks / path traversal escape?** `Baseline::capture` never follows
  symlinks (confirmed by code and by the pre-existing design comment at
  `baseline.rs:76-80`), which prevents the *walker itself* from escaping
  the root via a symlink target. **However, a genuinely new finding this
  session: a symlink freshly created at a path with no prior entry is
  completely invisible to `Baseline::diff` in either direction** —
  neither `before` nor `after` ever records it, so it never appears as
  `Added`, `Removed`, or `Modified`. *(executed:
  `scratch_new_symlink_creation_at_a_previously_nonexistent_path_is_invisible_to_diff`
  — created `new_link -> real_target.txt`, confirmed the diff was
  completely empty)*. Since a symlink could point outside the live
  repository entirely, **this is the most significant EC-03 gap found**:
  a misbehaving executor could plant a symlink inside the live repo
  pointing at an arbitrary external location with zero detection by this
  mechanism, as distinct from modifying an *existing* file (which, if it
  gets replaced by a symlink, correctly shows up as `Removed` — only
  *new* symlink paths are invisible).
- **Transient mutation that disappears before the final scan?** **Yes,
  invisible** — proven by
  `scratch_transient_write_then_delete_between_snapshots_is_invisible`
  (write a file, then delete it, before the "after" capture — empty
  diff). This is an inherent limitation of two-point-in-time snapshot
  diffing, not something fixable without continuous filesystem
  observation (e.g. `inotify`/`fsevents`), which is a materially larger
  mechanism than this session's scope and not something to build
  speculatively.
- **Does the mechanism prove "live workspace untouched," or only a
  Git-visible subset?** **Neither, precisely.** It proves something
  *broader* than "Git-visible" (since it includes gitignored/untracked
  files, unlike a `git status`-based check would) but *narrower* than "the
  live workspace" in the fullest sense (misses new symlinks, misses
  transient changes, is scoped to one given root only). The current
  module doc's phrase "the live repository's own Baseline" is accurate;
  a reader relying on the term "confinement" alone, without reading the
  implementation, could reasonably over-trust it in the three ways
  identified above.

### Recommendation for EC-03

Correct the documentation (in `conformance.rs`'s doc comment for
`check_workspace_confinement`, and this item's row wherever it's
summarized externally) to state the three limits explicitly: (1) gitignored
paths ARE included, a stricter-than-Blueprint behavior that will cause
false-positive violations against a real executor writing to legitimately
ignored paths; (2) newly-created symlinks are invisible; (3) transient
mutations between snapshots are invisible. Not proposing a mechanism
change this round (audit-only) — flagging for a scoped follow-up decision.

---

## 5. EC-04 through EC-10 Lifecycle Audit

Answering the eight questions per item as instructed. Since the answers to
questions 1–2 and 4–8 are structurally identical in *shape* across all
seven items (only the specific expected `ExecutionState` differs), they're
given once with per-item specifics called out; question 3 differs more
meaningfully per item and is broken out fully.

**1. What real event must occur?** A real subprocess (or whatever
mechanism a real Aider/Jcode adapter uses) must genuinely: complete
normally (EC-04); complete having applied only some of the intended change
(EC-05); exit with a non-zero code (EC-06); exceed its allotted time
budget (EC-07); receive and honor (or fail to honor) a cancellation
request (EC-08); terminate abnormally — segfault, killed, panicked (EC-09);
or terminate in a way that cannot be confidently classified as any of the
above (EC-10).

**2. What must the adapter return?** `Observation.final_executor_state`
set to the matching `ExecutionState` variant, ideally corroborated by
`exit_information`/`stdout_or_structured_events` — but **`check_lifecycle_outcome`
only ever reads `final_executor_state`**; it does not cross-check that
field against `exit_information` for consistency. A real adapter that
reported `final_executor_state: Completed` but `exit_information: "exit
137"` (a SIGKILL exit code) would currently pass EC-04 uncontested. This is
a gap: **no internal-consistency check exists between `Observation`'s own
fields.**

**3. What must Conductor independently observe?**
- EC-04/05/06: the actual file/workspace state, via `Baseline` diffing —
  mechanism exists (`check_workspace_confinement`'s machinery could be
  reused here, but currently isn't wired to `check_lifecycle_outcome` at
  all — they're independent functions with no cross-referencing).
- EC-07 (timeout): independent observation requires the Conductor's *own*
  wall-clock (or, per Blueprint §10.2, injected `Clock`) measurement of
  elapsed time against the `timeout_policy` in `ExecutionRequest` —
  **no such measurement exists anywhere in `conformance.rs`**; the
  function trusts the caller's assertion of what "expected" means.
- EC-08 (cancellation): independent observation of actual process
  termination (OS-level: is the PID still alive?) — **not implemented
  anywhere in this crate yet**; `CancellationReport.observed_termination_state`
  is a field on the trait's return type, but nothing in `conformance.rs`
  independently re-verifies it against the OS.
- EC-09 (crash): same OS-level observation gap as EC-08.
- EC-10 (Unknown): requires the *absence* of a confident classification
  from *both* the adapter's own report *and* Conductor's independent
  checks — this is the one item where "the adapter genuinely doesn't know
  either" and "the harness couldn't determine it" are meant to coincide,
  and no logic currently enforces that they must (a caller could report
  `Unknown` while independently corroborated evidence would have supported
  `Failed`, and nothing catches that).

**4. What evidence must be persisted?** Per Blueprint §4.9/§4.4: an event
in the tamper-evident event chain (`event_log.rs`) recording the attempt's
lifecycle transition, plus the `Evidence` record's provenance/freshness
fields (Blueprint §4.5, `evidence.rs`). **`conformance.rs` does not write
to `event_log.rs` at all** — `ConformanceResult`s are in-memory values
returned to whatever calls the check functions; nothing persists a
conformance run as an event. This matches this session's stated scope
(structural/synthetic-only foundation) but is worth stating plainly: **no
conformance run currently produces durable, auditable evidence** the way
every other accepted P1–P4 mechanism in this crate does (console.rs,
person_decision.rs both write through `EventLog`, per the prior session's
P5-W02 work).

**5. What state transition must occur?** Per Invariant 14/§7:
`AttemptStatus::Unknown → {Succeeded | Failed}` only via the (not-yet-built
in this crate) Verification Engine's full pipeline — `verification_engine.rs`
exists but its integration with a real executor's lifecycle result is
P5/P6-adjacent work not yet wired. `conformance.rs` correctly never
touches `AttemptStatus` (proven mechanically by EC-15's own check), which
is right for *this* module, but means **no code path currently exists
connecting "EC-04 passed" to an actual `AttemptStatus` transition** —
that connective tissue is P5-W03/W04 implementation work, appropriately
not built yet.

**6. What would distinguish FAILED from UNKNOWN?** Per
`reconciliation.rs`'s existing four-outcome logic (`NoChange`/
`ExpectedChange`/`UnexpectedChange`/`PartialChange`) plus Blueprint §9's
`FailureCategory` (`FailedClean`/`FailedWithChanges`/`FailedVerification`/
`FailedInfrastructure`) — **none of this is consulted by
`check_lifecycle_outcome`**, which only compares two `ExecutionState`
enum values directly. A real "was this actually Failed, or should it be
Unknown pending reconciliation" determination requires calling
`reconciliation::reconcile()` with the real before/after baselines, which
`check_lifecycle_outcome` does not do (nor is it wired to `check_workspace_confinement`,
which does have the baselines).

**7. How would a false executor claim be detected?** Currently: **it
would not be**, beyond the EC-13 credential scan and EC-03's independent
Baseline diff (which are the only two checks that observe anything the
adapter itself didn't directly report). `check_lifecycle_outcome` takes
the adapter's `Observation` at face value for the one field it checks —
this is stated honestly in the function's own doc comment ("does not — and
structurally cannot — manufacture the underlying scenario... only honestly
evaluates what `observe()` returned") but bears restating here plainly:
**detecting a lying/buggy adapter's lifecycle claim requires
cross-referencing against Baseline diffs and/or OS-level process
observation, neither of which is wired in yet.**

**8. What test fixture or orchestration is required?** For a real
executor: (a) a real subprocess-spawning adapter (`aider.rs`/`jcode.rs`,
explicitly not built this session per instruction); (b) a controlled test
task with a genuinely reproducible failure mode for each of EC-06 through
EC-09 (e.g., a task designed to exceed a short timeout, a task that sends
SIGKILL to the child, a task with a deliberately malformed command to
force non-zero exit); (c) OS-level process liveness checking
(`std::process::Child::try_wait()`, PID existence checks) that does not
exist in this crate yet; (d) wiring `check_lifecycle_outcome`'s result
into `check_workspace_confinement`'s baseline pair and into
`reconciliation::reconcile()`, none of which currently happens.

### Explicit statement, as instructed

**The current module only classifies caller-supplied observations for
EC-04 through EC-10.** It provides the vocabulary (`ConformanceItem`,
`EvidenceStatus`, honest `Blocked` defaults) and the discipline (never
fabricate a scenario) that real orchestration will need to report *into*,
but it does not itself contain any process-spawning, timing, or OS-level
observation machinery. No fake simulations were added to make these items
appear more complete than this — consistent with this round's explicit
instruction not to do so.

---

## 6. Restart / Reconciliation Audit (EC-11)

Tracing the required chain against the actual P2/P4 architecture:

```
REAL EXECUTION
  → INTERRUPTION
  → PROCESS DISAPPEARS
  → WORKSPACE MAY HAVE CHANGED
  → CONDUCTOR RESTARTS
  → ACTUAL STATE IS RECONCILED        <-- reconciliation::reconcile()
  → DUPLICATE SIDE EFFECT NOT REPEATED <-- IdempotencyStore::has_already_applied()
```

**Does `check_restart_reconciliation` reach either of the two marked
steps? No, to both, confirmed by reading `reconciliation.rs` and
`idempotency_store.rs`'s actual public APIs this session:**

- `reconciliation::reconcile(before: &Baseline, after: &Baseline, expected:
  &ExpectedOperation) -> ReconciliationOutcome` exists, is pure, and is
  exactly the function this scenario needs — **`check_restart_reconciliation`
  never calls it.** It instead does its own ad hoc diff-and-count directly
  on the two `Baseline`s, discarding the `Baseline::diff`'s
  `PathChange`-level detail into a bare count of `Modified` entries, and
  reports a generic "reconciliation ran, N paths differ" — not one of
  `NoChange`/`ExpectedChange`/`UnexpectedChange`/`PartialChange`.
- `IdempotencyStore::has_already_applied(key) -> Result<bool, ...>` exists
  in `idempotency_store.rs` — **`check_restart_reconciliation` never
  references it at all.** There is currently no way for this conformance
  check to prove "a duplicate side effect was not blindly repeated,"
  despite that being an explicit, named requirement in your instructions
  for this section and a core project invariant (Invariant 3/15/16).

### The exact missing bridge

`check_restart_reconciliation` needs to be rewritten (not this round —
audit only) to:

1. Accept an `ExpectedOperation` (already exists, `reconciliation.rs`) and
   call `reconciliation::reconcile(before, after, expected)` instead of
   doing its own diff-and-count.
2. Assert the resulting `ReconciliationOutcome` is one of the four
   currently-reachable variants and is *consistent* with what the
   real-executor scenario was trying to prove (e.g., a restart-after-
   partial-edit scenario should assert specifically `PartialChange`, not
   merely "some outcome was produced").
3. Accept an idempotency key and consult `IdempotencyStore::has_already_applied`
   both before and after the simulated/real restart, asserting the
   post-restart continuation path did not re-invoke a side effect the
   store already knows completed.

**This is exactly the kind of structural gap this audit was commissioned
to find before P5-W05 begins for real** — implementing P5-W05 against the
current shallow `check_restart_reconciliation` would have produced a
conformance item that looks green while never having exercised the real
Reconciliation Engine at all. Correctly not fixed this round (would be
implementing P5-W05-adjacent logic prematurely, which this round's
instructions explicitly forbid) — recorded as the single most
consequential finding of this audit.

---

## 7. EC-12 / EC-13 Evidence Audit (partial — your message was cut off here)

Answering what was specified before the cutoff:

- **stdout:** covered — `Observation.stdout_or_structured_events` is
  scanned by `check_credential_redaction`.
- **stderr:** **not separately representable at all.** Neither the
  Adapter Contract's `Observation` type nor this crate's
  `execution_adapter.rs` implementation has a distinct stderr field —
  everything is folded into the single `stdout_or_structured_events`
  string by convention. If a real adapter keeps stderr genuinely separate
  internally and only surfaces stdout through this field, **a credential
  leaked exclusively via stderr would never reach this scanner at all.**
  This is an upstream type-design question (`execution_adapter.rs`'s
  `Observation` shape), not something fixable inside `conformance.rs`
  alone.
- **structured adapter output:** covered by the same
  `stdout_or_structured_events` field, plus `tool_activity` and
  `changed_workspace_observations` — but only insofar as a real adapter
  actually puts structured output into one of those three fields; there's
  no guarantee a real adapter's structured events (e.g. a JSON event
  stream) would land there rather than in some adapter-specific side
  channel this harness has no visibility into.
- **error (as far as the sentence goes):** `Observation.exit_information`
  and `EvidenceRefs.exit_result` are both scanned. **`EvidenceRefs.process_ref`
  is not scanned** — confirmed by re-reading `check_credential_redaction`'s
  haystack list this session — a small, concrete, fixable gap (a one-line
  addition, not attempted this round per audit-only scope).

Your message ends mid-sentence after "- error". I have not guessed at
what came after it. Send the rest and I'll extend this section (and any
further numbered sections you had planned) without re-doing 1–6.

---

## Summary of findings requiring a decision

1. **AC-13 resolution** — recommendation given above (C+B hybrid); awaiting
   your confirmation before producing the standalone mapping document.
2. **`ConformanceResult` type hardening** (Section 3) — recommend private
   fields + validating constructors; not yet done.
3. **EC-03 documentation correction** (Sections 2 & 4) — three specific
   overstatements to fix in the doc comment; not yet done.
4. **EC-11/`check_restart_reconciliation` rewrite** (Section 6) — the
   most consequential finding; needs to actually call
   `reconciliation::reconcile()` and `IdempotencyStore`; explicitly not
   attempted this round (would be premature P5-W05 implementation).
5. **EC-12 gap**: no check for `failure_classification` propagation
   (Section 2, Section 1 finding #3).
6. **EC-13 gap**: `EvidenceRefs.process_ref` unscanned (Section 2, Section
   7); stderr has no dedicated field anywhere upstream (Section 7).
7. **EC-04..EC-10**: confirmed classification-only, no independent
   OS-level or Baseline-cross-referenced observation exists yet (Section
   5) — expected and previously disclosed, not a new problem, but now
   precisely enumerated per-question as instructed.

No gate was weakened to produce this report. No code was committed. 286/286
tests, clean tree, `Cargo.lock` unchanged, confirmed immediately before
writing this document.
