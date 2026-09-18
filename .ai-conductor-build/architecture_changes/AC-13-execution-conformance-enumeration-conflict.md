> **Supersession note (added during the P5 conformance-foundation
> correction pass, historical record — nothing below this note was
> altered):** the inline comparison table below is now also maintained,
> in expanded form, as a standalone canonical artifact at
> `GOVERNANCE/EC_ENUMERATION_MAPPING.md`. That document is the one
> `conformance.rs` links to going forward. This record remains the
> historical account of how and why the conflict was first discovered;
> its STATUS line below is unchanged and still governs.

TITLE:
Three non-identical "EC-xx" execution-conformance enumerations exist across
the governing documents, with different counts and different item meanings
at the same numbers.

CURRENT RULE:
Phase Manifest §9.3 (P5-W03..W08) and Verification Gates §66 both say a
"common executor conformance suite" gates real-executor promotion, and both
assume a single canonical numbered list exists.

BLUEPRINT REFERENCES:
- Master Blueprint §10.7.6 "Common Executor Conformance Tests": lists
  EC-01 through EC-16 (16 items).
- Execution Adapter Contract v1.0 §14 "Conformance Suite": lists
  EC-01 through EC-17 (17 items).
- Verification Gates v1.1 §66 amendment "Execution Harness Conformance
  Gates": lists VG-P5-EXEC-01 through VG-P5-EXEC-12 (12 items, different
  naming convention entirely).

OBSERVED PROBLEM:
The three lists do not line up numerically or by content at the same
number:

| # | Blueprint §10.7.6 | Adapter Contract §14 |
|---|---|---|
| EC-01 | identity/capability discovery (combined) | identity (only) |
| EC-02 | authorized workspace execution | capabilities (only) |
| EC-03 | forbidden-path rejection | workspace confinement |
| EC-12 | provider failure propagation | evidence collection |
| EC-17 | *(does not exist)* | executor replacement |

Blueprint has no item for "evidence collection" as its own numbered entry
and no EC-17 at all. Adapter Contract has no item for "forbidden-path
rejection" as its own numbered entry and no item for "provider failure
propagation" as its own numbered entry. The two lists are 16 vs 17 items
and diverge starting at EC-01/EC-02's boundary.

Verification Gates §66 is a third, non-numerically-comparable scheme (12
items, named not numbered: "Workspace Confinement," "Lifecycle
Observation," "Unknown Safety," etc.) that reads as a *consolidated*
grouping of the same underlying concerns rather than a third distinct
requirement set, but this is inferred, not stated anywhere.

EVIDENCE:
Direct text comparison of the three sections, reproduced verbatim in the
table above. No later document (v2.5.1 Blueprint corrections, v1.1
harness-neutral amendments) resolves the discrepancy or references it.

WHY CURRENT DESIGN FAILS:
Any conformance-suite implementation that claims "passes EC-04" is
ambiguous about which document's EC-04 it means (Blueprint: "successful
execution evidence"; Adapter Contract: "successful execution" — these
happen to mean approximately the same thing at #04, but the drift starting
at #01-03 and the extra #17 mean this alignment cannot be assumed to hold
at every number without checking).

WHAT THIS RECORD DOES NOT DO:
It does not silently pick one numbering and present it as if the conflict
didn't exist. It does not block all P5 conformance-foundation work either
— Task Contracts §1.3 and Build Protocol §14 call for stopping and
recording a genuine architecture contradiction, which this is, while
permitting continued work on whatever isn't actually in dispute.

PROPOSED RESOLUTION (provisional, pending human confirmation):
Adopt the **Execution Adapter Contract v1.0 §14 EC-01..EC-17** enumeration
as canonical for the `conformance.rs` implementation, because:
1. it is the most granular (17 vs 16 items) and the most specific document
   (its entire subject is this boundary, "Normative execution-boundary
   contract" per its own header);
2. its EC-03 "workspace confinement" is mechanically checkable using
   existing P2 primitives (`baseline.rs`) in a way Blueprint's EC-02/EC-03
   split ("authorized workspace execution" + "forbidden-path rejection")
   is not more precisely served by;
3. its EC-17 "executor replacement" directly operationalizes Adapter
   Contract §18's Replacement Principle and Phase Manifest P5-W08, which
   the Blueprint's 16-item list has no corresponding entry for at all —
   dropping it would leave a real requirement (§18, P5-W08) untested.

Blueprint §10.7.6's "forbidden-path rejection" and "provider failure
propagation" concerns are not dropped — they are folded into Adapter
Contract EC-03 (workspace confinement covers forbidden-path rejection by
construction: confinement failing *is* a forbidden-path write) and EC-12
"evidence collection" respectively (a provider failure is one of the
`FailureCategory` values `collect_evidence`/`observe` must surface,
per Adapter Contract §13).

Verification Gates §66's twelve VG-P5-EXEC items are treated as the
*phase-gate-level* grouping over the same seventeen mechanically-checked
items — e.g. VG-P5-EXEC-03 "Lifecycle Observation" is satisfied only when
EC-04 through EC-10 all pass; VG-P5-EXEC-10 "Executor Reversibility" maps
directly to EC-17. This mapping is recorded in `conformance.rs`'s module
documentation, not just here, so it survives independently of this file.

ALTERNATIVES CONSIDERED:
- Implement all three lists in parallel as separate enums: rejected as
  needless complexity that would triple the surface area for no
  corresponding increase in actual test coverage, since the underlying
  concerns overlap almost completely.
- Wait for human resolution before writing any conformance code: rejected
  per Task Contracts §1.3's guidance that a contradiction blocks the
  *affected path* (here: which exact numbering label to print), not all
  adjacent work. The mechanical checks themselves (workspace confinement,
  evidence-field population, credential-scan, structural no-Succeeded/
  no-merge grep) are identical in substance regardless of which of the
  three lists is used to label them.

INVARIANTS AFFECTED:
None. This is a documentation/labeling conflict, not a correctness or
authority-boundary conflict — Invariant 14 (Succeeded is verification-only)
and the merge-authority rule are unaffected either way, and are in fact
independently re-verified by `conformance.rs`'s structural checks
regardless of which numbering is used.

STATE MODEL AFFECTED:
None.

TESTS AFFECTED:
`conformance.rs`'s new tests are written against the Adapter Contract
numbering per the proposed resolution above; a future renumbering (if the
human resolves this differently) would rename `ConformanceItem` variants,
not change what each check actually verifies.

SECURITY IMPACT:
None.

PERFORMANCE IMPACT:
None.

RECOMMENDATION:
Adopt the provisional resolution above for implementation purposes now
(unblocks the conformance foundation this session was asked to build);
flag it to the human for an explicit confirm/override at the next
decision point. Do not treat "provisional" as "final" in any later
session without that confirmation.

STATUS: pending human decision (provisional resolution in active use per
Build Protocol §14 — implementation may proceed on a documented
contradiction while awaiting confirmation, since none of the three
candidate resolutions changes an invariant, state model, or security
boundary).

---

> **HUMAN RULING (recorded, governance-only checkpoint, does not alter
> anything above this line):**
>
> **AC-13 = `PROVISIONAL_17_ITEM_EXECUTION_ADAPTER_ENUMERATION`.**
>
> The Execution Adapter Contract v1.0 §14 17-item enumeration remains the
> active *provisional* enumeration for P5 conformance work.
> `GOVERNANCE/EC_ENUMERATION_MAPPING.md` is the canonical mapping
> artifact for this provisional state. This ruling does **not** mean the
> underlying Blueprint §10.7.6 / Verification Gates §66-67 / Phase
> Manifest numbering disagreement has been permanently rewritten or
> resolved — none of those documents were renumbered or edited to match
> the Adapter Contract, and none should be inferred to agree with it
> merely because this ruling exists. The ruling is narrower than that: it
> formally authorizes continuing P5 structural/synthetic conformance work
> under the existing provisional choice, rather than leaving that choice
> perpetually unratified. Real-executor certification still must not cite
> this ruling, or the mapping document, as proof the numbering
> disagreement itself is settled.
>
> STATUS: **DECIDED (provisional).** Superseding the "pending human
> decision" line above for the narrow question of "may implementation
> continue under the 17-item enumeration" — the broader question of
> whether the governing documents should ever be reconciled numerically
> remains genuinely open and is not addressed by this ruling.
