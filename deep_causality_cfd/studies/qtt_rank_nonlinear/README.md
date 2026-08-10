<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_nonlinear` — nonlinear (forming-shock) rank probe

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_nonlinear
```

**What it tests.** `qtt_rank_dynamic` left one case OPEN. Its `QttLinear1d` vehicle is *linear*, so
it transports a fixed shape but never *forms* a shock. The Tier-B threat is the **nonlinear
steepening** of smooth data into a near-discontinuity. This study builds a self-contained **Burgers
marcher** from the existing QTT primitives, `u_t + ½(u²)_x = ν u_xx` in conservative flux form with
explicit Euler and per-step rounding, and measures the bond dimension as a shock *forms*.

**Findings (gated, exit nonzero on regression).**

- **A 1-D forming shock is cheap.** `u₀ = 0.5 + 0.4 sin(2πx)` steepens into a shock, and the bond
  peaks at **8** and stays there. Nonlinear steepening does not blow up 1-D rank, exactly as the
  static study predicted, since a 1-D discontinuity is rank 2 or less.
- **A 2-D forming *curved* shock raises rank, so the threat is real and dynamic.** A smooth radial
  bump self-advects into a curved front and the bond climbs **7 → 20**, at only 64², and it grows
  with resolution. This is the dynamic confirmation of the static 2-D result.
- **Thickening could NOT be tested as a lever here.** The explicit 2-D diffusion limit
  `ν·dt/dx² ≤ 0.25` at `dt = 0.2 dx` confines stable `ν` below `1.25 dx`, so this scheme has no room
  to thicken. The two viscosities run are `ν = 1 dx`, stable, and `ν = 6 dx`, whose diffusion number
  is 1.2 and which saturates to **full rank (64)**. That is a statement about the explicit scheme's
  stable window, not about thickness as a rank lever. You cannot simply crank artificial viscosity
  in an explicit scheme.
- **Curvature is not isolated here either.** No flat forming shock at the same `ν`, `dt` and front
  width runs in this study. The controlled comparison that holds thickness fixed at 2 cells and
  varies orientation is `qtt_rank_study`'s.

**Conclusion.** The nonlinear 2-D rank threat is confirmed. The candidate levers are **coordinate
alignment**, meaning a shock-aligned or body-fitted coordinate, which is the `qtt_rank_study` result,
and an **implicit / IMEX step** for stable dissipation within the diffusion CFL, which is gap **C3**.
Neither is exercised here, because this study *captures on a fixed Cartesian grid on purpose*, to
measure the cost of doing so. The shock-aligned confinement test is the next study, and both levers
are Tier-B design commitments. Analysis: `openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** Burgers is a scalar model, not the compressible Euler/NS system. Explicit Euler with
central differences is the minimal honest shock-former, not a production scheme. 64² and 128² are
small grids chosen for runtime, so the absolute 2-D rank is a lower bound that grows with `L`.

See `output.txt` for the recorded reference output.
