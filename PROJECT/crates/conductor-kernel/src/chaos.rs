//! Chaos Mode (P3-W03 / Blueprint §10.4).
//!
//! ```text
//! CHAOS MODE
//! ☑ Random provider failure        [ 10% ]
//! ☑ Random network timeout         [ 10% ]
//! ☑ Process termination            [  5% ]
//! ☑ Delayed response                [ 15% ]
//! ☑ Malformed model output          [  5% ]
//! ☑ Checkpoint interruption         [  5% ]
//! ☑ State persistence interruption  [  5% ]
//! ☑ Simulated user edit mid-attempt [  5% ]   ← exercises Conflict detection
//! ```
//!
//! Blueprint §10.4's hard rule: *"every chaos run records its
//! `chaos_seed`, `scenario_id`, and `virtual_time_seed`... random chaos
//! without a recorded seed is chaos you cannot debug."* [`ChaosConfig`]
//! carries all three; [`ChaosEngine::roll`] is deterministic given the
//! same seed, proven directly rather than assumed from "it uses a seed."
//!
//! Phase Manifest §7.3 P3-W03 singles out one category as required:
//! *"must include simulated mid-attempt user edits."*
//! [`simulate_user_edit_mid_attempt`] is that category made concrete —
//! and this module's own tests prove it actually triggers P2-W06's
//! Conflict detection end to end (`tests::simulated_user_edit_mid_attempt_triggers_conflict_detection`),
//! not just that the function exists.
//!
//! Scope, stated explicitly (this is P3-W03 only): `ProcessTermination`,
//! `CheckpointInterruption`, and `StatePersistenceInterruption` exist as
//! [`ChaosCategory`] variants (for weight-rolling and fidelity to
//! Blueprint's eight-item checklist) but have no defined mechanical
//! effect here — there is no attempt-execution orchestrator yet for a
//! process-termination event to interrupt, the same gap every P2 task
//! since W06 has independently confirmed. Wiring those three to
//! something real is P3-W04's or a later phase's job.

use crate::provider::FakeBehavior;
use std::path::Path;
use std::time::Duration;

/// Blueprint §10.4's eight chaos categories, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChaosCategory {
    RandomProviderFailure,
    RandomNetworkTimeout,
    ProcessTermination,
    DelayedResponse,
    MalformedModelOutput,
    CheckpointInterruption,
    StatePersistenceInterruption,
    SimulatedUserEditMidAttempt,
}

impl ChaosCategory {
    pub const ALL: [ChaosCategory; 8] = [
        ChaosCategory::RandomProviderFailure,
        ChaosCategory::RandomNetworkTimeout,
        ChaosCategory::ProcessTermination,
        ChaosCategory::DelayedResponse,
        ChaosCategory::MalformedModelOutput,
        ChaosCategory::CheckpointInterruption,
        ChaosCategory::StatePersistenceInterruption,
        ChaosCategory::SimulatedUserEditMidAttempt,
    ];

    /// Blueprint §10.4's default percentage for this category.
    pub fn default_weight_percent(&self) -> u8 {
        match self {
            ChaosCategory::RandomProviderFailure => 10,
            ChaosCategory::RandomNetworkTimeout => 10,
            ChaosCategory::ProcessTermination => 5,
            ChaosCategory::DelayedResponse => 15,
            ChaosCategory::MalformedModelOutput => 5,
            ChaosCategory::CheckpointInterruption => 5,
            ChaosCategory::StatePersistenceInterruption => 5,
            ChaosCategory::SimulatedUserEditMidAttempt => 5,
        }
    }

    /// Maps the provider-adapter-layer categories to a concrete
    /// `FakeBehavior` (P3-W01). Categories representing chaos at other
    /// layers (process/persistence) return `None` — see module docs for
    /// why that's this task's deliberate scope boundary, not an
    /// oversight.
    pub fn to_fake_behavior(&self) -> Option<FakeBehavior> {
        match self {
            ChaosCategory::RandomProviderFailure => Some(FakeBehavior::Crash),
            ChaosCategory::RandomNetworkTimeout => Some(FakeBehavior::Timeout),
            ChaosCategory::DelayedResponse => Some(FakeBehavior::SlowResponse(Duration::from_secs(30))),
            ChaosCategory::MalformedModelOutput => Some(FakeBehavior::MalformedOutput),
            ChaosCategory::ProcessTermination
            | ChaosCategory::CheckpointInterruption
            | ChaosCategory::StatePersistenceInterruption
            | ChaosCategory::SimulatedUserEditMidAttempt => None,
        }
    }
}

/// Blueprint §10.4's hard rule: every chaos run records these three
/// values, since unseeded chaos can't be replayed or debugged.
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    pub chaos_seed: u64,
    pub scenario_id: String,
    pub virtual_time_seed: u64,
    weights: [u8; 8],
}

impl ChaosConfig {
    /// Blueprint §10.4's default weights for every category.
    pub fn with_default_weights(chaos_seed: u64, scenario_id: impl Into<String>, virtual_time_seed: u64) -> Self {
        let mut weights = [0u8; 8];
        for (i, category) in ChaosCategory::ALL.iter().enumerate() {
            weights[i] = category.default_weight_percent();
        }
        ChaosConfig {
            chaos_seed,
            scenario_id: scenario_id.into(),
            virtual_time_seed,
            weights,
        }
    }

    pub fn weight_percent(&self, category: ChaosCategory) -> u8 {
        let index = ChaosCategory::ALL.iter().position(|c| *c == category).expect("category is in ALL");
        self.weights[index]
    }

    pub fn set_weight_percent(&mut self, category: ChaosCategory, percent: u8) {
        let index = ChaosCategory::ALL.iter().position(|c| *c == category).expect("category is in ALL");
        self.weights[index] = percent;
    }
}

/// A small, self-contained, deterministic PRNG (splitmix64) — no
/// external randomness crate. The only property this module actually
/// needs is exact seeded reproducibility, which splitmix64 provides
/// just as well as a general-purpose crate would, without adding a new
/// dependency for it.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// A value in `[0, 100)`, for percentage-threshold comparisons.
    fn next_percent(&mut self) -> u8 {
        (self.next_u64() % 100) as u8
    }
}

/// Seeded, deterministic chaos injector. [`ChaosEngine::roll`] evaluates
/// every category as an *independent* trial (not a mutually-exclusive
/// partition of one roll) — Blueprint §28.5's own composite-scenario
/// example combines multiple simultaneous chaos events in one scenario,
/// which independent trials support and a single exclusive slice would
/// not.
pub struct ChaosEngine {
    config: ChaosConfig,
    rng: SplitMix64,
}

impl ChaosEngine {
    pub fn new(config: ChaosConfig) -> Self {
        let rng = SplitMix64::new(config.chaos_seed);
        ChaosEngine { config, rng }
    }

    pub fn config(&self) -> &ChaosConfig {
        &self.config
    }

    /// Evaluate one independent trial per category against its
    /// configured weight; return every category that fired this roll
    /// (possibly empty, possibly more than one).
    pub fn roll(&mut self) -> Vec<ChaosCategory> {
        ChaosCategory::ALL
            .iter()
            .copied()
            .filter(|category| self.rng.next_percent() < self.config.weight_percent(*category))
            .collect()
    }
}

/// Blueprint §10.4's `SimulatedUserEditMidAttempt`, made concrete:
/// deterministically appends a marker line to `target_path` inside
/// `workspace_root`, simulating a real person editing a file in the live
/// workspace while an attempt is in flight — Phase Manifest §7.3 P3-W03's
/// explicitly required category, exercising P2-W06's Conflict detection
/// (see `tests::simulated_user_edit_mid_attempt_triggers_conflict_detection`
/// for the end-to-end proof).
///
/// Deterministic in `seed`: the same seed always appends the same exact
/// content, so a chaos run using this function is as replayable as the
/// rest of this module.
pub fn simulate_user_edit_mid_attempt(
    workspace_root: &Path,
    target_relative_path: &Path,
    seed: u64,
) -> std::io::Result<()> {
    let mut rng = SplitMix64::new(seed);
    let marker = rng.next_u64();
    let full_path = workspace_root.join(target_relative_path);
    let mut existing = std::fs::read_to_string(&full_path).unwrap_or_default();
    existing.push_str(&format!("\n// chaos: simulated mid-attempt user edit (seed marker {marker:016x})\n"));
    std::fs::write(&full_path, existing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::Baseline;
    use crate::conflict::{check_merge_safety, MergeCheckResult};
    use std::fs;

    fn workspace(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ck_chaos_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    #[test]
    fn all_eight_categories_present_with_blueprint_exact_default_weights() {
        let expected: [(ChaosCategory, u8); 8] = [
            (ChaosCategory::RandomProviderFailure, 10),
            (ChaosCategory::RandomNetworkTimeout, 10),
            (ChaosCategory::ProcessTermination, 5),
            (ChaosCategory::DelayedResponse, 15),
            (ChaosCategory::MalformedModelOutput, 5),
            (ChaosCategory::CheckpointInterruption, 5),
            (ChaosCategory::StatePersistenceInterruption, 5),
            (ChaosCategory::SimulatedUserEditMidAttempt, 5),
        ];
        for (category, weight) in expected {
            assert_eq!(category.default_weight_percent(), weight, "{category:?} weight mismatch");
        }
        assert_eq!(ChaosCategory::ALL.len(), 8);
    }

    #[test]
    fn provider_layer_categories_map_to_distinct_fake_behaviors() {
        assert_eq!(ChaosCategory::RandomProviderFailure.to_fake_behavior(), Some(FakeBehavior::Crash));
        assert_eq!(ChaosCategory::RandomNetworkTimeout.to_fake_behavior(), Some(FakeBehavior::Timeout));
        assert_eq!(ChaosCategory::MalformedModelOutput.to_fake_behavior(), Some(FakeBehavior::MalformedOutput));
        assert!(matches!(
            ChaosCategory::DelayedResponse.to_fake_behavior(),
            Some(FakeBehavior::SlowResponse(_))
        ));
    }

    #[test]
    fn non_provider_layer_categories_have_no_fake_behavior_mapping() {
        assert_eq!(ChaosCategory::ProcessTermination.to_fake_behavior(), None);
        assert_eq!(ChaosCategory::CheckpointInterruption.to_fake_behavior(), None);
        assert_eq!(ChaosCategory::StatePersistenceInterruption.to_fake_behavior(), None);
        assert_eq!(ChaosCategory::SimulatedUserEditMidAttempt.to_fake_behavior(), None);
    }

    #[test]
    fn roll_is_deterministic_across_repeated_seeded_runs() {
        let config_a = ChaosConfig::with_default_weights(42, "scenario-x", 0);
        let config_b = ChaosConfig::with_default_weights(42, "scenario-x", 0);
        let mut engine_a = ChaosEngine::new(config_a);
        let mut engine_b = ChaosEngine::new(config_b);

        for _ in 0..20 {
            assert_eq!(engine_a.roll(), engine_b.roll(), "identical seed must produce identical roll sequences");
        }
    }

    #[test]
    fn config_carries_all_three_hard_rule_fields() {
        let config = ChaosConfig::with_default_weights(893442, "chaos-4812", 0);
        assert_eq!(config.chaos_seed, 893442);
        assert_eq!(config.scenario_id, "chaos-4812");
        assert_eq!(config.virtual_time_seed, 0);
    }

    #[test]
    fn set_weight_percent_overrides_the_default() {
        let mut config = ChaosConfig::with_default_weights(1, "s", 0);
        config.set_weight_percent(ChaosCategory::RandomProviderFailure, 100);
        assert_eq!(config.weight_percent(ChaosCategory::RandomProviderFailure), 100);
        // A 100% weight must fire on every roll, deterministically.
        let mut engine = ChaosEngine::new(config);
        for _ in 0..10 {
            assert!(engine.roll().contains(&ChaosCategory::RandomProviderFailure));
        }
    }

    #[test]
    fn a_zero_weight_never_fires() {
        let mut config = ChaosConfig::with_default_weights(7, "s", 0);
        for category in ChaosCategory::ALL {
            config.set_weight_percent(category, 0);
        }
        let mut engine = ChaosEngine::new(config);
        for _ in 0..20 {
            assert!(engine.roll().is_empty());
        }
    }

    #[test]
    fn simulate_user_edit_mid_attempt_is_deterministic_by_exact_content() {
        let ws_a = workspace("determinism_a");
        let ws_b = workspace("determinism_b");
        fs::write(ws_a.join("shared.txt"), "original\n").unwrap();
        fs::write(ws_b.join("shared.txt"), "original\n").unwrap();

        simulate_user_edit_mid_attempt(&ws_a, std::path::Path::new("shared.txt"), 555).unwrap();
        simulate_user_edit_mid_attempt(&ws_b, std::path::Path::new("shared.txt"), 555).unwrap();

        let content_a = fs::read_to_string(ws_a.join("shared.txt")).unwrap();
        let content_b = fs::read_to_string(ws_b.join("shared.txt")).unwrap();
        assert_eq!(content_a, content_b, "identical seed must produce byte-identical appended content");

        let _ = fs::remove_dir_all(&ws_a);
        let _ = fs::remove_dir_all(&ws_b);
    }

    /// The concrete proof Phase Manifest §7.3 P3-W03 asks for: the
    /// mid-attempt-user-edit chaos category actually exercises Conflict
    /// detection end to end, not just as an isolated unit of the chaos
    /// function alone.
    #[test]
    fn simulated_user_edit_mid_attempt_triggers_conflict_detection() {
        let ws = workspace("triggers_conflict");
        fs::write(ws.join("shared.txt"), "original content\n").unwrap();

        let original_baseline = Baseline::capture(&ws).expect("capture baseline");

        simulate_user_edit_mid_attempt(&ws, std::path::Path::new("shared.txt"), 999).expect("simulate edit");

        let live_at_merge_time = Baseline::capture(&ws).expect("capture after chaos edit");
        let result = check_merge_safety(&original_baseline, &live_at_merge_time);

        match result {
            MergeCheckResult::Conflict { changed_paths } => {
                assert_eq!(changed_paths, vec![std::path::PathBuf::from("shared.txt")]);
            }
            MergeCheckResult::Safe => panic!("a mid-attempt user edit must be detected as Conflict, not Safe"),
        }

        let _ = fs::remove_dir_all(&ws);
    }
}
