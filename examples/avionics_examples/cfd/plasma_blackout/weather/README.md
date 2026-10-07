[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Weather-Dispersion Table

This example generates a dispersion table for the
[plasma-blackout corridor](../corridor/README.md): six weather conditions, each a **counterfactual world alternated from
one validated baseline description**, flown concurrently through the full coupled physics and reduced to one table. A
flight computer flies lookup tables like this one, and engineers generate them in simulation with a digital twin. Every
dispersion row carries the `!!ContextAlternation!!` audit marker naming the baseline it alternates, so the table carries
its own provenance trail.

## Why this matters

The example performs the environmental-envelope characterization behind an operational-limitations table: hold the
vehicle description constant, vary one declared environmental parameter, measure the operational consequences, record
pass/fail against pinned acceptance criteria, and trace every result back to its condition. That activity usually runs
as a campaign of independent simulation jobs, where "everything else was identical" is a claim in a report. Here it is
one program: every dispersion world shares the validated baseline bit-identically except for its declared difference,
the audit log records that difference, the acceptance criteria are executable gates, and the whole campaign reruns
deterministically (parallel and sequential runs produce identical bits). A certification-by-analysis evidence chain
needs these properties; batch campaigns reconstruct them by hand.

Three gaps separate this from certification-grade evidence. The physics is validated at one flight anchor (RAM-C II),
through the uncalibrated finite-rate ionization network, so off-anchor rows are single-anchor extrapolation. The
parameter set is temperature and density, not the full DO-160-style list (humidity, winds, statistical coverage, and so
on). And this table is one artifact in a certification chain. A sixty-condition version follows the same blueprint.

## How to Run

```bash
cargo run --release -p avionics_examples --example plasma_blackout_weather
```

The campaign is six conditions times eight deterministic receiver-noise draws, 48 full descents, flown concurrently by
the scoped fork-join (`deep_causality_par::scoped_map`), one per core: about three minutes of wall-clock on a 16-core
machine.

## The Question the Table Answers

**Navigation precision versus weather.** In high-altitude avionics, when air pressure and temperature fall far enough,
the INS departs from the behavior assumed under standard conditions. Two mechanisms couple weather to navigation here,
and the table separates them:

1. **The window.** The atmosphere sets the ionization, the ionization sets the blackout window, and the window sets how
   long the dead-reckoning drift integrates. Denser, colder air ionizes earlier (measured onset spread: 2.8 s across the
   table).
2. **The instrument.** The accelerometer bias departs from its calibration point with temperature (a labeled
   tactical-grade thermal coefficient, `1 + 0.01/K` of departure), while the navigation filter keeps its standard-day
   priors in every world. The mismatch between the instrument flown and the instrument assumed is the phenomenon under
   study.

A measured table from the pinned configuration:

| world        | dT (K) | density | IMU departure | onset (s) | dwell (s) | drift in the dark (m) | terminal (m)      |
|--------------|--------|---------|---------------|-----------|-----------|-----------------------|-------------------|
| standard_day | 0      | 1.00    | 1.00          | 10.6      | 55.9      | 42.13 +- 3.05         | 0.125 (max 0.198) |
| hot_day      | +20    | 0.90    | 1.20          | 11.1      | 56.2      | 50.44 +- 1.95         | 0.140 (max 0.210) |
| cold_day     | -25    | 1.10    | 1.25          | 10.2      | 55.6      | 50.86 +- 4.36         | 0.128 (max 0.194) |
| polar_winter | -40    | 1.20    | 1.40          | 9.8       | 55.3      | 58.20 +- 4.09         | 0.130 (max 0.209) |
| thin_day     | -5     | 0.75    | 1.05          | 12.1      | 56.2      | 43.19 +- 1.76         | 0.143 (max 0.207) |
| dense_day    | +5     | 1.30    | 1.05          | 9.3       | 55.5      | 42.86 +- 3.43         | 0.133 (max 0.199) |

Drift and terminal cells are mean plus or minus one sample standard deviation over the eight receiver-noise draws; the
terminal cell also quotes the worst draw. The flow and window columns carry no error bar: the receiver noise never
touches the flow, the chemistry, or the truth trajectory, so they are draw-invariant by construction.

**Why the drift column spans 16 m.** The drift in the dark is dead-reckoning error growth, approximately
`1/2 * b_residual * dwell^2`: the unlearned part of the accelerometer bias integrated over the blackout. Only two
factors can differ between rows: dwell and temperature.

The dwell is the smaller factor. Deceleration physics sets the blackout duration, and these dispersions barely move
it: the dwell spans 55.3 to 56.2 s, a spread of about 1.6 percent, and entering squared it moves the drift by at most
3.3 percent, under 2 m of the sixteen.

The IMU departure accounts for nearly all of the measured deviation. A tactical-grade flight accelerometer is calibrated
(and its thermal compensation fitted) at standard conditions; the further the flight day sits from that calibration
point, the larger the uncompensated bias residue, in either direction. The model is `bias * (1 + 0.01/K * |dT|)`, so
polar winter at 40 K below standard flies 1.40x the calibrated bias, the largest departure in the table because it is
the condition farthest from where the instrument was characterized. The Kalman filter spends the aided descent
estimating the bias, but noise limits that learning, not time: over the pre-blackout fixes at 1 m noise it recovers a
roughly fixed *fraction* of whatever bias is flown, so the unlearned remainder, and with it the drift, inherits the
departure factor almost one-to-one. The check:
`predicted = 42.13 * departure * (dwell/55.9)^2` reproduces every mean in the table within 3.4 percent (57.7
predicted vs 58.20 +- 4.09 measured for polar winter). Each drift cell is the mean over eight deterministic
receiver-noise realizations, so the table quantifies the scatter of the filter's bias learning instead of sampling it
once. On an earlier chemistry pin, a single-draw table showed cold_day 8 percent below its prediction, and the Monte
Carlo mean identified the deviation as one lucky draw. Gate (4b) enforces the statistics:
the polar-standard separation must clear two combined standard deviations, and it clears 3.1.

## Gates

Eight gates pin the study: table integrity, the per-world alternation audit trail, flow-resolved blackout
windows in every weather, the onset spread (weather must move the window), the polar-winter mean drift factor (the
INS-does-not-behave-as-assumed gate), the statistical resolution of that effect (the polar-standard separation must
clear two combined sigma; measured 3.1), worst-draw reacquisition across all 48 descents, and the wall-clock budget.
`exit(1)` on regression, `exit(2)` on setup failure.

## Where Things Live

The physics, constants, stages, and coupling stack are shared with the corridor through
`avionics_examples::shared` (the crate's `src/` library). This example adds its own knobs ([
`constants.rs`](constants.rs): the six conditions, the IMU thermal coefficient, the gate thresholds), the world and row
logic ([`model.rs`](model.rs)), and the study itself ([`main.rs`](main.rs)). Beside the table the run writes
`weather_trace.csv` (draw 0 of each world, one row per coupled step: altitude, plasma frequency, link state, navigation
error, Mach, speed, peak electron density, peak heat flux) and `weather_draws.csv` (every draw's blackout window,
largest drift in the dark, and terminal error).
