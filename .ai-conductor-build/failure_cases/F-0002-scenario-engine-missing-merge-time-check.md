# Failure Case — F-0002

```json
{
  "failure_id": "F-0002",
  "recorded_at": "2026-08-30T15:30:00+00:00",
  "severity": "medium",
  "classification": "reconciliation_logic_gap",
  "title": "Scenario engine's first version only ran reconcile(), missing Invariant 13's merge-time re-check entirely",
  "status": "FIXED_AND_REGRESSION_TESTED",

  "what_happened": "P3-W04-T01's first implementation of scenario.rs::run_scenario called only reconciliation.rs::reconcile() to produce a scenario's actual outcome. cargo test genuinely failed changed_baseline_family (1/216): the scenario declared a.txt as an expected file, a mid-attempt user edit changed a.txt, and reconcile() reported ExpectedChange -- 'the file that was supposed to change did' -- because reconcile() has no way to see WHO changed a file, only THAT it changed. The test's own expectation (UnexpectedChange) was itself wrong for that scenario shape, but investigating why revealed the real gap underneath: several required failure families (changed_baseline, merge_conflict, and -- more subtly -- user_conflict, where reconcile() alone can look misleadingly clean) are fundamentally about Invariant 13's merge-time re-check (conflict.rs::check_merge_safety), not reconcile()'s whole-file classification. The engine simply never called it.",

  "detection": "cargo test --workspace, run as part of this session's standard verification discipline (never skipped, never assumed passing). 1 of 216 tests failed on the first run. Not caught by design review before implementation -- caught by actually running the test suite against real scenario execution, which is exactly the kind of gap a docs-only review would miss.",

  "root_cause_class": "single_check_used_where_two_independent_safety_checks_are_both_required",
  "root_cause": "P2 built two separate, independently-tested safety checks that exist for different reasons: reconcile() (P2-W03) classifies whole-file diffs against declared intent; check_merge_safety() (P2-W06, Invariant 13) re-checks the live workspace against the original baseline immediately before a merge, regardless of who caused any drift. Composing only one of the two into a new caller (the scenario engine) silently drops the coverage the other one provides. This is a composition-boundary risk, not a bug in either P2-W03 or P2-W06 individually -- both passed their own tasks' tests correctly in isolation.",

  "invariants_touched": [
    "Invariant 13 (two-phase merge re-check) -- the scenario engine's first version could not have exercised or proven this invariant for any scenario, despite several required failure families (SS7.4: changed_baseline, merge_conflict, user_conflict) being specifically about it."
  ],

  "reproduction": "Deterministic and reproducible on demand: construct a Scenario with expected_files containing a path, a behavior sequence containing only a SimulatedUserEdit to that same path (no attempt Edit step), and expected_reconciliation set to ExpectedChange -- run_scenario's pre-fix version would report ExpectedChange (matching that assertion) while genuinely never having checked whether the change was safe to merge.",

  "fix_applied": "run_scenario now calls BOTH reconcile() and check_merge_safety() unconditionally, and returns both results on ScenarioResult. Scenario gained an expected_merge_safe: Option<bool> field so a scenario can assert on whichever check is actually relevant to what it's testing. changed_baseline_family, merge_conflict_family, and user_conflict_family were rewritten to assert on check_merge_safety's result (expected_merge_safe: Some(false)) rather than only reconcile()'s outcome; user_conflict_family's own code comment now states explicitly that reconcile() alone would look clean here and that is exactly why the merge-time re-check exists as a safety net over it.",

  "regression_protection": {
    "immediate_rule": "Any future module that composes reconcile() and check_merge_safety() together (or drives an attempt lifecycle that will eventually merge) must call both, not just one -- they answer different questions (intent-alignment vs. baseline-drift-regardless-of-cause) and neither substitutes for the other.",
    "test_that_would_catch_a_regression": "scenario.rs::tests::user_conflict_family, changed_baseline_family, and merge_conflict_family all now assert on check_merge_safety's output specifically; removing that call from run_scenario would fail expected_merge_safe's assertion in all three, not just silently pass on reconcile()'s output alone.",
    "future_kernel_requirement": "Whichever future task builds the real attempt-execution orchestrator (repeatedly noted as not yet existing, since P2-W06) must call both checks at merge time, mirroring this fix -- flagged here so that task's own contract review has this precedent to check against."
  },

  "related": ["none -- first Phase 3 corpus entry; F-0001 is a build-process/tooling case, unrelated to kernel logic"]
}
```
