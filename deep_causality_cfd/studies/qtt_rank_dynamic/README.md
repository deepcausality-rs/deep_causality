<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_dynamic` — dynamic rank probe (does a marcher keep the field low-rank?)

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_dynamic
```

**What it tests.** The static study (`qtt_rank_study`) measured representability of *frozen* profiles.
This one tests the *live* question the Tier-B cost model actually depends on: a rollout repeatedly
applies a stencil operator and re-rounds — does the bond dimension **inflate over time**, or stay
bounded? Vehicle: the existing `QttLinear1d` marcher (linear advection–diffusion
`u ← round(u + Δt·(−c ∂ₓu + ν ∂ₓₓu))` on a periodic `2^L` grid).

**Findings (gated, exit nonzero on regression).**

- **Rank-safe under fixed-tolerance rounding.** Over 3000 steps a smooth bump holds at χ≈8 and a
  near-grid-scale steep top-hat settles at χ≈7–8 from an initial encode of 4 — it *settles* near its
  static representability rank, it does **not** run away.
- **More diffusion does not increase rank.** The high-ν (thickened) steep case peaks at or below the
  low-ν case (χ 7 vs 8) — the thickening lever is, at most, mildly rank-reducing here.

**Honest limit (why the diffusion lever looks weak).** `QttLinear1d` is **linear**: it transports a
fixed-shape feature but cannot **steepen** one. The strong dynamic-rank threat — a *nonlinear* shock
steepening into a near-discontinuity, with dispersive Gibbs growth — is therefore **not exercised here
and remains OPEN for Tier-B**. Testing it requires a nonlinear (Burgers / compressible) marcher, which
does not exist yet. So this study establishes the *reassuring half* (linear transport + rounding is
rank-safe) and explicitly scopes the *unresolved half* (nonlinear steepening). Analysis:
`openspec/notes/plasma-blackout/gap-2/`.
