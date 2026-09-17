//! State Engine — enforced lifecycle transition tables.
//!
//! Sources of truth (do not diverge from these):
//! - AC-01: AttemptStatus six states + structural table + authorization table
//! - AC-10: MissionStatus / StepStatus + transition tables (Blueprint §4.2.1)
//!
//! Structural legality and transition authority are separate concerns:
//! `try_transition_*` enforces the STRUCTURAL table. Authorization (who may
//! request which edge — e.g. only the Verification Engine may produce
//! `Succeeded`, AC-01 §5 / Invariant 14) is enforced at the call sites that
//! own those decisions; this module exposes distinct typed constructors so a
//! caller cannot accidentally claim an authority it does not have.

use thiserror::Error;

/// Errors produced when a requested edge is not in the structural transition
/// table, or an actor requests an edge it is not authorized to request.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum TransitionError {
    #[error("illegal transition {from:?} -> {to:?}: edge not in the structural table")]
    IllegalEdge {
        from: &'static str,
        to: &'static str,
    },
    #[error("actor {actor} is not authorized to request {from:?} -> {to:?}")]
    UnauthorizedActor {
        actor: &'static str,
        from: &'static str,
        to: &'static str,
    },
}

// ---------------------------------------------------------------------------
// AttemptStatus (AC-01 §1–§2)
// ---------------------------------------------------------------------------

/// Lifecycle-only enum. Exactly six states. Never carries reconciliation or
/// cancellation detail (AC-01 §4 type-separation rule; P1-W02).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}

impl AttemptStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttemptStatus::Pending => "Pending",
            AttemptStatus::Running => "Running",
            AttemptStatus::Succeeded => "Succeeded",
            AttemptStatus::Failed => "Failed",
            AttemptStatus::Cancelled => "Cancelled",
            AttemptStatus::Unknown => "Unknown",
        }
    }

    /// Structural edges from AC-01 §2:
    /// Pending → Running
    /// Running → Succeeded | Failed | Unknown | Cancelled
    /// Unknown → Succeeded | Failed
    /// Succeeded/Failed/Cancelled → terminal.
    pub fn can_transition_to(&self, to: AttemptStatus) -> bool {
        use AttemptStatus::*;
        matches!(
            (self, to),
            (Pending, Running)
                | (Running, Succeeded)
                | (Running, Failed)
                | (Running, Unknown)
                | (Running, Cancelled)
                | (Unknown, Succeeded)
                | (Unknown, Failed)
        )
    }
}

/// Actors that may request attempt transitions. Mirrors AC-01 §5's
/// authorization table; `VerificationEngine` exists so Invariant 14 has a
/// single named owner and every other actor is structurally denied the
/// Succeeded edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionActor {
    AuthoritativeExecutor,
    CancellationPath,
    VerificationEngine,
}

impl TransitionActor {
    fn as_str(&self) -> &'static str {
        match self {
            TransitionActor::AuthoritativeExecutor => "AuthoritativeExecutor",
            TransitionActor::CancellationPath => "CancellationPath",
            TransitionActor::VerificationEngine => "VerificationEngine",
        }
    }
}

/// Enforce structure AND authorization for one attempted edge.
///
/// Authorization per AC-01 §5:
/// - Pending→Running, Running→Unknown/Failed, Unknown→Failed: executor
/// - Running→Cancelled: cancellation path
/// - Running→Succeeded, Unknown→Succeeded: Verification Engine ONLY (Invariant 14)
pub fn try_attempt_transition(
    actor: TransitionActor,
    from: AttemptStatus,
    to: AttemptStatus,
) -> Result<(), TransitionError> {
    if !from.can_transition_to(to) {
        return Err(TransitionError::IllegalEdge {
            from: from.as_str(),
            to: to.as_str(),
        });
    }
    use AttemptStatus::*;
    let authorized = match (from, to) {
        (Pending, Running)
        | (Running, Unknown)
        | (Running, Failed)
        | (Unknown, Failed) => actor == TransitionActor::AuthoritativeExecutor,
        (Running, Cancelled) => actor == TransitionActor::CancellationPath,
        // Both Succeeded edges: Verification Engine ONLY. No other actor,
        // no reconciliation result, may construct these (Invariant 14).
        (Running, Succeeded) | (Unknown, Succeeded) => {
            actor == TransitionActor::VerificationEngine
        }
        _ => false,
    };
    if !authorized {
        return Err(TransitionError::UnauthorizedActor {
            actor: actor.as_str(),
            from: from.as_str(),
            to: to.as_str(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// MissionStatus (AC-10 §1, §3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionStatus {
    Draft,
    Planning,
    Running,
    Paused,
    Blocked,
    Succeeded,
    Failed,
    Cancelled,
}

impl MissionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MissionStatus::Draft => "Draft",
            MissionStatus::Planning => "Planning",
            MissionStatus::Running => "Running",
            MissionStatus::Paused => "Paused",
            MissionStatus::Blocked => "Blocked",
            MissionStatus::Succeeded => "Succeeded",
            MissionStatus::Failed => "Failed",
            MissionStatus::Cancelled => "Cancelled",
        }
    }

    /// Structural edges from AC-10 §3:
    /// Draft → Planning → Running → { Succeeded | Failed | Cancelled }
    /// Running ⇄ Paused (person-initiated, reversible)
    /// Running → Blocked → { Running | Failed | Cancelled } (system-initiated)
    pub fn can_transition_to(&self, to: MissionStatus) -> bool {
        use MissionStatus::*;
        matches!(
            (self, to),
            (Draft, Planning)
                | (Planning, Running)
                | (Running, Succeeded)
                | (Running, Failed)
                | (Running, Cancelled)
                | (Running, Paused)
                | (Paused, Running)
                | (Running, Blocked)
                | (Blocked, Running)
                | (Blocked, Failed)
                | (Blocked, Cancelled)
        )
    }
}

// ---------------------------------------------------------------------------
// StepStatus (AC-10 §2, §3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Blocked,
    Succeeded,
    Failed,
    Skipped,
}

impl StepStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            StepStatus::Pending => "Pending",
            StepStatus::Running => "Running",
            StepStatus::Blocked => "Blocked",
            StepStatus::Succeeded => "Succeeded",
            StepStatus::Failed => "Failed",
            StepStatus::Skipped => "Skipped",
        }
    }

    /// Structural edges from AC-10 §3:
    /// Pending → Running → { Succeeded | Failed | Skipped }
    /// Running → Blocked → Running (mirrors owning Mission pause/block).
    /// Blocked is reachable ONLY from Running — a step that never started
    /// cannot be "blocked"; it stays Pending.
    pub fn can_transition_to(&self, to: StepStatus) -> bool {
        use StepStatus::*;
        matches!(
            (self, to),
            (Pending, Running)
                | (Running, Succeeded)
                | (Running, Failed)
                | (Running, Skipped)
                | (Running, Blocked)
                | (Blocked, Running)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_ACTORS: [TransitionActor; 3] = [
        TransitionActor::AuthoritativeExecutor,
        TransitionActor::CancellationPath,
        TransitionActor::VerificationEngine,
    ];

    // -- Exhaustive legality tests: every ordered pair, all three machines --

    #[test]
    fn attempt_table_is_exactly_ac01() {
        use AttemptStatus::*;
        let all = [Pending, Running, Succeeded, Failed, Cancelled, Unknown];
        let legal: &[(AttemptStatus, AttemptStatus)] = &[
            (Pending, Running),
            (Running, Succeeded),
            (Running, Failed),
            (Running, Unknown),
            (Running, Cancelled),
            (Unknown, Succeeded),
            (Unknown, Failed),
        ];
        for from in all {
            for to in all {
                let expected = legal.iter().any(|&(a, b)| a == from && b == to);
                assert_eq!(
                    from.can_transition_to(to),
                    expected,
                    "{:?} -> {:?}",
                    from,
                    to
                );
            }
        }
        assert_eq!(legal.len(), 7, "AC-01 defines exactly 7 legal edges");
    }

    #[test]
    fn mission_table_is_exactly_ac10() {
        use MissionStatus::*;
        let all = [
            Draft, Planning, Running, Paused, Blocked, Succeeded, Failed, Cancelled,
        ];
        let legal: &[(MissionStatus, MissionStatus)] = &[
            (Draft, Planning),
            (Planning, Running),
            (Running, Succeeded),
            (Running, Failed),
            (Running, Cancelled),
            (Running, Paused),
            (Paused, Running),
            (Running, Blocked),
            (Blocked, Running),
            (Blocked, Failed),
            (Blocked, Cancelled),
        ];
        for from in all {
            for to in all {
                let expected = legal.iter().any(|&(a, b)| a == from && b == to);
                assert_eq!(from.can_transition_to(to), expected, "{:?} -> {:?}", from, to);
            }
        }
        assert_eq!(legal.len(), 11, "AC-10 defines exactly 11 legal edges");
    }

    #[test]
    fn step_table_is_exactly_ac10() {
        use StepStatus::*;
        let all = [Pending, Running, Blocked, Succeeded, Failed, Skipped];
        let legal: &[(StepStatus, StepStatus)] = &[
            (Pending, Running),
            (Running, Succeeded),
            (Running, Failed),
            (Running, Skipped),
            (Running, Blocked),
            (Blocked, Running),
        ];
        for from in all {
            for to in all {
                let expected = legal.iter().any(|&(a, b)| a == from && b == to);
                assert_eq!(from.can_transition_to(to), expected, "{:?} -> {:?}", from, to);
            }
        }
        assert_eq!(legal.len(), 6, "AC-10 defines exactly 6 legal edges");
    }

    // -- Terminal states accept nothing --

    #[test]
    fn attempt_terminal_states_have_no_outgoing_edges() {
        use AttemptStatus::*;
        for terminal in [Succeeded, Failed, Cancelled] {
            for to in [Pending, Running, Succeeded, Failed, Cancelled, Unknown] {
                assert!(
                    !terminal.can_transition_to(to),
                    "terminal {:?} must not transition to {:?}",
                    terminal,
                    to
                );
            }
        }
    }

    // -- Unknown restrictions (AC-01 notes) --

    #[test]
    fn unknown_cannot_go_to_running_cancelled_or_pending() {
        use AttemptStatus::*;
        assert!(!Unknown.can_transition_to(Running));
        assert!(!Unknown.can_transition_to(Cancelled));
        assert!(!Unknown.can_transition_to(Pending));
    }

    // -- Authorization enforcement (Invariant 14 / AC-01 §5) --

    #[test]
    fn succeeded_requires_verification_engine_from_running() {
        use AttemptStatus::*;
        for actor in ALL_ACTORS {
            let res = try_attempt_transition(actor, Running, Succeeded);
            if actor == TransitionActor::VerificationEngine {
                assert!(res.is_ok());
            } else {
                assert_eq!(
                    res,
                    Err(TransitionError::UnauthorizedActor {
                        actor: actor.as_str(),
                        from: "Running",
                        to: "Succeeded",
                    }
                ));
            }
        }
    }

    #[test]
    fn succeeded_requires_verification_engine_from_unknown() {
        use AttemptStatus::*;
        for actor in ALL_ACTORS {
            let res = try_attempt_transition(actor, Unknown, Succeeded);
            if actor == TransitionActor::VerificationEngine {
                assert!(res.is_ok());
            } else {
                assert!(matches!(res, Err(TransitionError::UnauthorizedActor { .. })));
            }
        }
    }

    #[test]
    fn cancellation_path_only_may_cancel() {
        use AttemptStatus::*;
        for actor in ALL_ACTORS {
            let res = try_attempt_transition(actor, Running, Cancelled);
            if actor == TransitionActor::CancellationPath {
                assert!(res.is_ok());
            } else {
                assert!(matches!(res, Err(TransitionError::UnauthorizedActor { .. })));
            }
        }
    }

    #[test]
    fn executor_edges_accept_executor_and_reject_others() {
        use AttemptStatus::*;
        let executor_edges = [
            (Pending, Running),
            (Running, Unknown),
            (Running, Failed),
            (Unknown, Failed),
        ];
        for (from, to) in executor_edges {
            for actor in ALL_ACTORS {
                let res = try_attempt_transition(actor, from, to);
                if actor == TransitionActor::AuthoritativeExecutor {
                    assert!(res.is_ok(), "{:?} -> {:?}", from, to);
                } else {
                    assert!(matches!(res, Err(TransitionError::UnauthorizedActor { .. })));
                }
            }
        }
    }

    #[test]
    fn illegal_edge_reported_before_authorization() {
        // Even the Verification Engine cannot make a structurally illegal edge.
        use AttemptStatus::*;
        let res = try_attempt_transition(
            TransitionActor::VerificationEngine,
            Pending,
            Succeeded,
        );
        assert_eq!(
            res,
            Err(TransitionError::IllegalEdge {
                from: "Pending",
                to: "Succeeded"
            }
        ));
    }

    #[test]
    fn full_happy_path_walk_succeeds() {
        use AttemptStatus::*;
        let mut st = Pending;
        for to in [Running, Unknown, Succeeded] {
            assert!(try_attempt_transition(
                if to == Succeeded {
                    TransitionActor::VerificationEngine
                } else {
                    TransitionActor::AuthoritativeExecutor
                },
                st,
                to,
            )
            .is_ok());
            st = to;
        }
        assert_eq!(st, Succeeded);
    }

    // -- Mission / Step walk-throughs --

    #[test]
    fn mission_pause_resume_and_block_resume_walks() {
        use MissionStatus::*;
        // Draft → Planning → Running → Paused → Running → Blocked → Running → Succeeded
        let path = [
            (Draft, Planning),
            (Planning, Running),
            (Running, Paused),
            (Paused, Running),
            (Running, Blocked),
            (Blocked, Running),
            (Running, Succeeded),
        ];
        for (from, to) in path {
            assert!(from.can_transition_to(to), "{:?} -> {:?}", from, to);
        }
    }

    #[test]
    fn step_blocked_only_reachable_from_running() {
        use StepStatus::*;
        assert!(!Pending.can_transition_to(Blocked));
        assert!(Running.can_transition_to(Blocked));
        assert!(Blocked.can_transition_to(Running));
        // Blocked mirrors pause/block; it is not a failure of its own.
        assert!(!Blocked.can_transition_to(Failed));
        assert!(!Blocked.can_transition_to(Succeeded));
    }

    #[test]
    fn mission_terminal_states_have_no_outgoing_edges() {
        use MissionStatus::*;
        for terminal in [Succeeded, Failed, Cancelled] {
            for to in [
                Draft, Planning, Running, Paused, Blocked, Succeeded, Failed, Cancelled,
            ] {
                assert!(!terminal.can_transition_to(to));
            }
        }
    }

    // -- Type separation guard (AC-01 §4): statuses are distinct types with
    //    distinct string identities; a compile-time guarantee by construction.
    //    This test pins the observable identity strings.

    #[test]
    fn status_identity_strings_are_stable() {
        assert_eq!(AttemptStatus::Unknown.as_str(), "Unknown");
        assert_eq!(MissionStatus::Blocked.as_str(), "Blocked");
        assert_eq!(StepStatus::Skipped.as_str(), "Skipped");
    }
}
