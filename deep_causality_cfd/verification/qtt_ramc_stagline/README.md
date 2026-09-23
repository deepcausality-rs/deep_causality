<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# RAM-C stagnation line — Tier-B Stage 4 (shock fitting + reused Tier-A LER)

This example runs the buildable Tier-B milestone on the RAM-C stagnation line. There the bow shock
is a 1-D **fitted interface**: the freestream crosses it and the **exact Rankine–Hugoniot jump** sets
the post-shock state. No flux is marched *through* the front (see `studies/qtt_repin_marcher`), so
each side stays smooth and `O(1)` rank. The post-shock translational temperature `T₂` is the
**transported energy**, in place of the Tier-A recovery-temperature *reconstruction*. The smooth
post-shock relaxation zone then drives the reused Tier-A ionization kernels (Saha / Park-2T →
electron density → plasma frequency → blackout).

```bash
cargo run --release -p deep_causality_cfd --example qtt_ramc_stagline
```

## What it gates (self-verifying, exit nonzero on regression)

1. **Post-shock temperature band.** `T₂` lies in the realistic ≈10⁴ K band.
2. **Peak electron density vs RAM-C II.** The closed-form Park-2T controller, with the N₂–N₂
   reduced mass `μ = 14.007`, lands −1.27 decades below the `1e19 m⁻³` flight anchor. The gate is
   a tripwire on that value; its band stays narrow.
3. **Blackout onset.** The plasma frequency exceeds the comms band.
4. **`O(1)` rank.** The smooth post-shock relaxation profile stays low tensor-train rank.
5. **The uncalibrated network inside its earned band.** The finite-rate prediction lies within
   ±0.7 decades of the anchor, with no Saha calibration target anywhere in the path.
6. **Electron impact is a refinement.** The associative channel carries the prediction; the
   thresholded impact channels add at most one decade (measured: +19 percent).
7. **The carried arm self-limits.** See the sheath-renewal A/B under recombination below.

## The uncalibrated finite-rate network

The `FiniteRateIonizationStage` evaluates the three-channel RP-1232 network with no calibration
knob: associative ionization `N + O -> NO+ + e-` with its dissociative-recombination reverse,
thresholded electron-impact ionization at `T_e = T_ve`, and a lagged neutral atom pool whose
N clock carries both direct dissociation and the low-activation Zeldovich exchange
`N2 + O -> NO + N`. Each rate runs at its controlling temperature: ionization at the calibrated
geometric mean, dissociation at Park's published `T_tr^0.7 T_ve^0.3`, electron channels at
`T_e = T_ve`. The parcel age on the stagnation line is the knob-free transit-age profile
`age(ξ) = t_res·ln(1/(1−ξ))` from the linear stagnation-line deceleration, and the gate reads
the profile's peak, the quantity the flight reflectometers measured. Measured: channel 1 plus
the pool `1.887e19` (+0.28 dec), full network `2.251e19` (+0.35 dec, ~2.25x) against the `1e19`
anchor, inside the production-code context (DPLR, LAURA, and US3D land 2x to 3x on this peak).

**Sheath-renewal A/B under recombination.** Both integration modes are measured over the same
transit-age profile: the renewal arm peaks at `2.251e19` (+0.35 dec) and the carried arm at
`1.768e18` (−0.75 dec). The example uses renewal. Its clock is evaluated at the network fixed
point, which equals the true Riccati relaxation rate `sqrt(production·β)` of the two-way balance
near equilibrium, and it realizes the transit-age closure the anchor gate is pinned on. The
carried arm rates its clock at the young carried population and under-relaxes young parcels, but
the recombination channel makes it self-limit at or below the closed-form arm, so explicit renewal
is not needed to prevent runaway (a forward-only surrogate diverges without it).

## The physics, honestly

- **Effective γ.** Perfect-gas `γ = 1.4` over-predicts `T₂` badly (≈30 000 K) because it ignores the
  dissociation/vibration that absorb the post-shock energy. The engineering effective value for
  strongly-dissociated hypersonic air is `≈1.1–1.2`; `γ = 1.1` lands `T₂ ≈ 8000 K`, the band RAM-C ionizes
  in. This is an effective-γ closure, not perfect gas.
- **Nonequilibrium lag is essential.** Saha *equilibrium* at 8000 K gives ~15% ionization, orders above
  RAM-C. The measured `n_e ≈ 1e19` is the **nonequilibrium lagged** value: the residence time
  `t_res = standoff/u₂` is short against the ionization time `τ_ion = 1/(k_f·n₂)`, with `k_f` the **dominant
  associative-ionization rate** N + O → NO⁺ + e⁻ (Park / Gupta), a grounded rate rather than a free fit. The
  closed-form LER relaxation `α = α_eq·(1 − e^{−t_res/τ_ion})` pulls the peak below equilibrium toward the
  flight value.
- **The closed-form controller and the network prediction, kept separate.** The Park-2T controller
  path is the closed-form Saha surrogate. It uses the N₂–N₂ reduced mass `μ = 14.007` (the N–N atomic
  pair, `μ = 7.0`, has no vibrational mode and would land near the anchor for the wrong reason) and
  lands 1.27 decades below the anchor, reported as an offset. The finite-rate network path is the
  independent prediction: the same anchor, approached with no calibration target, lands at +0.35 dec
  (~2.25x). The example prints and gates both numbers, so the distinction stays measurable.

## References

- RAM-C II flight experiment, NASA Langley (1970) — the canonical ionized-reentry electron-density dataset.
- Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990); Gupta–Yos–Thompson–Lee, NASA RP-1232
  (1990) — the two-temperature model and the associative-ionization rate.
- `openspec/notes/archive/cfd-plasma-blackout/gap-2/` — the Tier-B design and the studies this milestone rests on.
