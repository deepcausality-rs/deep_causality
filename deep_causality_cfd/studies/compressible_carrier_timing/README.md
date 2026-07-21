<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `compressible_carrier_timing` — can the compressible carrier hold the corridor budget? (Task 0)

```bash
cargo run --release -p deep_causality_cfd --example compressible_carrier_timing
```

**What it tests.** Task 0 of `add-compressible-blackout-carrier`, and a go/no-go rather than a
physics probe. Can the body-fitted 3-D compressible marcher carry the plasma-blackout corridor
inside the minutes-not-hours budget? Everything downstream hangs on two measured numbers per
configuration:

1. **Per-step wall-clock** of `CompressibleMarcher3dFitted::step`. The corridor marches roughly 200
   coupled steps, three leg spans plus the counterfactual branch study, so the projected corridor
   time is `per_step × 200` plus coupling overhead.
2. **Assembly cost** of the marcher, meaning metric plus acoustic inverse. The continuous-descent
   host rebuilds the solver when the scheduled freestream drifts, so the rebuild budget in
   "equivalent steps" sets the freestream-drift tolerance.

The budget is 200 corridor steps inside **600 s**. The 3-D shell is measured at the smallest
candidate only, 16³ at bond cap 16, because those numbers already decide the family and larger 3-D
grids are a foregone conclusion. The 2-D fallback is swept at two resolutions and two bond caps.

**Findings (gated, exit nonzero on regression).**

| case | grid | bond cap | assembly | per-step | projected corridor | verdict |
|---|---|---|---|---|---|---|
| 3d-fitted | 16³ | 16 | 0.004 s | 10.805 s | 2161.0 s (36.0 min) | **over budget** |
| 2d | 32² | 16 | 0.000 s | 0.027 s | 5.4 s (0.1 min) | fits |
| 2d | 32² | 32 | 0.000 s | 0.042 s | 8.3 s (0.1 min) | fits |
| 2d | 64² | 16 | 0.000 s | 0.062 s | 12.4 s (0.2 min) | fits |
| 2d | 64² | 32 | 0.001 s | 0.175 s | 35.0 s (0.6 min) | fits |

- **The 3-D fitted shell is out, by more than 3×.** Even the smallest candidate projects to 36
  minutes against a 10-minute budget.
- **The 2-D fallback fits with room to spare.** The largest in-budget configuration runs the
  corridor in 35 s, a 17× margin.
- **Assembly is free at this scale.** One rebuild costs at most 0.01 equivalent steps, and roughly
  10 rebuilds per run add **0.04 %** to the march. Freestream-drift rebuilds need no rationing.

**Conclusion: GO, the corridor carrier is 2-D at 64², bond cap 32** (0.175 s/step, peak bond 32).
The gate is that at least one configuration must fit, and the recommended configuration, the
largest one inside budget, is printed as the go/no-go record. A regression where nothing fits exits
nonzero, which is the documented trigger for revisiting the `CompressibleMarcher2d` fallback
decision.

**Caveats.** Wall-clock on one machine (Apple M3 Max, release), so re-measure on the target
hardware. The projection is `per_step × 200` and excludes coupling overhead, making it a floor
rather than a forecast. Five timed steps per case after one untimed warmup is a small sample, and
the 3-D verdict is extrapolated from a single grid by design.

See `output.txt` for the recorded reference output.
