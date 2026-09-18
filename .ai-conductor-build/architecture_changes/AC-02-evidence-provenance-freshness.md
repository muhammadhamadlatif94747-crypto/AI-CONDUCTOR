# Architecture Contract AC-02 — Evidence Provenance & Freshness
# Task: P0-W02-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-02
task_id: P0-W02-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §4.5 (Evidence — Structured Provenance)"
  - "Verification Gates §7 (Evidence Model), §8 (Trust Rules), §9 (Freshness)"
  - "Phase Manifest P0-C02"
  - "Build Protocol §12 (OBSERVED/VERIFIED/DOCUMENTED/INFERRED/UNKNOWN)"
```

## 1. What evidence is

An Evidence record is the only admissible input by which verification decisions are
made. A model claim, a provider HTTP 200, or an agent's own statement is never,
by itself, evidence of completion. Every record is structured — trust category is a
field of the record, not a global ranking.

## 2. Canonical record fields (required proof #1)

Every persisted Evidence record MUST carry all of:

```text
evidence_id        unique id (EV-...)
provenance         one of the six provenance classes below
observation_type   what kind of check (test_output | diff | filesystem_observation |
                   git_observation | runtime_observation | provider_observation |
                   security_check | browser_check | manual_review | requirement_check)
freshness          produced_at timestamp + duration since production, against the
                   clock source that governed the run
clock_source       real | virtual   (virtual-clock runs are valid evidence inside
                   simulation; real-clock runs required for real-world gates)
execution_id       ties the record to ONE specific run — never "a run once"
source             command/system that produced it (e.g. "cargo test -- quiet")
correlation_ids    mission_id / step_id / attempt_id chain per Blueprint §4.1.1
state_binding      commit SHA and/or worktree ID and/or content fingerprints the
                   evidence was produced against
reproducibility    Deterministic | Repeatable | Flaky | Unknown
integrity          IndependentlyVerified | SelfReported | Invalid
result             pass | fail | unknown
artifact_reference pointer to full output (report/log path); record stays lightweight
```

`state_binding` is mandatory wherever applicable; evidence without it cannot be
freshness-checked and is treated as stale by default (fail-closed).

## 3. Provenance classes (required proof #2)

Exactly six classes — where evidence came from, not how much to trust it:

```text
ModelClaim                 a model/provider asserted something
ProcessObservation         observed process exit/output
FilesystemObservation      observed file state/diff/hashes
LocalTestRunner            a test runner executed locally in this environment
CiSystem                   CI/external build system result
ExternalSystemObservation  any other external system (OmniRoute health, etc.)
```

No class implies another's trustworthiness. Selection between classes happens per
criterion via the Mission Acceptance Contract (AC-08), which may demand a specific
combination (e.g., `provenance = LocalTestRunner AND freshness ≤ 5min AND
execution_id = current_attempt`). This structurally prevents a stale CI badge from
satisfying a criterion meant for fresh local proof.

## 4. Freshness & invalidation rules (required proof #2 continued)

- Freshness is always evaluated against `produced_at`, the governing `clock_source`,
  and `state_binding`.
- Invalidation triggers (non-exhaustive): source changed after the run → build/test
  evidence invalid; config changed → test evidence invalid; worktree changed after
  diff inspection → diff evidence invalid; capability data older than its staleness
  policy → capability evidence suspect.
- An evidence record whose `state_binding` no longer matches the current state is
  INVALID for acceptance decisions — it may be retained for history but cannot pass
  a gate.
- UNKNOWN evidence causes BLOCKED or FAIL within a gate; it never becomes PASS for
  convenience.
- INFERRED may not silently satisfy a criterion requiring executable proof.

## 5. Trust-class discipline (required proof #3)

The five process-level classes OBSERVED / VERIFIED / DOCUMENTED / INFERRED / UNKNOWN
govern what an agent may *claim*; AC-02's provenance/integrity/freshness fields
govern what a *gate* may *accept*. The mapping rule:

```text
VERIFIED claims require reproducible execution (LocalTestRunner or equivalent) with
         intact freshness/state_binding.
OBSERVED alone is admissible evidence but not automatically sufficient for acceptance.
DOCUMENTED establishes expected contracts, never runtime behavior.
INFERRED never satisfies executable-proof criteria.
UNKNOWN  → BLOCKED/FAIL per §4.
```

## 6. Non-goals

Acceptance-contract format itself (AC-08), verification pipeline mechanics (P4),
provider capability registry staleness policy internals (P0-C07 covers the table;
§4 here governs its evidence).

## 7. Invariants preserved

2, 3, 4, 7, 8, 14. None weakened.
