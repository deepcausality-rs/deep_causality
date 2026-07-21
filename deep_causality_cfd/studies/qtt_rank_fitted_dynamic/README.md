<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_fitted_dynamic` — does the rank lever survive *marching*? (Res 5)

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_fitted_dynamic
```

**What it tests.** `qtt_rank_nonlinear` measured that a *captured* 2-D curved shock raises rank as
it forms, 7 to 20. `qtt_rank_3d` measured the Cartesian upper bound `χ ~ √side`. Both left the
decisive cell open. Does a marcher that keeps the feature **aligned to a coordinate axis** hold
the bond bounded *and* resolution-independent over the march, the thing Resolution 5 claims "by
construction"? Three marched viscous-Burgers cases answer it, each at two resolutions, so the
**resolution scaling** is the headline rather than any single number.

**Findings (gated, exit nonzero on regression).** Peak `max_bond` over the march:

| case | L=6 | L=7 | reading |
|---|---|---|---|
| Cartesian **curved** (misaligned) | 20 | 25 | the threat: grows with resolution |
| Axis-**aligned** planar | 7 | 7 | alignment: low and flat, the lever |
| **Static** body-fitted polar march | 25 | 35 | set once: grows, so needs D9 re-pin |

- **The captured threat is real and dynamic.** The misaligned curved shock grows 20 to 25 with
  resolution, closing the cell `qtt_rank_nonlinear` left open.
- **Alignment bounds the bond under marching.** The axis-aligned front holds 7 / 7, flat in
  resolution. When the feature stays on a grid axis throughout, the marcher bounds the bond by
  construction. That is the Res-5 lever, confirmed dynamically rather than statically.
- **A static body-fitted coordinate is not enough.** Set once, the fitted chart grows 25 to 35, no
  better than the capture. Cartesian fluxes evolve the front off the fixed coordinate, so the rank
  climbs anyway.

**Conclusion.** Alignment is the lever; maintaining it is the mechanism. A one-time fitted
coordinate does not stay aligned once the flux march moves the front, which makes Resolution 5's
**feedback re-pinning (D9)** necessary rather than optional. `qtt_repin_marcher` carries the
finding further and shows that re-pinning alone is still not sufficient. Analysis:
`openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** Viscous Burgers is a scalar shock-former, not compressible Euler/NS. 64² and 128² are
small grids chosen for runtime, so the absolute bonds are lower bounds that grow with `L`. Two
resolutions establish a direction, not a scaling law; the `√side` law is read in `qtt_rank_3d`.

See `output.txt` for the recorded reference output.
