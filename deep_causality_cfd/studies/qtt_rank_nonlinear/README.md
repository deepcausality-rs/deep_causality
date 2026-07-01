<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_nonlinear` — nonlinear (forming-shock) rank probe

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_nonlinear
```

**What it tests.** `qtt_rank_dynamic` left one case OPEN: its `QttLinear1d` vehicle is *linear*, so it
transports a fixed shape but never *forms* a shock. The Tier-B threat is the **nonlinear steepening** of
smooth data into a near-discontinuity. This study builds a self-contained **Burgers marcher** from the
existing QTT primitives — `u_t + ½(u²)_x = ν u_xx`, conservative flux form, explicit Euler + per-step
rounding — and measures the bond dimension as a shock *forms*.

**Findings (gated, exit nonzero on regression).**

- **1-D forming shock is cheap.** `u₀ = 0.5 + 0.4 sin(2πx)` steepens into a shock; the bond peaks at **8**
  and stays there — nonlinear steepening does not blow 1-D rank, exactly as the static study predicted (a
  1-D discontinuity is rank ≤ 2).
- **2-D forming *curved* shock raises rank — the threat is real and dynamic.** A smooth radial bump
  self-advects into a curved front; the bond climbs **7 → 20** (at only 64²; it grows with resolution).
  This is the dynamic confirmation of the static 2-D result.
- **Thickening is NOT the curved-shock lever (the hypothesis was refuted by running it).** The rank of a
  curved shock is set by its **curvature / mis-alignment** with the grid axes, which adding viscosity
  cannot remove. Worse, **naive over-thickening is diffusion-CFL-unstable**: at `ν = 6 dx` the diffusion
  number `ν·dt/dx² = 1.2 ≫ 0.25` and the run blows up to **full rank (64)**. So you cannot simply crank
  artificial viscosity in an explicit scheme.

**Conclusion / scoping.** The nonlinear 2-D rank threat is confirmed. The levers are **coordinate
alignment** (shock-aligned / body-fitted coordinate — the `qtt_rank_study` result) and an **implicit /
IMEX step** for stable dissipation within the diffusion CFL (gap **C3**). Neither is exercised here — this
study *captures on a fixed Cartesian grid on purpose*, to measure the cost of doing so. The shock-aligned
confinement test is the next study, and both levers are Tier-B design commitments. Analysis:
`openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** Burgers is a scalar model, not the compressible Euler/NS system; explicit Euler + central
differences is the minimal honest shock-former, not a production scheme; 64²/128² are small grids chosen
for runtime, so the absolute 2-D rank is a lower bound (it grows with `L`).
