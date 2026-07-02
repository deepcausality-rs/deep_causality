[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Weather-Dispersion Table

When a new flight vehicle is built, the lookup tables its flight computer flies are sourced from
somewhere: Contemporary engineering uses a digital twin to create those tables in simulation. 
This example is the digital-twin table generator for the
[plasma-blackout corridor](../plasma_blackout_corridor/README.md): six weather conditions, each
a **counterfactual world alternated from one validated baseline description**, flown
concurrently through the full coupled physics and reduced to one dispersion table. Every
dispersion row carries the `!!ContextAlternation!!` audit marker naming exactly what it is a
counterfactual of, so the table has a provenance trail instead of a folder of filenames.

## Why this matters

The example is the environmental-envelope characterization behind an operational-limitations table. 
Hold the vehicle description constant, vary one declared environmental parameter, measure the
operational consequences, record pass/fail against pinned acceptance criteria, and keep
traceability from every result back to its condition. That activity normally runs as a campaign
of independent simulation jobs whose "everything else was identical". 
Here it is one program, and every dispersion world shares the validated baseline bit-identically 
except for its declared difference, the difference is stamped into the audit log, 
the acceptance criteria are executable gates, and the
whole campaign reruns deterministically (parallel and sequential runs produce identical bits).
Those are precisely the properties a certification-by-analysis evidence chain needs and batch
campaigns reconstruct by hand.

The honest distance to certification-grade evidence is also short to state: the physics is
validated at one flight anchor (RAM-C II), so off-anchor rows are model extrapolation until the
finite-rate chemistry lands; the parameter set here is temperature and density, not the full
DO-160-style list (humidity, winds, statistical coverage,etc); and this table is one artifact in a
certification chain. However, updating the example to a more complete, say sixty-condition
version, follows the same blueprint.  

## How to Run

```bash
cargo run --release -p avionics_examples --example plasma_blackout_weather
```

The campaign is six conditions times eight deterministic receiver-noise draws, 48 full
descents, flown concurrently by the scoped fork-join (`deep_causality_par::scoped_map`), one
per core: about four minutes of wall-clock on a 16-core machine.

## The Question the Table Answers

**Navigation precision versus weather.** It is a known phenomenon in high-altitude avionics
that when air pressure and temperature fall far enough, the INS does not necessarily behave the
way it was assumed to under standard conditions. Two real mechanisms couple weather to
navigation here, and the table separates them:

1. **The window.** The atmosphere sets the ionization, the ionization sets the blackout window,
   and the window sets how long the dead-reckoning drift integrates. Denser, colder air ionizes
   earlier (measured onset spread: 3.1 s across the table).
2. **The instrument.** The accelerometer bias departs from its calibration point with
   temperature (a labeled tactical-grade thermal coefficient, `1 + 0.01/K` of departure), while
   the navigation filter keeps its standard-day priors in every world. The mismatch between the
   instrument flown and the instrument assumed is the phenomenon under study.

A measured table from the pinned configuration:

| world | dT (K) | density | IMU departure | onset (s) | dwell (s) | drift in the dark (m) | terminal (m) |
|---|---|---|---|---|---|---|---|
| standard_day | 0 | 1.00 | 1.00 | 14.3 | 55.6 | 40.79 +- 2.06 | 0.14 (max 0.20) |
| hot_day | +20 | 0.90 | 1.20 | 14.8 | 55.4 | 47.96 +- 2.44 | 0.14 (max 0.22) |
| cold_day | -25 | 1.10 | 1.25 | 13.8 | 56.0 | 51.95 +- 2.75 | 0.15 (max 0.21) |
| polar_winter | -40 | 1.20 | 1.40 | 13.4 | 56.2 | 57.50 +- 1.50 | 0.14 (max 0.20) |
| thin_day | -5 | 0.75 | 1.05 | 16.1 | 54.7 | 40.91 +- 1.61 | 0.14 (max 0.20) |
| dense_day | +5 | 1.30 | 1.05 | 13.0 | 56.4 | 43.82 +- 1.71 | 0.13 (max 0.20) |

Drift and terminal cells are mean plus or minus one sample standard deviation over the eight
receiver-noise draws; the terminal cell also quotes the worst draw. The flow and window columns
carry no error bar because the receiver noise never touches the flow, the chemistry, or the
truth trajectory; they are draw-invariant by construction.

**Why the drift column spans 18 m?.** The drift in the
dark is dead-reckoning error growth, approximately `1/2 * b_residual * dwell^2`: the unlearned
part of the accelerometer bias integrated over the blackout. Only two factors can differ
between rows: Dwell and temperature.

The dwell is not relevant beccase the blackout duration is set by the deceleration physics,
which is robust to these dispersions: the dwell spans 54.7 to 56.4 s, a spread of 1.5 percent,
and entering squared it moves the drift by at most 3 percent, one or two meters of the
eighteen.

The IMU departure accounts for nearly all of the measured deviation.A tactical-grade flight accelerometer is calibrated (and its
thermal compensation fitted) at standard conditions; the further the flight day sits from that
calibration point, the larger the uncompensated bias residue, in either direction. The model is
`bias * (1 + 0.01/K * |dT|)`, so polar winter at 40 K below standard flies 1.40x the calibrated
bias, the largest departure in the table simply because it is the condition farthest from where
the instrument was characterized. The Kalman filter does spend the aided descent estimating the
bias, but that learning is noise-limited, not time-limited: over the ~140 pre-blackout fixes at
1 m noise it recovers a roughly fixed *fraction* of whatever bias is flown, so the unlearned
remainder, and with it the drift, inherits the departure factor almost one-to-one. The check:
`predicted = 40.79 * departure * (dwell/55.6)^2` reproduces every mean in the table within
about 2 percent (58.6 predicted vs 57.50 +- 1.50 measured for polar winter). The error bars are
the certification-grade part: each drift cell is the mean over eight deterministic
receiver-noise realizations, so the scatter of the filter's bias learning is quantified instead
of sampled once. That machinery also settled a diagnosis: an earlier single-draw table showed
cold_day at 47.8 m, 8 percent below its prediction of 51.7 m, and the Monte Carlo mean came
back at 51.95 +- 2.75 m, confirming the deviation was one lucky draw. Gate (4b)
makes the statistics load-bearing: the polar-standard separation must clear two combined
standard deviations, and it clears 6.6.


## Gates

Eight self-verifying gates pin the study: table integrity, the per-world alternation audit
trail, flow-resolved blackout windows in every weather, the onset spread (weather must move the
window), the polar-winter mean drift factor (the INS-does-not-behave-as-assumed gate), the
statistical resolution of that effect (the polar-standard separation must clear two combined
sigma; measured 6.6), worst-draw reacquisition across all 48 descents, and the wall-clock
budget. `exit(1)` on regression, `exit(2)` on setup failure.

## Where Things Live

The physics, constants, stages, and coupling stack are shared with the corridor through
`avionics_examples::blackout` (the crate's `src/` library). This example adds only its own
knobs ([`constants.rs`](constants.rs): the six conditions, the IMU thermal coefficient, the gate
thresholds), the world and row logic ([`model.rs`](model.rs)), and the study itself
([`main.rs`](main.rs)).
