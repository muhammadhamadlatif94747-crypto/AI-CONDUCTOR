# Failure Case — F-0001

```json
{
  "failure_id": "F-0001",
  "recorded_at": "2026-08-26T05:45:00+05:00",
  "severity": "medium",
  "classification": "process_tooling",
  "title": "Concurrent edits to the same JSON file clobbered each other (lost update)",
  "status": "ROOT_CAUSE_IDENTIFIED_REGRESSION_PLANNED",

  "what_happened": "Two parallel SearchReplace tool calls targeted the same BUILD_STATE.json in one batch. The second call's write was based on the pre-first-edit content, so the first edit (current_task -> null + task_history ACCEPTED entry) was silently lost while the second (checkpoint commit hash) survived. Discovered during post-edit verification when task_history keys came back empty.",

  "detection": "Verification probe: parse BUILD_STATE.json and read back task_history.P0-W00-T01 — returned empty. Build-state verification caught it before any downstream use.",

  "root_cause_class": "lost_update_write_write_conflict",
  "root_cause": "Two writers, same file, no serialization. Applies to any future multi-writer state mutation (agent tools, later the Conductor's own persistence shell).",

  "invariants_touched": [
    "Invariant 15 analog at build-process level: multi-record logical update must be durable and consistent; a lost update is exactly what Invariant 15/16 discipline exists to prevent."
  ],

  "reproduction": "Deterministic: issue two same-file writes in parallel against stale snapshots. Reproduced once in-session.",

  "fix_applied": "Re-applied the lost edit sequentially after detecting the conflict; verified by re-parse.",

  "regression_protection": {
    "immediate_rule": "Never issue two write/edit calls to the same file in one parallel batch. Same-file mutations are strictly sequential.",
    "future_kernel_requirement": "The Conductor persistence layer must serialize all writes to a state store behind a single writer (or file lock). Recorded as an implementation requirement for P1-W05 atomic persistence work."
  },

  "related": ["F-ENV-0001"]
}
```
