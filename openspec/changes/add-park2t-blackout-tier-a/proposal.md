<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Why

Gap 1 of the [plasma-blackout corridor](../../notes/plasma-blackout/plasma-blackout-corridor.md) is closed:
`deep_causality_cfd` has a verified QTT incompressible flowfield with an immersed body and neutral surface
observables. But the flagship's *regime driver* — vibrational–electron nonequilibrium → ionization → electron
density → plasma frequency → comms/GNSS blackout — does not exist yet (Gap 2,
[gap-analysis §4](../../notes/plasma-blackout/gap-analysis.md)). Without it the corridor is a flowfield with no
plasma, so the counterfactual "does this trajectory black out?" question cannot be answered at all.

This change builds and **verifies against published reference results** the buildable **Tier-A** slice of Gap 2
([gap-2 note §6 steps 1–3](../../notes/plasma-blackout/gap-2/gap-two-reacting-plasma.md)): the Park-2T reacting
kernels and the blackout closure, riding the **existing incompressible** QTT rollout. The full compressible
shock-capturing marcher (Tier-B) remains open research and is **explicitly out of scope** — named and deferred,
not attempted.

The Tier-A slice is only buildable because three otherwise-fatal contradictions (stiff chemistry on an explicit
stage; a driving temperature that cannot emerge from an incompressible flow; nonequilibrium lag without a stiff
network) are resolved by one mechanism — the **Lagging-Equilibrium Relaxation (LER) stage**
([gap-2 §1.4](../../notes/plasma-blackout/gap-2/gap-two-reacting-plasma.md) + the three resolution notes). This
change makes that mechanism real, and proves it.

## What Changes

- **New pointwise Park-2T kernels** in `deep_causality_physics/src/kernels/hypersonic/` — vibrational
  relaxation (Landau–Teller / Millikan–White), Arrhenius rate, ionization fraction (Saha + rate-based),
  Rankine–Hugoniot normal-shock temperature jump, recovery-temperature reconstruction, and the Tier-A
  ionization surrogate — each a free `fn<R: RealField>(…) -> Result<Quantity<R>, PhysicsError>` with a
  `PropagatingEffect` wrapper. Plasma frequency reuses `mhd/plasma.rs`.
- **New quantity newtypes** in `deep_causality_physics`: `ElectronDensity`, `IonizationFraction`,
  `ElectronTemperature`, `VibrationalTemperature`, `MassFraction`, `ReactionRate` (reuse `PlasmaFrequency`,
  `DebyeLength`).
- **A new kernel contract for stiff sources** — the relaxation kernels return the **integrated increment over
  `Δt`** (a closed-form exponential / linearly-implicit step), not a rate. This is the LER mechanism; the
  marcher and the `PhysicsStage` seam are untouched.
- **New CFD coupling stages** in `deep_causality_cfd`: `IonizationStage` and `EosStage` as LER stages reading a
  **state-derived** `T_tr`, and a `BlackoutTrigger` (`n_e → plasma frequency → comms-band compare → GNSS-denied
  flag`) on the `CausalFlow`/`bind_or_error` seam — driving the kernels over the temperature/species scalar
  fields of the existing QTT rollout via `advance_scalar`.
- **New blackout observables** (`n_e`, plasma frequency, blackout dwell) emitted by the QTT march run.
- **A new self-verifying verification example** under `deep_causality_cfd/verification/` (config/main/print_utils
  /baseline/README style, mirroring `qtt_taylor_green` / `qtt_cylinder`) that gates exit-nonzero on the LER
  verification criteria and reports the published reference cross-references with honest disclaimers.
- The **dynamic-by-construction invariant** is binding: every temperature/density/fraction/frequency is computed
  from state or supplied as config; only constants of nature and cited model coefficients are literal, in
  `constants/`, lifted via `R::from_f64`.
- **Explicitly NOT in this change — specified in the sibling Tier-B change
  [`add-cfd-compressible-qtt-marcher`](../add-cfd-compressible-qtt-marcher/proposal.md):** the compressible
  shock-capturing QTT marcher, density/energy transport, the reacting `*_rhs` family on a compressible solver,
  multi-mode relaxation spectra, and shock-rank control. That change **reuses every kernel, newtype, and LER
  stage built here, unchanged** — Tier-A is stage 1 of the Gap-2 program, Tier-B raises the flowfield fidelity
  on the same physics layer.

## Capabilities

### New Capabilities

- `park2t-ionization-kernels`: the pure pointwise hypersonic Park-2T kernels and their quantity newtypes in
  `deep_causality_physics` (vibrational relaxation, Arrhenius rate, ionization/Saha, Rankine–Hugoniot jump,
  recovery temperature, the Tier-A surrogate, plasma-frequency reuse), each with a `PropagatingEffect` wrapper —
  validated **pointwise, in isolation**, against Park tables / RAM-C / the Saha limit before any solver use.
- `lagging-equilibrium-relaxation`: the LER stage mechanism — the kernel contract that returns the
  closed-form integrated increment over `Δt`, and the between-step `PhysicsStage` that relaxes a carried scalar
  toward a state-derived equilibrium target. Unconditionally stable under stiffness; first-order Lie split
  (Strang noted as the timing upgrade).
- `plasma-blackout-trigger`: the CFD wiring — the state-derived recovery-temperature reconstruction, the
  `IonizationStage`/`EosStage` LER stages over the existing QTT rollout, the `BlackoutTrigger` classifier on the
  `CausalFlow` seam, and the blackout observables (`n_e`, plasma frequency, dwell).
- `park2t-blackout-validation`: the self-verifying verification example and the close-and-verify acceptance
  gates — stability-at-stiffness, exponential exactness, RH-jump temperature magnitude, lag + Saha limit,
  counterfactual path-dependence, nonzero ionization — plus the published reference cross-references
  (RAM-C II, Park 2-T, Saha, Apollo dwell) with Tier-A honesty disclaimers.

### Modified Capabilities

<!-- None. Blackout observables are new requirements scoped to plasma-blackout-trigger, not a change to the
     existing qtt-observe requirements; the QTT marcher and PhysicsStage seam are reused unchanged. -->

## Impact

- **`deep_causality_physics`** — new `kernels/hypersonic/` domain (kernels + wrappers + `mod.rs`, flattened at
  `lib.rs`); new quantity newtypes under `quantities/`; new Park / Millikan–White coefficient constants under
  `constants/`. Reuses `quantities/mhd/` (`PlasmaFrequency`, `DebyeLength`) and `mhd/plasma.rs`.
- **`deep_causality_cfd`** — new `PhysicsStage` LER stages and `BlackoutTrigger` in `types/flow/`; blackout
  observables on the QTT march run; reuses `QttImmersed2d::advance_scalar` and the `Coupling`/`CausalFlow`
  seams unchanged; new `verification/qtt_park2t_blackout/` example registered in `Cargo.toml` and
  `verification/README.md`.
- **Dependencies** — no new external crates. `deep_causality_cfd` already depends on `deep_causality_physics`.
- **Tests / Bazel** — new test modules mirror the src tree, registered in `mod.rs` and the `tests/BUILD.bazel`
  targets of both crates; 100% coverage of new code.
- **Notes** — on completion, marks Gap 2 **Tier-A closed** in
  [gap-analysis §4](../../notes/plasma-blackout/gap-analysis.md) and the gap-2 note §6, with Tier-B still open.
- **Constraints honored** — static dispatch only; no `dyn`/`unsafe`/lib-macros; crate-root imports; lib float
  literals confined to `constants/` mapping via `from_f64`.
