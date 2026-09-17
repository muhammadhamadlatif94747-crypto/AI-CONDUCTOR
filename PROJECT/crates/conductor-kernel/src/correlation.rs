//! Correlation IDs — the canonical hierarchy (Phase Manifest P1-W09):
//!
//!   mission → step → attempt → event → provider_request → tool_call
//!
//! Rule this module exists to enforce: there is exactly ONE identifier type
//! per hierarchy level, defined here and reused everywhere that level is
//! referenced. No module is permitted to invent its own parallel scheme for
//! naming a mission, step, attempt, provider request, or tool call — that is
//! precisely the "independent ad hoc ID system" the Phase Manifest forbids.
//!
//! Two things are deliberately NOT this module's job:
//! - Minting fresh IDs. Callers (mission planner, step scheduler, etc.) own
//!   ID generation; this module only gives that already-chosen identifier a
//!   single canonical type so it cannot be silently swapped for an
//!   unrelated bare `String` as it crosses module boundaries.
//! - Enforcing that a referenced parent ID actually exists in durable state
//!   (that is a State Engine / store concern). What IS enforced here is
//!   structural: a context cannot claim a child-level ID while skipping the
//!   parent level (e.g. an `attempt_id` with no `step_id`), fail-closed.

use serde::{Deserialize, Serialize};

macro_rules! id_newtype {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
    };
}

id_newtype!(MissionId, "Identifies a mission — the root of the hierarchy.");
id_newtype!(StepId, "Identifies a step within a mission.");
id_newtype!(AttemptId, "Identifies an attempt within a step.");
id_newtype!(
    ProviderRequestId,
    "Identifies a single provider request made within an attempt."
);
id_newtype!(
    ToolCallId,
    "Identifies a single tool call made within a provider request."
);

/// A context skips a hierarchy level: a child-level ID is present while its
/// required parent-level ID is absent. Fail-closed — callers must supply
/// the full lineage, not just the leaf they care about.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CorrelationError {
    #[error("attempt_id set without step_id: hierarchy requires mission -> step -> attempt")]
    AttemptWithoutStep,
    #[error(
        "provider_request_id set without attempt_id: hierarchy requires step -> attempt -> provider_request"
    )]
    ProviderRequestWithoutAttempt,
    #[error(
        "tool_call_id set without provider_request_id: hierarchy requires attempt -> provider_request -> tool_call"
    )]
    ToolCallWithoutProviderRequest,
}

/// The canonical correlation context threaded through events and, downstream
/// of an attempt, through every provider request and tool call made in
/// service of it. This is the single hierarchy-threading type for the crate;
/// `event::EventContext` is this type (see `event.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationContext {
    pub mission_id: MissionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<AttemptId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_request_id: Option<ProviderRequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<ToolCallId>,
}

impl CorrelationContext {
    /// Start a context at the root of the hierarchy. Every context has a
    /// mission_id; nothing narrower is optional-at-the-root.
    pub fn for_mission(mission_id: impl Into<MissionId>) -> Self {
        Self {
            mission_id: mission_id.into(),
            step_id: None,
            attempt_id: None,
            provider_request_id: None,
            tool_call_id: None,
        }
    }

    /// Narrow to a step of this mission. Steps never require an attempt, so
    /// this cannot fail structurally.
    pub fn with_step(mut self, step_id: impl Into<StepId>) -> Self {
        self.step_id = Some(step_id.into());
        self
    }

    /// Narrow to an attempt of this step. Fails if no step is set yet — an
    /// attempt without a step is not a valid position in the hierarchy.
    pub fn with_attempt(mut self, attempt_id: impl Into<AttemptId>) -> Result<Self, CorrelationError> {
        if self.step_id.is_none() {
            return Err(CorrelationError::AttemptWithoutStep);
        }
        self.attempt_id = Some(attempt_id.into());
        Ok(self)
    }

    /// Narrow to a provider request made within this attempt.
    pub fn with_provider_request(
        mut self,
        provider_request_id: impl Into<ProviderRequestId>,
    ) -> Result<Self, CorrelationError> {
        if self.attempt_id.is_none() {
            return Err(CorrelationError::ProviderRequestWithoutAttempt);
        }
        self.provider_request_id = Some(provider_request_id.into());
        Ok(self)
    }

    /// Narrow to a tool call made within this provider request.
    pub fn with_tool_call(mut self, tool_call_id: impl Into<ToolCallId>) -> Result<Self, CorrelationError> {
        if self.provider_request_id.is_none() {
            return Err(CorrelationError::ToolCallWithoutProviderRequest);
        }
        self.tool_call_id = Some(tool_call_id.into());
        Ok(self)
    }

    /// Re-check structural well-formedness for a context that was assembled
    /// some other way (struct literal, deserialized off disk). Applies the
    /// same rule the builder enforces, defensively, at trust boundaries.
    pub fn validate(&self) -> Result<(), CorrelationError> {
        if self.attempt_id.is_some() && self.step_id.is_none() {
            return Err(CorrelationError::AttemptWithoutStep);
        }
        if self.provider_request_id.is_some() && self.attempt_id.is_none() {
            return Err(CorrelationError::ProviderRequestWithoutAttempt);
        }
        if self.tool_call_id.is_some() && self.provider_request_id.is_none() {
            return Err(CorrelationError::ToolCallWithoutProviderRequest);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_hierarchy_builds_in_order() {
        let ctx = CorrelationContext::for_mission("m-1")
            .with_step("s-1")
            .with_attempt("a-1")
            .expect("attempt has a step")
            .with_provider_request("pr-1")
            .expect("provider_request has an attempt")
            .with_tool_call("tc-1")
            .expect("tool_call has a provider_request");

        assert_eq!(ctx.mission_id, MissionId::from("m-1"));
        assert_eq!(ctx.step_id, Some(StepId::from("s-1")));
        assert_eq!(ctx.attempt_id, Some(AttemptId::from("a-1")));
        assert_eq!(ctx.provider_request_id, Some(ProviderRequestId::from("pr-1")));
        assert_eq!(ctx.tool_call_id, Some(ToolCallId::from("tc-1")));
    }

    #[test]
    fn mission_only_context_is_valid() {
        let ctx = CorrelationContext::for_mission("m-1");
        assert!(ctx.validate().is_ok());
        assert_eq!(ctx.step_id, None);
    }

    #[test]
    fn mission_and_step_without_attempt_is_valid() {
        let ctx = CorrelationContext::for_mission("m-1").with_step("s-1");
        assert!(ctx.validate().is_ok());
    }

    #[test]
    fn attempt_without_step_is_rejected() {
        let err = CorrelationContext::for_mission("m-1")
            .with_attempt("a-1")
            .unwrap_err();
        assert_eq!(err, CorrelationError::AttemptWithoutStep);
    }

    #[test]
    fn provider_request_without_attempt_is_rejected() {
        let err = CorrelationContext::for_mission("m-1")
            .with_step("s-1")
            .with_provider_request("pr-1")
            .unwrap_err();
        assert_eq!(err, CorrelationError::ProviderRequestWithoutAttempt);
    }

    #[test]
    fn tool_call_without_provider_request_is_rejected() {
        let err = CorrelationContext::for_mission("m-1")
            .with_step("s-1")
            .with_attempt("a-1")
            .expect("attempt has a step")
            .with_tool_call("tc-1")
            .unwrap_err();
        assert_eq!(err, CorrelationError::ToolCallWithoutProviderRequest);
    }

    #[test]
    fn validate_catches_hand_assembled_skip_of_every_level() {
        // A context built by struct literal (e.g. off a deserialized record)
        // rather than the builder can still skip a level; `validate` must
        // catch each case independently, not just the first one it finds.
        let attempt_without_step = CorrelationContext {
            mission_id: MissionId::from("m"),
            step_id: None,
            attempt_id: Some(AttemptId::from("a")),
            provider_request_id: None,
            tool_call_id: None,
        };
        assert_eq!(
            attempt_without_step.validate().unwrap_err(),
            CorrelationError::AttemptWithoutStep
        );

        let provider_request_without_attempt = CorrelationContext {
            mission_id: MissionId::from("m"),
            step_id: Some(StepId::from("s")),
            attempt_id: None,
            provider_request_id: Some(ProviderRequestId::from("pr")),
            tool_call_id: None,
        };
        assert_eq!(
            provider_request_without_attempt.validate().unwrap_err(),
            CorrelationError::ProviderRequestWithoutAttempt
        );

        let tool_call_without_provider_request = CorrelationContext {
            mission_id: MissionId::from("m"),
            step_id: Some(StepId::from("s")),
            attempt_id: Some(AttemptId::from("a")),
            provider_request_id: None,
            tool_call_id: Some(ToolCallId::from("tc")),
        };
        assert_eq!(
            tool_call_without_provider_request.validate().unwrap_err(),
            CorrelationError::ToolCallWithoutProviderRequest
        );
    }

    #[test]
    fn json_round_trip_preserves_full_hierarchy() {
        let ctx = CorrelationContext::for_mission("m-1")
            .with_step("s-1")
            .with_attempt("a-1")
            .unwrap()
            .with_provider_request("pr-1")
            .unwrap()
            .with_tool_call("tc-1")
            .unwrap();

        let json = serde_json::to_string(&ctx).expect("serialize");
        let back: CorrelationContext = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, ctx);
    }

    #[test]
    fn json_omits_absent_levels() {
        let ctx = CorrelationContext::for_mission("m-1");
        let json = serde_json::to_value(&ctx).expect("serialize");
        let obj = json.as_object().expect("object");
        assert!(!obj.contains_key("step_id"));
        assert!(!obj.contains_key("attempt_id"));
        assert!(!obj.contains_key("provider_request_id"));
        assert!(!obj.contains_key("tool_call_id"));
    }

    #[test]
    fn distinct_id_types_are_not_interchangeable_at_the_type_level() {
        // This is a compile-time property, not a runtime assertion: MissionId
        // and StepId are distinct types, so a caller cannot pass a StepId
        // where a MissionId is expected. Documented here as a witness test —
        // if someone "fixes" the newtypes into one shared type, this comment
        // is where the regression would need to be re-introduced and caught
        // in review, since the compiler can no longer catch it for us.
        let mission = MissionId::from("x");
        let step = StepId::from("x");
        assert_eq!(mission.as_str(), step.as_str());
        // Same string content, different types — `mission == step` does not
        // even typecheck, which is the point.
    }
}
