[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Weather-Dispersion Table

When a new flight vehicle is built, the tables its flight computer flies are sourced from
somewhere: either a prototype flies into the ground until the envelope is mapped, or a digital
twin maps it in simulation. This example is the digital-twin table factory for the
[plasma-blackout corridor](../plasma_blackout_corridor/README.md): six weather conditions, each
a **counterfactual world alternated from one validated baseline description**, flown
concurrently through the full coupled physics and reduced to one dispersion table. Every
dispersion row carries the `!!ContextAlternation!!` audit marker naming exactly what it is a
counterfactual of, so the table has a provenance trail instead of a folder of filenames.

## How to Run

```bash
cargo run --release -p avionics_examples --example plasma_blackout_weather
```

The whole six-world table costs one descent of wall-clock (about 50 s): the scoped fork-join
(`deep_causality_par::scoped_map`) flies all six atmospheres concurrently, one per core.

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
| standard_day | 0 | 1.00 | 1.00 | 14.3 | 55.6 | 40.8 | 0.11 |
| hot_day | +20 | 0.90 | 1.20 | 14.8 | 55.4 | 48.8 | 0.10 |
| cold_day | -25 | 1.10 | 1.25 | 13.8 | 56.0 | 47.8 | 0.10 |
| polar_winter | -40 | 1.20 | 1.40 | 13.4 | 56.2 | 58.2 | 0.12 |
| thin_day | -5 | 0.75 | 1.05 | 16.1 | 54.7 | 42.5 | 0.11 |
| dense_day | +5 | 1.30 | 1.05 | 13.0 | 56.4 | 43.8 | 0.12 |

The polar-winter row is the star: the densest window and the largest thermal departure compound
to 1.43x the standard-day blackout drift. The thin day pits the mechanisms against each other
(less ionization, but less deceleration too). Every world reacquires to about 0.1 m once the
sheath clears, which is itself a table-worthy result: weather moves the *exposure*, not the
recovery.

## Gates

Seven self-verifying gates pin the study: table integrity, the per-world alternation audit
trail, flow-resolved blackout windows in every weather, the onset spread (weather must move the
window), the polar-winter drift factor (the INS-does-not-behave-as-assumed gate), universal
reacquisition, and the wall-clock budget. `exit(1)` on regression, `exit(2)` on setup failure.

## Where Things Live

The physics, constants, stages, and coupling stack are shared with the corridor through
`avionics_examples::blackout` (the crate's `src/` library). This example adds only its own
knobs ([`constants.rs`](constants.rs): the six conditions, the IMU thermal coefficient, the gate
thresholds), the world and row logic ([`model.rs`](model.rs)), and the study itself
([`main.rs`](main.rs)).
