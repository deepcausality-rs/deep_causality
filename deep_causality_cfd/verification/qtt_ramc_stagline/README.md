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

## The flight condition and the anchor

The harness flies two cited RAM-C II freestream stations (Parent, Thoguluva Rajendran & Omprakas,
"Electron Losses in Hypersonic Flows", arXiv:2111.09432). Each is stated as Mach number, static
temperature and dynamic pressure; velocity and number density are derived from those three values
and the US-1976 air molar mass.

| Station | Mach | T∞ (K) | q (Pa) | V (m/s, derived) | n∞ (m⁻³, derived) |
|---|---|---|---|---|---|
| 71 km (anchor comparison) | 25.9 | 217.9 | 2 280 | 7 664.4 | 1.614e21 |
| 61 km (lower bound) | 23.9 | 255.9 | 8 000 | 7 664.5 | 5.663e21 |

The anchor is the flight's station-1 (x/D = 0.15) Ka-band critical-density crossing. Grantham
(1970, NASA TN D-6062, p. 18) places it at 236 000 ft (71.93 km). His critical-density relation
`N_e,cr = 1.287e-8 f²` (cm⁻³, f in Hz) and slope-technique ratio `N_e,pk/N_e,cr = 0.63` (p. 11), at
the 35 000 MHz Ka-band (Table I), give `N_e,pk = 9.93e18 m⁻³`. The 71 km station sits 0.93 km below
the crossing. Below 71.93 km the flight's station 1 is overdense at Ka-band, the highest frequency
flown, so at 61 km the same value is a lower bound.

## What it gates (self-verifying, exit nonzero on regression)

1. **Post-shock temperature band.** `T₂` lies in the realistic ≈10⁴ K band (measured 7509 K).
2. **The closed-form Park-2T controller.** With the N₂–N₂ reduced mass `μ = 14.007` it lands
   `1.070e17 m⁻³`, −1.97 decades below the anchor. The gate is a tripwire on that value; its band
   stays narrow.
3. **Blackout onset.** The plasma frequency exceeds the comms band.
4. **`O(1)` rank.** The smooth post-shock relaxation profile stays low tensor-train rank (bond 2).
5. **The uncalibrated network inside its allowance.** The finite-rate prediction lies within
   ±0.70 decades of the anchor, with no Saha calibration target anywhere in the path. The width is a
   chemistry-model-spread allowance (rate sets spread 2x to 5x), so the gate is a tripwire.
6. **Electron impact is a refinement.** The associative channel carries the prediction; the
   thresholded impact channels add at most one decade (measured: +29 percent).
7. **The carried arm self-limits.** See the sheath-renewal A/B under recombination below.
8. **The 61 km lower bound** (`[reference]`). The network's peak `n_e` at the cited 61 km station is
   at least the anchor value. Measured `1.811e20 m⁻³`, 1.26 decades above the bound.

## The uncalibrated finite-rate network

The `FiniteRateIonizationStage` evaluates the three-channel RP-1232 network with no calibration
knob: associative ionization `N + O -> NO+ + e-` with its dissociative-recombination reverse,
thresholded electron-impact ionization at `T_e = T_ve`, and a lagged neutral atom pool whose
N clock carries both direct dissociation and the low-activation Zeldovich exchange
`N2 + O -> NO + N`. Each rate runs at its controlling temperature: ionization at the calibrated
geometric mean, dissociation at Park's published `T_tr^0.7 T_ve^0.3`, electron channels at
`T_e = T_ve`. The parcel age on the stagnation line is the knob-free transit-age profile
`age(ξ) = t_res·ln(1/(1−ξ))` from the linear stagnation-line deceleration, and the gate reads
the profile's peak, the quantity the flight reflectometers measured. Measured at 71 km: channel 1
plus the pool `1.278e19` (+0.11 dec), full network `1.643e19` (+0.22 dec, ~1.65x) against the
`9.93e18` anchor, inside the production-code context (DPLR, LAURA, and US3D land 2x to 3x on this
peak).

**Sheath-renewal A/B under recombination.** Both integration modes are measured over the same
transit-age profile: the renewal arm peaks at `1.643e19` (+0.22 dec) and the carried arm at
`6.598e17` (−1.18 dec). The example uses renewal. Its clock is evaluated at the network fixed
point, which equals the true Riccati relaxation rate `sqrt(production·β)` of the two-way balance
near equilibrium, and it realizes the transit-age closure the anchor gate reads. The carried arm
rates its clock at the young carried population and under-relaxes young parcels, but the
recombination channel makes it self-limit at or below the closed-form arm, so explicit renewal is
not needed to prevent runaway (a forward-only surrogate diverges without it).

## The physics, honestly

- **Effective γ.** Perfect-gas `γ = 1.4` over-predicts `T₂` badly (≈30 000 K) because it ignores the
  dissociation/vibration that absorb the post-shock energy. The engineering effective value for
  strongly-dissociated hypersonic air is `≈1.1–1.2`; `γ = 1.1` lands `T₂ = 7509 K` at the 71 km
  station, the band RAM-C ionizes in. This is an effective-γ closure, not perfect gas.
- **Nonequilibrium lag is essential.** Saha *equilibrium* at the 71 km post-shock state gives
  `n_e = 7.04e21 m⁻³`, about 21 % of the post-shock density, orders above RAM-C. The flight value is
  the **nonequilibrium lagged** one: the residence time `t_res = standoff/u₂ = 2.02e-5 s` is short
  against the ionization time `τ_ion = 1/(k_f·n₂)`, with `k_f` the **dominant associative-ionization
  rate** N + O → NO⁺ + e⁻ (Park / Gupta), a grounded rate rather than a free fit.
- **The closed-form controller and the network prediction, kept separate.** The Park-2T controller
  path is the closed-form Saha surrogate. It uses the N₂–N₂ reduced mass `μ = 14.007` (the N–N atomic
  pair, `μ = 7.0`, has no vibrational mode) and lands 1.97 decades below the anchor, reported as an
  offset. The finite-rate network path is the independent prediction: the same anchor, approached with
  no calibration target, lands at +0.22 dec (~1.65x). The example prints and gates both numbers, so the
  distinction stays measurable.

## References

- Grantham, W. L. (1970). *Flight Results of a 25 000-Foot-per-Second Reentry Experiment Using
  Microwave Reflectometers to Measure Plasma Electron Density and Standoff Distance.* NASA TN D-6062,
  NASA Langley. PDF: `deep_causality_physics/papers/ram_c_ii_nasa_tn_d6062.pdf`.
- Parent, B., Thoguluva Rajendran, P. & Omprakas, A. *Electron Losses in Hypersonic Flows.*
  arXiv:2111.09432. The RAM-C II freestream at 61 and 71 km.
- Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990); Gupta–Yos–Thompson–Lee, NASA RP-1232
  (1990) — the two-temperature model and the associative-ionization rate.
- `openspec/notes/archive/cfd-plasma-blackout/gap-2/` — the Tier-B design and the studies this milestone rests on.
