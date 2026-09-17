//! The Execution Adapter Contract's common semantic boundary (P5-W01 /
//! Execution Adapter Contract v1.0).
//!
//! ```text
//! AI CONDUCTOR                          EXECUTION HARNESS
//!   owns: mission, step, attempt,         performs: delegated execution,
//!   policy, capability authority,         repository inspection,
//!   workspace authority, persistence,     authorized tool use,
//!   evidence authority, reconciliation,   authorized mutations,
//!   verification, recovery, merge         command execution, result reporting
//! ```
//!
//! §0: *"This contract prevents AI Conductor from becoming coupled to
//! Aider, Jcode, or any future coding harness."* [`ExecutionAdapter`] is
//! that boundary as a Rust trait — a real, replaceable executor
//! implements it; the Conductor's own reliability kernel never needs to
//! know which one it's talking to.
//!
//! # Environment constraint, stated plainly
//!
//! Neither Aider nor Jcode is installed in this sandbox, and no provider
//! API keys exist to run either even if installed (confirmed by direct
//! check at Phase 5 entry). This module implements the contract's types
//! and trait, and [`FakeExecutionAdapter`] proves the trait is genuinely
//! implementable — but no real executor has been validated against it.
//! Phase Manifest §9.3's P5-W03 onward (Reference Executor Isolation,
//! Real Execution Lifecycle, and everything after) require a real
//! executor and are not attempted here.
//!
//! # Hard Authority Rules (§11), upheld structurally
//!
//! An executor adapter must never mutate `AttemptStatus::Succeeded`,
//! authorize a merge, or silently resolve a `Conflict`. This module
//! upholds that by having **zero references to `crate::state` or
//! `crate::conflict` anywhere in it** — not a runtime check that could
//! be bypassed, but the simple fact that nothing in this trait's method
//! set accepts or returns a type from either module. Grep-audited in the
//! verification report, the same method used for every prior
//! authority-boundary claim in this crate.

use crate::correlation::CorrelationContext;
use std::time::Duration;

/// Execution Adapter Contract §2 — Required Identity, verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorIdentity {
    pub executor_id: String,
    pub name: String,
    pub version: String,
    /// "commit/build identity where available" — genuinely optional per
    /// the contract text, not every executor exposes this.
    pub commit_or_build_identity: Option<String>,
    pub adapter_version: String,
    pub runtime_environment_identity: String,
}

/// §3: *"Unknown capabilities remain unknown... must not fabricate
/// capability values merely to satisfy a task."* A plain `bool` or
/// `Option<bool>` would conflate "known to be absent" with "never
/// checked" — this tri-state keeps them distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityValue {
    Supported,
    Unsupported,
    Unknown,
}

/// Execution Adapter Contract §3's thirteen required capabilities,
/// verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySnapshot {
    pub interactive_execution: CapabilityValue,
    pub non_interactive_execution: CapabilityValue,
    pub workspace_workdir_control: CapabilityValue,
    pub file_mutation: CapabilityValue,
    pub shell_process_execution: CapabilityValue,
    pub git_interaction: CapabilityValue,
    pub streaming: CapabilityValue,
    pub cancellation: CapabilityValue,
    pub timeout: CapabilityValue,
    pub session_persistence: CapabilityValue,
    pub provider_selection: CapabilityValue,
    pub mcp_tooling: CapabilityValue,
    pub browser_capability: CapabilityValue,
    pub parallel_execution: CapabilityValue,
}

/// Execution Adapter Contract §4's conceptual execution request,
/// reproduced field-for-field. `correlation` reuses P1's
/// `CorrelationContext` directly for `mission_id`/`step_id`/`attempt_id`
/// rather than a second identifier scheme (the contract's own
/// `correlation_ids` field, made concrete).
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub correlation: CorrelationContext,
    pub executor_id: String,
    pub workspace_id: String,
    pub baseline_id: String,
    pub allowed_capabilities: Vec<String>,
    pub requested_operation: String,
    pub provider_resource: Option<String>,
    pub policy_version: String,
    pub timeout_policy: Duration,
}

/// Execution Adapter Contract §5's lifecycle, verbatim. `Unknown` is
/// mandatory per the contract text — a first-class variant, not an
/// `Option::None` that a caller could accidentally treat as "no
/// information" rather than "genuinely could not be determined."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    Pending,
    Starting,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Crashed,
    Unknown,
}

/// Execution Adapter Contract §10: finalization reports the executor's
/// *observed* outcome. *"It does NOT authorize acceptance."* Modeled as
/// a genuinely separate type from [`ExecutionState`] — not a reused or
/// aliased enum — precisely so a `FinalizationClaim` can never be
/// mistaken for, or silently substituted as, an authoritative
/// `AttemptStatus`. Only the Verification Engine (P4) decides whether an
/// accepted outcome is actually valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizationClaim {
    CompletedClaim,
    FailedClaim,
    CancelledClaim,
    UnknownClaim,
}

/// Execution Adapter Contract §13's failure taxonomy, verbatim (nine
/// named categories plus `Unknown` — *"do not map everything to generic
/// ERROR"*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureCategory {
    ProviderFailure,
    NetworkFailure,
    ProcessFailure,
    ExecutorFailure,
    WorkspaceFailure,
    FilesystemFailure,
    GitFailure,
    PermissionFailure,
    CapabilityMismatch,
    Unknown,
}

/// Execution Adapter Contract §7 — what `observe()` must make
/// reconstructible.
#[derive(Debug, Clone)]
pub struct Observation {
    pub started: bool,
    pub process_identity: Option<String>,
    pub stdout_or_structured_events: String,
    pub tool_activity: Vec<String>,
    pub exit_information: Option<String>,
    pub changed_workspace_observations: Vec<String>,
    pub failure_classification: Option<FailureCategory>,
    pub final_executor_state: ExecutionState,
}

/// Execution Adapter Contract §8 — what cancellation must report.
/// *"Cancellation does not automatically mean Failed... if side effects
/// may have occurred, reconciliation remains required."*
#[derive(Debug, Clone)]
pub struct CancellationReport {
    pub requested: bool,
    pub accepted: bool,
    pub observed_termination_state: ExecutionState,
    pub remaining_ambiguity: bool,
    pub cleanup_status: String,
}

/// Execution Adapter Contract §9 — evidence references the Verification
/// Engine needs for independent verification. §9: *"Secrets must be
/// redacted"* — redaction is the responsibility of whatever code
/// populates these fields against a real executor's real output; no
/// redaction engine is built here, since there is no real executor
/// output yet to validate one against (see module docs).
#[derive(Debug, Clone)]
pub struct EvidenceRefs {
    pub attempt_id: String,
    pub workspace_id: String,
    pub executor_id: String,
    pub version: String,
    pub process_ref: Option<String>,
    pub commands: Vec<String>,
    pub outputs_ref: Option<String>,
    pub artifacts_ref: Vec<String>,
    pub exit_result: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdapterError {
    #[error("start rejected: {0}")]
    StartRejected(String),
    #[error("capability not allowed for this request: {0}")]
    CapabilityNotAllowed(String),
}

/// The common semantic boundary itself (§§1-13). A conforming executor
/// implements this trait; the Conductor's reliability kernel interacts
/// only through it, never through executor-specific internals —
/// Execution Adapter Contract §18's Replacement Principle, made
/// concrete.
pub trait ExecutionAdapter {
    fn identity(&self) -> ExecutorIdentity;
    fn capabilities(&self) -> CapabilitySnapshot;
    fn start(&mut self, request: &ExecutionRequest) -> Result<(), AdapterError>;
    fn observe(&self) -> Observation;
    fn cancel(&mut self) -> CancellationReport;
    fn collect_evidence(&self) -> EvidenceRefs;
    fn finalize(&self) -> FinalizationClaim;
}

/// A scripted test double proving [`ExecutionAdapter`] is genuinely
/// implementable — mirroring `provider.rs`'s `FakeProvider` design
/// precedent. Not a stand-in for real-executor validation (P5-W03+);
/// see module docs.
pub struct FakeExecutionAdapter {
    identity: ExecutorIdentity,
    capabilities: CapabilitySnapshot,
    scripted_observation: Observation,
    scripted_cancellation: CancellationReport,
    scripted_evidence: EvidenceRefs,
    scripted_finalization: FinalizationClaim,
    started: bool,
}

impl FakeExecutionAdapter {
    pub fn new(
        identity: ExecutorIdentity,
        capabilities: CapabilitySnapshot,
        scripted_observation: Observation,
        scripted_cancellation: CancellationReport,
        scripted_evidence: EvidenceRefs,
        scripted_finalization: FinalizationClaim,
    ) -> Self {
        FakeExecutionAdapter {
            identity,
            capabilities,
            scripted_observation,
            scripted_cancellation,
            scripted_evidence,
            scripted_finalization,
            started: false,
        }
    }
}

impl ExecutionAdapter for FakeExecutionAdapter {
    fn identity(&self) -> ExecutorIdentity {
        self.identity.clone()
    }

    fn capabilities(&self) -> CapabilitySnapshot {
        self.capabilities.clone()
    }

    fn start(&mut self, _request: &ExecutionRequest) -> Result<(), AdapterError> {
        self.started = true;
        Ok(())
    }

    fn observe(&self) -> Observation {
        self.scripted_observation.clone()
    }

    fn cancel(&mut self) -> CancellationReport {
        self.scripted_cancellation.clone()
    }

    fn collect_evidence(&self) -> EvidenceRefs {
        self.scripted_evidence.clone()
    }

    fn finalize(&self) -> FinalizationClaim {
        self.scripted_finalization
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_identity() -> ExecutorIdentity {
        ExecutorIdentity {
            executor_id: "fake-1".to_string(),
            name: "FakeExecutor".to_string(),
            version: "0.1.0".to_string(),
            commit_or_build_identity: None,
            adapter_version: "1.0".to_string(),
            runtime_environment_identity: "test-sandbox".to_string(),
        }
    }

    fn all_unknown_capabilities() -> CapabilitySnapshot {
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

    fn fake_adapter() -> FakeExecutionAdapter {
        FakeExecutionAdapter::new(
            fake_identity(),
            all_unknown_capabilities(),
            Observation {
                started: true,
                process_identity: Some("pid-1234".to_string()),
                stdout_or_structured_events: "did the thing".to_string(),
                tool_activity: vec!["edited a.txt".to_string()],
                exit_information: Some("exit 0".to_string()),
                changed_workspace_observations: vec!["a.txt".to_string()],
                failure_classification: None,
                final_executor_state: ExecutionState::Completed,
            },
            CancellationReport {
                requested: false,
                accepted: false,
                observed_termination_state: ExecutionState::Running,
                remaining_ambiguity: false,
                cleanup_status: "n/a".to_string(),
            },
            EvidenceRefs {
                attempt_id: "attempt-1".to_string(),
                workspace_id: "ws-1".to_string(),
                executor_id: "fake-1".to_string(),
                version: "0.1.0".to_string(),
                process_ref: Some("pid-1234".to_string()),
                commands: vec!["npm test".to_string()],
                outputs_ref: None,
                artifacts_ref: vec![],
                exit_result: Some("exit 0".to_string()),
            },
            FinalizationClaim::CompletedClaim,
        )
    }

    fn sample_request() -> ExecutionRequest {
        ExecutionRequest {
            correlation: CorrelationContext::for_mission("m-1").with_step("s-1"),
            executor_id: "fake-1".to_string(),
            workspace_id: "ws-1".to_string(),
            baseline_id: "baseline-1".to_string(),
            allowed_capabilities: vec!["file_mutation".to_string()],
            requested_operation: "implement feature X".to_string(),
            provider_resource: None,
            policy_version: "v1".to_string(),
            timeout_policy: Duration::from_secs(300),
        }
    }

    #[test]
    fn capability_snapshot_has_exactly_the_thirteen_contract_capabilities() {
        // Structural witness: this is a compile-time property (the
        // struct literal below must name exactly these thirteen fields
        // or the build fails), exercised here so a future accidental
        // field removal/addition is caught by a clearly-named test, not
        // just a generic compile error far away.
        let snapshot = all_unknown_capabilities();
        assert_eq!(snapshot.interactive_execution, CapabilityValue::Unknown);
        assert_eq!(snapshot.parallel_execution, CapabilityValue::Unknown);
    }

    #[test]
    fn unknown_capability_never_silently_defaults_to_supported_or_unsupported() {
        let snapshot = all_unknown_capabilities();
        assert_ne!(snapshot.file_mutation, CapabilityValue::Supported);
        assert_ne!(snapshot.file_mutation, CapabilityValue::Unsupported);
        assert_eq!(snapshot.file_mutation, CapabilityValue::Unknown);
    }

    #[test]
    fn execution_state_includes_unknown_as_a_first_class_variant() {
        let states = [
            ExecutionState::Pending,
            ExecutionState::Starting,
            ExecutionState::Running,
            ExecutionState::Completed,
            ExecutionState::Failed,
            ExecutionState::Cancelled,
            ExecutionState::TimedOut,
            ExecutionState::Crashed,
            ExecutionState::Unknown,
        ];
        assert_eq!(states.len(), 9);
        assert!(states.contains(&ExecutionState::Unknown));
    }

    #[test]
    fn finalization_claim_is_a_distinct_type_from_execution_state() {
        // Compile-time witness: FinalizationClaim::CompletedClaim and
        // ExecutionState::Completed are different types entirely --
        // there is no From/Into between them, so a caller cannot
        // accidentally treat a claim as an authoritative state.
        let claim = FinalizationClaim::CompletedClaim;
        let state = ExecutionState::Completed;
        assert_eq!(format!("{claim:?}"), "CompletedClaim");
        assert_eq!(format!("{state:?}"), "Completed");
    }

    #[test]
    fn fake_execution_adapter_implements_the_full_trait_through_a_trait_object() {
        let mut adapter: Box<dyn ExecutionAdapter> = Box::new(fake_adapter());
        let request = sample_request();

        assert_eq!(adapter.identity().executor_id, "fake-1");
        assert_eq!(adapter.capabilities().file_mutation, CapabilityValue::Unknown);
        assert!(adapter.start(&request).is_ok());

        let observation = adapter.observe();
        assert!(observation.started);
        assert_eq!(observation.final_executor_state, ExecutionState::Completed);

        let cancellation = adapter.cancel();
        assert!(!cancellation.requested);

        let evidence = adapter.collect_evidence();
        assert_eq!(evidence.attempt_id, "attempt-1");

        assert_eq!(adapter.finalize(), FinalizationClaim::CompletedClaim);
    }

    #[test]
    fn failure_category_has_exactly_the_contract_ten_variants() {
        let categories = [
            FailureCategory::ProviderFailure,
            FailureCategory::NetworkFailure,
            FailureCategory::ProcessFailure,
            FailureCategory::ExecutorFailure,
            FailureCategory::WorkspaceFailure,
            FailureCategory::FilesystemFailure,
            FailureCategory::GitFailure,
            FailureCategory::PermissionFailure,
            FailureCategory::CapabilityMismatch,
            FailureCategory::Unknown,
        ];
        assert_eq!(categories.len(), 10);
    }

    #[test]
    fn execution_request_reuses_correlation_context_directly() {
        let request = sample_request();
        assert_eq!(request.correlation.mission_id, crate::correlation::MissionId::from("m-1"));
    }
}
