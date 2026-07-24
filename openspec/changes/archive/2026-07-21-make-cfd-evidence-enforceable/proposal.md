# Make the CFD evidence layer enforceable

## Why

The pre-certification audit ([`openspec/notes/cfd_audit/AUDIT-REPORT.md`](../../notes/cfd_audit/AUDIT-REPORT.md))
found the `deep_causality_cfd` numerical core sound — Rankine–Hugoniot exact, the Ghia lid-cavity primary
vortex matched to four decimal places, Sod against the exact Riemann solution — but the **assurance layer**
unable to support it. Two facts decide this change:

- **No CI job, script, or Bazel target executes any of the 13 `verification/` binaries.** `cargo test`
  compiles them and never runs them, so every quantitative accuracy claim in the crate README and
  `verification/README.md` is unenforced and can silently rot.
- **72 of 294 verified findings are tautology or circular-reasoning defects.** Several headline gates are
  algebraically incapable of failing; for the QTT immersed cylinder, *no* gate constrains drag correctness.

The risk this creates is not that an engineer gets a wrong number from the marchers — on every closed-form
reference available they get the right one. The risk is that an engineer reads "all gates passed" and
concludes something was checked that was not. This change closes that gap and is the highest
value-per-effort item in the audit's path to certification.

## What Changes

Phase 1 of the audit's remediation path. **Scope is the evidence layer only** — `verification/`,
`studies/`, `examples/`, `tests/` and CI. **No solver source changes and no physics fixes**; those are
Phase 2 and are deliberately excluded so this change stays reviewable and its gates stay interpretable.

- **CI executes the verification suite.** The 10 fast harnesses (≤5 s each) run on every PR; the three slow
  ones (`dec_cylinder` ~510 s, `dec_cylinder_wake` ~155 s, `dec_lid_cavity`) run nightly. They already
  self-verify and exit nonzero by design, so this is wiring, not new gate logic. Resolves audit blocker **B-2**.
- **`dec_cylinder_verification` gets a real gate.** It currently holds zero `process::exit` calls and zero
  assertions; on a solver `Err` it prints, breaks the march, then reports Strouhal and drag from the
  *truncated* series and returns 0 — the opposite of the convention `verification/README.md` advertises.
  Gate St and C_d against the cited Williamson / Dröge–Verstappen bands; exit nonzero on solver error.
  Resolves audit blocker **B-3**.
- **Every gate that cannot fail is repaired or removed.** Confirmed instances: `qtt_park2t_blackout` gates
  (ii) and (iv); the `qtt_taylor_green` convection-amplitude gate; the weather example's "(0) table
  integrity" gate; the corridor "(4f) fine sweep refines the coarse winner" gate; `qtt_rank_dynamic` G2;
  the compressible MMS `continuity_error` gate; and the energy-budget CI gate.
  **BREAKING (spec-level):** `park2t-blackout-validation` currently *mandates* one of these tautologies —
  it requires gate (ii) to assert "exactness of the closed-form exponential … equality to round-off",
  which is a transcription of `ler_step`'s own body. That requirement is replaced, not merely re-implemented.
- **The QTT cylinder gate set is re-founded so something constrains drag.** Measured: `SMOOTH_CELLS = 2.0`
  moves C_d **6.1×** (7.70 → 12.33 → 23.76 → 35.81 → 47.27 across 0.5→4 cells) while the no-slip gate is
  *provably invariant* over that range; C_d is **non-monotone** in `ETA` (17.4 → 26.2 → 21.4) with no η→0
  limit; and `CONVERGENCE_BOUND = 0.10` sits eleven orders above the measured 1.89e-11 difference. Add η and
  smoothing-width ladders as first-class gates and tighten the convergence bound to the phenomenon's scale.
- **Every gate bound is classified by evidence class.** Each bound is labelled either a **reference gate**
  (analytic or published external value) or a **pinned regression tripwire** (a bound derived from this
  code's own prior output), in code and in the printed output. Some harnesses already disclose this in prose
  — `qtt_ramc_stagline` says "band ±0.70 dec, pinned from the measurement" — but it is neither systematic nor
  machine-visible, so a tripwire currently reads as validation against flight data.
- **The truncated lid-cavity `baseline.txt` is regenerated.** It is 11 lines, stops at t=44.99 of 14223
  steps, and carries no RMSE, no vortex table and no verdict — the only truncated baseline of the 12.

Explicitly **not** in scope: `REDUCED_MASS_AMU` (**B-1**), the `BlendedMap` fold check (**B-4**), the ESKF
defects, the Brinkman η/resolution envelope, and `wall_heat_flux`. Those are Phase 2. Where this change
touches a harness whose *physics* is under Phase 2 review, it gates the quantity as-is and labels the bound
a tripwire rather than pre-empting the physics decision.

## Capabilities

### New Capabilities

- `verification-gate-integrity`: the cross-cutting contract the evidence layer must satisfy — every shipped
  gate is falsifiable and carries a documented breaking input; every bound declares its evidence class
  (reference gate vs pinned regression tripwire) in code and in printed output; and the verification suite
  is executed by CI on a defined cadence with a nonzero exit failing the build. This capability owns the
  gate repairs in `studies/` and `examples/` that no existing spec mandates.

### Modified Capabilities

- `dec-ns-validation`: `dec_cylinder_verification` moves from "reports St and C_d" to "gates St and C_d
  against the cited reference bands and exits nonzero on solver error"; the lid-cavity baseline artifact
  must be a complete run carrying RMSE, the vortex table and a verdict.
- `park2t-blackout-validation`: LER acceptance gates (ii) and (iv) are replaced. The present requirement
  mandates asserting an identity of the implementation against itself; it becomes a falsifiable check of
  the relaxation against an independently-derived reference.
- `qtt-immersed-body`: the immersed-cylinder validation gains η and smoothing-width ladders as first-class
  gates, and the bond-convergence bound is tightened to the scale of the phenomenon it measures, so at
  least one gate constrains the reported drag.

## Impact

**Code**
- `.github/workflows/` — new/extended job running the verification binaries (PR + nightly cadence).
- `deep_causality_cfd/verification/` — `dec_cylinder_verification` (gates + exit path),
  `qtt_cylinder_verification` (η/smoothing ladders, tightened bound), `qtt_park2t_blackout` (gates ii, iv),
  `qtt_taylor_green_verification` (convection amplitude), `dec_lid_cavity_re1000_verification/baseline.txt`.
- `deep_causality_cfd/studies/qtt_rank_dynamic` — gate G2.
- `deep_causality_cfd/src/types/flow/mms.rs` and `deep_causality_cfd/tests/solvers/dec/energy_budget_tests.rs`
  — the MMS `continuity_error` and energy-budget gates. *(Test/gate logic only; the marchers are untouched.)*
- `examples/avionics_examples/cfd/plasma_blackout/{weather,corridor}/model.rs` — two example gates.
- `deep_causality_cfd/verification/README.md` — evidence-class labelling for every documented bound.

**CI and runtime**
- PR wall-clock grows by the fast suite (~20 s total, 10 harnesses). The nightly job adds ~12 min,
  dominated by `dec_cylinder`.
- Some gates will fail on first wiring — that is the intended outcome, not a regression. Any harness whose
  measured value sits outside a corrected bound is triaged against the audit's per-finding evidence in
  [`ACTION-LIST.md`](../../notes/cfd_audit/ACTION-LIST.md) before the bound is moved.

**Risk**
- Tightening `CONVERGENCE_BOUND` and adding η/smoothing ladders may surface that the QTT cylinder drag has
  no converged value. That is a real finding, not a blocker for this change: the gate set must be able to
  say so, and Phase 2 resolves the envelope.
- No public API changes. No solver behaviour changes. `deep_causality_cfd` is `publish = false`, so there is
  no downstream release impact.
