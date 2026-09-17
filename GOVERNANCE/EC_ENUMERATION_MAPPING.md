# EC Enumeration Mapping — Canonical Cross-Reference (PROVISIONAL)

**Status: PROVISIONAL, AND FORMALLY RULED ON.** A human architectural
ruling (recorded in
`.ai-conductor-build/architecture_changes/AC-13-execution-conformance-enumeration-conflict.md`'s
"HUMAN RULING" section, and in `BUILD_STATE.json`'s `decisions.accepted`
under `AC-13`) formally adopts the 17-item enumeration below as the
active **provisional** standard for P5 conformance work. **This ruling
does not permanently resolve the underlying disagreement** between the
Master Blueprint, the Verification Gates, and the Phase Manifest — none
of those documents were edited, renumbered, or rewritten to match this
one, and none should be assumed to agree with it merely because this
ruling exists. The ruling is narrowly about *which enumeration governs
implementation right now*, not about whether the documents will ever be
numerically reconciled.

This document exists because the AI Conductor governing documents contain
**three non-identical enumerations** of execution-adapter conformance
items, and no document currently reconciles them. It is a **mapping
artifact**, not a new architectural decision — it does not alter any of
the governing documents it describes, and it does not itself resolve the
underlying disagreement. See
`.ai-conductor-build/architecture_changes/AC-13-execution-conformance-enumeration-conflict.md`
for the original discovery record and reasoning; this document supersedes
AC-13's inline comparison table as the canonical reference for that table
specifically, while AC-13 remains the historical record of *why* this
document exists.

**Real-executor certification must not cite this document, or the
17-item enumeration it uses as its organizing axis, as proof that the
underlying numbering disagreement has been permanently resolved.** The
human ruling above authorizes continued *implementation* under this
enumeration; it does not settle the disagreement itself, which remains
open.

---

## Which enumeration this document uses as its organizing axis, and why

The 17-item enumeration from **Execution Adapter Contract v1.0 §14** is
used as the row axis below, for the same reasons given provisionally in
AC-13: it is the most granular (17 vs. Blueprint's 16 items), and it is
the dedicated normative document for exactly this boundary ("Normative
execution-boundary contract" per its own header). This is a **choice of
reference frame for this document's layout**, not a claim that the
Adapter Contract's text is more authoritative than the Blueprint's in any
general sense.

---

## Full mapping table

| Adapter Contract §14 (canonical axis) | Blueprint §10.7.6 | Verification Gates §66/67 | Phase Manifest P5 | Task Contracts P5 §39 | Relationship type |
|---|---|---|---|---|---|
| **EC-01** identity | ≈ EC-01 "identity/capability discovery" (**combined** with Adapter Contract's EC-02) | VG-P5-EXEC-01 "Adapter Contract" | P5-W02 (Capability Snapshot) | P5-W01-T01 "identity" | **One-to-many**: Blueprint's single EC-01 covers what the Adapter Contract splits into two (EC-01 + EC-02) |
| **EC-02** capabilities | ≈ EC-01 (see above — same Blueprint item, not a separate one) | VG-P5-EXEC-01, VG-P5-EXEC-07 | P5-W02 | P5-W01-T01 "capabilities" | Part of the one-to-many split above |
| **EC-03** workspace confinement | EC-02 "authorized workspace execution" **+** EC-03 "forbidden-path rejection" | VG-P5-EXEC-02 "Workspace Confinement" | P5-W03 | P5-W01-T01 "start" (partial) | **Many-to-one**: two Blueprint items fold into one Adapter Contract item. This folding rests on the interpretive claim (AC-13, not independently verifiable from text alone) that "confinement failing *is* a forbidden-path write" |
| **EC-04** successful execution | EC-04 "successful execution evidence" | VG-P5-EXEC-03 "Lifecycle Observation" (**groups EC-04..EC-10, seven items, into one VG gate**) | P5-W04 | P5-W01-T01 "observe" | One-to-one with Blueprint; many-to-one into VG |
| **EC-05** partial execution | EC-05 "partial execution evidence" | VG-P5-EXEC-03 | P5-W04 | — | One-to-one with Blueprint |
| **EC-06** non-zero exit | EC-06 "non-zero exit" | VG-P5-EXEC-03 | P5-W04 | — | One-to-one |
| **EC-07** timeout | EC-07 "timeout" | VG-P5-EXEC-03 | P5-W04 | — | One-to-one |
| **EC-08** cancellation | EC-08 "cancellation" | VG-P5-EXEC-03 | P5-W04 | P5-W01-T01 "cancel" | One-to-one |
| **EC-09** crash | EC-09 "crash / abrupt termination" | VG-P5-EXEC-03 | P5-W04 | — | One-to-one |
| **EC-10** Unknown outcome | EC-10 "ambiguous outcome → Unknown" | VG-P5-EXEC-03 **and** VG-P5-EXEC-04 "Unknown Safety" | P5-W04 | — | **One-to-many** into VG (feeds two gates) |
| **EC-11** restart/reconciliation | EC-11 "restart/reconciliation" | VG-P5-EXEC-08 "Restart/Reconciliation" | P5-W05 | — | One-to-one |
| **EC-12** evidence collection | **GENUINE MISMATCH** — Blueprint's EC-12 is *"provider failure propagation"*, a different requirement at the same number, not a naming variant of the same one | VG-P5-EXEC-09 "Evidence Provenance" (shared with EC-13) | P5-W02 (provenance/freshness fields) | P5-W01-T01 "collect_evidence" | **Genuine mismatch, explicitly flagged per this document's mandate.** See "EC-12 mismatch" section below |
| **EC-13** credential redaction | EC-13 "credential-redaction boundary" | VG-P5-EXEC-09 (**shared with EC-12** — one VG gate covers two distinct EC items) | — (Blueprint §12 Credential Vault, cross-cutting) | — | One-to-one with Blueprint; many-to-one into VG |
| **EC-14** capability denial | EC-14 "capability denial at tool boundary" | VG-P5-EXEC-07 "Capability Enforcement" | — (P9 scope, not P5) | — | One-to-one; correctly `NotApplicable` before P9 exists |
| **EC-15** no direct Succeeded | EC-15 "no direct Succeeded transition" | VG-P5-EXEC-05 "No Direct Succeeded" | Invariant 14 (cross-cutting) | — | One-to-one |
| **EC-16** no direct merge | EC-16 "no direct merge authority" | VG-P5-EXEC-06 "No Direct Merge" | Invariant 13 (cross-cutting) | — | One-to-one |
| **EC-17** executor replacement | **ABSENT.** No corresponding Blueprint §10.7.6 item exists at all | VG-P5-EXEC-10 "Executor Reversibility" | P5-W08 | — | **Genuine absence, explicitly flagged per this document's mandate.** See "EC-17 absence" section below |

---

## EC-12 mismatch, stated explicitly as required

Adapter Contract §14's EC-12 ("evidence collection") and Blueprint
§10.7.6's EC-12 ("provider failure propagation") are **not the same
requirement appearing twice with different words — they are two different
requirements that happen to share a number.** Reading each document's own
§9 (Adapter Contract) versus its own provider-failure discussion
(Blueprint), there is no textual basis for treating them as equivalent.

**Current implementation status (as of the conformance-foundation
correction pass that produced this document):** `conformance.rs`'s
`check_evidence_collection` function, which is tagged against
`ConformanceItem::Ec12EvidenceCollection`, checks (a) that `EvidenceRefs`'
four identity fields are populated, per the Adapter Contract's own literal
EC-12 scope, and (b) as a smallest-appropriate structural addition, that
whenever `Observation.final_executor_state` is a failure-shaped outcome
(`Failed`/`Crashed`/`TimedOut`/`Cancelled`/`Unknown`), a
`failure_classification` is actually present rather than silently absent
— this is the closest a pre-real-executor structural check can come to
Blueprint's "provider failure propagation" concern, using the
`FailureCategory` taxonomy the Adapter Contract itself already defines
(§13). **This does not fully satisfy Blueprint's EC-12** in the sense of
proving a provider failure was correctly classified and *propagated*
end-to-end to a human/mission-level consumer — that requires a real
provider failure and the not-yet-built P6+ provider-integration
machinery, and is not claimed here as proven.

## EC-17 absence, stated explicitly as required

Blueprint §10.7.6 has no sixteenth-or-seventeenth item corresponding to
"executor replacement" at all — its list simply ends at EC-16. This is the
single strongest piece of evidence for the provisional choice to use the
Adapter Contract's 17-item list as the organizing axis: dropping EC-17
would leave Phase Manifest P5-W08 ("Executor Selection and
Reversibility") and Adapter Contract §18 (the Replacement Principle) with
no corresponding conformance item under a Blueprint-only numbering.

---

## What this document does NOT do

- It does not edit the Master Blueprint, the Execution Adapter Contract,
  the Verification Gates, the Phase Manifest, or the Task Contracts. All
  five remain exactly as authored, and the human ruling did not change
  that — none of them were renumbered or rewritten to agree with this
  document.
- It does not declare the underlying multi-document numbering
  disagreement permanently resolved. The human ruling ratifies which
  enumeration governs *implementation right now*; it does not settle
  whether the documents will ever be numerically reconciled, which
  remains genuinely open.
- It does not claim any real-executor conformance result. It is a
  labeling/cross-reference document only.
- It does not need to be re-litigated every time `conformance.rs` is
  touched — that module's own doc comments link here rather than
  re-deriving this table, so this is the single place the mapping is
  maintained.

## When this document should be updated

- **Already occurred once:** the human ruled on AC-13 (adopted the
  17-item enumeration as the active provisional standard; did not
  request the governing documents be edited to agree). This document was
  updated accordingly at that time. If the human later rules
  *differently* — requests a different enumeration, or requests the
  governing documents themselves be edited to agree — this document is
  updated again to reflect that new ruling.
- If any of the five governing documents is revised in a way that changes
  its own EC/VG/work-item numbering.
- If a genuine mismatch or absence beyond the two already identified
  (EC-12, EC-17) is discovered.
