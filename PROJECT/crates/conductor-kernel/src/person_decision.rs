//! Person decision path (P2-W07 / Blueprint §4.3, §21, AC-07;
//! Invariant 11).
//!
//! Once P2-W06's `decide_merge` returns `Halt`, Invariant 11 says the
//! only safe action is to stop and ask — never auto-resolve. Blueprint
//! §4.3 specifies exactly four options:
//!
//! ```text
//! [Keep user version] [Keep AI version] [Merge] [Review diff]
//! ```
//!
//! This module renders a `ConflictReason`'s real evidence in plain text,
//! accepts one of those four choices, and durably records it — mirroring
//! P1-W11's `console.rs::request_attempt_cancellation` exactly: real
//! `EventLog` read/write, no special-cased bypass, every outcome written
//! as a real event, never silently dropped. "Functional decision UI, not
//! polished Mission Mode" (Phase Manifest §6.3 P2-W07) is the same
//! plain-text discipline `console.rs::render` already established; this
//! module reuses that precedent rather than inventing a second style.
//!
//! Scope, stated explicitly (this is P2-W07 only): this module records
//! WHAT the person chose. It does not execute the choice — no merge
//! write, no "keep AI version" file write — because the workspace
//! mutation lock and a merge-executor were both explicitly scoped out of
//! P2-W06 and remain unbuilt; there is nothing yet for "Merge" or "Keep
//! AI version" to safely act through. It also does not drive any
//! `MissionStatus` transition, for the same reason P2-W06 didn't:
//! `state.rs` still has no actor-authorization wrapper for mission-level
//! transitions (re-confirmed before writing this module).

use crate::conflict::ConflictReason;
use crate::console::{ConsoleError, StorePaths};
use crate::event::{ClockSource, EventContext, NewEvent};
use crate::event_log::EventLog;
use std::path::Path;

/// Blueprint §4.3's four-way choice, verbatim. No fifth "auto-resolve"
/// option exists — Invariant 11 is upheld by construction, not by a
/// runtime check that could be bypassed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonDecision {
    KeepUserVersion,
    KeepAiVersion,
    Merge,
    ReviewDiff,
}

impl PersonDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            PersonDecision::KeepUserVersion => "keep_user_version",
            PersonDecision::KeepAiVersion => "keep_ai_version",
            PersonDecision::Merge => "merge",
            PersonDecision::ReviewDiff => "review_diff",
        }
    }

    /// `KeepUserVersion`/`KeepAiVersion`/`Merge` resolve the conflict;
    /// `ReviewDiff` does not — a person may review any number of times
    /// before eventually choosing one of the other three. This is
    /// informational metadata only: no decision-loop orchestrator
    /// enforcing that ordering exists yet in this crate (out of scope —
    /// see module docs), so nothing in this module *enforces* the
    /// distinction; it is recorded truthfully for whatever later caller
    /// does build that loop.
    pub fn is_terminal(&self) -> bool {
        !matches!(self, PersonDecision::ReviewDiff)
    }
}

/// Render a `ConflictReason`'s actual evidence in plain text — the real
/// changed paths and/or conflicting regions, not a generic "a conflict
/// occurred" placeholder. This is what a person reads before choosing.
pub fn describe_conflict(reason: &ConflictReason) -> String {
    let mut out = String::new();
    match reason {
        ConflictReason::MergeTimeDrift { changed_paths } => {
            out.push_str("The live workspace changed since the attempt's baseline was captured:\n");
            for p in changed_paths {
                out.push_str(&format!("  - {}\n", p.display()));
            }
        }
        ConflictReason::RegionOverlap { regions } => {
            out.push_str("The AI's change and a user change overlap in the same region:\n");
            for r in regions {
                out.push_str(&format!(
                    "  - {} lines {}-{}\n",
                    r.path.display(),
                    r.baseline_range.0,
                    r.baseline_range.1
                ));
            }
        }
        ConflictReason::Both { changed_paths, regions } => {
            out.push_str("The live workspace changed since baseline, AND overlapping regions were found:\n");
            for p in changed_paths {
                out.push_str(&format!("  - workspace drift: {}\n", p.display()));
            }
            for r in regions {
                out.push_str(&format!(
                    "  - overlapping region: {} lines {}-{}\n",
                    r.path.display(),
                    r.baseline_range.0,
                    r.baseline_range.1
                ));
            }
        }
    }
    out
}

/// The actual plain text a person would see, listing all four options
/// alongside the conflict's real evidence.
pub fn render_decision_prompt(reason: &ConflictReason) -> String {
    let mut out = String::new();
    out.push_str("AI CONDUCTOR -- CONFLICT: PERSON DECISION REQUIRED\n");
    out.push_str("(plain/functional -- not the Mission Mode UI; Blueprint section 21)\n\n");
    out.push_str(&describe_conflict(reason));
    out.push_str("\nChoose one:\n");
    out.push_str("  [1] Keep user version\n");
    out.push_str("  [2] Keep AI version\n");
    out.push_str("  [3] Merge\n");
    out.push_str("  [4] Review diff\n");
    out
}

/// Durably record a person's decision. Mirrors
/// `console::request_attempt_cancellation` exactly: same `StorePaths`,
/// same `EventLog::open`/`NewEvent` path, `now_ms` caller-supplied (no
/// direct system-clock read, per AC-12). Writes the decision AND the
/// evidence it was made in response to, so the log alone is enough to
/// reconstruct why the person chose what they chose.
pub fn record_person_decision(
    store_dir: &Path,
    context: EventContext,
    reason: &ConflictReason,
    decision: PersonDecision,
    now_ms: u64,
) -> Result<(), ConsoleError> {
    let paths = StorePaths::under(store_dir);
    let mut log = EventLog::open(&paths.events)?;
    log.append(NewEvent {
        event_type: "person.conflict_decision_recorded".to_string(),
        context,
        payload: serde_json::json!({
            "decision": decision.as_str(),
            "terminal": decision.is_terminal(),
            "evidence": describe_conflict(reason),
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
    use crate::mutation_attribution::{MutationAttribution, RegionClassification};
    use std::path::PathBuf;

    fn dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ck_person_decision_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("mkdir");
        d
    }

    fn ctx() -> EventContext {
        CorrelationContext::for_mission("m-1").with_step("s-1")
    }

    fn drift_reason() -> ConflictReason {
        ConflictReason::MergeTimeDrift {
            changed_paths: vec![PathBuf::from("auth.ts"), PathBuf::from("config.rs")],
        }
    }

    fn overlap_reason() -> ConflictReason {
        ConflictReason::RegionOverlap {
            regions: vec![MutationAttribution {
                path: PathBuf::from("model.py"),
                baseline_range: (40, 45),
                classification: RegionClassification::ConflictRegion,
            }],
        }
    }

    // -- describe_conflict: real evidence, not generic text --

    #[test]
    fn describe_conflict_surfaces_real_paths_for_merge_time_drift() {
        let text = describe_conflict(&drift_reason());
        assert!(text.contains("auth.ts"), "must surface the real changed path, got: {text}");
        assert!(text.contains("config.rs"), "must surface the real changed path, got: {text}");
    }

    #[test]
    fn describe_conflict_surfaces_real_path_and_line_range_for_region_overlap() {
        let text = describe_conflict(&overlap_reason());
        assert!(text.contains("model.py"), "must surface the real path, got: {text}");
        assert!(text.contains("40"), "must surface the real range start, got: {text}");
        assert!(text.contains("45"), "must surface the real range end, got: {text}");
    }

    #[test]
    fn describe_conflict_surfaces_both_kinds_of_evidence_for_both_reason() {
        let both = ConflictReason::Both {
            changed_paths: vec![PathBuf::from("auth.ts")],
            regions: vec![MutationAttribution {
                path: PathBuf::from("model.py"),
                baseline_range: (10, 12),
                classification: RegionClassification::ConflictRegion,
            }],
        };
        let text = describe_conflict(&both);
        assert!(text.contains("auth.ts"));
        assert!(text.contains("model.py"));
        assert!(text.contains("10"));
    }

    // -- render_decision_prompt: all four options listed --

    #[test]
    fn render_decision_prompt_lists_all_four_options() {
        let text = render_decision_prompt(&drift_reason());
        assert!(text.contains("Keep user version"));
        assert!(text.contains("Keep AI version"));
        assert!(text.contains("Merge"));
        assert!(text.contains("Review diff"));
    }

    #[test]
    fn render_decision_prompt_includes_the_real_evidence_too() {
        let text = render_decision_prompt(&drift_reason());
        assert!(text.contains("auth.ts"), "prompt must include real evidence, not just option labels");
    }

    // -- record_person_decision: real event, all four variants --

    #[test]
    fn record_person_decision_writes_a_real_readable_event_for_every_variant() {
        let store = dir("all_variants");
        let variants = [
            PersonDecision::KeepUserVersion,
            PersonDecision::KeepAiVersion,
            PersonDecision::Merge,
            PersonDecision::ReviewDiff,
        ];
        for (i, decision) in variants.iter().enumerate() {
            record_person_decision(&store, ctx(), &drift_reason(), *decision, 1000 + i as u64)
                .unwrap_or_else(|e| panic!("record_person_decision failed for {decision:?}: {e}"));
        }

        let paths = StorePaths::under(&store);
        let log = EventLog::open(&paths.events).expect("reopen log");
        let events = log.read_all().expect("read all events");
        assert_eq!(events.len(), 4);
        for (event, decision) in events.iter().zip(variants.iter()) {
            assert_eq!(event.event_type, "person.conflict_decision_recorded");
            assert_eq!(
                event.payload.get("decision").and_then(|v| v.as_str()),
                Some(decision.as_str())
            );
            let expected_terminal = decision.is_terminal();
            assert_eq!(event.payload.get("terminal").and_then(|v| v.as_bool()), Some(expected_terminal));
        }
        let _ = std::fs::remove_dir_all(&store);
    }

    #[test]
    fn recorded_event_carries_the_real_evidence_in_its_payload() {
        let store = dir("evidence_in_payload");
        record_person_decision(&store, ctx(), &overlap_reason(), PersonDecision::Merge, 1000).unwrap();

        let paths = StorePaths::under(&store);
        let log = EventLog::open(&paths.events).expect("reopen log");
        let events = log.read_all().expect("read all events");
        let evidence = events[0].payload.get("evidence").and_then(|v| v.as_str()).expect("evidence field");
        assert!(evidence.contains("model.py"));
        let _ = std::fs::remove_dir_all(&store);
    }

    #[test]
    fn is_terminal_matches_blueprint_four_way_choice_semantics() {
        assert!(PersonDecision::KeepUserVersion.is_terminal());
        assert!(PersonDecision::KeepAiVersion.is_terminal());
        assert!(PersonDecision::Merge.is_terminal());
        assert!(!PersonDecision::ReviewDiff.is_terminal());
    }
}
