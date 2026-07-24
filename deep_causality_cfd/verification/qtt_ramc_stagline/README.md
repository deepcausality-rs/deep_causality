<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# RAM-C stagnation line — Tier-B Stage 4 (shock fitting + reused Tier-A LER)

The buildable Tier-B milestone. On the stagnation streamline the bow shock is a 1-D **fitted interface**:
the freestream crosses it and the **exact Rankine–Hugoniot jump** sets the post-shock state. No flux is
marched *through* the front (the `studies/qtt_repin_marcher` lesson), so each side stays smooth and `O(1)`
rank. The post-shock translational temperature `T₂` is the **real transported energy**, which retires the
Tier-A recovery-temperature *reconstruction*. The smooth post-shock relaxation zone then drives the reused
Tier-A ionization kernels (Saha / Park-2T → electron density → plasma frequency → blackout).

```bash
cargo run --release -p deep_causality_cfd --example qtt_ramc_stagline
```

## What it gates (self-verifying, exit nonzero on regression)

1. **Post-shock temperature band** — `T₂` in the realistic ≈10⁴ K band.
2. **Peak electron density vs RAM-C II** — the closed-form Park-2T controller, reported at −1.27
   decades below the `1e19 m⁻³` flight anchor after the N₂–N₂ `μ = 14.007` correction. A tripwire on
   the corrected value; the band is not widened to re-admit the old within-3x headline.
3. **Blackout onset** — the plasma frequency exceeds the comms band.
4. **`O(1)` rank** — the smooth post-shock relaxation profile stays low tensor-train rank.
5. **The uncalibrated network inside its earned band** — the finite-rate prediction within
   ±0.7 decades of the anchor, with no Saha calibration target anywhere in the path.
6. **Electron impact is a refinement** — the associative channel carries the prediction; the
   thresholded impact channels add at most one decade (measured: +19 percent).
7. **The carried arm self-limits** — the sheath-renewal A/B under recombination (see below).

## The uncalibrated finite-rate network

The `FiniteRateIonizationStage` evaluates the three-channel RP-1232 network with no calibration
knob: associative ionization `N + O -> NO+ + e-` with its dissociative-recombination reverse,
thresholded electron-impact ionization at `T_e = T_ve`, and a lagged neutral atom pool whose
N clock carries both direct dissociation and the low-activation Zeldovich exchange
`N2 + O -> NO + N`. Each rate runs at its controlling temperature: ionization at the calibrated
geometric mean, dissociation at Park's published `T_tr^0.7 T_ve^0.3`, electron channels at
`T_e = T_ve`. The parcel age on the stagnation line is the knob-free transit-age profile
`age(ξ) = t_res·ln(1/(1−ξ))` from the linear stagnation-line deceleration, and the gate reads
the profile's peak, which is what the flight reflectometers measured. Measured: channel 1 plus
the pool `1.887e19` (+0.28 dec), full network `2.251e19` (+0.35 dec, ~2.25x) against the `1e19`
anchor, inside the production-code context (DPLR, LAURA, and US3D land 2x to 3x on this peak).

**Sheath-renewal A/B under recombination.** Both integration modes are measured over the same
transit-age profile: the renewal arm peaks at `2.251e19` (+0.35 dec) and the carried arm at
`1.768e18` (−0.75 dec). Renewal is kept. Its clock is evaluated at the network fixed point,
which equals the true Riccati relaxation rate `sqrt(production·β)` of the two-way balance near
equilibrium, and it realizes the transit-age closure the anchor gate is pinned on. The carried
arm rates its clock at the young carried population and under-relaxes young parcels, but it
self-limits at or below the closed-form arm — the property the recombination channel was added
for, and the reason explicit renewal is no longer load-bearing against runaway (the forward-only
surrogate diverged without it). This supersedes the first A/B's record.

## The physics, honestly

- **Effective γ.** Perfect-gas `γ = 1.4` over-predicts `T₂` badly (≈30 000 K) because it ignores the
  dissociation/vibration that absorb the post-shock energy. The engineering effective value for
  strongly-dissociated hypersonic air is `≈1.1–1.2`; `γ = 1.1` lands `T₂ ≈ 8000 K`, the band RAM-C ionizes
  in. This is an effective-γ closure, not perfect gas.
- **Nonequilibrium lag is essential.** Saha *equilibrium* at 8000 K gives ~15% ionization — orders above
  RAM-C. The measured `n_e ≈ 1e19` is the **nonequilibrium lagged** value: the residence time
  `t_res = standoff/u₂` is short against the ionization time `τ_ion = 1/(k_f·n₂)`, with `k_f` the **dominant
  associative-ionization rate** N + O → NO⁺ + e⁻ (Park / Gupta), grounded — not a free fit. The closed-form
  LER relaxation `α = α_eq·(1 − e^{−t_res/τ_ion})` pulls the peak below equilibrium toward the flight value.
- **The closed-form controller and the network prediction, kept separate.** The Park-2T controller
  path is the closed-form Saha surrogate. Its former near-anchor landing was an artifact of an invalid
  `μ = 7.0` (the N–N atomic pair, which has no vibrational mode). With the reduced mass corrected to the
  N₂–N₂ `μ = 14.007` it lands 1.27 decades below the anchor, reported as an offset. The finite-rate
  network path is the independent prediction: the same anchor, approached with no calibration target,
  lands at +0.35 dec (~2.25x). Both numbers are printed and gated, so the distinction stays measurable.

## References

- RAM-C II flight experiment, NASA Langley (1970) — the canonical ionized-reentry electron-density dataset.
- Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990); Gupta–Yos–Thompson–Lee, NASA RP-1232
  (1990) — the two-temperature model and the associative-ionization rate.
- `openspec/notes/plasma-blackout/gap-2/` — the Tier-B design and the studies this milestone rests on.
