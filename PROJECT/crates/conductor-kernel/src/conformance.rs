//! Execution Adapter Conformance harness — P5 real-executor foundation.
//!
//! Builds the *minimum* deterministic, reusable infrastructure needed to
//! test any real [`crate::execution_adapter::ExecutionAdapter`] (Aider,
//! Jcode, or a future harness) against the same contract, without giving
//! either executor special treatment — Execution Adapter Contract v1.0
//! §17 (Selection) and §18 (Replacement Principle), Phase Manifest §9.3
//! (P5-W03/W04/W05), Verification Gates §66/§67.
//!
//! # Numbering conflict, resolved provisionally
//!
//! The governing documents contain **three non-identical** enumerations
//! of "EC-xx" conformance items (Blueprint §10.7.6: 16 items; Execution
//! Adapter Contract §14: 17 items; Verification Gates §66: 12
//! differently-named gates). They diverge starting at EC-01/EC-02.
//! **Canonical cross-reference (kept up to date independently of this
//! module): `GOVERNANCE/EC_ENUMERATION_MAPPING.md`.** That document is
//! explicitly marked PROVISIONAL and records the two genuine mismatches
//! (EC-12, EC-17) found so far — it is the authoritative place to look,
//! not this comment, which only summarizes. The historical discovery
//! record is `.ai-conductor-build/architecture_changes/AC-13-execution-conformance-enumeration-conflict.md`.
//! This module proceeds using the **Adapter Contract v1.0 §14
//! EC-01..EC-17** enumeration as canonical (most granular; the dedicated
//! normative document for this exact boundary) — provisionally, pending
//! explicit human confirmation. **No real-executor certification
//! produced by this module may be cited as evidence that the numbering
//! disagreement itself has been permanently resolved.**
//!
//! # The hard architectural rule this module exists to uphold
//!
//! Execution Adapter Contract §11: "An executor adapter MUST NOT... set
//! `Succeeded` directly... authorize merge... silently resolve
//! Conflict." This harness can *test* an executor. It must never become
//! an alternative authority to the State Engine (`state.rs`) or the
//! Verification Engine (`verification_engine.rs`). Structurally enforced
//! the same way `execution_adapter.rs` already enforces it: this file has
//! zero references to `crate::state` or `crate::conflict`, verified
//! mechanically in this module's own tests (not just claimed in prose) by
//! scanning `execution_adapter.rs`'s actual source text for the specific
//! symbols that would constitute a violation (`self::tests::ec15_*`).
//! A `ConformanceResult` is evidence *about* an adapter; it is never
//! itself an `AttemptStatus` or a merge authorization, and nothing in
//! this module can construct one.
//!
//! # Synthetic vs. real evidence — the rule that must never be violated
//!
//! [`ExecutorProvenance::Synthetic`] (a [`crate::execution_adapter::FakeExecutionAdapter`]
//! or any other test double) proves the *harness* works. It can never
//! satisfy a real-executor gate. This is not a comment developers are
//! trusted to remember — [`ConformanceSuiteReport::disposition`] is
//! computed by a pure function that caps the result at
//! [`SuiteDisposition::SyntheticOnly`] the moment **any** result in the
//! suite has `synthetic() == true`, regardless of how many items
//! otherwise passed. There is no code path that produces
//! [`SuiteDisposition::RealConformancePassed`] from synthetic evidence.
//!
//! # `ConformanceResult` construction is a closed set (hardened this pass)
//!
//! A forensic audit of the first version of this module found that
//! `ConformanceResult`'s fields were all `pub`, which made logically
//! impossible combinations (e.g. `status: Blocked` carrying
//! `passed: Some(true)`) compile and construct without error — safe only
//! by accident of match-arm ordering in [`ConformanceSuiteReport::disposition`],
//! not by any actual type guarantee. All fields are now private; the
//! only way to build a [`ConformanceResult`] from outside this module is
//! through [`ConformanceResult::observed`], [`ConformanceResult::verified`],
//! [`ConformanceResult::blocked`], [`ConformanceResult::unknown`], or
//! [`ConformanceResult::not_applicable`] — each of which hard-codes its
//! own `status`/`passed` combination in its signature, so there is no
//! parameter list through which a caller could ask for `Blocked` *and*
//! supply a `passed` value at all. (Rust's module-privacy rules mean this
//! module's own child `tests` submodule retains access to the private
//! fields; the hardening's actual guarantee is against every other
//! module in this crate and any future `aider.rs`/`jcode.rs` adapter
//! module, which is the boundary that matters — this module's own test
//! code exercises only the public constructor surface below, by choice,
//! not by compiler force, to prove that surface alone is sufficient.)

use crate::baseline::{Baseline, PathChange};
use crate::execution_adapter::{
    CapabilitySnapshot, EvidenceRefs, ExecutionState, ExecutorIdentity, FinalizationClaim,
    Observation,
};
use crate::worktree::AttemptWorkspace;
use std::collections::BTreeMap;
use std::path::Path;

/// Execution Adapter Contract §14, canonical per this module's docs (see
/// `GOVERNANCE/EC_ENUMERATION_MAPPING.md` for why, and for the full
/// cross-reference). Each variant's doc comment records a short pointer;
/// the mapping document is authoritative, this comment is a summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConformanceItem {
    /// Feeds VG-P5-EXEC-01. See mapping doc: one-to-many with Blueprint
    /// EC-01 (which combines this with EC-02).
    Ec01Identity,
    /// Feeds VG-P5-EXEC-01, VG-P5-EXEC-07. Part of the same one-to-many
    /// split as EC-01.
    Ec02Capabilities,
    /// Feeds VG-P5-EXEC-02. Many-to-one: folds Blueprint's EC-02
    /// ("authorized workspace execution") and EC-03 ("forbidden-path
    /// rejection") into one item.
    Ec03WorkspaceConfinement,
    /// Feeds VG-P5-EXEC-03 (one of seven EC items grouped into that gate).
    Ec04SuccessfulExecution,
    /// Feeds VG-P5-EXEC-03.
    Ec05PartialExecution,
    /// Feeds VG-P5-EXEC-03.
    Ec06NonZeroExit,
    /// Feeds VG-P5-EXEC-03.
    Ec07Timeout,
    /// Feeds VG-P5-EXEC-03.
    Ec08Cancellation,
    /// Feeds VG-P5-EXEC-03.
    Ec09Crash,
    /// Feeds VG-P5-EXEC-03 and VG-P5-EXEC-04 (one-to-many into VG).
    Ec10UnknownOutcome,
    /// Feeds VG-P5-EXEC-08.
    Ec11RestartReconciliation,
    /// Feeds VG-P5-EXEC-09 (shared with EC-13). **Genuine mismatch with
    /// Blueprint's differently-scoped EC-12 ("provider failure
    /// propagation") — see `GOVERNANCE/EC_ENUMERATION_MAPPING.md`'s
    /// "EC-12 mismatch" section, not this comment, for the full,
    /// current-status-accurate account.**
    Ec12EvidenceCollection,
    /// Feeds VG-P5-EXEC-09 (shared with EC-12).
    Ec13CredentialRedaction,
    /// Feeds VG-P5-EXEC-07. Out of scope for THIS module (see
    /// [`ConformanceItem::is_applicable_before_p9`]) — real
    /// tool-boundary enforcement is P9 gstack Skill Runtime work.
    Ec14CapabilityDenial,
    /// Feeds VG-P5-EXEC-05.
    Ec15NoDirectSucceeded,
    /// Feeds VG-P5-EXEC-06.
    Ec16NoDirectMerge,
    /// Feeds VG-P5-EXEC-10. **Genuinely absent from the Blueprint's
    /// 16-item list entirely** — see the mapping doc's "EC-17 absence"
    /// section.
    Ec17ExecutorReplacement,
}

impl ConformanceItem {
    pub fn all() -> [ConformanceItem; 17] {
        use ConformanceItem::*;
        [
            Ec01Identity,
            Ec02Capabilities,
            Ec03WorkspaceConfinement,
            Ec04SuccessfulExecution,
            Ec05PartialExecution,
            Ec06NonZeroExit,
            Ec07Timeout,
            Ec08Cancellation,
            Ec09Crash,
            Ec10UnknownOutcome,
            Ec11RestartReconciliation,
            Ec12EvidenceCollection,
            Ec13CredentialRedaction,
            Ec14CapabilityDenial,
            Ec15NoDirectSucceeded,
            Ec16NoDirectMerge,
            Ec17ExecutorReplacement,
        ]
    }

    pub fn code(&self) -> &'static str {
        use ConformanceItem::*;
        match self {
            Ec01Identity => "EC-01",
            Ec02Capabilities => "EC-02",
            Ec03WorkspaceConfinement => "EC-03",
            Ec04SuccessfulExecution => "EC-04",
            Ec05PartialExecution => "EC-05",
            Ec06NonZeroExit => "EC-06",
            Ec07Timeout => "EC-07",
            Ec08Cancellation => "EC-08",
            Ec09Crash => "EC-09",
            Ec10UnknownOutcome => "EC-10",
            Ec11RestartReconciliation => "EC-11",
            Ec12EvidenceCollection => "EC-12",
            Ec13CredentialRedaction => "EC-13",
            Ec14CapabilityDenial => "EC-14",
            Ec15NoDirectSucceeded => "EC-15",
            Ec16NoDirectMerge => "EC-16",
            Ec17ExecutorReplacement => "EC-17",
        }
    }

    /// P9 (gstack Skill Runtime / tool-execution-boundary enforcement,
    /// Blueprint §16) does not exist in the repository yet — confirmed by
    /// this crate's own module list (`lib.rs`) and by grep for
    /// `capability_check`/`tool_boundary` returning zero matches at the
    /// time this module was written. EC-14 is therefore structurally
    /// `NotApplicable` to this harness until that phase exists, not a gap
    /// in this module.
    pub fn is_applicable_before_p9(&self) -> bool {
        !matches!(self, ConformanceItem::Ec14CapabilityDenial)
    }
}

/// Epistemic status of a single check — deliberately not a plain
/// pass/fail bool. Mirrors the project-wide evidence vocabulary (Build
/// Protocol §12, Verification Gates §2) rather than inventing a fourth
/// one for this module specifically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceStatus {
    /// Directly observed by this harness against the adapter/workspace
    /// actually in front of it right now.
    Observed,
    /// Confirmed by a reproducible, re-runnable check (the structural
    /// EC-15/EC-16 source-text scan below is the one example currently
    /// implemented this way).
    Verified,
    /// The harness could not determine this — never silently promoted to
    /// Observed/Verified.
    Unknown,
    /// A genuine external dependency prevents this item from being
    /// checked at all right now (e.g. no real executor present).
    Blocked,
    /// This item does not apply at the current phase (EC-14 pre-P9) or
    /// to the current executor kind.
    NotApplicable,
}

/// The result of checking one [`ConformanceItem`] once, against one
/// adapter, once.
///
/// **Construction is closed** — see the module-level docs' "construction
/// is a closed set" section. There is no public way to build one of
/// these except through the five named constructors below, each of
/// which fixes its own valid `status`/`passed` combination.
#[derive(Debug, Clone)]
pub struct ConformanceResult {
    item: ConformanceItem,
    status: EvidenceStatus,
    passed: Option<bool>,
    synthetic: bool,
    detail: String,
}

impl ConformanceResult {
    pub fn item(&self) -> ConformanceItem {
        self.item
    }

    pub fn status(&self) -> EvidenceStatus {
        self.status
    }

    /// `None` when `status()` is `Unknown`/`Blocked`/`NotApplicable` (there
    /// is nothing to have passed or failed yet) — and, after this pass's
    /// hardening, that is no longer merely a documented convention: it is
    /// impossible to construct any other combination through the public
    /// API. `Some(false)` is a real, important, non-erased outcome — a
    /// *failed* conformance check, distinct from one that was never run.
    pub fn passed(&self) -> Option<bool> {
        self.passed
    }

    /// `true` if produced against a test double (`FakeExecutionAdapter`
    /// or equivalent). Every constructor requires the caller to state
    /// this explicitly (except [`ConformanceResult::not_applicable`],
    /// which never depends on an adapter at all — see that
    /// constructor's own doc comment).
    pub fn synthetic(&self) -> bool {
        self.synthetic
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// The only way to build a `status: Observed` result. `status` and
    /// the fact that `passed` is `Some(_)` are both fixed by this
    /// constructor's signature — a caller cannot ask for `Observed` with
    /// no `passed` value, nor for a different status with one.
    pub fn observed(
        item: ConformanceItem,
        passed: bool,
        synthetic: bool,
        detail: impl Into<String>,
    ) -> Self {
        ConformanceResult {
            item,
            status: EvidenceStatus::Observed,
            passed: Some(passed),
            synthetic,
            detail: detail.into(),
        }
    }

    /// The only way to build a `status: Verified` result. Same guarantee
    /// as [`ConformanceResult::observed`], for the stronger evidence
    /// class.
    pub fn verified(
        item: ConformanceItem,
        passed: bool,
        synthetic: bool,
        detail: impl Into<String>,
    ) -> Self {
        ConformanceResult {
            item,
            status: EvidenceStatus::Verified,
            passed: Some(passed),
            synthetic,
            detail: detail.into(),
        }
    }

    /// The only way to build a `status: Blocked` result. There is no
    /// `passed` parameter at all — `Blocked` + any `passed` value is
    /// unrepresentable, not merely undocumented.
    pub fn blocked(item: ConformanceItem, synthetic: bool, detail: impl Into<String>) -> Self {
        ConformanceResult {
            item,
            status: EvidenceStatus::Blocked,
            passed: None,
            synthetic,
            detail: detail.into(),
        }
    }

    /// The only way to build a `status: Unknown` result. Same guarantee
    /// as [`ConformanceResult::blocked`]: no `passed` parameter exists.
    pub fn unknown(item: ConformanceItem, synthetic: bool, detail: impl Into<String>) -> Self {
        ConformanceResult {
            item,
            status: EvidenceStatus::Unknown,
            passed: None,
            synthetic,
            detail: detail.into(),
        }
    }

    /// The only way to build a `status: NotApplicable` result.
    /// Deliberately takes **no** `synthetic` parameter, unlike the other
    /// four constructors: every current use of `NotApplicable` (EC-14,
    /// pre-P9) never looks at an adapter at all, so "was a test double
    /// involved" has no meaning to fix one way or the other, and `false`
    /// is recorded as the honest, non-adapter-dependent default rather
    /// than a guess. If a future `NotApplicable` case genuinely does
    /// depend on which adapter was used, add a `synthetic` parameter to
    /// this constructor then — do not synthesize a value for the current
    /// cases, which don't need one.
    pub fn not_applicable(item: ConformanceItem, detail: impl Into<String>) -> Self {
        ConformanceResult {
            item,
            status: EvidenceStatus::NotApplicable,
            passed: None,
            synthetic: false,
            detail: detail.into(),
        }
    }
}

/// What kind of adapter produced a batch of results. The harness never
/// infers this from the adapter's reported `ExecutorIdentity` (a real
/// adapter's identity fields are exactly the kind of thing a broken or
/// dishonest adapter could misreport) — the *caller* states it, because
/// the caller is the one who constructed the adapter and genuinely knows.
///
/// **Irreducible limitation, stated for the record (forensic audit
/// finding):** nothing in this module can prevent a caller from
/// asserting `Real` for what is actually a `FakeExecutionAdapter`, or
/// vice versa — `&mut dyn ExecutionAdapter` is structurally identical
/// either way. This rests on caller honesty (an operational/process
/// control, e.g. code review of whatever P5-W03 code eventually
/// constructs a `Real` value), not something this module's types can
/// enforce by themselves. Flagging this is not the same as having fixed
/// it; no fix is claimed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorProvenance {
    Synthetic { double_kind: String },
    Real { name: String, version: String },
}

impl ExecutorProvenance {
    fn is_synthetic(&self) -> bool {
        matches!(self, ExecutorProvenance::Synthetic { .. })
    }
}

// ---------------------------------------------------------------------
// Individual mechanical checks
// ---------------------------------------------------------------------

/// EC-01. Can only confirm the adapter *reports* an identity, and that
/// the required fields are non-empty — it cannot verify the reported
/// version/commit is actually truthful (that would require independent
/// ground truth this harness doesn't have access to for any given
/// executor binary).
pub fn check_identity(
    identity: &ExecutorIdentity,
    provenance: &ExecutorProvenance,
) -> ConformanceResult {
    let populated = !identity.executor_id.is_empty()
        && !identity.name.is_empty()
        && !identity.version.is_empty()
        && !identity.adapter_version.is_empty()
        && !identity.runtime_environment_identity.is_empty();
    ConformanceResult::observed(
        ConformanceItem::Ec01Identity,
        populated,
        provenance.is_synthetic(),
        if populated {
            format!(
                "identity fields populated: executor_id={}, name={}, version={}",
                identity.executor_id, identity.name, identity.version
            )
        } else {
            "one or more required identity fields were empty".to_string()
        },
    )
}

/// EC-02. Same limitation as EC-01: reports that the call succeeded and
/// returned a value, not that the value is truthful.
pub fn check_capabilities(
    _snapshot: &CapabilitySnapshot,
    provenance: &ExecutorProvenance,
) -> ConformanceResult {
    ConformanceResult::observed(
        ConformanceItem::Ec02Capabilities,
        true,
        provenance.is_synthetic(),
        "capabilities() returned a snapshot without panicking; truthfulness of \
         individual capability values cannot be independently verified by this \
         harness and must not be assumed",
    )
}

/// EC-03, the harness's most substantive check. Genuinely re-usable
/// across any real executor: captures the live repository's own
/// [`Baseline`] before and after the adapter runs, and if anything
/// outside the isolated [`AttemptWorkspace`] changed, that is a real,
/// mechanically-detected confinement violation — not a property asserted
/// from documentation. Reuses P2-W02's `Baseline` directly rather than
/// re-implementing hashing.
///
/// # Precisely what this proves, and what it does not (forensic audit,
/// corrected here per that audit's Part 3 instruction)
///
/// This function proves **"the live repository's own regular-file tree,
/// as captured by `Baseline`, is unchanged"** — a narrower and in one
/// respect *stricter* claim than "workspace confinement" or "the live
/// workspace is untouched" might suggest to a reader who hasn't read
/// `baseline.rs`. Six specific, empirically-confirmed limits (each
/// verified with an executed test during the forensic audit, then
/// reverted before that audit's own report was written — not asserted
/// from reading the code alone):
///
/// 1. **Gitignored and untracked regular files under the captured root
///    ARE included** — `Baseline` has no `.gitignore` awareness at all;
///    it walks the filesystem directly, not `git status`. This makes
///    this check **stricter than Blueprint §5.1's stated intent**
///    ("File matches `.gitignore`" → "excluded from baseline comparison
///    ... the Conductor never manages ignored files"). A real executor's
///    legitimate write to a gitignored path (e.g. a build tool
///    regenerating `dist/` or `target/`) will currently register as a
///    confinement **violation** here — a false positive relative to
///    Blueprint's own tolerance, not a security gap.
/// 2. **A brand-new symlink created at a path with no prior entry is
///    completely invisible** to `Baseline::diff` in either direction —
///    `Baseline` never follows symlinks and never records their
///    existence at all, so a symlink appearing at a previously-empty
///    path produces no `Added`/`Removed`/`Modified` entry whatsoever.
///    Since a symlink can point outside the live repository entirely,
///    this is the most significant gap in this mechanism: a misbehaving
///    executor could plant such a symlink with zero detection here.
///    (An *existing* file being replaced by a symlink of the same name
///    IS detected, correctly, as `Removed` — only genuinely new symlink
///    paths are invisible.)
/// 3. **A transient write-then-delete occurring entirely between the
///    `before` and `after` captures is invisible** — an inherent
///    limitation of two-point-in-time snapshot diffing, not fixable
///    without continuous filesystem observation (e.g. `inotify`), which
///    is materially outside this function's/module's current scope.
/// 4. **Only the filesystem beneath the one root path passed to
///    `Baseline::capture` is observed, by construction** — a mutation
///    anywhere else (a different repository, `/tmp`, the executor's own
///    working directory unrelated to any repository) is structurally
///    unobservable to this mechanism, not merely untested.
/// 5. Therefore: **"repository-tree snapshot integrity" is what this
///    function actually proves; "absolute machine-wide filesystem
///    confinement" is a materially stronger claim this function does
///    NOT make and must not be described as making.**
/// 6. None of the above are proposed to be fixed in this pass —
///    `Baseline` is explicitly not being redesigned and no filesystem
///    watcher is being added, per this correction task's own scope
///    boundary. This doc comment exists so the mechanism is not
///    overclaimed, not so it is silently strengthened.
pub fn check_workspace_confinement(
    live_repo_root: &Path,
    _workspace: &AttemptWorkspace,
    before: &Baseline,
    after: &Baseline,
    provenance: &ExecutorProvenance,
) -> ConformanceResult {
    let diff = before.diff(after);
    let violated = !diff.is_empty();
    let detail = if violated {
        let paths: Vec<String> = diff
            .keys()
            .take(10)
            .map(|p| p.display().to_string())
            .collect();
        format!(
            "the captured repository-tree snapshot at {} changed during the adapter's \
             execution window ({} path(s) affected, first up to 10: {:?}) -- repository-tree \
             snapshot integrity VIOLATED. This is NOT a claim about machine-wide filesystem \
             activity outside {}; see this function's doc comment for the exact scope.",
            live_repo_root.display(),
            diff.len(),
            paths,
            live_repo_root.display()
        )
    } else {
        format!(
            "the captured repository-tree snapshot at {} is unchanged across the adapter's \
             execution window (repository-tree snapshot integrity only -- new symlinks, \
             transient writes, and activity outside this root are not covered; see this \
             function's doc comment)",
            live_repo_root.display()
        )
    };
    ConformanceResult::observed(
        ConformanceItem::Ec03WorkspaceConfinement,
        !violated,
        provenance.is_synthetic(),
        detail,
    )
}

/// EC-04 through EC-10 share one shape: classify whatever
/// [`Observation::final_executor_state`] the adapter actually reported
/// against the state the scenario was trying to exercise.
///
/// # Hard boundary (forensic audit Part 6, restated as a permanent
/// constraint on this function, not just a one-time note)
///
/// This function does not, and must not be made to, manufacture the
/// underlying scenario. It cannot and does not claim: real timeout
/// observation, real cancellation observation, real crash observation,
/// real partial-execution detection, or a real Unknown-outcome
/// determination. It only classifies what a caller-orchestrated real (or
/// synthetic, honestly labeled) scenario's `Observation` already
/// contains. Any future change that adds subprocess-spawning,
/// timing, or OS-level process observation belongs in a different,
/// explicitly-scoped task (P5-W03/W04 real-execution work) — not as a
/// quiet expansion of this function.
pub fn check_lifecycle_outcome(
    item: ConformanceItem,
    observation: &Observation,
    expected: ExecutionState,
    provenance: &ExecutorProvenance,
) -> ConformanceResult {
    debug_assert!(matches!(
        item,
        ConformanceItem::Ec04SuccessfulExecution
            | ConformanceItem::Ec05PartialExecution
            | ConformanceItem::Ec06NonZeroExit
            | ConformanceItem::Ec07Timeout
            | ConformanceItem::Ec08Cancellation
            | ConformanceItem::Ec09Crash
            | ConformanceItem::Ec10UnknownOutcome
    ));
    let matched = observation.final_executor_state == expected;
    ConformanceResult::observed(
        item,
        matched,
        provenance.is_synthetic(),
        format!(
            "expected final_executor_state={expected:?}, observed={:?} ({}) -- this is a \
             classification of a caller-supplied Observation only; it does not independently \
             verify that the underlying real-world event (timeout/crash/cancellation/etc.) \
             actually occurred",
            observation.final_executor_state,
            if matched { "match" } else { "MISMATCH" }
        ),
    )
}

/// EC-11.
///
/// # TODO(P5-W05) — required future integration, deliberately NOT
/// implemented in this pass
///
/// The forensic audit established that this function does **not**
/// bridge into the real Reconciliation Engine: it never calls
/// [`crate::reconciliation::reconcile`] and never consults
/// [`crate::idempotency_store::IdempotencyStore::has_already_applied`].
/// It only performs a shallow before/after `Baseline` diff-and-count.
/// **Building that bridge is P5-W05's actual implementation scope, not
/// this correction task's** — per this task's explicit boundary
/// ("do NOT rewrite `check_restart_reconciliation` to implement the
/// actual P5-W05 behavior"), it is documented here and left deferred:
///
/// ```text
/// REAL EXECUTION
///   → interruption
///   → process disappearance
///   → workspace observation                    (this function today)
///   → restart
///   → reconciliation::reconcile(before, after, expected)   <- NOT called yet
///   → ReconciliationOutcome (NoChange/ExpectedChange/UnexpectedChange/PartialChange)
///   → IdempotencyStore::has_already_applied(key)            <- NOT consulted yet
///   → safe continuation
/// ```
///
/// Until that bridge exists, this function's result must not be read as
/// proof that reconciliation was semantically correct — only that
/// *something* diffable happened between two `Baseline` captures the
/// caller asserts spanned a genuine restart.
pub fn check_restart_reconciliation(
    before_interruption: &Baseline,
    after_restart_reconciliation: &Baseline,
    reconciliation_actually_ran: bool,
    provenance: &ExecutorProvenance,
) -> ConformanceResult {
    if !reconciliation_actually_ran {
        return ConformanceResult::blocked(
            ConformanceItem::Ec11RestartReconciliation,
            provenance.is_synthetic(),
            "no restart/reconciliation was actually exercised by the caller",
        );
    }
    let diff = before_interruption.diff(after_restart_reconciliation);
    let unexplained: Vec<_> = diff
        .iter()
        .filter(|(_, change)| matches!(change, PathChange::Modified { .. }))
        .collect();
    ConformanceResult::observed(
        ConformanceItem::Ec11RestartReconciliation,
        true,
        provenance.is_synthetic(),
        format!(
            "reconciliation ran; {} path(s) differ pre/post -- NOTE: this does not call \
             reconciliation::reconcile() or consult IdempotencyStore (see this function's \
             TODO(P5-W05) doc comment); caller is responsible for confirming these are the \
             *expected* reconciled changes and that no side effect was duplicated",
            unexplained.len()
        ),
    )
}

/// EC-12. Two independent structural sub-checks, combined into one
/// result (a suite can only hold one result per [`ConformanceItem`], so
/// splitting this into two separately-tagged functions would silently
/// let one overwrite the other in [`ConformanceSuiteReport::from_results`]
/// — combining them here avoids that collision):
///
/// 1. The four required [`EvidenceRefs`] identity fields are populated —
///    this is the Adapter Contract §14 EC-12's own literal scope
///    ("evidence collection", per §9's reference list).
/// 2. **New in this pass**, per the forensic audit's Part 4 finding:
///    whenever `observation.final_executor_state` is a failure-shaped
///    outcome (`Failed`/`Crashed`/`TimedOut`/`Cancelled`/`Unknown`), a
///    `failure_classification` must actually be present (`Some(_)`)
///    rather than silently absent. This uses the Adapter Contract §13
///    `FailureCategory` taxonomy the crate already defines, and is the
///    closest a pre-real-executor structural check can come to
///    Blueprint's differently-scoped EC-12 ("provider failure
///    propagation") — see `GOVERNANCE/EC_ENUMERATION_MAPPING.md`'s
///    "EC-12 mismatch" section for the honest limits of what this does
///    and does not prove. **It does not prove a provider failure was
///    correctly classified and propagated end-to-end** — that requires a
///    real provider failure and P6+ machinery that does not exist yet.
pub fn check_evidence_collection(
    evidence: &EvidenceRefs,
    observation: &Observation,
    provenance: &ExecutorProvenance,
) -> ConformanceResult {
    let fields_populated = !evidence.attempt_id.is_empty()
        && !evidence.workspace_id.is_empty()
        && !evidence.executor_id.is_empty()
        && !evidence.version.is_empty();

    let is_failure_shaped = matches!(
        observation.final_executor_state,
        ExecutionState::Failed
            | ExecutionState::Crashed
            | ExecutionState::TimedOut
            | ExecutionState::Cancelled
            | ExecutionState::Unknown
    );
    let failure_classification_present = observation.failure_classification.is_some();
    let failure_classification_consistent = !is_failure_shaped || failure_classification_present;

    let passed = fields_populated && failure_classification_consistent;

    let mut detail = String::new();
    detail.push_str(if fields_populated {
        "required EvidenceRefs identity fields (attempt_id/workspace_id/executor_id/version) \
         are populated"
    } else {
        "one or more required EvidenceRefs identity fields were empty"
    });
    detail.push_str("; ");
    detail.push_str(&if !is_failure_shaped {
        "final_executor_state is not failure-shaped, so failure_classification presence is \
         not required"
            .to_string()
    } else if failure_classification_present {
        format!(
            "final_executor_state={:?} is failure-shaped and failure_classification={:?} \
             is present, as required",
            observation.final_executor_state, observation.failure_classification
        )
    } else {
        format!(
            "VIOLATION: final_executor_state={:?} is failure-shaped but \
             failure_classification is None -- Adapter Contract SS13's taxonomy exists \
             precisely so this is not silently absent",
            observation.final_executor_state
        )
    });
    detail.push_str(
        "; this check does NOT prove a real provider failure was correctly classified and \
         propagated end-to-end -- see GOVERNANCE/EC_ENUMERATION_MAPPING.md's EC-12 mismatch \
         section",
    );

    ConformanceResult::observed(
        ConformanceItem::Ec12EvidenceCollection,
        passed,
        provenance.is_synthetic(),
        detail,
    )
}

/// EC-13, the other substantive mechanical check available without a
/// real executor's real secrets: given a set of substrings the caller
/// asserts are secret-shaped for this run (never the secret values
/// themselves in a committed fixture -- see the test module for how this
/// is exercised safely with synthetic markers), scan every text-bearing
/// field the adapter surfaced. A hit is a real, mechanically-detected
/// redaction failure.
///
/// # Known coverage limits (forensic audit Part 7, corrected this pass
/// where fixable; documented where not)
///
/// - `EvidenceRefs.process_ref` **is now scanned** (was missing before
///   this pass — a real, small, fixed gap).
/// - **`stderr` has no dedicated representation anywhere upstream of
///   this function.** Neither the Adapter Contract's `Observation` type
///   nor this crate's `execution_adapter.rs` distinguishes stdout from
///   stderr — both are folded into the single
///   `stdout_or_structured_events` string by convention. If a real
///   adapter keeps stderr genuinely separate internally and only
///   surfaces stdout through this field, a credential leaked exclusively
///   via stderr would never reach this scanner. This is an upstream
///   type-design question, not fixable inside this function alone, and
///   is deliberately not "solved" here by inventing a new field on
///   `Observation` (out of this task's scope; would need its own
///   change).
/// - This remains a **known-marker scanner**, not a secret-shape
///   detector (no entropy/pattern heuristics) — a credential the caller
///   didn't think to list as a marker is not caught.
pub fn check_credential_redaction(
    observation: &Observation,
    evidence: &EvidenceRefs,
    known_secret_markers: &[String],
    provenance: &ExecutorProvenance,
) -> ConformanceResult {
    if known_secret_markers.is_empty() {
        return ConformanceResult::unknown(
            ConformanceItem::Ec13CredentialRedaction,
            provenance.is_synthetic(),
            "no secret markers were supplied to scan for -- absence of a hit proves \
             nothing when nothing secret was ever in scope for this run",
        );
    }
    let mut haystacks: Vec<&str> = vec![observation.stdout_or_structured_events.as_str()];
    haystacks.extend(observation.tool_activity.iter().map(|s| s.as_str()));
    haystacks.extend(
        observation
            .changed_workspace_observations
            .iter()
            .map(|s| s.as_str()),
    );
    if let Some(exit) = observation.exit_information.as_deref() {
        haystacks.push(exit);
    }
    haystacks.extend(evidence.commands.iter().map(|s| s.as_str()));
    if let Some(outputs) = evidence.outputs_ref.as_deref() {
        haystacks.push(outputs);
    }
    haystacks.extend(evidence.artifacts_ref.iter().map(|s| s.as_str()));
    if let Some(process_ref) = evidence.process_ref.as_deref() {
        haystacks.push(process_ref);
    }
    if let Some(exit_result) = evidence.exit_result.as_deref() {
        haystacks.push(exit_result);
    }

    let mut hits = Vec::new();
    for marker in known_secret_markers {
        if haystacks.iter().any(|h| h.contains(marker.as_str())) {
            hits.push(marker.clone());
        }
    }

    let leaked = !hits.is_empty();
    ConformanceResult::observed(
        ConformanceItem::Ec13CredentialRedaction,
        !leaked,
        provenance.is_synthetic(),
        if leaked {
            format!(
                "{} known secret marker(s) found unredacted in adapter-surfaced evidence: {:?}",
                hits.len(),
                hits
            )
        } else {
            format!(
                "{} known secret marker(s) scanned for (including process_ref/exit_result); \
                 none found unredacted. NOTE: stderr has no dedicated field upstream and may \
                 not be covered -- see this function's doc comment",
                known_secret_markers.len()
            )
        },
    )
}

/// EC-14. Structurally not this module's to check pre-P9 (see
/// [`ConformanceItem::is_applicable_before_p9`]).
pub fn check_capability_denial_not_applicable() -> ConformanceResult {
    ConformanceResult::not_applicable(
        ConformanceItem::Ec14CapabilityDenial,
        "tool-execution-boundary capability enforcement is P9 gstack Skill Runtime scope \
         (Blueprint SS16); no capability_check()/tool_boundary module exists in this crate \
         yet, confirmed by module-list and grep audit. Not a gap in this harness.",
    )
}

/// EC-15/EC-16, verified the same way `execution_adapter.rs`'s own
/// module docs claim it ("grep-audited... the same method used for every
/// prior authority-boundary claim in this crate") — except here it is an
/// actual, automatically re-run mechanical check rather than a claim in
/// prose. Reads `execution_adapter.rs`'s own compiled-in source text via
/// `include_str!` and asserts the specific symbols that would constitute
/// a violation are genuinely absent. This turns a documentation claim
/// into a permanent regression test: if a future edit to
/// `execution_adapter.rs` ever adds a `crate::state::AttemptStatus` or
/// `crate::conflict` reference, this check fails immediately rather than
/// relying on someone re-reading the module docs and noticing.
///
/// **Scope limit, stated explicitly (forensic audit Section 2 finding):**
/// this only ever protects `execution_adapter.rs` itself. It says
/// nothing about `conformance.rs`, or any future `aider.rs`/`jcode.rs`
/// adapter module, making the same violation.
pub fn check_no_direct_succeeded_or_merge_authority() -> (ConformanceResult, ConformanceResult) {
    let source = include_str!("execution_adapter.rs");
    // Only real code lines count as a violation. This crate's style never
    // uses block comments (`/* */` -- confirmed absent crate-wide), so a
    // trimmed-line-prefix filter on `//` is sufficient and does not
    // silently mask a genuine code-level reference: a real `use
    // crate::state::...;` or `crate::conflict::...` call site cannot
    // start with `//` and still compile. Without this filter, this
    // module's own doc comment prose (which *names* `crate::state` and
    // `crate::conflict` while explaining the rule it upholds) would
    // false-positive as a violation -- caught by this check's own test
    // the first time it ran against the real file, not assumed correct.
    let code_lines: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    let forbidden_state_refs = [
        "crate::state::",
        "AttemptStatus::Succeeded",
        "use crate::state",
    ];
    let forbidden_conflict_refs = ["crate::conflict::", "use crate::conflict"];

    let state_hits: Vec<&&str> = forbidden_state_refs
        .iter()
        .filter(|needle| code_lines.contains(*needle))
        .collect();
    let conflict_hits: Vec<&&str> = forbidden_conflict_refs
        .iter()
        .filter(|needle| code_lines.contains(*needle))
        .collect();

    let ec15 = ConformanceResult::verified(
        ConformanceItem::Ec15NoDirectSucceeded,
        state_hits.is_empty(),
        false,
        if state_hits.is_empty() {
            "execution_adapter.rs's own source text contains no reference to crate::state \
             or AttemptStatus::Succeeded -- structurally cannot construct a Succeeded \
             transition from this module (scope limit: this only protects \
             execution_adapter.rs itself)"
                .to_string()
        } else {
            format!(
                "VIOLATION: execution_adapter.rs now references forbidden symbol(s): {:?}",
                state_hits
            )
        },
    );
    let ec16 = ConformanceResult::verified(
        ConformanceItem::Ec16NoDirectMerge,
        conflict_hits.is_empty(),
        false,
        if conflict_hits.is_empty() {
            "execution_adapter.rs's own source text contains no reference to crate::conflict \
             -- structurally cannot silently resolve a Conflict or authorize a merge from \
             this module (scope limit: this only protects execution_adapter.rs itself)"
                .to_string()
        } else {
            format!(
                "VIOLATION: execution_adapter.rs now references forbidden symbol(s): {:?}",
                conflict_hits
            )
        },
    );
    (ec15, ec16)
}

/// EC-17. Unlike every other lifecycle item, this one genuinely does not
/// require a *real* executor to prove — it is a property of the kernel-
/// facing calling code, not of any particular executor's real-world
/// behavior. Proves the same generic function drives two differently-
/// identified adapters with zero code change, which is exactly Adapter
/// Contract §18's claim. Still marked `synthetic` when built from
/// synthetic adapters (honesty about *what kind* of adapter was used),
/// but its `passed` value is meaningful regardless, since replaceability
/// is a compile-time/calling-convention property, not a runtime-behavior
/// property that only a real executor could exhibit.
pub fn check_executor_replacement(
    claim_a: FinalizationClaim,
    claim_b: FinalizationClaim,
    identity_a: &ExecutorIdentity,
    identity_b: &ExecutorIdentity,
    provenance_a: &ExecutorProvenance,
    provenance_b: &ExecutorProvenance,
) -> ConformanceResult {
    let different_executors = identity_a.executor_id != identity_b.executor_id;
    ConformanceResult::observed(
        ConformanceItem::Ec17ExecutorReplacement,
        different_executors,
        provenance_a.is_synthetic() || provenance_b.is_synthetic(),
        format!(
            "same generic drive-through-the-trait-object calling code produced \
             claim_a={claim_a:?} (executor_id={}) and claim_b={claim_b:?} (executor_id={}) \
             without any kernel-level code change between them",
            identity_a.executor_id, identity_b.executor_id
        ),
    )
}

// ---------------------------------------------------------------------
// Suite aggregation
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuiteDisposition {
    /// The harness itself is implemented and exercisable (this is what
    /// running the suite against `FakeExecutionAdapter` proves).
    StructurallyReady,
    /// Every applicable item ran and reported a result, but at least one
    /// result came from a synthetic adapter. Can NEVER be upgraded to
    /// `RealConformancePassed` regardless of how many items passed.
    SyntheticOnly,
    /// At least one item ran against a real executor and passed, but not
    /// every applicable item has real, passing evidence yet.
    RealConformancePartial,
    /// Every applicable item (per `ConformanceItem::is_applicable_before_p9`)
    /// has real (non-synthetic), passing evidence.
    RealConformancePassed,
    /// One or more applicable items are `Blocked`/`Unknown` and the suite
    /// cannot be meaningfully scored yet.
    Blocked,
}

#[derive(Debug, Clone)]
pub struct ConformanceSuiteReport {
    results: BTreeMap<ConformanceItem, ConformanceResult>,
}

impl ConformanceSuiteReport {
    pub fn from_results(results: Vec<ConformanceResult>) -> Self {
        ConformanceSuiteReport {
            results: results.into_iter().map(|r| (r.item(), r)).collect(),
        }
    }

    pub fn result_for(&self, item: &ConformanceItem) -> Option<&ConformanceResult> {
        self.results.get(item)
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Pure function; the one place `SuiteDisposition` is decided.
    /// Deliberately conservative: any single synthetic result caps the
    /// whole suite, any single Blocked/Unknown applicable result prevents
    /// a "passed" verdict, and a missing applicable item is treated the
    /// same as Blocked (never silently ignored).
    pub fn disposition(&self) -> SuiteDisposition {
        let applicable_items: Vec<ConformanceItem> = ConformanceItem::all()
            .into_iter()
            .filter(|i| i.is_applicable_before_p9())
            .collect();

        let mut any_synthetic = false;
        let mut any_blocked_or_unknown = false;
        let mut any_missing = false;
        let mut any_real_pass = false;
        let mut all_real_pass = true;

        for item in &applicable_items {
            match self.results.get(item) {
                None => {
                    any_missing = true;
                    all_real_pass = false;
                }
                Some(result) => {
                    if result.synthetic() {
                        any_synthetic = true;
                    }
                    match result.status() {
                        EvidenceStatus::Blocked | EvidenceStatus::Unknown => {
                            any_blocked_or_unknown = true;
                            all_real_pass = false;
                        }
                        EvidenceStatus::NotApplicable => {}
                        EvidenceStatus::Observed | EvidenceStatus::Verified => {
                            let passed = result.passed().unwrap_or(false);
                            if !result.synthetic() && passed {
                                any_real_pass = true;
                            }
                            if result.synthetic() || !passed {
                                all_real_pass = false;
                            }
                        }
                    }
                }
            }
        }

        if any_missing || any_blocked_or_unknown {
            return SuiteDisposition::Blocked;
        }
        if any_synthetic {
            return SuiteDisposition::SyntheticOnly;
        }
        if all_real_pass {
            return SuiteDisposition::RealConformancePassed;
        }
        if any_real_pass {
            return SuiteDisposition::RealConformancePartial;
        }
        SuiteDisposition::StructurallyReady
    }
}

// ---------------------------------------------------------------------
// Phase 5 readiness assessment (the A..J questions)
// ---------------------------------------------------------------------

/// Facts about the external environment that this harness cannot
/// determine on its own and must never guess. Every field defaults to
/// `Unknown` via [`ExternalEnvironmentFacts::unconfirmed`] -- a caller
/// must explicitly supply real evidence to claim anything stronger, and
/// that evidence's source is recorded alongside it so a later reader can
/// judge its freshness (Blueprint SS4.5 evidence-freshness philosophy,
/// applied here rather than re-invented).
#[derive(Debug, Clone)]
pub struct ExternalEnvironmentFacts {
    pub aider_available: EvidenceStatus,
    pub aider_evidence_source: String,
    pub jcode_available: EvidenceStatus,
    pub jcode_evidence_source: String,
    pub provider_credentials_available: EvidenceStatus,
    pub credentials_evidence_source: String,
}

impl ExternalEnvironmentFacts {
    pub fn unconfirmed() -> Self {
        ExternalEnvironmentFacts {
            aider_available: EvidenceStatus::Unknown,
            aider_evidence_source: "not checked this session".to_string(),
            jcode_available: EvidenceStatus::Unknown,
            jcode_evidence_source: "not checked this session".to_string(),
            provider_credentials_available: EvidenceStatus::Unknown,
            credentials_evidence_source: "not checked this session".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Phase5ReadinessAnswer {
    pub label: &'static str,
    pub question: &'static str,
    pub status: EvidenceStatus,
    pub detail: String,
}

/// Answers questions A..J exactly as posed, from real inputs only --
/// never infers C/D/E from anything the conformance suite itself
/// observed (the suite cannot see the human's machine).
pub fn assess_phase5_readiness(
    suite: &ConformanceSuiteReport,
    env: &ExternalEnvironmentFacts,
) -> Vec<Phase5ReadinessAnswer> {
    let disposition = suite.disposition();

    let a_status = if ConformanceItem::all()
        .iter()
        .all(|i| !i.is_applicable_before_p9() || suite.result_for(i).is_some())
    {
        EvidenceStatus::Verified
    } else {
        EvidenceStatus::Unknown
    };

    let b_status = match disposition {
        SuiteDisposition::StructurallyReady
        | SuiteDisposition::SyntheticOnly
        | SuiteDisposition::RealConformancePartial
        | SuiteDisposition::RealConformancePassed => EvidenceStatus::Observed,
        SuiteDisposition::Blocked => EvidenceStatus::Blocked,
    };

    let real_ready = matches!(
        disposition,
        SuiteDisposition::RealConformancePartial | SuiteDisposition::RealConformancePassed
    );

    let f_status = if real_ready
        && env.aider_available == EvidenceStatus::Observed
        && env.provider_credentials_available == EvidenceStatus::Observed
    {
        EvidenceStatus::Observed
    } else if real_ready
        && env.jcode_available == EvidenceStatus::Observed
        && env.provider_credentials_available == EvidenceStatus::Observed
    {
        EvidenceStatus::Observed
    } else {
        EvidenceStatus::Blocked
    };

    let j_gate: String = if f_status == EvidenceStatus::Observed {
        "VG-P5-EXEC-01 (Adapter Contract) may be attempted next, followed by \
         VG-P5-EXEC-02 (Workspace Confinement)"
            .to_string()
    } else {
        "none -- P5-W03/W04/W05 remain BLOCKED until F is Observed".to_string()
    };

    vec![
        Phase5ReadinessAnswer {
            label: "A",
            question: "Is the conformance infrastructure implemented?",
            status: a_status,
            detail: format!(
                "{}/{} applicable ConformanceItem(s) have a recorded result",
                suite.len(),
                ConformanceItem::all()
                    .iter()
                    .filter(|i| i.is_applicable_before_p9())
                    .count()
            ),
        },
        Phase5ReadinessAnswer {
            label: "B",
            question: "Can it exercise a real ExecutionAdapter?",
            status: b_status,
            detail: format!(
                "structurally yes (trait-object driven, no Aider/Jcode-specific code); \
                 current suite disposition: {disposition:?}. A synthetic-only disposition \
                 proves the harness runs, not that a real executor has been exercised."
            ),
        },
        Phase5ReadinessAnswer {
            label: "C",
            question: "Is Aider actually available?",
            status: env.aider_available,
            detail: env.aider_evidence_source.clone(),
        },
        Phase5ReadinessAnswer {
            label: "D",
            question: "Is Jcode actually available?",
            status: env.jcode_available,
            detail: env.jcode_evidence_source.clone(),
        },
        Phase5ReadinessAnswer {
            label: "E",
            question: "Is at least one usable provider/model credential available?",
            status: env.provider_credentials_available,
            detail: env.credentials_evidence_source.clone(),
        },
        Phase5ReadinessAnswer {
            label: "F",
            question: "Can the real executor perform a controlled test task?",
            status: f_status,
            detail: "requires: real conformance disposition AND (Aider-or-Jcode Observed) \
                      AND credentials Observed. Not inferred from A/B alone."
                .to_string(),
        },
        Phase5ReadinessAnswer {
            label: "G",
            question: "Can the workspace isolation requirement be observed?",
            status: suite
                .result_for(&ConformanceItem::Ec03WorkspaceConfinement)
                .map(|r| r.status())
                .unwrap_or(EvidenceStatus::Unknown),
            detail: suite
                .result_for(&ConformanceItem::Ec03WorkspaceConfinement)
                .map(|r| r.detail().to_string())
                .unwrap_or_else(|| "EC-03 not yet run".to_string()),
        },
        Phase5ReadinessAnswer {
            label: "H",
            question: "Can execution interruption be observed?",
            status: [
                ConformanceItem::Ec07Timeout,
                ConformanceItem::Ec08Cancellation,
                ConformanceItem::Ec09Crash,
            ]
            .iter()
            .map(|i| {
                suite
                    .result_for(i)
                    .map(|r| r.status())
                    .unwrap_or(EvidenceStatus::Unknown)
            })
            .min_by_key(evidence_status_rank)
            .unwrap_or(EvidenceStatus::Unknown),
            detail: "aggregated across EC-07/EC-08/EC-09; weakest of the three".to_string(),
        },
        Phase5ReadinessAnswer {
            label: "I",
            question: "Can post-interruption reconciliation be observed?",
            status: suite
                .result_for(&ConformanceItem::Ec11RestartReconciliation)
                .map(|r| r.status())
                .unwrap_or(EvidenceStatus::Unknown),
            detail: suite
                .result_for(&ConformanceItem::Ec11RestartReconciliation)
                .map(|r| r.detail().to_string())
                .unwrap_or_else(|| "EC-11 not yet run".to_string()),
        },
        Phase5ReadinessAnswer {
            label: "J",
            question: "Which exact P5 gate can now be attempted?",
            status: if f_status == EvidenceStatus::Observed {
                EvidenceStatus::Observed
            } else {
                EvidenceStatus::Blocked
            },
            detail: j_gate,
        },
    ]
}

fn evidence_status_rank(status: &EvidenceStatus) -> u8 {
    // Lower = weaker; used to pick the "weakest of several" for H.
    match status {
        EvidenceStatus::Blocked => 0,
        EvidenceStatus::Unknown => 1,
        EvidenceStatus::NotApplicable => 2,
        EvidenceStatus::Observed => 3,
        EvidenceStatus::Verified => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_adapter::{
        CancellationReport, CapabilityValue, ExecutionAdapter, ExecutionRequest as ExecReq,
        FailureCategory, FakeExecutionAdapter,
    };
    use crate::worktree::WorktreeManager;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    /// Mirrors `worktree.rs`'s own test helper exactly (no new dependency
    /// introduced for this module -- `std::env::temp_dir()` plus a
    /// process-id-qualified tag is the established pattern in this
    /// crate).
    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ck_conformance_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    fn synthetic() -> ExecutorProvenance {
        ExecutorProvenance::Synthetic {
            double_kind: "FakeExecutionAdapter".to_string(),
        }
    }

    fn fake_identity(id: &str) -> ExecutorIdentity {
        ExecutorIdentity {
            executor_id: id.to_string(),
            name: format!("Fake-{id}"),
            version: "0.1.0".to_string(),
            commit_or_build_identity: None,
            adapter_version: "1.0".to_string(),
            runtime_environment_identity: "conformance-test-sandbox".to_string(),
        }
    }

    fn unknown_capabilities() -> CapabilitySnapshot {
        CapabilitySnapshot {
            interactive_execution: CapabilityValue::Unknown,
            non_interactive_execution: CapabilityValue::Unknown,
            workspace_workdir_control: CapabilityValue::Unknown,
            file_mutation: CapabilityValue::Unknown,
            shell_process_execution: CapabilityValue::Unknown,
            git_interaction: CapabilityValue::Unknown,
            streaming: CapabilityValue::Unknown,
            cancellation: CapabilityValue::Unknown,
            timeout: CapabilityValue::Unknown,
            session_persistence: CapabilityValue::Unknown,
            provider_selection: CapabilityValue::Unknown,
            mcp_tooling: CapabilityValue::Unknown,
            browser_capability: CapabilityValue::Unknown,
            parallel_execution: CapabilityValue::Unknown,
        }
    }

    fn fake_adapter(id: &str, final_state: ExecutionState) -> FakeExecutionAdapter {
        FakeExecutionAdapter::new(
            fake_identity(id),
            unknown_capabilities(),
            Observation {
                started: true,
                process_identity: Some(format!("pid-{id}")),
                stdout_or_structured_events: "did the thing".to_string(),
                tool_activity: vec!["edited a.txt".to_string()],
                exit_information: Some("exit 0".to_string()),
                changed_workspace_observations: vec!["a.txt".to_string()],
                failure_classification: None,
                final_executor_state: final_state,
            },
            CancellationReport {
                requested: false,
                accepted: false,
                observed_termination_state: ExecutionState::Running,
                remaining_ambiguity: false,
                cleanup_status: "n/a".to_string(),
            },
            EvidenceRefs {
                attempt_id: format!("attempt-{id}"),
                workspace_id: format!("ws-{id}"),
                executor_id: id.to_string(),
                version: "0.1.0".to_string(),
                process_ref: Some(format!("pid-{id}")),
                commands: vec!["npm test".to_string()],
                outputs_ref: None,
                artifacts_ref: vec![],
                exit_result: Some("exit 0".to_string()),
            },
            FinalizationClaim::CompletedClaim,
        )
    }

    fn sample_request() -> ExecReq {
        ExecReq {
            correlation: crate::correlation::CorrelationContext::for_mission("m-1")
                .with_step("s-1"),
            executor_id: "fake-1".to_string(),
            workspace_id: "ws-1".to_string(),
            baseline_id: "baseline-1".to_string(),
            allowed_capabilities: vec!["file_mutation".to_string()],
            requested_operation: "implement feature X".to_string(),
            provider_resource: None,
            policy_version: "v1".to_string(),
            timeout_policy: std::time::Duration::from_secs(300),
        }
    }

    fn init_git_repo(dir: &Path) {
        let run = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(dir)
                .status()
                .expect("git available");
            assert!(status.success(), "git {args:?} failed");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "conformance@example.com"]);
        run(&["config", "user.name", "Conformance Test"]);
        std::fs::write(dir.join("live.txt"), b"live content\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-q", "-m", "initial"]);
    }

    #[test]
    fn ec01_reports_observed_pass_for_populated_identity() {
        let result = check_identity(&fake_identity("x"), &synthetic());
        assert_eq!(result.status(), EvidenceStatus::Observed);
        assert_eq!(result.passed(), Some(true));
        assert!(result.synthetic());
    }

    #[test]
    fn ec01_fails_on_empty_required_field() {
        let mut identity = fake_identity("x");
        identity.name = String::new();
        let result = check_identity(&identity, &synthetic());
        assert_eq!(result.passed(), Some(false));
    }

    #[test]
    fn ec03_detects_no_confinement_violation_when_live_repo_untouched() {
        let root = dir("ec03_clean");
        let live_repo = root.join("live");
        fs::create_dir_all(&live_repo).unwrap();
        init_git_repo(&live_repo);

        let workspaces_root = root.join("workspaces");
        let manager = WorktreeManager::new(&live_repo, &workspaces_root);
        let workspace = manager.create("attempt-ec03-clean", None).unwrap();

        let before = Baseline::capture(&live_repo).unwrap();
        // Simulate an adapter that only writes inside the attempt workspace.
        fs::write(workspace.path.join("only_in_workspace.txt"), b"ok\n").unwrap();
        let after = Baseline::capture(&live_repo).unwrap();

        let result =
            check_workspace_confinement(&live_repo, &workspace, &before, &after, &synthetic());
        assert_eq!(result.passed(), Some(true));

        manager.remove(&workspace).unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn ec03_detects_a_real_confinement_violation() {
        let root = dir("ec03_violation");
        let live_repo = root.join("live");
        fs::create_dir_all(&live_repo).unwrap();
        init_git_repo(&live_repo);

        let workspaces_root = root.join("workspaces");
        let manager = WorktreeManager::new(&live_repo, &workspaces_root);
        let workspace = manager.create("attempt-ec03-violation", None).unwrap();

        let before = Baseline::capture(&live_repo).unwrap();
        // Simulate a misbehaving adapter that writes OUTSIDE its workspace,
        // directly into the live repository -- exactly what EC-03 exists
        // to catch.
        fs::write(live_repo.join("live.txt"), b"TAMPERED\n").unwrap();
        let after = Baseline::capture(&live_repo).unwrap();

        let result =
            check_workspace_confinement(&live_repo, &workspace, &before, &after, &synthetic());
        assert_eq!(result.passed(), Some(false));
        assert!(result.detail().contains("VIOLATED"));

        manager.remove(&workspace).unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn ec03_gitignored_style_paths_are_still_captured_stricter_than_blueprint() {
        // Pins the forensic-audit finding as a permanent regression case:
        // Baseline has no .gitignore awareness, so EC-03 is stricter than
        // Blueprint SS5.1's stated tolerance for ignored files. This is
        // documented behavior now, not a silent surprise.
        let root = dir("ec03_gitignore");
        let live_repo = root.join("live");
        fs::create_dir_all(&live_repo).unwrap();
        init_git_repo(&live_repo);
        fs::write(live_repo.join(".gitignore"), b"node_modules/\n").unwrap();

        let workspaces_root = root.join("workspaces");
        let manager = WorktreeManager::new(&live_repo, &workspaces_root);
        let workspace = manager.create("attempt-ec03-gitignore", None).unwrap();

        let before = Baseline::capture(&live_repo).unwrap();
        fs::create_dir_all(live_repo.join("node_modules")).unwrap();
        fs::write(live_repo.join("node_modules/pkg.js"), b"stuff").unwrap();
        let after = Baseline::capture(&live_repo).unwrap();

        let result =
            check_workspace_confinement(&live_repo, &workspace, &before, &after, &synthetic());
        // A write to a gitignored path inside the live repo IS flagged --
        // stricter than Blueprint's stated intent, documented as such.
        assert_eq!(result.passed(), Some(false));

        manager.remove(&workspace).unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn ec04_through_ec10_classify_match_and_mismatch_honestly() {
        let adapter = fake_adapter("x", ExecutionState::Completed);
        let observation = adapter.observe();
        let matched = check_lifecycle_outcome(
            ConformanceItem::Ec04SuccessfulExecution,
            &observation,
            ExecutionState::Completed,
            &synthetic(),
        );
        assert_eq!(matched.passed(), Some(true));

        let mismatched = check_lifecycle_outcome(
            ConformanceItem::Ec07Timeout,
            &observation,
            ExecutionState::TimedOut,
            &synthetic(),
        );
        assert_eq!(mismatched.passed(), Some(false));
        assert!(mismatched.detail().contains("MISMATCH"));
    }

    #[test]
    fn ec11_is_blocked_when_no_reconciliation_actually_ran() {
        let baseline = Baseline::capture(std::env::temp_dir()).unwrap();
        let result = check_restart_reconciliation(&baseline, &baseline, false, &synthetic());
        assert_eq!(result.status(), EvidenceStatus::Blocked);
        assert_eq!(result.passed(), None);
    }

    #[test]
    fn ec12_fails_on_empty_evidence_field() {
        let mut evidence = EvidenceRefs {
            attempt_id: "a-1".to_string(),
            workspace_id: String::new(),
            executor_id: "e-1".to_string(),
            version: "1.0".to_string(),
            process_ref: None,
            commands: vec![],
            outputs_ref: None,
            artifacts_ref: vec![],
            exit_result: None,
        };
        let observation = fake_adapter("x", ExecutionState::Completed).observe();
        let result = check_evidence_collection(&evidence, &observation, &synthetic());
        assert_eq!(result.passed(), Some(false));
        evidence.workspace_id = "ws-1".to_string();
        let result = check_evidence_collection(&evidence, &observation, &synthetic());
        assert_eq!(result.passed(), Some(true));
    }

    #[test]
    fn ec12_requires_failure_classification_when_final_state_is_failure_shaped() {
        let adapter = fake_adapter("x", ExecutionState::Crashed);
        let mut observation = adapter.observe();
        let evidence = adapter.collect_evidence();

        // failure_classification left None on a Crashed outcome -- must fail.
        assert!(observation.failure_classification.is_none());
        let result = check_evidence_collection(&evidence, &observation, &synthetic());
        assert_eq!(result.passed(), Some(false));
        assert!(result.detail().contains("VIOLATION"));

        // Populate it -- must now pass (assuming identity fields are fine).
        observation.failure_classification = Some(FailureCategory::ProcessFailure);
        let result = check_evidence_collection(&evidence, &observation, &synthetic());
        assert_eq!(result.passed(), Some(true));
    }

    #[test]
    fn ec12_does_not_require_failure_classification_on_a_non_failure_outcome() {
        let adapter = fake_adapter("x", ExecutionState::Completed);
        let observation = adapter.observe();
        let evidence = adapter.collect_evidence();
        assert!(observation.failure_classification.is_none());
        let result = check_evidence_collection(&evidence, &observation, &synthetic());
        assert_eq!(result.passed(), Some(true));
    }

    #[test]
    fn ec13_detects_an_unredacted_secret_marker() {
        let adapter = fake_adapter("x", ExecutionState::Completed);
        let mut observation = adapter.observe();
        // A deliberately fake, synthetic marker -- not a real credential
        // shape, never a plausible real secret -- used only to prove the
        // scanner actually scans.
        observation.stdout_or_structured_events =
            "output included CONFORMANCE-TEST-MARKER-not-a-real-secret".to_string();
        let evidence = adapter.collect_evidence();
        let markers = vec!["CONFORMANCE-TEST-MARKER-not-a-real-secret".to_string()];
        let result = check_credential_redaction(&observation, &evidence, &markers, &synthetic());
        assert_eq!(result.passed(), Some(false));
        assert!(result.detail().contains("found unredacted"));
    }

    #[test]
    fn ec13_detects_a_marker_planted_only_in_process_ref() {
        // Regression case for the forensic-audit-found gap: process_ref
        // was not scanned before this pass.
        let adapter = fake_adapter("x", ExecutionState::Completed);
        let observation = adapter.observe();
        let mut evidence = adapter.collect_evidence();
        evidence.process_ref =
            Some("pid-1234 token=CONFORMANCE-TEST-PROCESS-REF-MARKER".to_string());
        let markers = vec!["CONFORMANCE-TEST-PROCESS-REF-MARKER".to_string()];
        let result = check_credential_redaction(&observation, &evidence, &markers, &synthetic());
        assert_eq!(
            result.passed(),
            Some(false),
            "a marker present only in EvidenceRefs.process_ref must be caught"
        );
    }

    #[test]
    fn ec13_passes_clean_and_reports_unknown_when_no_markers_supplied() {
        let adapter = fake_adapter("x", ExecutionState::Completed);
        let observation = adapter.observe();
        let evidence = adapter.collect_evidence();

        let clean = check_credential_redaction(
            &observation,
            &evidence,
            &["definitely-absent-marker".to_string()],
            &synthetic(),
        );
        assert_eq!(clean.passed(), Some(true));

        let unscanned = check_credential_redaction(&observation, &evidence, &[], &synthetic());
        assert_eq!(unscanned.status(), EvidenceStatus::Unknown);
        assert_eq!(unscanned.passed(), None);
    }

    #[test]
    fn ec14_is_structurally_not_applicable_before_p9() {
        let result = check_capability_denial_not_applicable();
        assert_eq!(result.status(), EvidenceStatus::NotApplicable);
        assert!(!ConformanceItem::Ec14CapabilityDenial.is_applicable_before_p9());
    }

    #[test]
    fn ec15_and_ec16_pass_against_the_actual_current_source() {
        let (ec15, ec16) = check_no_direct_succeeded_or_merge_authority();
        assert_eq!(ec15.status(), EvidenceStatus::Verified);
        assert_eq!(ec15.passed(), Some(true));
        assert_eq!(ec16.status(), EvidenceStatus::Verified);
        assert_eq!(ec16.passed(), Some(true));
    }

    /// Regression case: `execution_adapter.rs`'s own module-doc prose
    /// names `crate::state` and `crate::conflict` in backticks while
    /// explaining the very rule this check verifies -- a naive
    /// substring scan over the whole file (no comment filtering) failed
    /// this exact assertion the first time this check was run against
    /// the real file (caught here, not assumed away). This test pins
    /// that the comment-filtering fix stays in place.
    #[test]
    fn ec15_check_is_not_fooled_by_the_forbidden_symbols_appearing_in_doc_comment_prose() {
        let source = include_str!("execution_adapter.rs");
        assert!(
            source.contains("crate::state") || source.contains("crate::conflict"),
            "this test's premise requires the doc comments to still mention these symbols; \
             if this fails, execution_adapter.rs's docs changed and this regression test's \
             premise should be re-checked"
        );
        let (ec15, ec16) = check_no_direct_succeeded_or_merge_authority();
        assert_eq!(
            ec15.passed(),
            Some(true),
            "doc-comment mentions of crate::state must not be treated as code-level violations"
        );
        assert_eq!(
            ec16.passed(),
            Some(true),
            "doc-comment mentions of crate::conflict must not be treated as code-level violations"
        );
    }

    #[test]
    fn ec17_proves_two_different_adapters_drive_through_identical_calling_code() {
        fn drive(adapter: &mut dyn ExecutionAdapter, request: &ExecReq) -> FinalizationClaim {
            adapter.start(request).expect("start ok");
            let _ = adapter.observe();
            adapter.finalize()
        }

        let mut adapter_a = fake_adapter("executor-a", ExecutionState::Completed);
        let mut adapter_b = fake_adapter("executor-b", ExecutionState::Completed);
        let request = sample_request();

        let claim_a = drive(&mut adapter_a, &request);
        let claim_b = drive(&mut adapter_b, &request);

        let result = check_executor_replacement(
            claim_a,
            claim_b,
            &adapter_a.identity(),
            &adapter_b.identity(),
            &synthetic(),
            &synthetic(),
        );
        assert_eq!(result.passed(), Some(true));
        assert!(result.synthetic());
    }

    #[test]
    fn suite_disposition_is_synthetic_only_even_when_every_applicable_item_passes() {
        let mut results = Vec::new();
        for item in ConformanceItem::all() {
            if !item.is_applicable_before_p9() {
                continue;
            }
            // EC-15/16 are the exception: genuinely non-synthetic even in
            // a synthetic-adapter run, since they check this module's own
            // source text, not the adapter.
            let synthetic = !matches!(
                item,
                ConformanceItem::Ec15NoDirectSucceeded | ConformanceItem::Ec16NoDirectMerge
            );
            results.push(ConformanceResult::observed(
                item,
                true,
                synthetic,
                "synthetic",
            ));
        }
        let report = ConformanceSuiteReport::from_results(results);
        assert_eq!(report.disposition(), SuiteDisposition::SyntheticOnly);
    }

    #[test]
    fn suite_disposition_is_blocked_when_an_applicable_item_is_missing() {
        let results = vec![ConformanceResult::observed(
            ConformanceItem::Ec01Identity,
            true,
            true,
            "only one item present",
        )];
        let report = ConformanceSuiteReport::from_results(results);
        assert_eq!(report.disposition(), SuiteDisposition::Blocked);
    }

    #[test]
    fn suite_disposition_never_reaches_real_conformance_passed_from_synthetic_evidence() {
        // Even a maximally favorable synthetic run must not claim real
        // conformance -- this is the single most important behavioral
        // guarantee this module provides.
        let mut results = Vec::new();
        for item in ConformanceItem::all() {
            if item.is_applicable_before_p9() {
                results.push(ConformanceResult::observed(
                    item,
                    true,
                    true,
                    "all-green synthetic run",
                ));
            } else {
                results.push(ConformanceResult::not_applicable(
                    item,
                    "not applicable pre-P9",
                ));
            }
        }
        let report = ConformanceSuiteReport::from_results(results);
        assert_ne!(
            report.disposition(),
            SuiteDisposition::RealConformancePassed
        );
        assert_ne!(
            report.disposition(),
            SuiteDisposition::RealConformancePartial
        );
    }

    #[test]
    fn readiness_assessment_never_reports_f_observed_without_real_credentials() {
        let mut results = Vec::new();
        for item in ConformanceItem::all() {
            if item.is_applicable_before_p9() {
                results.push(ConformanceResult::observed(item, true, true, "synthetic"));
            } else {
                results.push(ConformanceResult::not_applicable(
                    item,
                    "not applicable pre-P9",
                ));
            }
        }
        let report = ConformanceSuiteReport::from_results(results);
        // Environment facts deliberately left at the honest default.
        let env = ExternalEnvironmentFacts::unconfirmed();
        let answers = assess_phase5_readiness(&report, &env);
        let f = answers.iter().find(|a| a.label == "F").unwrap();
        assert_eq!(f.status, EvidenceStatus::Blocked);
        let j = answers.iter().find(|a| a.label == "J").unwrap();
        assert_eq!(
            j.detail,
            "none -- P5-W03/W04/W05 remain BLOCKED until F is Observed"
        );
    }

    #[test]
    fn readiness_assessment_defaults_never_fabricate_environment_facts() {
        let env = ExternalEnvironmentFacts::unconfirmed();
        assert_eq!(env.aider_available, EvidenceStatus::Unknown);
        assert_eq!(env.jcode_available, EvidenceStatus::Unknown);
        assert_eq!(env.provider_credentials_available, EvidenceStatus::Unknown);
    }

    // -------------------------------------------------------------
    // Adversarial construction tests (Part 2 requirement): prove the
    // hardened public constructor surface cannot express the invalid
    // combinations the forensic audit found representable before this
    // pass. These deliberately use ONLY the public API (constructors +
    // accessors), the same surface any external caller/module is
    // restricted to, rather than relying on this submodule's incidental
    // access to private fields.
    // -------------------------------------------------------------

    #[test]
    fn blocked_can_never_carry_a_passed_value() {
        let r = ConformanceResult::blocked(ConformanceItem::Ec11RestartReconciliation, true, "x");
        // There is no method to set `passed` on a Blocked result -- this
        // assertion is the only possible outcome of the constructor, not
        // one of several a caller could have chosen.
        assert_eq!(r.status(), EvidenceStatus::Blocked);
        assert_eq!(r.passed(), None);
    }

    #[test]
    fn unknown_can_never_carry_a_favorable_passed_value() {
        let r = ConformanceResult::unknown(ConformanceItem::Ec13CredentialRedaction, false, "x");
        assert_eq!(r.status(), EvidenceStatus::Unknown);
        assert_eq!(r.passed(), None);
    }

    #[test]
    fn not_applicable_can_never_masquerade_as_passed() {
        let r = ConformanceResult::not_applicable(ConformanceItem::Ec14CapabilityDenial, "x");
        assert_eq!(r.status(), EvidenceStatus::NotApplicable);
        assert_eq!(r.passed(), None);
        assert!(!r.synthetic());
    }

    #[test]
    fn observed_and_verified_still_work_for_every_existing_valid_call_site_shape() {
        let o = ConformanceResult::observed(ConformanceItem::Ec01Identity, true, false, "x");
        assert_eq!(o.status(), EvidenceStatus::Observed);
        assert_eq!(o.passed(), Some(true));
        assert!(!o.synthetic());

        let v =
            ConformanceResult::verified(ConformanceItem::Ec15NoDirectSucceeded, false, false, "x");
        assert_eq!(v.status(), EvidenceStatus::Verified);
        assert_eq!(v.passed(), Some(false));
    }

    #[test]
    fn missing_applicable_evidence_remains_blocked_after_hardening() {
        // Re-confirms suite_disposition_is_blocked_when_an_applicable_item_is_missing's
        // guarantee still holds through the hardened constructor surface.
        let results = vec![ConformanceResult::observed(
            ConformanceItem::Ec01Identity,
            true,
            false,
            "only one item present",
        )];
        let report = ConformanceSuiteReport::from_results(results);
        assert_eq!(report.disposition(), SuiteDisposition::Blocked);
    }
}
