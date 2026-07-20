<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_dynamic` — dynamic rank probe (does a marcher keep the field low-rank?)

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_dynamic
```

**What it tests.** The static study (`qtt_rank_study`) measured representability of *frozen*
profiles. This one tests the live question the Tier-B cost model actually depends on. A rollout
repeatedly applies a stencil operator and re-rounds, so does the bond dimension **inflate over
time**, or stay bounded? The vehicle is the existing `QttLinear1d` marcher, linear
advection–diffusion `u ← round(u + Δt·(−c ∂ₓu + ν ∂ₓₓu))` on a periodic `2^L` grid.

**Findings (gated, exit nonzero on regression).** Measured at L=12, N=4096, c=1, dt=3.66e-5, 3000
steps, tolerance 1e-8:

| case | init | **peak** | final |
|---|---|---|---|
| A: smooth, low ν | 8 | **8** | 8 |
| B: steep, low ν (4e-5) | 4 | **8** | 7 |
| C: steep, high ν (6e-4) | 4 | **7** | 7 |

- **Rank-safe under fixed-tolerance rounding.** Over 3000 steps the smooth bump holds at χ = 8, and
  the near-grid-scale steep top-hat settles at χ = 7 from an initial encode of 4. Both settle near
  their static representability rank; neither runs away. The trajectory is flat from step 300
  onward, so this is a settled state and not a slow climb.
- **More diffusion does not increase rank.** The high-ν thickened case peaks at or below the low-ν
  case, 7 against 8, so the thickening lever is at most mildly rank-reducing here.

**Honest limit (why the diffusion lever looks weak).** `QttLinear1d` is **linear**. It transports a
fixed-shape feature but cannot **steepen** one. The strong dynamic-rank threat is a *nonlinear*
shock steepening into a near-discontinuity with dispersive Gibbs growth, and that is not exercised
here; it remains OPEN for Tier-B. Testing it needs a nonlinear Burgers or compressible marcher,
which did not exist when this study ran. So this study establishes the reassuring half, that linear
transport plus rounding is rank-safe, and explicitly scopes the unresolved half.
`qtt_rank_nonlinear` closes it. Analysis: `openspec/notes/plasma-blackout/gap-2/`.

See `output.txt` for the recorded reference output.
