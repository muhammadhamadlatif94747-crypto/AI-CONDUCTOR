//! Executor Capability Snapshot (P5-W02 / Phase Manifest §9.3).
//!
//! *"Record executor identity, version/commit, capabilities, workspace
//! behavior, lifecycle semantics, and verification provenance."* Six
//! dimensions; this module gives each a real place to live and a real,
//! durable way to be recorded — not an in-memory struct that
//! disappears when the process exits.
//!
//! - **Identity, version/commit, capabilities** — pulled directly from a
//!   real `&dyn ExecutionAdapter` (P5-W01): `identity()` and
//!   `capabilities()`.
//! - **Workspace behavior** — does the executor ever mutate the live
//!   workspace directly, or only the Conductor-created isolated one
//!   (Execution Adapter Contract §12's Workspace Rule)? This cannot come
//!   from the adapter's own methods — it requires actually running a
//!   scenario against a real executor and observing what it touched,
//!   which is P5-W03/W04's job. Supplied by the caller here.
//! - **Lifecycle semantics** — does this executor reliably distinguish
//!   `Cancelled` from `Unknown`, does cancellation actually stop work?
//!   Same situation: requires real observation, supplied by the caller.
//! - **Verification provenance** — reuses P4-W02's `Evidence` fields
//!   directly (`EvidenceProvenance`, `IntegrityLevel`, a captured
//!   timestamp), not a second provenance concept invented here.
//!
//! # Environment constraint, stated plainly
//!
//! No real executor is available in this sandbox (confirmed at Phase 5
//! entry). This module's own tests capture a snapshot from
//! `FakeExecutionAdapter` with caller-supplied workspace-behavior and
//! lifecycle-semantics values — the *mechanism* is real and tested; the
//! *content* in this task's tests is clearly a test double's, never
//! presented as data about Aider or Jcode.

use crate::console::{ConsoleError, StorePaths};
use crate::event::{ClockSource, EventContext, NewEvent};
use crate::event_log::EventLog;
use crate::evidence::{EvidenceProvenance, IntegrityLevel};
use crate::execution_adapter::{CapabilitySnapshot, CapabilityValue, ExecutionAdapter, ExecutorIdentity};
use std::path::Path;

/// Execution Adapter Contract §12's Workspace Rule, made observable.
/// Each field is a [`CapabilityValue`] tri-state (P5-W01's type, reused
/// directly) — an unobserved behavior is `Unknown`, never silently
/// assumed favorable or unfavorable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBehaviorNotes {
    /// Did every observed mutation stay inside the Conductor-created
    /// isolated workspace (never touching the live workspace directly)?
    pub respects_isolated_workspace_boundary: CapabilityValue,
    /// Did the executor ever attempt an operation outside the
    /// workspace_id it was given?
    pub attempted_out_of_scope_access: CapabilityValue,
}

/// Lifecycle behavior specific to this executor, beyond the generic
/// state machine P5-W01 already defines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleSemanticsNotes {
    /// Does a cancellation request reliably stop further mutation, or
    /// can work continue after `cancel()` returns?
    pub cancellation_is_effective: CapabilityValue,
    /// Does this executor report `Unknown` for a genuinely ambiguous
    /// exit, rather than guessing `Completed`/`Failed`?
    pub reports_unknown_for_ambiguous_exit: CapabilityValue,
}

/// The durable record: all six of Phase Manifest §9.3 P5-W02's named
/// dimensions in one place.
#[derive(Debug, Clone)]
pub struct ExecutorCapabilitySnapshotRecord {
    pub identity: ExecutorIdentity,
    pub capabilities: CapabilitySnapshot,
    pub workspace_behavior: WorkspaceBehaviorNotes,
    pub lifecycle_semantics: LifecycleSemanticsNotes,
    pub provenance: EvidenceProvenance,
    pub integrity: IntegrityLevel,
    pub captured_at_ms: u64,
}

/// Pull identity and capabilities directly from a real adapter (through
/// the trait object, not re-typed by hand) and combine them with the
/// caller-supplied observations that only a real execution run could
/// produce.
pub fn capture_snapshot(
    adapter: &dyn ExecutionAdapter,
    workspace_behavior: WorkspaceBehaviorNotes,
    lifecycle_semantics: LifecycleSemanticsNotes,
    provenance: EvidenceProvenance,
    integrity: IntegrityLevel,
    captured_at_ms: u64,
) -> ExecutorCapabilitySnapshotRecord {
    ExecutorCapabilitySnapshotRecord {
        identity: adapter.identity(),
        capabilities: adapter.capabilities(),
        workspace_behavior,
        lifecycle_semantics,
        provenance,
        integrity,
        captured_at_ms,
    }
}

fn capability_value_as_str(v: CapabilityValue) -> &'static str {
    match v {
        CapabilityValue::Supported => "supported",
        CapabilityValue::Unsupported => "unsupported",
        CapabilityValue::Unknown => "unknown",
    }
}

/// Durably record a snapshot, mirroring `console::request_attempt_cancellation`
/// and `person_decision::record_person_decision`'s established pattern
/// exactly: same `StorePaths`, same `EventLog::open`/`NewEvent` path,
/// caller-supplied `now_ms` (no direct clock read, AC-12).
pub fn record_executor_capability_snapshot(
    store_dir: &Path,
    context: EventContext,
    record: &ExecutorCapabilitySnapshotRecord,
    now_ms: u64,
) -> Result<(), ConsoleError> {
    let paths = StorePaths::under(store_dir);
    let mut log = EventLog::open(&paths.events)?;
    log.append(NewEvent {
        event_type: "execution_adapter.capability_snapshot_recorded".to_string(),
        context,
        payload: serde_json::json!({
            "executor_id": record.identity.executor_id,
            "executor_name": record.identity.name,
            "executor_version": record.identity.version,
            "commit_or_build_identity": record.identity.commit_or_build_identity,
            "adapter_version": record.identity.adapter_version,
            "runtime_environment_identity": record.identity.runtime_environment_identity,
            "workspace_behavior": {
                "respects_isolated_workspace_boundary": capability_value_as_str(record.workspace_behavior.respects_isolated_workspace_boundary),
                "attempted_out_of_scope_access": capability_value_as_str(record.workspace_behavior.attempted_out_of_scope_access),
            },
            "lifecycle_semantics": {
                "cancellation_is_effective": capability_value_as_str(record.lifecycle_semantics.cancellation_is_effective),
                "reports_unknown_for_ambiguous_exit": capability_value_as_str(record.lifecycle_semantics.reports_unknown_for_ambiguous_exit),
            },
            "provenance": format!("{:?}", record.provenance),
            "integrity": format!("{:?}", record.integrity),
            "captured_at_ms": record.captured_at_ms,
        }),
        timestamp_ms: now_ms,
        clock_source: ClockSource::Virtual,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correlation::CorrelationContext;
    use crate::execution_adapter::{CancellationReport, ExecutionState, FakeExecutionAdapter, FinalizationClaim, Observation};

    fn dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ck_executor_snapshot_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("mkdir");
        d
    }

    fn ctx() -> EventContext {
        CorrelationContext::for_mission("m-1").with_step("s-1")
    }

    fn fake_adapter() -> FakeExecutionAdapter {
        FakeExecutionAdapter::new(
            ExecutorIdentity {
                executor_id: "fake-1".to_string(),
                name: "FakeExecutor".to_string(),
                version: "0.1.0".to_string(),
                commit_or_build_identity: Some("abc123".to_string()),
                adapter_version: "1.0".to_string(),
                runtime_environment_identity: "test-sandbox".to_string(),
            },
            CapabilitySnapshot {
                interactive_execution: CapabilityValue::Unknown,
                non_interactive_execution: CapabilityValue::Supported,
                workspace_workdir_control: CapabilityValue::Supported,
                file_mutation: CapabilityValue::Supported,
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
            },
            Observation {
                started: true,
                process_identity: None,
                stdout_or_structured_events: String::new(),
                tool_activity: vec![],
                exit_information: None,
                changed_workspace_observations: vec![],
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
            crate::execution_adapter::EvidenceRefs {
                attempt_id: "attempt-1".to_string(),
                workspace_id: "ws-1".to_string(),
                executor_id: "fake-1".to_string(),
                version: "0.1.0".to_string(),
                process_ref: None,
                commands: vec![],
                outputs_ref: None,
                artifacts_ref: vec![],
                exit_result: None,
            },
            FinalizationClaim::CompletedClaim,
        )
    }

    fn unknown_workspace_behavior() -> WorkspaceBehaviorNotes {
        WorkspaceBehaviorNotes {
            respects_isolated_workspace_boundary: CapabilityValue::Unknown,
            attempted_out_of_scope_access: CapabilityValue::Unknown,
        }
    }

    fn unknown_lifecycle_semantics() -> LifecycleSemanticsNotes {
        LifecycleSemanticsNotes {
            cancellation_is_effective: CapabilityValue::Unknown,
            reports_unknown_for_ambiguous_exit: CapabilityValue::Unknown,
        }
    }

    #[test]
    fn capture_snapshot_pulls_identity_and_capabilities_from_a_real_adapter_through_the_trait_object() {
        let adapter: Box<dyn ExecutionAdapter> = Box::new(fake_adapter());
        let snapshot = capture_snapshot(
            adapter.as_ref(),
            unknown_workspace_behavior(),
            unknown_lifecycle_semantics(),
            EvidenceProvenance::ProcessObservation,
            IntegrityLevel::SelfReported,
            1_000,
        );

        assert_eq!(snapshot.identity.executor_id, "fake-1");
        assert_eq!(snapshot.identity.commit_or_build_identity, Some("abc123".to_string()));
        assert_eq!(snapshot.capabilities.file_mutation, CapabilityValue::Supported);
        assert_eq!(snapshot.capabilities.interactive_execution, CapabilityValue::Unknown);
    }

    #[test]
    fn unobserved_workspace_and_lifecycle_fields_are_unknown_not_silently_favorable() {
        let adapter: Box<dyn ExecutionAdapter> = Box::new(fake_adapter());
        let snapshot = capture_snapshot(
            adapter.as_ref(),
            unknown_workspace_behavior(),
            unknown_lifecycle_semantics(),
            EvidenceProvenance::ProcessObservation,
            IntegrityLevel::SelfReported,
            1_000,
        );

        assert_eq!(snapshot.workspace_behavior.respects_isolated_workspace_boundary, CapabilityValue::Unknown);
        assert_ne!(snapshot.workspace_behavior.respects_isolated_workspace_boundary, CapabilityValue::Supported);
        assert_eq!(snapshot.lifecycle_semantics.cancellation_is_effective, CapabilityValue::Unknown);
        assert_ne!(snapshot.lifecycle_semantics.cancellation_is_effective, CapabilityValue::Unsupported);
    }

    #[test]
    fn record_executor_capability_snapshot_writes_a_real_readable_event() {
        let store = dir("record_snapshot");
        let adapter: Box<dyn ExecutionAdapter> = Box::new(fake_adapter());
        let snapshot = capture_snapshot(
            adapter.as_ref(),
            WorkspaceBehaviorNotes {
                respects_isolated_workspace_boundary: CapabilityValue::Supported,
                attempted_out_of_scope_access: CapabilityValue::Unsupported,
            },
            LifecycleSemanticsNotes {
                cancellation_is_effective: CapabilityValue::Supported,
                reports_unknown_for_ambiguous_exit: CapabilityValue::Unknown,
            },
            EvidenceProvenance::ProcessObservation,
            IntegrityLevel::IndependentlyVerified,
            2_000,
        );

        record_executor_capability_snapshot(&store, ctx(), &snapshot, 2_000).expect("record snapshot");

        let paths = StorePaths::under(&store);
        let log = EventLog::open(&paths.events).expect("reopen log");
        let events = log.read_all().expect("read all events");
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.event_type, "execution_adapter.capability_snapshot_recorded");
        assert_eq!(event.payload.get("executor_id").and_then(|v| v.as_str()), Some("fake-1"));
        assert_eq!(
            event.payload.get("workspace_behavior").and_then(|w| w.get("respects_isolated_workspace_boundary")).and_then(|v| v.as_str()),
            Some("supported")
        );
        assert_eq!(
            event.payload.get("lifecycle_semantics").and_then(|l| l.get("reports_unknown_for_ambiguous_exit")).and_then(|v| v.as_str()),
            Some("unknown")
        );
        assert_eq!(event.payload.get("captured_at_ms").and_then(|v| v.as_u64()), Some(2_000));

        let _ = std::fs::remove_dir_all(&store);
    }
}
