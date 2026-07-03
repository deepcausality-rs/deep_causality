---
marp: true
theme: default
paginate: true
---

<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# DeepCausality CFD

## MDAO in one process

Fluid dynamics, reacting-gas physics, navigation, and control,
coupled in one typed flow. Optimized by counterfactual branching.
Uncertainty carried end to end.

`deep_causality_cfd`, a DeepCausality workspace crate

---

# The problem the field names

CFD University's survey of the six biggest unsolved challenges in CFD
puts multidisciplinary analysis and optimization on the list, twice over:

- Coupling CFD with structural, thermal, and flight-dynamics solvers
  **"lacks standardized frameworks"**
- The **"propagation of uncertainties from solver to solver"** that would
  track overall simulation uncertainty is **simply absent**

The status quo: each discipline is its own code with its own world.
Solvers exchange interface files. The coupling framework lives in the gaps.

---

# The claim

**Coupling stops being a framework problem when there is nothing to couple across.**

In `deep_causality_cfd`, the disciplines are stages of one typed composition
stepping one shared evolved state:

- No interface files
- No co-simulation orchestrator
- No time-lag between disciplines: every stage sees the state its
  predecessors just wrote, inside the same step

That is not an integration layer over solvers. It is the program itself.

---

# The coupling stack, verbatim

```rust
Coupling::between_steps()
    .then(VibrationalLagStage::new(/* Millikan-White bath */))
    .then(FiniteRateIonizationStage::new(n_tot).with_density_field("n_tot"))
    .then(RegimeClassify::new(l_char, trigger))
    .then(BankSteeredLift::new(rho_ref, cda_over_m, l_over_d))
    .then(TrajectoryNav::new(q_diag, gnss_var, optical_var).with_imu(imu))
    .then(CyberneticCorrect::new(SafetyEnvelope::new(q_max, g_max, bank_max)))
    .build()
```

A static cons-tuple; no `dyn`. An `Err` from any stage
(an envelope breach, a physics guard) short-circuits the whole step.

---

# Four disciplines, one loop

```text
flow        compressible tensor-train marcher, shock-fitted inflow
  |
plasma      finite-rate ionization network on the evolved per-cell state
  |
navigation  the evolved electron density gates which measurements
  |         the Kalman filter may fold: GNSS denial is a flow event
  |
control     the safety gate clamps the bank command; the aero stage
  |         FLIES the clamped command
  '--> the steered trajectory selects the next step's freestream
```

The loop is closed. CFD feeds estimation, estimation feeds control,
and control feeds the CFD's boundary condition.

---

# The analysis half: multi-paradigm CFD

Three solver families, one language, one scalar type:

- **Calculus-based.** The DEC-native Navier-Stokes solver: velocity as an
  edge 1-form, the Leray projector as the incompressibility equation,
  un-projected time-stepping a compile error
- **Compression-based.** QTT compressible marchers: a `2^L` grid at cost
  `chi^2 * L`, shock fitting instead of shock capturing
- **Analytic and pointwise.** Exact Rankine-Hugoniot jumps, Park two-temperature
  closures, the finite-rate ionization network

Paradigm choice is an engineering decision inside one program,
not a code migration.

---

# The optimization half: counterfactual branching

The flagship pauses its march on a flow-resolved event (blackout onset),
then forks the paused state in O(1) through copy-on-write:

- **Coarse round:** six bank commands, 0 to 40 degrees, flown concurrently
- **Fine round:** eleven 0.5-degree candidates around the coarse winner,
  forked from the *same* paused state, scored against the *same* aim
- Seventeen worlds total; the best one is committed **mid-descent**

Result: committed miss 2.39 m vs 20.00 m ballistic (8.4x). The sweep stops
there deliberately: 2.39 m equals the INS drift at the blackout peak,
the vehicle's knowledge floor. Steering tighter than you can navigate buys nothing.

---

# The uncertainty half: propagation through the whole chain

The weather-dispersion example is the survey's missing capability, running:

- Six counterfactual atmospheres times eight deterministic noise draws,
  48 full descents, pushed through flow, plasma, blackout window,
  and navigation drift
- Every cell reported with Monte Carlo error bars
- The headline effect (cold-day INS degradation, 1.50x) must clear
  two combined sigma; it clears 4.0, **and that check is itself a gate**

Next step on the roadmap: dispersions declared as `Uncertain<T>` values,
significance as `probability_exceeds(...)`, sample counts adaptive via
sequential hypothesis testing.

---

# Evidence, not promises

- The chemistry is **uncalibrated**: RP-1232 rate pairs, no tuning knob
  anywhere in the prediction path. It lands **3.0x** on the RAM-C II
  flight anchor, inside the band production codes achieve on the same peak
- **Thirteen** self-verifying verification programs, gated against analytic
  solutions and published references, each exiting nonzero on regression
- **Precision is a parameter**: the corridor reran at 106-bit precision and
  reproduced every gate, so the error budget is closures and grid, not round-off
- **Determinism**: parallel and sequential runs produce identical bits
- The whole flagship: about **40 seconds** on a laptop

---

# Honest limits

- Validated at **one** flight anchor (RAM-C II); off-anchor rows are
  model extrapolation
- The corridor world is deterministic point-mass 3-DOF with a fixed
  atmosphere; dispersions live in the companion weather study
- Turbulence modelling is **in scope, staged**: energy-spectra observables
  land first, then LES on the DEC solver with a gated resolved-TKE fraction
- Out of scope at any tier: million-core scaling, CAD repair

Every simplification is labeled in the examples' `constants.rs`,
with its consequence stated.

---

# Roadmap, least to most effort

1. Objective convergence gates as the default workflow
2. More tensor-train-native observables (spectra unlock turbulence)
3. Dispersion sweeps as a first-class combinator, carried by `Uncertain<T>`
4. Reduced-order model export from QTT states
5. Self-describing multidisciplinary results
6. Experimental data fusion (`MaybeUncertain<T>` for intermittent gauges)
7. Meshless complex geometry via watertight-surface rasterization

Then: turbulence, as its own change series.
Full note: `openspec/notes/cfd-roadmap/cfd-roadmap.md`

---

# The one-slide summary

**MDAO, by definition:** couple multiple disciplines' analyses,
optimize over the coupled result, track the uncertainty.

**This crate, by construction:** flow, plasma, navigation, and control
as stages of one typed composition; optimization as counterfactual
forks of the running simulation; uncertainty pushed through the entire
chain with gated statistics.

The definition and the architecture are the same sentence.

```bash
cargo run --release -p avionics_examples --example plasma_blackout_corridor
```
