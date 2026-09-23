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
the scoped fork-join (`deep_causality_par::scoped_map`), one per core: about four minutes of wall-clock on a 16-core
machine.

## The Question the Table Answers

**Navigation precision versus weather.** In high-altitude avionics, when air pressure and temperature fall far enough,
the INS departs from the behavior assumed under standard conditions. Two mechanisms couple weather to navigation here,
and the table separates them:

1. **The window.** The atmosphere sets the ionization, the ionization sets the blackout window, and the window sets how
   long the dead-reckoning drift integrates. Denser, colder air ionizes earlier (measured onset spread: 4.2 s across the
   table).
2. **The instrument.** The accelerometer bias departs from its calibration point with temperature (a labeled
   tactical-grade thermal coefficient, `1 + 0.01/K` of departure), while the navigation filter keeps its standard-day
   priors in every world. The mismatch between the instrument flown and the instrument assumed is the phenomenon under
   study.

A measured table from the pinned configuration:

| world        | dT (K) | density | IMU departure | onset (s) | dwell (s) | drift in the dark (m) | terminal (m)    |
|--------------|--------|---------|---------------|-----------|-----------|-----------------------|-----------------|
| standard_day | 0      | 1.00    | 1.00          | 11.9      | 58.2      | 45.93 +- 2.35         | 0.15 (max 0.20) |
| hot_day      | +20    | 0.90    | 1.20          | 12.2      | 58.1      | 53.16 +- 2.38         | 0.15 (max 0.22) |
| cold_day     | -25    | 1.10    | 1.25          | 11.1      | 58.9      | 57.72 +- 2.05         | 0.15 (max 0.22) |
| polar_winter | -40    | 1.20    | 1.40          | 9.9       | 60.0      | 68.75 +- 5.19         | 0.16 (max 0.24) |
| thin_day     | -5     | 0.75    | 1.05          | 12.9      | 58.0      | 45.72 +- 1.96         | 0.13 (max 0.20) |
| dense_day    | +5     | 1.30    | 1.05          | 8.7       | 61.1      | 52.03 +- 5.73         | 0.15 (max 0.21) |

Drift and terminal cells are mean plus or minus one sample standard deviation over the eight receiver-noise draws; the
terminal cell also quotes the worst draw. The flow and window columns carry no error bar: the receiver noise never
touches the flow, the chemistry, or the truth trajectory, so they are draw-invariant by construction.

**Why the drift column spans 23 m.** The drift in the dark is dead-reckoning error growth, approximately
`1/2 * b_residual * dwell^2`: the unlearned part of the accelerometer bias integrated over the blackout. Only two
factors can differ between rows: dwell and temperature.

The dwell is the smaller factor. Deceleration physics sets the blackout duration, and these dispersions barely move
it: the dwell spans 58.0 to 61.1 s, a spread of about 5 percent, and entering squared it moves the drift by at most 10
percent, a few meters of the twenty-three.

The IMU departure accounts for nearly all of the measured deviation. A tactical-grade flight accelerometer is calibrated
(and its thermal compensation fitted) at standard conditions; the further the flight day sits from that calibration
point, the larger the uncompensated bias residue, in either direction. The model is `bias * (1 + 0.01/K * |dT|)`, so
polar winter at 40 K below standard flies 1.40x the calibrated bias, the largest departure in the table because it is
the condition farthest from where the instrument was characterized. The Kalman filter spends the aided descent
estimating the bias, but noise limits that learning, not time: over the pre-blackout fixes at 1 m noise it recovers a
roughly fixed *fraction* of whatever bias is flown, so the unlearned remainder, and with it the drift, inherits the
departure factor almost one-to-one. The check:
`predicted = 45.93 * departure * (dwell/58.2)^2` reproduces every mean in the table within about 5 percent (68.3
predicted vs 68.75 +- 5.19 measured for polar winter). Each drift cell is the mean over eight deterministic
receiver-noise realizations, so the table quantifies the scatter of the filter's bias learning instead of sampling it
once. On an earlier chemistry pin, a single-draw table showed cold_day 8 percent below its prediction, and the Monte
Carlo mean identified the deviation as one lucky draw. Gate (4b) enforces the statistics:
the polar-standard separation must clear two combined standard deviations, and it clears 4.0.

## Gates

Eight gates pin the study: table integrity, the per-world alternation audit trail, flow-resolved blackout
windows in every weather, the onset spread (weather must move the window), the polar-winter mean drift factor (the
INS-does-not-behave-as-assumed gate), the statistical resolution of that effect (the polar-standard separation must
clear two combined sigma; measured 4.0), worst-draw reacquisition across all 48 descents, and the wall-clock budget.
`exit(1)` on regression, `exit(2)` on setup failure.

## Where Things Live

The physics, constants, stages, and coupling stack are shared with the corridor through
`avionics_examples::shared` (the crate's `src/` library). This example adds its own knobs ([
`constants.rs`](constants.rs): the six conditions, the IMU thermal coefficient, the gate thresholds), the world and row
logic ([`model.rs`](model.rs)), and the study itself ([`main.rs`](main.rs)).
