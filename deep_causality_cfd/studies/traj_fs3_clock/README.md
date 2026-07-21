<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `traj_fs3_clock` — is the "two-time" parameter proper time, and does a forward `dτ/dt` kernel work?

```bash
cargo run --release -p deep_causality_cfd --example traj_fs3_clock
```

**What it tests.** Gap-3 Resolution-3, de-risking item ⑤. Two findings, one conceptual and one
numerical.

- **Conceptual.** The fictitious time of FS-1, the eccentric anomaly / KS `s` with
  `dt = (r/na)·ds`, is a *regularising reparametrisation*, **not** proper time. Proper time `τ` is a
  separate GR+SR integral. Resolution 1 conflates them ("τ↔t is what 2T physics is about"); they
  are distinct roles, and the spec must carry **two** clocks.
- **Numerical.** The missing forward clock kernel `dτ/dt = 1 + Φ/c² − v²/(2c²)`, with `Φ = −μ/r`, is
  checked against the textbook GPS relativistic split, the canonical falsifiable anchor.

The forward clock rate offset has since been **promoted out of this study into a reusable physics
kernel** (`relativistic_clock_offset_kernel`, capability ⑤). The study now consumes the shipped
kernel rather than defining its own.

**Findings (gated, exit nonzero on regression).** GPS satellite clock against a geoid clock, at an
orbital speed of `v = 3874.0 m/s`:

| effect | measured | textbook |
|---|---|---|
| gravitational (higher potential, so faster) | **+45.65 µs/day** | +45.7 |
| velocity (time dilation, so slower) | **−7.21 µs/day** | −7.2 |
| net | **+38.44 µs/day** | +38.5 |

Reentry blackout carry, vehicle clock against surface over a 180 s denied window at `v = 7650 m/s`
and 71 km altitude: the accumulated `τ − t` offset is **−57.2 ns**, which is **17.2 m** of ranging
drift if uncorrected.

**Conclusion.** The forward `dτ/dt` kernel reproduces the textbook GPS split to sub-µs/day, so
ns-level onboard timing is feasible with the existing constants. The linearising `s` and the proper
time `τ` are **distinct clocks** and the spec must carry both, which is a conceptual fix to
Resolution 1 rather than an implementation detail. Over a 3-minute reentry blackout the uncorrected
clock drifts tens of metres, quantifying why the correction must be carried internally (B3).
Analysis: `openspec/notes/plasma-blackout/gap-3/gap-three-resolution-3-trajectory-axis.md`.

**Caveats.** The GPS split uses a circular orbit and an equatorial-radius geoid clock, the textbook
idealisation, so agreement to sub-µs/day validates the kernel rather than a flight-grade timing
model. There is no J₂, no Earth rotation or Sagnac term, and no eccentricity periodic term. The
blackout carry is a single representative point, and `×0.3 m/ns` is the vacuum-ranging conversion.

See `output.txt` for the recorded reference output.
