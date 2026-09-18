# Architecture Contract AC-11 — External Side-Effect Idempotency Classification
# Task: P0-W11-T01 (Phase 0 — no production code; this document IS the deliverable)

```yaml
contract_id: AC-11
task_id: P0-W11-T01
contract_version: 1
status: DEFINED

authority:
  blueprint: "2.4"
  protocol: "1.0"
  phase_manifest: "1.0"

section_refs:
  - "Blueprint §10.5 (External Side-Effect Idempotency), §10.1, §20"
  - "Blueprint §3 Invariant 16"
  - "AC-02 (Unknown handling), AC-05 (failure classes)"
  - "Phase Manifest P0-C11"
```

## 1. IdempotencySupport classification model (required)

Every external side effect carries an explicit classification carried per provider/combo
in the Capability Registry — retry behavior branches on it, **never assumed**:

```rust
enum IdempotencySupport {
    ServerSideGuaranteed, // provider documents/confirms dedup by idempotency key → safe to retry
    Unknown,              // retry-with-same-key behavior not confirmed → NOT safe to blind-retry
    NotIdempotent,        // provider explicitly does not deduplicate → retry after ambiguous
                          //   outcome is a genuinely new side effect
}
```

**Default: `Unknown`** for any provider/combo not explicitly verified otherwise.
The safe default is "don't assume" — same philosophy as the §10.3 staleness policy.

## 2. Key vs guarantee distinction

`ProviderRequest.idempotency_key` (§10.1) is a *local* signal of intent; it says nothing
about whether the *provider* treats repeated requests with that key as a no-op.
Generating a key never upgrades a classification. A dropped connection after the provider
already processed a request, followed by a naive retry, can double-execute even when local
bookkeeping looks perfectly idempotent (Invariant 16's exact gap).

## 3. Ambiguous-outcome decision rule (required)

```text
Request outcome ambiguous (timeout, disconnect, no confirmed response)
        ↓
ServerSideGuaranteed → safe to retry with the SAME idempotency_key
Unknown / NotIdempotent → do NOT blind-retry:
    Queryable? ("did request abc-123 complete?")
        Yes → query, reconcile against ground truth, proceed from real state
        No  → surface as Unknown (AC-01 outcome) to the person — an explicit
              "not sure it went through" beats a silent possible double-execution
```

## 4. Scope and store separation

Applies to every process the Conductor doesn't fully control: provider API calls,
verification commands, any spawned tool execution. Local deterministic operations
(state transitions under §17 executor) are out of scope.

Two distinct artifacts (per Blueprint module map):
- `idempotency.rs` — the IdempotencySupport classification model (this contract);
- `idempotency_store.rs` — the local "already applied?" retry-detection store,
  which is necessary but NEVER sufficient without the classification above.

## 5. Non-goals

Capability Registry population/staleness internals (§10.3, resource phase),
outbox mechanics (P1-W06), provider adapter implementations (Phases 6–7).

## 6. Invariants preserved

3 (resume never blindly repeats an accepted side effect), 5, 8, 16 (core subject).
None weakened.
