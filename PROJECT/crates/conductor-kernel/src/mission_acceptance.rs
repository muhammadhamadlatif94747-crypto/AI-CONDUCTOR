//! Mission Acceptance Contracts (P4-W03 / Blueprint §8).
//!
//! ```text
//! Model says "completed"
//!         ↓
//! Verification Engine (Section 7) runs
//!         ↓
//! Each criterion's declared verification method actually executes, right now
//!         ↓
//! Result's Evidence record (Section 4.5) compared against the criterion's required_evidence
//!         ↓
//! PASS  → Attempt = Succeeded, Checkpoint accepted   (Verification Engine only — Invariant 14)
//! FAIL  → specific unmet criteria become the next repair request — never a vague "try again"
//! ```
//!
//! [`evaluate_contract`] is the middle two arrows of that diagram, made
//! real: it reuses P4-W02's `Evidence::matches` directly (no second
//! provenance/freshness comparison invented here), and it produces
//! per-criterion detail — which criteria failed and *why* (no evidence
//! at all, versus evidence that exists but doesn't match) — because
//! Blueprint §8 is explicit that a failure must become "the next repair
//! request," not a vague "try again."
//!
//! Scope, stated explicitly (this is P4-W03 only): this module produces
//! a [`MacEvaluationResult`]. It does not touch `AttemptStatus` at all —
//! the top arrow ("PASS → Attempt = Succeeded") is P4-W04's job
//! (Invariant 14; Phase Manifest §8.5: *"No code path other than the
//! Verification Engine may create Succeeded"*). It also does not modify
//! P4-W01's already-accepted pipeline to use this richer schema — that
//! module's own contract explicitly deferred "the full MAC schema" to
//! this task; wiring the two together is a future orchestration
//! decision, not a reason to reopen an accepted module.

use crate::evidence::{Evidence, EvidenceProvenance};
use std::collections::HashMap;
use std::time::Duration;

/// Blueprint §8's own worked full-contract example names exactly these
/// five verification types — the concrete, complete set the text
/// provides, not extended with plausible-sounding additions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationType {
    DomCheck,
    TestCommand,
    IntegrationTest,
    TestSuiteDiff,
    DiffScopeCheck,
}

/// Blueprint §8's example shows `"expected_result": "exit_code_0"`.
/// `Other(String)` is this task's own fallback for anything the single
/// shown example doesn't cover — an explicit interpretation, not
/// presented as verbatim Blueprint content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedResult {
    ExitCodeZero,
    Other(String),
}

/// Blueprint §8's `required_evidence` block. Reuses P4-W02's
/// `EvidenceProvenance` directly rather than a second provenance concept.
#[derive(Debug, Clone)]
pub struct RequiredEvidenceSpec {
    pub provenance: EvidenceProvenance,
    pub max_freshness: Duration,
    pub execution_id: String,
}

/// One Mission Acceptance Contract criterion, matching Blueprint §8's
/// JSON shape field-for-field.
#[derive(Debug, Clone)]
pub struct Criterion {
    pub id: String,
    pub description: String,
    pub verification_type: VerificationType,
    pub command: Option<String>,
    pub expected_result: ExpectedResult,
    pub required_evidence: RequiredEvidenceSpec,
    pub blocking: bool,
}

/// "A full contract is a list of these, not prose" — Blueprint §8.
#[derive(Debug, Clone)]
pub struct MissionAcceptanceContract {
    pub mission: String,
    pub criteria: Vec<Criterion>,
}

/// Why one criterion did or did not pass — distinguishing "no evidence
/// was ever provided" from "evidence exists but doesn't match" (wrong
/// provenance, stale, wrong execution_id, or a failed underlying
/// result), since a future repair request needs to say something more
/// specific than "it failed."
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CriterionOutcome {
    Satisfied,
    NoEvidenceProvided,
    EvidenceDidNotMatch,
}

#[derive(Debug, Clone)]
pub struct CriterionResult {
    pub criterion_id: String,
    pub blocking: bool,
    pub outcome: CriterionOutcome,
}

#[derive(Debug, Clone)]
pub struct MacEvaluationResult {
    pub mission: String,
    pub criteria: Vec<CriterionResult>,
}

impl MacEvaluationResult {
    /// The contract as a whole passes only if every *blocking* criterion
    /// is `Satisfied`. Non-blocking criteria are recorded but never
    /// affect this — see
    /// `tests::a_non_blocking_criterion_failure_does_not_fail_the_contract`.
    pub fn all_blocking_satisfied(&self) -> bool {
        self.criteria
            .iter()
            .filter(|c| c.blocking)
            .all(|c| c.outcome == CriterionOutcome::Satisfied)
    }

    /// The specific unmet criteria — Blueprint §8: "specific unmet
    /// criteria become the next repair request, never a vague 'try
    /// again.'"
    pub fn unmet_blocking_criteria(&self) -> Vec<&CriterionResult> {
        self.criteria
            .iter()
            .filter(|c| c.blocking && c.outcome != CriterionOutcome::Satisfied)
            .collect()
    }
}

/// The middle two arrows of Blueprint §8's flow diagram, made real:
/// checks each criterion's required evidence against what was actually
/// produced (`evidence_by_criterion_id`, keyed by criterion `id`), via
/// P4-W02's own `Evidence::matches` — reused directly, not
/// re-implemented. A criterion with no corresponding entry in
/// `evidence_by_criterion_id` fails as `NoEvidenceProvided`, explicitly
/// — never silently treated as satisfied.
pub fn evaluate_contract(
    contract: &MissionAcceptanceContract,
    evidence_by_criterion_id: &HashMap<String, Evidence>,
    now_ms: u64,
) -> MacEvaluationResult {
    let criteria = contract
        .criteria
        .iter()
        .map(|criterion| {
            let outcome = match evidence_by_criterion_id.get(&criterion.id) {
                None => CriterionOutcome::NoEvidenceProvided,
                Some(evidence) => {
                    let req = &criterion.required_evidence;
                    if evidence.matches(req.provenance, req.max_freshness, &req.execution_id, now_ms) {
                        CriterionOutcome::Satisfied
                    } else {
                        CriterionOutcome::EvidenceDidNotMatch
                    }
                }
            };
            CriterionResult {
                criterion_id: criterion.id.clone(),
                blocking: criterion.blocking,
                outcome,
            }
        })
        .collect();

    MacEvaluationResult {
        mission: contract.mission.clone(),
        criteria,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{IntegrityLevel, ObservationType, Reproducibility};

    fn satisfying_evidence(execution_id: &str, produced_at_ms: u64) -> Evidence {
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

    fn criterion(id: &str, blocking: bool) -> Criterion {
        Criterion {
            id: id.to_string(),
            description: "test criterion".to_string(),
            verification_type: VerificationType::TestCommand,
            command: Some("npm test -- auth.test.ts".to_string()),
            expected_result: ExpectedResult::ExitCodeZero,
            required_evidence: RequiredEvidenceSpec {
                provenance: EvidenceProvenance::LocalTestRunner,
                max_freshness: Duration::from_secs(300),
                execution_id: "current_attempt".to_string(),
            },
            blocking,
        }
    }

    #[test]
    fn a_contract_with_all_criteria_satisfied_passes() {
        let contract = MissionAcceptanceContract {
            mission: "add authentication".to_string(),
            criteria: vec![criterion("auth-03", true)],
        };
        let mut evidence = HashMap::new();
        evidence.insert("auth-03".to_string(), satisfying_evidence("current_attempt", 0));

        let result = evaluate_contract(&contract, &evidence, 60_000);
        assert!(result.all_blocking_satisfied());
        assert!(result.unmet_blocking_criteria().is_empty());
    }

    #[test]
    fn a_blocking_criterion_with_no_evidence_fails_the_contract() {
        let contract = MissionAcceptanceContract {
            mission: "add authentication".to_string(),
            criteria: vec![criterion("auth-03", true)],
        };
        let evidence = HashMap::new(); // nothing provided at all

        let result = evaluate_contract(&contract, &evidence, 60_000);
        assert!(!result.all_blocking_satisfied());
        assert_eq!(result.criteria[0].outcome, CriterionOutcome::NoEvidenceProvided);
    }

    #[test]
    fn a_blocking_criterion_with_non_matching_evidence_fails_the_contract() {
        let contract = MissionAcceptanceContract {
            mission: "add authentication".to_string(),
            criteria: vec![criterion("auth-03", true)],
        };
        let mut evidence = HashMap::new();
        // Wrong execution_id -- evidence from a stale prior run, not this attempt.
        evidence.insert("auth-03".to_string(), satisfying_evidence("stale-prior-run", 0));

        let result = evaluate_contract(&contract, &evidence, 60_000);
        assert!(!result.all_blocking_satisfied());
        assert_eq!(result.criteria[0].outcome, CriterionOutcome::EvidenceDidNotMatch);
    }

    #[test]
    fn no_evidence_and_non_matching_evidence_are_distinguishable_outcomes() {
        let contract = MissionAcceptanceContract {
            mission: "m".to_string(),
            criteria: vec![criterion("a", true), criterion("b", true)],
        };
        let mut evidence = HashMap::new();
        // "a" gets no evidence at all; "b" gets evidence that doesn't match.
        evidence.insert("b".to_string(), satisfying_evidence("wrong-id", 0));

        let result = evaluate_contract(&contract, &evidence, 60_000);
        assert_eq!(result.criteria[0].outcome, CriterionOutcome::NoEvidenceProvided);
        assert_eq!(result.criteria[1].outcome, CriterionOutcome::EvidenceDidNotMatch);
    }

    #[test]
    fn a_non_blocking_criterion_failure_does_not_fail_the_contract() {
        let contract = MissionAcceptanceContract {
            mission: "add authentication".to_string(),
            criteria: vec![criterion("auth-03", true), criterion("auth-08-nice-to-have", false)],
        };
        let mut evidence = HashMap::new();
        evidence.insert("auth-03".to_string(), satisfying_evidence("current_attempt", 0));
        // auth-08 gets no evidence at all, but it's non-blocking.

        let result = evaluate_contract(&contract, &evidence, 60_000);
        assert!(
            result.all_blocking_satisfied(),
            "a non-blocking criterion's failure must not fail the contract"
        );
        assert!(result.unmet_blocking_criteria().is_empty());
        // But it's still recorded, not silently dropped.
        assert_eq!(
            result.criteria.iter().find(|c| c.criterion_id == "auth-08-nice-to-have").unwrap().outcome,
            CriterionOutcome::NoEvidenceProvided
        );
    }

    /// Blueprint §8's own seven-criterion worked "auth" example,
    /// reproduced literally and evaluated end to end.
    #[test]
    fn blueprint_worked_example_seven_criterion_auth_contract() {
        let mission_criteria = [
            ("auth-01", VerificationType::DomCheck, EvidenceProvenance::FilesystemObservation),
            ("auth-02", VerificationType::TestCommand, EvidenceProvenance::LocalTestRunner),
            ("auth-03", VerificationType::TestCommand, EvidenceProvenance::LocalTestRunner),
            ("auth-04", VerificationType::IntegrationTest, EvidenceProvenance::LocalTestRunner),
            ("auth-05", VerificationType::TestCommand, EvidenceProvenance::LocalTestRunner),
            ("auth-06", VerificationType::TestSuiteDiff, EvidenceProvenance::LocalTestRunner),
            ("auth-07", VerificationType::DiffScopeCheck, EvidenceProvenance::FilesystemObservation),
        ];

        let criteria: Vec<Criterion> = mission_criteria
            .iter()
            .map(|(id, vtype, provenance)| Criterion {
                id: id.to_string(),
                description: format!("criterion {id}"),
                verification_type: *vtype,
                command: None,
                expected_result: ExpectedResult::ExitCodeZero,
                required_evidence: RequiredEvidenceSpec {
                    provenance: *provenance,
                    max_freshness: Duration::from_secs(300),
                    execution_id: "current_attempt".to_string(),
                },
                blocking: true,
            })
            .collect();

        let contract = MissionAcceptanceContract {
            mission: "Add authentication".to_string(),
            criteria,
        };

        let mut evidence = HashMap::new();
        for (id, _vtype, provenance) in &mission_criteria {
            evidence.insert(
                id.to_string(),
                Evidence {
                    provenance: *provenance,
                    observation_type: ObservationType::TestExecution,
                    produced_at_ms: 0,
                    execution_id: "current_attempt".to_string(),
                    source: format!("check for {id}"),
                    reproducibility: Reproducibility::Deterministic,
                    integrity: IntegrityLevel::IndependentlyVerified,
                    result: true,
                },
            );
        }

        let result = evaluate_contract(&contract, &evidence, 60_000);
        assert!(result.all_blocking_satisfied(), "all seven criteria should be satisfied: {result:?}");
        assert_eq!(result.criteria.len(), 7);
    }
}
