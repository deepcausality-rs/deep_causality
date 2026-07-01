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
2. **Peak electron density vs RAM-C II** — within ~2 decades of the `1e19 m⁻³` flight anchor (order of
   magnitude).
3. **Blackout onset** — the plasma frequency exceeds the comms band.
4. **`O(1)` rank** — the smooth post-shock relaxation profile stays low tensor-train rank.

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
- **Order-of-magnitude anchor, not a calibrated match.** The Park-2T `T_e = T_ve` lumping over-predicts peak
  `n_e` ~2× vs a 3-T closure; the single-rate lag is a surrogate for the full nonequilibrium kinetics. The
  result lands ~1 decade above the RAM-C anchor — a legitimate order-of-magnitude match for the Tier-A
  surrogate, with the exact value requiring the full reacting-plasma kinetics.

## References

- RAM-C II flight experiment, NASA Langley (1970) — the canonical ionized-reentry electron-density dataset.
- Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990); Gupta–Yos–Thompson–Lee, NASA RP-1232
  (1990) — the two-temperature model and the associative-ionization rate.
- `openspec/notes/plasma-blackout/gap-2/` — the Tier-B design and the studies this milestone rests on.
