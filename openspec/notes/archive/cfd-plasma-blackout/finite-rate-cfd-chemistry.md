<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Finite-rate CFD chemistry: preparation for Option A, transition commentary for Option B

> **STATUS: Option A shipped (2026-07-03).** The finite-rate ionization network landed through
> `openspec/changes/archive/2026-07-03-add-finite-rate-ionization-network/` with two design revisions this note did
> not anticipate, both recorded in
> [Resolution 3](gap-3/gap-three-resolution-3-chemistry-fidelity.md): the atom pool ships
> *lagged* (not partial-equilibrium), and the stagnation-line parcel age is the transit-age
> profile `age(ξ) = t_res·ln(1/(1−ξ))`, whose peak the anchor gate reads. Measured uncalibrated:
> **+0.48 decades (3.0×)** of the RAM-C II anchor on the stagnation line; the plasma-blackout corridor
> and weather table fly the network with all gates green. The Option B commentary below stands
> unchanged as the transition path.
>
> *(Original preparation note, 2026-07-02, follows.)*

## 1. Where the chemistry stands

The lever ladder of the chemistry-fidelity gap is two-thirds discharged:

- **Lever 1, T_ve-controlled ionization: shipped.** Saha target and rate driven off
  `T_a = sqrt(T_tr * T_ve)`; the single-temperature 12x over-prediction fell to ~1.1x on the
  stagnation line, and the continuous-descent corridor holds 1.43x on the RAM-C II anchor.
- **Lever 2, explicit 3-T electron energy: investigated and closed.** It brackets the model
  spread instead of improving the calibrated result; the durable insight (electrons are created
  in the post-shock bath, so `T_e(0) = T_ve`) is recorded in
  [Resolution 2](gap-3/gap-three-resolution-2-tve-controlled-ionization.md).
- **Lever 3, the finite-rate network: shipped (see STATUS above).** Option A below is the
  design it started from.

The prerequisite the gap note named is now satisfied: since
`2026-07-02-add-compressible-blackout-carrier`, the corridor consumes **evolved** per-cell
`T_tr`, `n_tot`, and pressure from the compressible carrier, so a rate network has real
transported inputs instead of reconstructed ones.

The honest framing of the payoff, so nobody is surprised later: the current 1.43x is a
*calibrated* result. Production finite-rate codes (DPLR, LAURA, US3D) land 2x to 3x on RAM-C
peak `n_e`, with a 2x to 5x chemistry-model spread between them. Option A will therefore
probably *worsen* the printed number into that band while transforming what it means:
prediction from rate data instead of reproduction of a tuned closure. Calibration becomes
validation. The gates should be re-pinned accordingly (a ~3x band that is *earned* replaces the
5x band that was *granted*).

## 2. Option A: a two-way finite-rate ionization network in the existing stage architecture

**Scope.** Replace the `IonizationStage` closure ("Saha equilibrium target times one associative
rate, forward only") with a small reaction network evaluated per cell inside the same
`PhysicsStage` slot. No marcher change, no new transported fields; the quasi-steady
sheath-renewal picture keeps doing the transport bookkeeping until Option B replaces it.

**The network.** Three channels cover RAM-C-class entries (7 to 8 km/s):

1. `N + O <-> NO+ + e-`, associative ionization forward, **dissociative recombination**
   backward. The forward channel is what the current stage already rates; the backward channel
   is the physically load-bearing addition: it is the real blackout-exit mechanism and the
   reason the carried wake currently has to be frozen.
2. Electron-impact ionization (`N + e- -> N+ + 2e-` and the O analog), thresholded and rated at
   the electron temperature, taken as `T_e = T_ve` per the recorded lever-2 insight. Secondary
   at RAM-C speeds, but it shapes the `n_e` buildup slope and therefore the onset altitude.
3. The neutral atom pool (`N`, `O`) that channel 1 consumes. Decision point for the proposal:
   either a partial-equilibrium closure (atom fractions from dissociation equilibrium at `T_a`,
   with the existing LER lag), or two more LER scalars driven by the Park dissociation rates
   plus Zeldovich exchange. Start with partial equilibrium; promote to rated dissociation only
   if the stagnation-line comparison demands it.

**Integration.** LER-native, extending the pattern of
[`gap-2/gap-two-resolution-3-ionization-lag.md`](gap-2/gap-two-resolution-3-ionization-lag.md):
the electron density relaxes toward the network's fixed point with
`tau = 1/(k_f[M] + beta * n_e)`, exactly the extension the `IonizationStage` docstring already
documents as the deferred recombination channel, generalized to the two or three coupled
scalars the network carries. No stiff ODE integrator, no per-cell Newton solve.

**Kernels and data.** New rate kernels in `deep_causality_physics` from the Park (1990) rate
tables, with equilibrium constants from the Park curve fits so detailed balance holds by
construction. House rules apply: full citations in the kernel docstrings and the source PDFs in
`deep_causality_physics/papers/`.

**What must be re-examined: sheath renewal.** The measured A/B during the carrier re-pin showed
the carried fraction accumulating to equilibrium without explicit renewal (peak 268x over the
anchor, no exit) precisely *because no loss channel existed*. With dissociative recombination in
the network, the carried fraction self-limits, and the explicit renewal mode may become
redundant rather than required. Re-run that A/B as part of the change; whichever mode survives,
the reasoning must be recorded the way the first A/B was.

**Validation ladder.**

1. Unit tests per reaction: equilibrium recovery from both sides, detailed balance, frozen
   limits at low temperature.
2. The stagnation-line verification (`qtt_ramc_stagline`), where the 12x-to-1.1x history lives,
   re-measured against the network with **no Saha calibration target**.
3. The corridor re-pin: anchor band tightened toward 3x; blackout exit altitude compared against
   the RAM-C II window (the flight stayed dark to roughly 25 to 30 km; the surrogate exits at
   46 km, and the network should move that boundary toward flight); onset altitude recorded as a
   prediction.

**Effort.** One OpenSpec change: kernels plus tests, one stage replacing `IonizationStage` in
the corridor stack, the two A/Bs, and the re-pin. Comparable to one corridor stage of the
carrier change, not to the carrier change itself.

**Runtime impact.** Small. The chemistry stage today is closed-form arithmetic over 1024 cells
and is a minor fraction of the ~40 ms coupled step; the tensor-train advance and the
decode/publish round trip dominate. The network multiplies the chemistry arithmetic by roughly
3x to 5x (three channels, equilibrium constants, a coupled two-or-three-scalar update), which
lands at **+5 to +15 percent wall-clock** on the corridor example: about 35 s today, at most ~40 s
after. The 300 s budget gate does not move.

## 3. Option B commentary: the transition to species-transported reacting CFD

Option B replaces the quasi-steady per-cell picture with real species transport: the marched
state grows from 4 tensor trains `[rho, m_x, m_y, E]` to roughly 12 (add a
vibrational-electronic energy `E_ve` and partial densities for a 7-species air model: N2, O2,
NO, N, O, NO+, e-). This is the production-CFD architecture, and it is the road to
off-calibration credibility claims, catalytic-wall work, and publication-grade comparisons.

**Option A is the first third of Option B.** The rate kernels, the equilibrium
constants, the detailed-balance tests, and the reactor unit tests transfer verbatim: in Option
B they stop being an LER fixed point and become the source terms of the species continuity
equations. Nothing in Option A is throwaway work.

**What Option B adds beyond A:**

1. Species advection on tensor trains: each partial density fluxed like the bulk density, with
   per-species positivity floors and strip boundary conditions (post-shock frozen composition
   from the Rankine-Hugoniot state).
2. The two-temperature energy equation: `E_ve` transported with Millikan-White relaxation and
   chemistry-vibration coupling as sources, heats of formation folded into `E`.
3. Mixture thermodynamics: variable gamma and molecular weight from composition (Gupta or
   McBride fits), replacing the single `gamma_eff`.
4. **The design risk that needs its own D0-style timing study before any proposal:** stiff
   sources want pointwise dense evaluation, while the marcher wants to stay in train form. Every
   step pays a decode-react-encode round trip across ~12 components, and the encode side is SVD
   work. Whether reaction fronts stay low-rank under the bond cap, and what the round trip
   costs, must be measured first, exactly as the carrier's D0 study measured the 3-D marcher out
   of the corridor and the 2-D marcher in.

**Sequencing.** Option B belongs to the stagnation-line and 3-D fitted validation path first,
where its fidelity is cleanly measurable against published RAM-C profiles, and enters the
corridor only after the timing study re-verifies the minutes budget. 

**Triggers that would justify it:** a consumer needing predictions away from the RAM-C
calibration point; boundary-layer or catalytic-wall electron-density work (where the reflectometer
comparison actually becomes sensitive to wall chemistry); a publication requiring the
production band on first principles; or a revival of the 3-D fitted marcher as the validation
tool.

**Runtime impact.** Roughly 4x to 8x on the coupled step: ~3x from marching 12 trains instead
of 4, the remainder from the per-step dense react-and-re-encode round trip and modest source
sub-cycling. On the current corridor example that is 2.5 to 5 minutes of wall-clock against the 300 s
budget gate, so Option B in the corridor implies either the study-verified fast configuration
or a renegotiated budget. This is the concrete reason B rides the validation path first.

## 4. Counterfactual scaling under either option

The branch study fans out through `deep_causality_par::scoped_map` (scoped fork-join, one
branch per core up to `available_parallelism`), so the branch phase's wall-clock is one branch
duration, not the sum. Today that phase is ~100 coupled steps, about 10 s of the 35 s total;
the four sequential legs (~730 steps) dominate.

Growing the study from 3 to 6 counterfactuals therefore costs almost nothing on a machine with
6 or more cores: the fan-out runs 6 branches in the same one-branch wall time, and total
runtime moves by the scheduling noise, not by 2x. Under Option A the same holds at +5 to +15
percent overall. Under Option B each branch is 4x to 8x slower, but the fan-out still collapses
6 branches into one branch duration (roughly 40 to 80 s of branch-phase wall); the sequential
legs, not the counterfactuals, remain the budget item. The general rule this exposes: with the
scoped fan-out, the counterfactual *count* is nearly free up to the core count; the
counterfactual *length* (steps per branch) and the sequential descent legs are what buy
runtime.

## 5. Related

- [`gap-3/chemistry-fidelity-gap.md`](gap-3/chemistry-fidelity-gap.md): the lever ladder and
  the measured error anatomy this note continues.
- [`gap-3/gap-three-resolution-2-tve-controlled-ionization.md`](gap-3/gap-three-resolution-2-tve-controlled-ionization.md):
  lever 1 shipped, lever 2 closed, the `T_e = T_ve` insight channel 2 reuses.
- [`gap-2/gap-two-resolution-3-ionization-lag.md`](gap-2/gap-two-resolution-3-ionization-lag.md):
  the LER `(target, tau)` pattern the network extends.
- `openspec/changes/archive/2026-07-02-add-compressible-blackout-carrier/`: the evolved-state
  carrier this work rides on, including the sheath-renewal A/B that recombination re-opens.
- `examples/avionics_examples/cfd/plasma_blackout/corridor/`: the corridor example whose gates get
  re-pinned, and whose `constants.rs` labels the forward-only limitation this removes.
