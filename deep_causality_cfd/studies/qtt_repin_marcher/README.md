<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_repin_marcher` — what actually bounds the marched rank? (Res 5 / D9, the Stage-4 core)

```bash
cargo run --release -p deep_causality_cfd --example qtt_repin_marcher
```

**What it tests.** `qtt_rank_fitted_dynamic` measured the negative half. A **static** body-fitted
coordinate is not enough: under Cartesian fluxes a marched radial front drifts off a fixed
curvilinear chart and the bond climbs with resolution, 25 to 35, no better than the capture. The
fix Resolution 5 / D9 prescribes is **feedback re-pinning**, tracking the live front and moving
the coordinate with it so the feature stays coordinate-stationary. This study prototypes that and
asks whether it is sufficient.

**Method.** Each step, after one Burgers update in the polar frame, the study locates the front at
the steepest radial gradient, rolls the field back to a fixed computational band `η = ½`, and
slides the annulus inner radius `r₀` so the front's physical radius maps to that band. It then
rebuilds the low-rank metric for the new `r₀`. The front therefore never moves in computational
space.

**Findings (gated, exit nonzero on regression).** Peak bond at 64² and 128²:

| case | L=6 | L=7 | |
|---|---|---|---|
| **Part 1**: Cartesian fluxes marched *through* the curved front | | | |
| static body-fitted | 25 | 35 | +10 with resolution |
| **re-pinned (D9)** | 25 | 35 | +10; 8 re-pins at L6, 18 at L7 |
| **Part 2**: coordinate-*aligned* radial transport on a re-pinned tracked interface | | | |
| fitted + tracked | 8 | 8 | flat in resolution |

- **Re-pinning alone does not curb the growth.** The re-pinned march grows by the same +10 as the
  static chart, despite 18 re-pins at 128². The rank driver is therefore not the front's drift.
- **The driver is the flux, not the coordinate.** What inflates the bond is the angular structure
  a Cartesian-flux march injects by carrying fluxes **through** a curved front.
- **Aligning the transport with the coordinate is what works.** Radial flux, with the front as a
  tracked interface, holds bond 8 / 8 flat in resolution on that same re-pinned marcher.

**Conclusion.** The Stage-4 mechanism is re-pin **and** treat the front as an exact
Rankine–Hugoniot interface: smooth on each side, fluxes never marched across it. Re-pinning is
necessary but not sufficient. This pins down precisely which half of the Res-5 lever does the
work. Analysis: `openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** Scalar Burgers again, not the compressible Euler/NS system. The tracked-interface
case transports radially by construction, which is the idealisation an exact-RH interface would
have to earn in a real solver. Two resolutions show a direction, not a law.

See `output.txt` for the recorded reference output.
