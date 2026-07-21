## Context

The pre-certification audit established that `deep_causality_cfd`'s numerical core is sound but its
assurance layer cannot carry it. Two structural facts drive this design:

1. **The evidence is never executed.** `.github/workflows/run_tests.yml` runs `cargo build`,
   `cargo test --doc` and `cargo test` on pull requests. `cargo test` *compiles* the 13 `verification/`
   binaries — they are declared as `[[example]]` targets in `deep_causality_cfd/Cargo.toml` — but never
   runs them. There is no nightly workflow at all, and no Bazel target covers them either.
2. **Gate quality is uneven and unlabelled.** Gates in the suite currently mix three kinds: comparisons
   against analytic solutions (`qtt_sod` vs the exact Riemann solution — genuinely strong), comparisons
   against published tables (Ghia, Williamson), and bounds pinned from this code's own prior output. All
   three print through the same `[PASS]`/`[FAIL]` block, so a reader cannot tell them apart. Some
   harnesses already disclose their pinning in prose — `qtt_ramc_stagline` says "pinned from the
   measurement" in the gate text itself — which is the honesty this design generalizes rather than
   invents.

The harnesses already follow a consistent house style: a `main.rs` orchestrating the run, a `print_utils.rs`
rendering and verifying, a `config.rs` holding pinned constants, and `std::process::exit(1)` on a broken
gate. That existing convention is the seam this change builds on — the work is mostly wiring and
relabelling, not new architecture.

A relevant constraint: `Gates` (`src/types/flow/gates.rs`) exists as a public type documented as "the
`[PASS]`/`[FAIL]` block every self-verifying program prints", but **no** program uses it — every harness
goes through `Verdict`'s `Display` impl instead. Resolving that duplication is Phase 3 and an owner
decision; this design must not depend on either outcome.

## Goals / Non-Goals

**Goals:**

- Every gate in `verification/`, `studies/` and the CFD examples can demonstrably fail, with its breaking
  condition recorded next to it.
- Every numeric bound declares whether it is a **reference** gate or a **tripwire**, visibly in output.
- CI executes the verification suite and fails on a non-zero exit, split fast/PR and slow/nightly.
- The QTT cylinder harness gains gates that respond to the parameters that actually move its answer.
- The truncated lid-cavity baseline is replaced with a complete run.

**Non-Goals:**

- **No solver source changes.** `src/solvers/`, `src/theories/`, `src/coordinate/` and `src/navigation/`
  are untouched except where a *gate* lives in them (`src/types/flow/mms.rs`).
- **No physics fixes.** `REDUCED_MASS_AMU` (B-1), the `BlendedMap` fold check (B-4), the ESKF `Q`/`dt` and
  innovation-covariance defects, the Brinkman η envelope, and `wall_heat_flux` are Phase 2. This change
  makes the harnesses *able to detect* such conditions; it does not resolve them.
- **No re-baselining of physics results to make gates pass.** If a corrected gate fails, that is the
  finding.
- **Not resolving `Gates` vs `Verdict`.** Phase 3.

## Decisions

### D1 — Classify evidence with a two-value label, not a richer taxonomy

Each bound is `reference` or `tripwire`. Nothing else.

*Why:* the distinction that matters for certification is exactly one bit — *does this bound encode
knowledge from outside this codebase, or not?* An auditor reading "all gates passed" needs to know which
gates could have caught a physics regression versus which only catch drift from a previous run.

*Alternatives considered.* A four-tier scheme mirroring the verification README's existing prose
("analytic / flight-data order-of-magnitude / structural rank-lever / internal invariant") was rejected:
those tiers describe *what a harness verifies*, which is already documented per-harness, and conflating
them with *where the bound came from* is what produced the current ambiguity. Analytic and published
bounds both collapse to `reference` because both are falsifiable against something external.

### D2 — Carry the label in the gate call site, not in a side table

The label is a parameter at the point the gate is constructed, so it cannot drift from the bound. A
registry mapping gate names to classes would desynchronize the first time a gate is renamed.

Since harnesses render through `Verdict`/`GateSeq` rather than a shared gate type, the minimum viable form
is a convention on the gate label string plus the printed line — e.g. a `[reference]`/`[tripwire]` marker
emitted next to `[PASS]`/`[FAIL]`. This keeps the change inside the harnesses and avoids introducing a new
shared type while `Gates` is unresolved (see Context).

### D3 — Record each gate's breaking condition as a comment, and test it where cheap

Every gate carries a comment naming an input or mutation that makes it fail. For gates where constructing
that input is cheap, add a unit test asserting the gate predicate returns false for it.

*Why:* a comment is the cheapest durable defence against re-introducing a tautology, and it is what the
audit had to reconstruct by hand for each of the seven unfalsifiable gates. Mutation-testing the whole
suite would be stronger but is disproportionate for Phase 1.

*Alternatives considered.* Full mutation testing (`cargo-mutants`) was rejected on runtime and because the
suite must first *have* falsifiable gates before mutation coverage is meaningful — it is a natural Phase 4
follow-up.

### D4 — Two CI cadences, driven by an explicit list

A `verification` job on pull requests runs the fast harnesses; a scheduled nightly workflow runs
`dec_cylinder`, `dec_cylinder_wake` and `dec_lid_cavity`. The harness list is explicit in the workflow, and
a check asserts the union of both lists equals the `[[example]]` verification targets declared in
`Cargo.toml`, so a newly added harness cannot silently escape CI (a spec scenario).

*Why explicit over glob:* the fast/slow split is a runtime property that cannot be derived from the file
tree, and an explicit list makes the omission of a harness reviewable in the diff. The completeness check
recovers the safety a glob would have given.

*Why a separate nightly workflow:* the repo has no scheduled workflow today; adding `schedule` to
`run_tests.yml` (currently `on: [pull_request]`) would run the whole test suite nightly for no benefit.

*Cost:* PR wall-clock grows ~20 s for ten harnesses, against an existing full-workspace `cargo test`.
Nightly adds ~12 min, dominated by `dec_cylinder` at ~510 s.

### D5 — Repair tautological gates in place; delete only when nothing meaningful remains

Preference order: (a) make the predicate read the quantity under test rather than its reference —
e.g. the `qtt_taylor_green` amplitude gate computes `amp` from the analytic form and never touches the
solver output `cs[]`, so it becomes `amp` over `cs[]`; (b) replace with an independent reference —
e.g. `qtt_park2t_blackout` gate (ii) compares `ler_step` against a converged sub-stepped integration
instead of a transcription of its own body; (c) delete, and correct the advertised gate count.

*Why deletion is last:* several of these gates name a property worth checking; only the *implementation*
was circular. The weather "table integrity" gate is the likely deletion candidate — if `errored` is never
populated from a real per-row outcome, there is no check to recover.

*Note on scope interaction:* `qtt_park2t_blackout` gate (ii) is not merely a coding error — the
`park2t-blackout-validation` spec **mandates** "equality to round-off" against the closed form. The spec
delta must land with the code change, which is why that capability is a MODIFIED spec rather than a
straight fix.

### D6 — Where the physics is under Phase 2 review, gate the quantity as-is and label it `tripwire`

The QTT cylinder drag has no demonstrated η → 0 limit, so no `reference` bound can honestly be written for
it now. The design's answer is to add the η and smoothing ladders as gates that **report and gate the
trend**, and permit an explicit "not converging" outcome rather than forcing a pass/fail on an absolute
value.

*Why:* this is what lets Phase 1 complete without pre-empting Phase 2. A harness that can say "this
quantity does not converge under the parameter that dominates it" is strictly more useful than one that
passes silently — and that statement is itself the audit finding.

### D7 — Regenerate the cavity baseline at the configuration the documentation reports

The committed baseline is a 65² run truncated at t=44.99 of 14223 steps, while `verification/README.md`
reports 33²/t=40 figures. Regeneration fixes the truncation *and* the configuration mismatch, so the
artifact and the documented numbers describe the same run.

*Open sub-decision:* whether the README should instead report the 65² default. The audit found the default
65² result markedly better than the documented 33² row (primary vortex matching Ghia to 1e-4, RMSE 0.0617
vs 0.137). That is a documentation-accuracy question belonging to Phase 3; this change only ensures the
baseline artifact is internally consistent.

## Risks / Trade-offs

- **Gates will fail on first wiring.** → Intended. Triage each failure against the per-finding evidence in
  `openspec/notes/cfd_audit/ACTION-LIST.md` before moving any bound. Moving a bound to restore a pass,
  without evidence, reintroduces exactly the back-fitting this change exists to remove.
- **Tightening `CONVERGENCE_BOUND` and adding ladders may show the QTT cylinder drag has no converged
  value.** → Accepted, and handled by D6: report non-convergence as the result. Do not widen the bound.
- **A repaired `qtt_park2t_blackout` gate (ii) may fail** if the closed form and a sub-stepped integration
  disagree beyond tolerance. → That would be a genuine finding about the relaxation kernel, escalated to
  Phase 2 rather than absorbed by loosening the tolerance.
- **Nightly CI cost (~12 min) and flake exposure on long runs.** → Nightly-only, reported separately from
  the PR suite so a slow-harness flake never blocks a merge.
- **Label drift** — a bound reclassified without updating its marker. → D2 keeps the label at the call
  site; a doc test or review checklist item on `verification/README.md` keeps the table aligned.
- **Scope creep into Phase 2.** → The Non-Goals list is the control. Any change under `src/solvers/`,
  `src/navigation/` or `src/coordinate/` in this change's diff is out of scope by construction, with the
  single exception of gate logic in `src/types/flow/mms.rs`.

## Migration Plan

No runtime migration: `deep_causality_cfd` is `publish = false`, no public API changes, no solver
behaviour changes.

Sequencing matters for reviewability:

1. **Wire CI first, against the suite as it stands.** This establishes the current baseline — which
   harnesses pass today — before any gate is touched, so subsequent failures are attributable.
2. **Repair gates, one harness per commit.** Each commit carries the gate change, its breaking-condition
   comment, and any spec delta it depends on.
3. **Add evidence-class labels** across the suite and reconcile `verification/README.md`.
4. **Re-found the QTT cylinder gate set** last, since it is the only item likely to surface a
   non-convergence result needing discussion.

Rollback is per-commit; the CI job can be made non-blocking by moving it to the nightly workflow if the PR
suite proves too slow, without reverting any gate repair.

## Open Questions

- **Does the repaired `qtt_park2t_blackout` gate (ii) pass?** Unknown until the sub-stepped reference is
  implemented. If it fails, the finding escalates to Phase 2.
- **Does the QTT cylinder drag converge under any η the harness can afford?** The measured ladder is still
  drifting at η = 0.008. If no affordable η converges, the gate records non-convergence (D6) and Phase 2
  owns the resolution.
- **Should the cavity README row report the 65² default instead of 33²?** Phase 3 documentation question,
  flagged here because D7 touches the same artifact.
- **Which of the seven unfalsifiable gates end in deletion rather than repair?** Decided per-gate during
  implementation using D5's preference order; the weather "table integrity" gate is the likely candidate.
