# Architecture Contract AC-12 — Clock Trait
# Task: P0-W12-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-12
task_id: P0-W12-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §10.2 (Deterministic Virtual Clock + hard architecture rule)"
  - "Verification Gates §7 (clock_source field), §13 checklist"
  - "AC-02 (freshness), AC-04 (event timestamps)"
  - "Phase Manifest P0-C12"
```

## 1. The Clock abstraction (required)

```rust
trait Clock {
    fn now(&self) -> Timestamp;          // instant used by ALL time-dependent core logic
    fn advance(&self, d: Duration);      // VirtualClock only; test control of time
}
```

- `RealClock` — production implementation, backed by the system clock.
- `VirtualClock` — test implementation; advancing it instantly exercises cooldowns,
  quota resets, retry windows, and backoff in milliseconds of real time
  ("quota resets after 60 seconds" without waiting 60 seconds).

## 2. Hard rule (required)

**No file in the reliability core may call the real system clock directly**
(`SystemTime::now()` or equivalent) for any time-dependent logic. All such reads go
through the injectable `Clock` trait. This is not a style preference: a single stray
direct clock call anywhere in cooldown/quota/retry/backoff logic silently reintroduces
slow, flaky, real-time-dependent tests exactly where they are most expensive to have.

Enforcement expectation (P1+): a source-level check/grep gate for direct system-clock
calls in core modules belongs in the verification pipeline; a violation fails the gate.

## 3. Consumers and recorded source

Every timestamp-bearing record states its `clock_source: real | virtual`
(Verification Gates §7): evidence freshness (AC-02), event chain timestamps (AC-04),
checkpoint times (AC-03), health/staleness windows (§10.3). Virtual-clock-derived
evidence can never satisfy real-world freshness requirements — provenance rules apply.

## 4. Non-goals

Timer/scheduling mechanics (retry backoff engine internals, Phase 3+),
FakeProvider behavior matrix itself (§10.2 enum, owned by its phase),
UI clock display.

## 5. Invariants preserved

5 (deterministic recovery — virtual clock makes recovery tests reproducible),
8 (events carry consistent timestamps). None weakened.
