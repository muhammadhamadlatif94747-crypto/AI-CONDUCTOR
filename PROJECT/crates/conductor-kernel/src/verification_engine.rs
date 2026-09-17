//! The Verification Engine's final decision (P4-W04 / Blueprint §7, §8;
//! Invariant 14).
//!
//! ```text
//! PipelineResult (P4-W01)   MacEvaluationResult (P4-W03)   Evidence (P4-W02)
//!         │                          │                          │
//!         └──────────────┬───────────┴──────────────────────────┘
//!                        decide()
//!                          │
//!              Succeed  |  Fail(reasons)
//!                  │
//!         attempt_succeeded_transition()
//!                  │
//!   state::try_attempt_transition(VerificationEngine, Running, Succeeded)
//! ```
//!
//! `state.rs`'s gate has restricted the `Succeeded` edge to
//! `TransitionActor::VerificationEngine` since Phase 1, re-confirmed at
//! P2-W04. That answers *who* may call it. This module answers the
//! other half: does the one caller who's authorized only call it when
//! it has actually been earned. [`attempt_succeeded_transition`] calls
//! the real gate exactly once, inside the `Succeed` branch — the `Fail`
//! branch returns an error before reaching that call at all, proven for
//! every one of Phase Manifest §8.4's eight required scenarios, not just
//! asserted.
//!
//! # Two checks this task adds on top of P4-W02/P4-W03, not inside them
//!
//! Neither is a literal field in Blueprint §8's JSON schema; both are
//! justified by Principle 2 / Invariant 7 (a model's own claim is never
//! authoritative) and by Phase Manifest §8.4's own required tests, which
//! neither P4-W02 nor P4-W03 individually covers:
//!
//! - **Wrong verification method**: a blocking criterion's evidence must
//!   have come from the *kind* of check the criterion actually declared
//!   (`verification_type` ↔ `observation_type`), via
//!   [`verification_type_expects`] — an explicit, documented,
//!   conservative mapping, not presented as verbatim Blueprint content.
//! - **False model success**: a blocking criterion's evidence must be
//!   `IntegrityLevel::IndependentlyVerified`, never merely
//!   `SelfReported` — mirroring the rule P4-W01 already established for
//!   `RequirementVerificationClass::ModelAssisted` (a model's own
//!   assessment alone can never satisfy a blocking requirement).

use crate::evidence::{Evidence, IntegrityLevel, ObservationType};
use crate::mission_acceptance::{MacEvaluationResult, MissionAcceptanceContract, VerificationType};
use crate::state::{try_attempt_transition, AttemptStatus, TransitionActor, TransitionError};
use crate::verification::PipelineResult;
use std::collections::HashMap;

/// This task's own explicit, documented mapping from a criterion's
/// declared verification method to the observation type its evidence
/// must have come from — not found verbatim in the Blueprint text.
fn verification_type_expects(vtype: VerificationType) -> ObservationType {
    match vtype {
        VerificationType::TestCommand => ObservationType::TestExecution,
        VerificationType::DomCheck => ObservationType::DomCheck,
        VerificationType::IntegrationTest => ObservationType::TestExecution,
        VerificationType::TestSuiteDiff => ObservationType::TestExecution,
        VerificationType::DiffScopeCheck => ObservationType::DiffCheck,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuccessDecision {
    Succeed,
    Fail { reasons: Vec<String> },
}

/// Compose P4-W01's pipeline result, P4-W03's MAC evaluation, and the
/// two additional checks above into one decision. `Succeed` requires
/// ALL of: the pipeline passed entirely; every blocking criterion is
/// `Satisfied`; every blocking criterion's evidence's observation_type
/// matches what its verification_type expects; every blocking
/// criterion's evidence is `IndependentlyVerified`, not `SelfReported`.
pub fn decide(
    pipeline_result: &PipelineResult,
    contract: &MissionAcceptanceContract,
    mac_evaluation: &MacEvaluationResult,
    evidence_by_criterion_id: &HashMap<String, Evidence>,
) -> SuccessDecision {
    let mut reasons = Vec::new();

    if !pipeline_result.all_passed() {
        reasons.push("verification pipeline did not pass every stage".to_string());
    }

    if !mac_evaluation.all_blocking_satisfied() {
        for unmet in mac_evaluation.unmet_blocking_criteria() {
            reasons.push(format!("blocking criterion {} unmet: {:?}", unmet.criterion_id, unmet.outcome));
        }
    }

    for criterion in contract.criteria.iter().filter(|c| c.blocking) {
        // Only meaningful to check method/integrity if the criterion at
        // least had evidence at all -- a missing-evidence failure is
        // already captured above via mac_evaluation.
        if let Some(evidence) = evidence_by_criterion_id.get(&criterion.id) {
            let expected_observation = verification_type_expects(criterion.verification_type);
            if evidence.observation_type != expected_observation {
                reasons.push(format!(
                    "criterion {}: wrong verification method (evidence observation_type {:?}, expected {:?})",
                    criterion.id, evidence.observation_type, expected_observation
                ));
            }
            if evidence.integrity != IntegrityLevel::IndependentlyVerified {
                reasons.push(format!(
                    "criterion {}: false model success -- evidence integrity is {:?}, not independently verified",
                    criterion.id, evidence.integrity
                ));
            }
        }
    }

    if reasons.is_empty() {
        SuccessDecision::Succeed
    } else {
        SuccessDecision::Fail { reasons }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VerificationEngineError {
    #[error("verification did not pass: {0:?}")]
    VerificationFailed(Vec<String>),
    #[error("state transition rejected: {0:?}")]
    StateTransitionRejected(TransitionError),
}

/// The one function in this crate permitted to call
/// `try_attempt_transition` for the `Succeeded` edge. Calls it exactly
/// once, only inside the `Succeed` branch — the `Fail` branch returns
/// before reaching that call at all.
pub fn attempt_succeeded_transition(decision: &SuccessDecision) -> Result<(), VerificationEngineError> {
    match decision {
        SuccessDecision::Succeed => {
            try_attempt_transition(TransitionActor::VerificationEngine, AttemptStatus::Running, AttemptStatus::Succeeded)
                .map_err(VerificationEngineError::StateTransitionRejected)
        }
        SuccessDecision::Fail { reasons } => Err(VerificationEngineError::VerificationFailed(reasons.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::Baseline;
    use crate::evidence::{EvidenceProvenance, Reproducibility};
    use crate::mission_acceptance::{evaluate_contract, Criterion, ExpectedResult, RequiredEvidenceSpec};
    use crate::reconciliation::ExpectedOperation;
    use crate::verification::{run_pipeline, Requirement, RequirementVerificationClass};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::time::Duration;

    fn baseline(entries: &[(&str, &str)]) -> Baseline {
        Baseline {
            entries: entries.iter().map(|(p, h)| (PathBuf::from(p), h.to_string())).collect::<BTreeMap<_, _>>(),
        }
    }

    fn good_evidence(execution_id: &str, produced_at_ms: u64) -> Evidence {
        Evidence {
            provenance: EvidenceProvenance::LocalTestRunner,
            observation_type: ObservationType::TestExecution,
            produced_at_ms,
            execution_id: execution_id.to_string(),
            source: "npm test -- auth.test.ts".to_string(),
            reproducibility: Reproducibility::Deterministic,
            integrity: IntegrityLevel::IndependentlyVerified,
            result: true,
        }
    }

    fn one_criterion_contract() -> MissionAcceptanceContract {
        MissionAcceptanceContract {
            mission: "add auth".to_string(),
            criteria: vec![Criterion {
                id: "auth-03".to_string(),
                description: "login test passes".to_string(),
                verification_type: VerificationType::TestCommand,
                command: Some("npm test -- auth.test.ts".to_string()),
                expected_result: ExpectedResult::ExitCodeZero,
                required_evidence: RequiredEvidenceSpec {
                    provenance: EvidenceProvenance::LocalTestRunner,
                    max_freshness: Duration::from_secs(300),
                    execution_id: "current_attempt".to_string(),
                },
                blocking: true,
            }],
        }
    }

    fn passing_pipeline() -> PipelineResult {
        let before = baseline(&[("a.txt", "h1")]);
        let after = baseline(&[("a.txt", "h2")]);
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "test");
        run_pipeline(&before, &after, &expected, Ok(()), Ok(()), Ok(()), &[])
    }

    // -- Phase Manifest SS8.4's eight required scenarios --

    #[test]
    fn passing_acceptance_criteria_reaches_a_real_succeeded_transition() {
        let contract = one_criterion_contract();
        let mut evidence = HashMap::new();
        evidence.insert("auth-03".to_string(), good_evidence("current_attempt", 0));
        let mac_eval = evaluate_contract(&contract, &evidence, 60_000);
        let pipeline = passing_pipeline();

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        assert_eq!(decision, SuccessDecision::Succeed);

        let result = attempt_succeeded_transition(&decision);
        assert!(result.is_ok(), "the real state.rs gate must accept this: {result:?}");
    }

    #[test]
    fn failing_acceptance_criteria_never_reaches_succeeded() {
        let contract = one_criterion_contract();
        let mut evidence = HashMap::new();
        // Wrong execution_id -- criterion is genuinely unmet.
        evidence.insert("auth-03".to_string(), good_evidence("stale-prior-run", 0));
        let mac_eval = evaluate_contract(&contract, &evidence, 60_000);
        let pipeline = passing_pipeline();

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        assert!(matches!(decision, SuccessDecision::Fail { .. }));
        assert!(attempt_succeeded_transition(&decision).is_err());
    }

    #[test]
    fn stale_evidence_never_reaches_succeeded() {
        let contract = one_criterion_contract();
        let mut evidence = HashMap::new();
        evidence.insert("auth-03".to_string(), good_evidence("current_attempt", 0));
        // now_ms far beyond max_freshness (300s = 300_000ms).
        let mac_eval = evaluate_contract(&contract, &evidence, 999_999_999);
        let pipeline = passing_pipeline();

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        assert!(matches!(decision, SuccessDecision::Fail { .. }));
        assert!(attempt_succeeded_transition(&decision).is_err());
    }

    #[test]
    fn insufficient_evidence_never_reaches_succeeded() {
        let contract = one_criterion_contract();
        let evidence = HashMap::new(); // nothing provided at all
        let mac_eval = evaluate_contract(&contract, &evidence, 60_000);
        let pipeline = passing_pipeline();

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        assert!(matches!(decision, SuccessDecision::Fail { .. }));
        assert!(attempt_succeeded_transition(&decision).is_err());
    }

    #[test]
    fn wrong_verification_method_never_reaches_succeeded() {
        let contract = one_criterion_contract(); // declares TestCommand
        let mut evidence = HashMap::new();
        // Evidence is real, fresh, correct execution_id and provenance --
        // but it came from a DOM check, not a test command.
        let mut wrong_method_evidence = good_evidence("current_attempt", 0);
        wrong_method_evidence.observation_type = ObservationType::DomCheck;
        evidence.insert("auth-03".to_string(), wrong_method_evidence);
        let mac_eval = evaluate_contract(&contract, &evidence, 60_000);
        let pipeline = passing_pipeline();

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        match &decision {
            SuccessDecision::Fail { reasons } => {
                assert!(reasons.iter().any(|r| r.contains("wrong verification method")));
            }
            other => panic!("expected Fail, got {other:?}"),
        }
        assert!(attempt_succeeded_transition(&decision).is_err());
    }

    #[test]
    fn partial_completion_never_reaches_succeeded() {
        let contract = one_criterion_contract();
        let mut evidence = HashMap::new();
        evidence.insert("auth-03".to_string(), good_evidence("current_attempt", 0));
        let mac_eval = evaluate_contract(&contract, &evidence, 60_000);

        // Pipeline reports PartialChange at the diff-verification stage:
        // baseline expected two files to change, only one did.
        let before = baseline(&[("a.txt", "h1"), ("b.txt", "h1")]);
        let after = baseline(&[("a.txt", "h2"), ("b.txt", "h1")]); // b.txt never landed
        let expected = ExpectedOperation::new([PathBuf::from("a.txt"), PathBuf::from("b.txt")], "test");
        let pipeline = run_pipeline(&before, &after, &expected, Ok(()), Ok(()), Ok(()), &[]);

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        assert!(matches!(decision, SuccessDecision::Fail { .. }));
        assert!(attempt_succeeded_transition(&decision).is_err());
    }

    #[test]
    fn false_model_success_never_reaches_succeeded() {
        let contract = one_criterion_contract();
        let mut evidence = HashMap::new();
        // Correct provenance, freshness, execution_id -- but the model
        // (or its own runner) merely reported success; nothing
        // independently confirmed it.
        let mut self_reported = good_evidence("current_attempt", 0);
        self_reported.integrity = IntegrityLevel::SelfReported;
        evidence.insert("auth-03".to_string(), self_reported);
        let mac_eval = evaluate_contract(&contract, &evidence, 60_000);
        let pipeline = passing_pipeline();

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        match &decision {
            SuccessDecision::Fail { reasons } => {
                assert!(reasons.iter().any(|r| r.contains("false model success")));
            }
            other => panic!("expected Fail, got {other:?}"),
        }
        assert!(attempt_succeeded_transition(&decision).is_err());
    }

    #[test]
    fn a_favorable_reconciliation_alone_never_reaches_succeeded_without_full_mac_verification() {
        // The diff-verification stage alone reports ExpectedChange
        // (favorable) and every other pipeline stage passes -- but no
        // MAC evidence was ever produced. A favorable reconciliation
        // outcome must never, by itself, be treated as sufficient.
        let contract = one_criterion_contract();
        let evidence = HashMap::new(); // no MAC evidence at all
        let mac_eval = evaluate_contract(&contract, &evidence, 60_000);
        let pipeline = passing_pipeline(); // diff verification alone is favorable here

        assert_eq!(
            pipeline.outcome_for(crate::verification::VerificationStage::DiffVerification).unwrap().status,
            crate::verification::StageStatus::Passed,
            "diff verification must genuinely be favorable in this scenario"
        );

        let decision = decide(&pipeline, &contract, &mac_eval, &evidence);
        assert!(
            matches!(decision, SuccessDecision::Fail { .. }),
            "a favorable diff alone must not grant Succeeded without full MAC verification"
        );
        assert!(attempt_succeeded_transition(&decision).is_err());
    }

    // -- structural: RequirementVerificationClass::ModelAssisted still
    // cannot satisfy a blocking requirement even inside the composed
    // pipeline (re-confirms P4-W01's own rule holds through composition)

    #[test]
    fn model_assisted_pipeline_requirement_still_cannot_alone_satisfy_blocking() {
        let before = baseline(&[("a.txt", "h1")]);
        let after = baseline(&[("a.txt", "h2")]);
        let expected = ExpectedOperation::new([PathBuf::from("a.txt")], "test");
        let model_only = Requirement {
            id: "model-only".to_string(),
            blocking: true,
            class: RequirementVerificationClass::ModelAssisted,
            raw_result: true,
        };
        let pipeline = run_pipeline(&before, &after, &expected, Ok(()), Ok(()), Ok(()), &[model_only]);
        assert!(!pipeline.all_passed(), "a ModelAssisted-only blocking requirement must still fail the pipeline");
    }
}
