[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Plasma-Blackout Corridor

This example flies a Mach-28 reentry through plasma blackout as **one continuous coupled descent** in a single composed
coupling: compressible flow, nonequilibrium plasma chemistry, regime classification, GNSS-denied navigation,
counterfactual guidance, and a cybernetic safety gate step together and write into one auditable provenance log.

The shock layer ionizes, and past a critical electron density the plasma sheath cuts every GNSS link; NASA's RAM-C II
flight measured this blackout in 1970. Through the dark the vehicle dead-reckons on its inertial navigation while a
bounded-correction gate keeps the bank command inside the certified envelope. When the sheath clears, one position fix
collapses the accumulated drift.

Continuous here means the *coupled field*. The navigation state, the truth trajectory, the evolved projections, and the
provenance log cross every leg boundary intact. The marched fluid layer does not. Resuming a leg through
`.from(pause.state())` rebuilds that leg's carrier and re-quantizes the world's uniform seed, so the evolved conserved
state, the inflow strip, and any acoustic-envelope drift the previous leg earned are discarded, and the layer
re-converges over the following steps. The descent crosses three such boundaries and logs each one: `leg re-seeded
from step N in world 'W': coupled field carried, marched fluid state re-seeded from the world
seed`. The quasi-steady closure (below) makes that acceptable at this fidelity; it does not provide fluid continuity.

The run verifies itself against the RAM-C II flight anchor and exits nonzero on any regression. It takes about 45
seconds.

## How to Run

From the repository root:

```bash
cargo run --release -p avionics_examples --example plasma_blackout_corridor
```

Precision is a parameter. `main.rs` carries a single alias, and the whole corridor (flow, plasma, navigation, control)
is generic over it:

```rust
pub type FloatType = f64; // or deep_causality_num::Float106
```

The counterfactual branch study fans out over scoped threads through the workspace `parallel`
feature (`deep_causality_par::scoped_map`; no external dependency). Results are bit-identical to the sequential run.

## What Happens When You Run It

The vehicle starts at 90 km at Mach 28, on a steep compressed trajectory sized so it crosses the RAM-C II anchor
altitude (71.93 km) at the flight's 7.66 km/s, through the US Standard Atmosphere 1976. Four legs and one branch study follow. The run finds every boundary as an event; no station switch is
scripted. Each of the three leg boundaries carries the coupled field and re-seeds the marched layer:

1. **Descent to blackout onset.** The evolved sheath's electron density climbs as the air thickens; at 76.8 km it
   crosses the GPS L1 cutoff and the classifier flips the link to DENIED. The march pauses on that flow-resolved event.
   The onset altitude is a prediction: no onset constant exists anywhere in the corridor.
2. **The counterfactual study, in two rounds.** The paused state forks once per candidate bank command (a six-candidate
   coarse sweep: 0, 5, 10, 15, 20, and 40 degrees), in O (1) through copy-on-write, and the scoped fan-out flies all six
   concurrently in the wall-clock of one branch. Each branch flies the *same* onset state in its own alternated world,
   scored by its trajectory-derived miss to a shared aim point. The coarse landscape descends 30.0, 16.5, down
   to 3.8 m at 10 degrees, then rises again through 11.5 and 25.0 m: the 40-degree command exceeds the envelope's
   0.5 rad cap, the gate bounds it every step, and the clamped branch overshoots to 48.4 m. A **fine round** then forks
   the same paused onset a second time, eleven 0.5-degree candidates bracketing the coarse winner, scored against the
   same aim: the landscape bottoms at 11.0 degrees with a 2.87 m miss. Two fork rounds resolve the optimum at 0.5-degree
   resolution for seventeen branches total. The 2.87 m residual sits 0.76 m above the vehicle's knowledge floor, the
   2.11 m INS drift when the branches are scored 100 steps after the fork (see "Why the sweep stops at 0.5 degrees and
   2.87 m" below).
3. **The committed dwell.** The winning world flies through the peak passage. At the RAM-C II anchor altitude, 71.93 km,
   the vehicle crosses at 7662 m/s against the flight's 7660, and the evolved peak electron density lands at 6.8e18 per
   cubic meter against the flight's station-1 Ka-band datum of 9.9e18 (−0.16 decades), inside the ±0.7-decade
   chemistry-spread allowance, with **no calibration target anywhere in the chemistry**. The INS dead-reckons; drift grows
   from 0.31 m at onset to 3.07 m at the 61 km passage and 42.8 m on the last step before the link returns
   (`corridor_trace.csv`). The descent's peak electron density, 2.8e20 per cubic meter, comes at 44.4 km, inside the
   blackout.
4. **Flow-resolved exit and reacquisition.** Drag decelerates the vehicle below the ionization threshold; at 45.7 km the
   renewed sheath stops ionizing past the cutoff and the link returns. Dissociative recombination
   `NO+ + e- -> N + O` drains the sheath and ends the blackout. The first fix folds on the exit step and cuts the error
   to 1.56 m; by the end of reacquisition it is 0.20 m.

Fourteen coupled validation gates then check the whole story: window ordering, the anchor comparison at matched
altitude and speed (with the 61 km value reported beside the flight's lower bound), the window altitudes (exit inside
its pinned band, reported against the RAM-C II 25-30 km flight window), the descent's peak inside the blackout, drift
and reacquisition, regime change, the multiphysics chain, real steering divergence, guidance precision from the sweep
(the committed branch must beat the ballistic miss at least 3x; it lands 10.5x better), the fine round refining the
coarse winner, tensor compression under the bond cap, bounded solver rebuilds, and the wall-clock budget.

## The Causal Chain

```text
[1] flow        CompressibleCarrier: 2-D compressible marcher on tensor trains; the truth
                vehicle's altitude and Mach select the freestream from an atmosphere schedule;
                the exact Rankine-Hugoniot jump is enforced on the inflow strip
[2] regime      RegimeClassify: freestream Knudsen number -> governing model;
                evolved n_e -> plasma frequency -> GNSS available / DENIED
[3] plasma      finite-rate ionization network on the EVOLVED state: Millikan-White clock on
                the evolved per-cell pressure, the three-channel RP-1232 network (associative
                ionization + dissociative recombination, electron impact, lagged atom pool with
                the Zeldovich exchange) on the evolved per-cell density, each rate at its
                controlling temperature, sheath renewal at the transit-age profile's peak
[4] navigation  TrajectoryNav: KS-regularized orbit predict with the aero-force channel as the
                kick; 17-state error-state Kalman filter; fixes gated by the REAL blackout flag;
                relativistic clock offset carried through the outage
[5] branches    run_until pauses at the flow-resolved onset; O(1) copy-on-write forks in
                two rounds (coarse sweep, then a 0.5-deg fine sweep around its winner);
                !!ContextAlternation!! per branch; trajectory-derived miss distances
[6] control     CyberneticCorrect clamps the commanded bank into the SafetyEnvelope;
                BankSteeredLift FLIES the clamped command (point-mass 3-DOF lift, one-step
                actuation lag)
[7] provenance  one EffectLog rides the coupled field end to end: regime transitions, nav-mode
                changes, solver rebuilds, bounded corrections, alternation markers
```

## What This Example Demonstrates

**Tensor-train compression as the mesh strategy.** Blackout is a scale-separation problem: the vehicle measures meters,
the sheath structure far less. Instead of adaptive mesh refinement, the carrier marches on quantized tensor trains, where
a `2^L` grid costs order `chi^2 * L`: logarithmic in point count, with sharp structure paid for only in bond dimension.
The rank studies in `deep_causality_cfd/studies/` found that *coordinate alignment* drives rank, sharpness does not. A
Cartesian-captured curved shock grows unboundedly in rank; a shock-aligned coordinate holds it at order 10. This example
uses a **shock-fitted inflow strip**: the exact Rankine-Hugoniot state is the *boundary* of the marched layer, so the
shock is never captured, and the final evolved state re-quantizes at peak bond 16 against a cap of 16.

**Uncalibrated finite-rate ionization chemistry.** The sheath chemistry is the three-channel RP-1232 (Gupta et al., NASA
1990) network with **no calibration target anywhere**: associative ionization `N + O -> NO+ + e-` with its
dissociative-recombination reverse (the physical blackout-exit mechanism), thresholded electron-impact ionization, and a
lagged neutral atom pool whose nitrogen clock carries the low-activation Zeldovich exchange `N2 + O -> NO + N`. Every
rate runs at its controlling temperature: ionization at the geometric mean
`sqrt(T_tr * T_ve)`, dissociation at Park's published `T_tr^0.7 * T_ve^0.3`, electron channels at `T_e = T_ve`, with the
Millikan-White relaxation clock on the **evolved per-cell pressure**
and the network on the **evolved per-cell density**. The network *predicts* the RAM-C II anchor from cited rate pairs
and geometry alone and lands 1.65x (+0.22 decades) on the stagnation line at the cited 71 km freestream, inside the band
production codes (DPLR, LAURA, US3D) achieve on the same peak. The sheath exposure is the transit-age profile's observable peak (`age(xi) =
t_res * ln(1/(1-xi))` from the linear stagnation-line deceleration; the reflectometer-visible near-body gas has aged ~
4.2 residence times). Sheath renewal stays in the flown closure: under recombination the carried mode self-limits
without it, but the renewal arm's fixed-point clock is the network's true Riccati timescale.

**Two-way flow-navigation coupling.** Navigation feeds flow: the truth vehicle's position and speed select the
freestream from the US Standard Atmosphere 1976 table, and the resulting
Rankine-Hugoniot jump drives the inflow strip each step. Flow feeds navigation: the evolved electron density gates which
measurements the Kalman filter may fold. When the scheduled inflow outgrows the solver's acoustic envelope, the carrier
rebuilds itself and logs the rebuild to provenance.

**GNSS-denied navigation.** Relativity matters in this problem on the trajectory axis, and only there: the
KS-regularized conformal propagator advances the orbit, a 17-state error-state Kalman filter (position, velocity,
attitude, accelerometer and gyro bias, clock) folds fixes when the link is up, and the relativistic clock offset is
carried internally through the outage with no satellite to reset it. The dead-reckoning drift comes from the real INS
mechanism: a tactical-grade accelerometer bias corrupting the sensed specific force, growing as t^2 through the dwell.

**Counterfactuals with the verbatim core vocabulary.** The branch study uses the same
`fork` / `alternate_context` / `continue_march` machinery as every other DeepCausality counterfactual: forks share the
paused tensor state by reference and clone copy-on-write at first write, each branch's log carries the
`!!ContextAlternation!!` marker, and candidate commands ride as world-published constants so branches differ only in the
world they fly. Branch misses are trajectory-derived: the distance from each branch's terminal truth state to a shared
aim point, with the analytic t^2 drift law printed beside it as a cross-check.

**Why the sweep stops at 0.5 degrees and 2.87 m.** A finer sweep would separate neighboring branches by less than the
vehicle's navigation error. The INS dead-reckoning error in this same run is 2.11 m when the branches are scored, 100
steps after the fork. Near the minimum, neighboring 0.5-degree candidates differ by 0.22 m (10.5 to 11.0 degrees),
0.40 m (11.0 to 11.5) and 0.84 m (11.5 to 12.0), and the 2.87 m guidance residual sits 0.76 m above the drift. The
error keeps growing through the rest of the blackout, to 42.8 m before the link returns. Steering
more precisely than the vehicle navigates buys nothing, because a real vehicle commands off the navigated state. (The
sweep scores against truth terminal states, which a flight system cannot see, so 2.87 m is optimistic.) The residual
itself is geometric: a single constant bank command traces a one-dimensional curve of reachable terminal states through
a 3-D miss space. Fitted through the eleven fine-round terminal states (a cubic per coordinate), that curve passes 2.86 m
from the aim at 10.9 degrees, 0.01 m closer than the committed branch.

**A cybernetic safety gate that steers.** `CyberneticCorrect` runs a
`CyberneticLoop::control_step` against the verified `SafetyEnvelope` each step and clamps the commanded bank into it;
`BankSteeredLift` then flies the clamped command as a point-mass 3-DOF lift vector rotated about the velocity by the
bank angle. The clamped command is the actuation. An unrecoverable envelope breach short-circuits the step.

## Validation Anchors

- **RAM-C II (Grantham 1970, NASA TN D-6062)**: the canonical ionized-reentry electron-density dataset. The anchor is
  the flight's station-1 Ka-band critical-density crossing, 9.93e18 per cubic meter at 71.93 km. The gate compares the
  corridor at that altitude, crossed at the flight's speed, within a ±0.7-decade chemistry-spread allowance; this run
  lands at 6.8e18 (−0.16 decades) on the evolved state with no calibration target. Below 71.93 km the flight's nose
  plasma is overdense, so the datum is a lower bound there; the corridor reports its 61 km value (1.3e20) beside it, and
  the stagnation-line harness gates the bound at RAM-C II's own 61 km freestream. The exit altitude (45.7 km) is gated
  in its own pinned band and reported against the flight's 25-30 km recovery window; the offset is the probe's
  deliberately light ballistic bundle, not chemistry.
- **Gupta-Yos-Thompson-Lee, NASA RP-1232 (1990)**: the Table II rate pairs behind every network channel (forward and
  backward tabulated together, detailed balance by construction).
- **Park's two-temperature model**: the `T_tr` / `T_ve` split and the controlling-temperature closures, including the
  published `q = 0.7` dissociation exponent.
- **Millikan-White vibrational relaxation** for the lagging bath; **Sutton-Graves** for stagnation-point heating.
- **US Standard Atmosphere 1976** for the descent table, at 1 km spacing, generated from the standard's defining
  constants (Tables 2 and 4) and checked against its Table I.

## Precision Is a Parameter

Because every derived number is computed in `FloatType`, the alias is a one-line probe of the error budget. Three runs,
same corridor (the precision study was recorded on the surrogate-era build; the network keeps the same SI-unit exponent
ranges that set its conclusion):

> **The `Float106` row does not reproduce on the current build.** Switching the alias fails to
> compile: 44 errors, most of them in this example's own `model.rs`, `main.rs` and
> `shared/utils.rs`, plus eight in the crate. The example contains `f64`-specific code.
> `turbulence_flow` demonstrates precision as a parameter end to end: it runs all three precisions
> from one rate field.

| Alias                | Outcome                                                                                                                                                                                                   |
|----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `f64`                | All gates pass in about 45 s. The default.                                                                                                                                                                 |
| `Float106` (106-bit) | Every gate and every discrete event step identical; continuous witnesses agree to 15-16 significant digits; about 11x the wall-clock.                                                                     |
| `f32`                | Crashes at step 1: `h^2` in the then-flown Saha kernel (4.4e-67) underflows the f32 exponent range, and the position ulp at Earth radius (0.5 m) would swallow the sub-meter navigation story regardless. |

`constants.rs` records the conclusion: the model closures and the grid set this corridor's error budget, and
floating-point round-off does not. `f64` carries the needed exponent range for SI-unit plasma constants at the bottom and
wastes nothing at the top.

## Limitations

Every simplification is documented in [`constants.rs`](constants.rs).

1) The chemistry is the finite-rate three-channel network as a single-point sheath closure (rates from RP-1232 Table II,
   valid to ~8 km/s; `T_e = T_ve` lumped; NO treated as transient; no spatially resolved reacting layer);

2) Time is compressed (each coupled step represents 0.1 s of flight, and the layer is quasi-steady per instant);

3) The marched layer is 2-D with the 3-D fitted marcher reserved for stagnation-line validation (a timing study showed
   it 5.1x over the minutes budget);

4) The flight corridor is a deterministic point-mass 3-DOF world with a fixed atmosphere. There are no winds, no
   aero-coefficient dispersions, and no density perturbations;

5) A leg boundary re-seeds the marched fluid layer. The coupled field is carried across it; the evolved conserved state,
   the inflow strip, the acoustic-envelope drift, and the leg's own rebuild count are not. The layer restarts from the
   world's uniform seed at each of the three boundaries and re-converges within a few steps, which the quasi-steady
   closure of (2) accepts

The counterfactual branches are exact because the world is deterministic, which follows partly from these limitations.
For higher fidelity, any step can be replaced with a different physics kernel, marcher, or coupling mechanism.
A [companion note](../../../../../openspec/notes/archive/cfd-plasma-blackout/finite-rate-ionization-chemistry.md)
documents the ionization-chemistry limitation.

## Where Things Live

| File                               | Contents                                                                                                                                  |
|------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
| [`main.rs`](main.rs)               | The descent: four legs, the branch study, provenance, the gates                                                                           |
| [`model.rs`](model.rs)             | The descent worlds, the bank commands, branch scoring, the leg snapshots, and both gating sequences (4 campaign + 10 leg = fourteen gates) |
| [`constants.rs`](constants.rs)     | The corridor's own knobs: the horizon, the bank sweep, the gate thresholds                                                                |
| [`utils_print.rs`](utils_print.rs) | Console rendering: the intro, the legs, the branch tables, the provenance                                                                 |
| `corridor_trace.csv`               | Written by the run: one row per coupled step of the flown descent, labelled by leg (altitude, Mach, `n_e`, link state, regime, nav error, loads, bank, position) |
| `corridor_branch_trace.csv`        | Written by the run: the same columns for every branch of both rounds, from the fork to the end of its continuation                       |

The physics constants, the numeric helpers, the example-local stages, and the coupling stack are shared with
the [weather-dispersion example](../weather/README.md) through the crate library module `avionics_examples::shared`
(under `examples/avionics_examples/src/`), which also carries the precision notes and the `FloatType` switch.

The library machinery this example exercises lives in `deep_causality_cfd` (the compressible carrier, the coupled-loop
seam, the corridor stages, the navigation engine) and
`deep_causality_par` (the scoped fork-join). 