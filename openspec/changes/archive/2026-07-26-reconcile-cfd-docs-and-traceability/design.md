## Context

This is the last change in the `deep_causality_cfd` audit. Phases 1 and 2 made the evidence bite and
closed the physics defects; the numerics are confirmed right against every closed-form reference the
audit could apply. What remains is the assurance the crate offers a reader: its prose, its load-bearing
constants, and its load-bearing tests. `AUDIT-REPORT.md` §9 items 16–24, systemic themes §7.

The audit's two largest themes by count survive here: 87 doc-overclaims and 39 doc-gaps. Both are the
same failure in opposite directions — prose that says more than the code does, and code that does more
than the prose says. Phase 3 is bidirectional truth-up across both. Phase 4 (traceability) is the
adjacent theme: a load-bearing constant with no source, and a test that pins the code's own prior output
instead of an independent reference.

Ground state, verified against the tree before planning (the §5c lesson: a claim about the tree is only
as good as the search behind it):

| Item | State | Anchor |
|---|---|---|
| 16 — doc-overclaims | OPEN, ~86 actionable rows; a minority closed by Phases 1–2 | `ACTION-LIST.md` (86 `doc-overclaim` rows); `blended.rs` fold, `boundary_zone` hook, `penalization_heat_integral` closed |
| 17 — DEC kernel + spectral comments | OPEN | `dec_ns_rate.rs:7,35` vs code `:621-652`; `projection.rs:157,169` vs `:158-163`; `mod.rs:21-26` vs `step.rs:6-13` |
| 18 — doc-gaps | OPEN; types exist with rustdoc, absent from README | `DuctMarchRun` `duct_march_run.rs:56`; `IgnitionCorridor` `throttle_guidance.rs:107`; `AcousticCoreInverse` `acoustic_inverse.rs:52`; resume `state_snapshot.rs` |
| 19 — RAM-C consistency | **DONE** (Phase 1) | `README.md:224`, `verification/README.md:127` both order-of-magnitude |
| 20 — lid-cavity row | **DONE** (Phase 1) | `verification/README.md:90,213` report the 65² default, RMSE 0.0617 |
| 21 — convergence qualification | OPEN | "clean 2nd-order" unqualified `qtt_taylor_green_verification/README.md:52`; temporal floor undocumented |
| 22 — `Gates` | OPEN, decision item | `Gates::new` only in `gates_tests.rs`; live code uses `GateSeq`/`Verdict`; sole `println!` in `src/`; empty `finish()`→true |
| 23 — constant provenance | PARTIAL; cylinder `ETA` DONE | `SMOOTH_CELLS=2.0` (2 sites, no source); park2t `ETA=0.016` no source; Mach-1.05 no justification; no penalization PDF |
| 24 — change-detector / QTT convection test | OPEN, needs new test | no test drives `rate_pair` with `u,v≠0`; TG harness re-assembles via `gradient_*` not `rate_pair` |

Items 19, 20, and the cylinder site of 23 are already delivered. They are re-verified here and captured
in the spec so they cannot silently rot, but they are not re-implemented.

## Goals / Non-Goals

**Goals:**

- Every doc-overclaim in the ACTION-LIST catalogue reconciled: the prose describes the code, or marks
  intent as intent.
- Every kernel docstring and comment names the operator the code marches; no comment contradicts the code
  beneath it.
- Every load-bearing public capability documented where a user looks for it.
- Every convergence claim states the order it holds in and the regime it holds over.
- Every load-bearing constant carries a source, its units, and a `papers/` entry where a publication
  backs it.
- One load-bearing test references an independent truth: the shipped QTT convection path exercised with
  `u,v ≠ 0` against an analytic reference.
- `Gates` resolved per the owner's decision.

**Non-Goals:**

- **Any change to a marcher, kernel, or gate bound.** The behaviour is confirmed correct; this change
  corrects the prose, constants, and tests that describe it. `Gates::finish()` refusing an empty set and
  the new convection test are the only code touched, and neither changes a marched number.
- **The physics-math and magic-number *value* corrections.** Those were Phase 2 (`REDUCED_MASS_AMU`,
  the pressure floor, the mask clamp). This change traces the *provenance* of constants whose values
  stand, not their values.
- **Replacing every `[tripwire]` gate with a reference gate.** A back-fitted bound is legitimate when
  disclosed and labelled (§7); Phase 1 labelled them. This change converts a tripwire to a reference gate
  only where an independent reference exists — the QTT convection path is the concrete one — and leaves
  the honestly-labelled regression tripwires as they are.
- **The tautology-circular gate theme.** Owned by `verification-gate-integrity` (Phase 1); not reopened.

## Decisions

### D1 — Two cross-cutting capabilities, not edits threaded through every behavioural spec

The doc-overclaims sit across many capabilities — `dec-ns-rate`, `qtt-surface-observables`,
`boundary-zone-abstraction`, and more — whose behaviour is confirmed correct and unchanged. Two new
cross-cutting capabilities own the discipline: `documentation-code-parity` (Phase 3) and
`constant-and-test-traceability` (Phase 4).

*Why:* it is the pattern Phase 1 used. `verification-gate-integrity` owns the gate repairs across
`studies/` and `examples/` that no behavioural spec mandates, rather than editing each behavioural spec to
say "and its gate must bite". Prose parity and constant provenance are the same shape of cross-cutting
contract. Threading a prose edit through `dec-ns-rate`'s behavioural spec would imply the behaviour
changed; it does not.

### D2 — Work the catalogue, record the true count; disprove each claim before rewriting it

The unit of work is the ACTION-LIST row, not the audit's headline number. "87" is 86 actionable rows and
91 raw occurrences; "39" is the doc-gap category total, a mix of README sections and docstring gaps. The
change reconciles the catalogue and records the count it actually closed.

*Why:* the §5c lessons apply directly. Lesson 1 — a "by construction" claim may be false, not merely
unchecked (B-4 was: 275 configurations folded). So each overclaim is checked against the code before its
prose is rewritten. If a claim is false *and* the property it asserts is unenforced, that is a
Phase-2-class correctness finding, not a doc edit — it is escalated and recorded, not papered over by
softening the prose. Rewriting prose to match broken code would bury a defect under honest-sounding
words.

### D3 — Truth-up is bidirectional

Item 16 corrects prose that claims more than the code delivers. Item 18 documents code that delivers more
than the prose admits. Both are in scope and both are "make the document tell the truth about the code".

*Why:* the audit's own Phase 3 title is "Documentation truth-up (bidirectional)". A doc-gap
(`AcousticCoreInverse` shipping unmentioned in the README) misleads as surely as an overclaim — a reader
concludes the capability is absent. Ship-more-than-you-say is the safer direction but still a parity
defect.

### D4 — `Gates` is the owner's decision; the default path deletes nothing

Item 22 is a decision, not a mechanical fix, and one arm of it — retirement — is a deletion. The golden
rule holds: no deletion without owner approval.

The options:

**(a) Adopt `Gates`** across the self-verifying programs so the README's claim becomes true and the type
earns its export. Cost: every program migrates from `GateSeq`/`Verdict` to `Gates`, and `Gates` gains the
`EvidenceClass` labelling Phase 1 built into the live path — otherwise adoption would regress the
evidence-class discipline.

**(b) Retire `Gates`** — remove the dead type and its five `println!`, the sole `println!` in `src/`.
Cost: a deletion, so owner approval; and the README/`flight_envelope_placard` prose that names `Gates`
must move to `GateSeq`.

Independent of (a)/(b), and taken now: correct the README to name the type the programs use, and make
`Gates::finish()` refuse an empty gate set (it returns success for one today — a gate harness that
registered nothing should not report pass). These are non-deleting and true under either resolution.

*Recommendation:* retire. `GateSeq` is what the crate uses, carries the evidence-class labelling, and is
the live contract; `Gates` is a parallel API that has never been exercised outside its own test. But that
is a deletion, so it waits on the owner. The change lands the non-deleting corrections and records the
decision as open.

### D5 — Constant provenance follows the crate's paper convention; orphans are indexed, not deleted

A load-bearing constant gets a source, its units, and — where a publication fixes it — the PDF in
`papers/` with a full author-year citation at the definition, the convention the crate already follows
for Kirkpatrick (2003) and Dröge & Verstappen (2005). The penalization method the immersed-body harness
rests on gets Angot / Bruneau & Fabrie (1999) added to `papers/` and cited from the harness.

Two PDFs in `papers/` — `mittal2005.pdf`, `mohamed2016.pdf` — are cited by author name nowhere in the
crate. They are either the sources for constants this change is tracing (in which case they get cited) or
orphans. Removing an orphan PDF is a deletion and waits on the owner; the default is to index `papers/`
(a short README mapping each PDF to what cites it) so an orphan is visible rather than silently carried.

### D6 — The test references an independent truth, through the public API, and the harness calls the shipped path

Item 24 has two halves. The new test drives `rate_pair` with `u,v ≠ 0` against an analytic convection
reference chosen so the projection does not annihilate it — the reason the existing tests are blind is
that Taylor–Green's convection is a pure gradient the projection removes. The harness re-route points the
Taylor–Green convection check at the shipped `rate_pair` instead of a `gradient_x`/`gradient_y`
re-assembly.

*Why:* the standing rule from §5c — no test may assert on source text; behavioural coverage through the
public API only, verified under `bazel test //...` (the sandbox that caught the `include_str!` regression
`cargo test` hid). And the gate-BM-A lesson — a gate that re-implements the code it checks verifies the
copy, not the code; the harness must call the shipped convection, not a look-alike.

### D7 — Run the adversarial pass over the finished diff

Every Phase-2 change found its own overclaims when a refute-by-default review was run over the completed
diff — a correctness gap in one, a cluster of doc/spec overclaims in each. A documentation truth-up change
is the most exposed to this failure of all: its deliverable *is* prose, and prose claiming "all
reconciled" is itself an overclaim if a catalogue row was missed. The finished diff gets the same
multi-dimension refute-by-default review, and its residue is recorded honestly — "N reconciled, M
deferred", never "clean".

## Risks / Trade-offs

- **The change's own prose overclaims.** → D7. The one deliverable most able to claim more than it did is
  a truth-up change. The count is recorded from the catalogue, not asserted; the adversarial pass is
  mandatory; a deferred tail is stated, not hidden.
- **Reconciling an overclaim surfaces a real defect.** → D2. If a "by construction" claim is false and
  unenforced, it is escalated as a correctness finding, not softened into honest-sounding prose. This is
  the good outcome — it is what caught B-4 — but it means the change may spawn a small Phase-2-style
  follow-up rather than close cleanly.
- **High count, low individual risk, easy to leave a tail.** → Work the catalogue row by row and record
  the closed count against it; the completeness is checkable against the ACTION-LIST, not against memory.
- **The analytic reference for `rate_pair` is annihilated by the projection.** → D6. The reference is
  chosen pre-projection (a convection field whose gradient is not pure), which is the specific reason the
  existing tests miss the shipped path.
- **`Gates` and orphan-PDF deletions.** → D4, D5. Deletions wait on the owner; the change lands only the
  non-deleting corrections unless directed otherwise.

## Migration Plan

No runtime migration; `publish = false`, no downstream consumers, no marched number moves.

1. **Doc-overclaim reconciliation** (items 16, 17, 21) — independent prose edits, each revertible; work
   the ACTION-LIST catalogue, escalating any false-and-unenforced claim per D2.
2. **Doc-gap closure** (item 18) — document the undocumented capabilities in the README.
3. **Constant provenance** (item 23) — sources, units, the penalization PDF, the `papers/` index.
4. **The convection test and harness re-route** (item 24) — the only behavioural work; verified under
   `bazel test //...`.
5. **`Gates`** (item 22) — land the non-deleting corrections; surface the adopt/retire decision to the
   owner.
6. **Re-verify** items 19, 20, and the cylinder `ETA` are still correct (Phases 1–2 delivered them).
7. **Adversarial pass** over the finished diff (D7); record the residue.

## Open Questions

- **Adopt or retire `Gates`?** Owner decision (D4). Recommendation: retire, since `GateSeq` is the live
  contract — but retirement is a deletion.
- **The orphan PDFs (`mittal2005`, `mohamed2016`) — cite, index, or remove?** Owner decision (D5).
  Default: index; removal is a deletion.
- **How many `[tripwire]` gates have an independent reference available to convert?** Unknown until the
  gate inventory is surveyed against available references. The QTT convection path is the one this change
  commits to; others are recorded, not promised.
- **Does reconciling the doc-overclaim catalogue surface a false-and-unenforced claim** (a B-4 repeat)?
  Unknown until each is checked against the code (D2). If it does, it becomes a follow-up correctness
  change, not a silent prose softening.
