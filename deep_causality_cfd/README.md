<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# DeepCausality CFD: Counterfactual Fluid Dynamics

DeepCausality CFD provides Counterfactual Fluid Dynamics and Multidisciplinary analysis and optimization (MDAO) by coupling fluid dynamics, multiple physics, navigation, and control, in one typed dynamic process. 
DeepCausality CFD couples several disciplines' analyses, optimizes over the coupled result, 
and keeps track of the uncertainty along the way: the plasma-blackout
example marches a compressible flow, reacts its plasma chemistry, gates a Kalman filter on the
result, flies the control command it selects, and picks that command by forking the running
simulation into counterfactual worlds. Multiple solvers, multi-physics, multi-regime,
counterfactual dynamics, and precision as a parameter, in one crate.

## Usage

The crate is unpublished, but you can add it [as a git dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories):

you can pin the repo by:
* branch e.g. 'branch = "main"'
* tag e.g. 'tag = "0.10.3"'
* commit e.g. 'rev = "0c09903..."'

```toml
[dependencies]
deep_causality_cfd = { git = "https://github.com/deepcausality-rs/deep_causality.git", branch = "main" }

```

The fastest way to see the whole crate work end to end:

```bash
cargo run --release -p avionics_examples --example plasma_blackout_corridor
```

## Three Solver Paradigms, One Language

**Calculus-based: the DEC-native Navier-Stokes solver.** Velocity lives as an edge 1-form on a
discrete exterior calculus. Each time step marches the Leray-projected rate, so the field stays
divergence-free at every step, and the `SolenoidalField` type-state rejects time-stepping an
unprojected field at compile time. Validated against Taylor-Green decay, exact Couette and
Poiseuille states, the Ghia et al. (1982) lid-driven cavity tables, and cylinder wake
references.

**Compression-based: the QTT marchers.** The compressible Euler marchers (1-D through 3-D,
including a body-fitted variant) run on quantized tensor trains, where a `2^L` grid costs order
`chi^2 * L`: logarithmic in point count, with sharp structure paid for in bond dimension. The
rank studies in `studies/` measured the decisive caveat (the rank driver is coordinate
alignment, not sharpness), and the compressible carrier answers it with a shock-fitted inflow
strip: the exact Rankine-Hugoniot state is the boundary of the marched layer, so the shock is
never captured at all.

**Analytic and pointwise: fitted closures.** Exact Rankine-Hugoniot jumps, the Park
two-temperature relaxation closures, the finite-rate ionization network, and the pointwise
Navier-Stokes regime evaluators with their causal-effect wrappers. A stagnation line with a
fitted shock runs entirely on these, with no grid.

All three families sit behind the same `CfdFlow` language and the same scalar type, so one
program can pick per problem: the DEC solver for an incompressible cavity, the QTT marcher for
a reentry layer, a fitted closure for the stagnation line.

## Counterfactual Dynamics

`CfdFlow` is a two-level language. At the **trajectory** level, `CfdFlow::march` marches a coupled
run until a predicate fires and yields a resumable pause; at the **campaign** level,
`CfdFlow::study` runs a family of counterfactual cases forked from that pause to a `Verdict`. A
fork shares the paused state in O(1) through copy-on-write (tensor fields, navigation engine, and
provenance log included) and continues each branch in its own alternated world. From the
plasma-blackout corridor, condensed:

```rust
// Trajectory level: march until the evolved sheath's n_e crosses the GPS L1 cutoff.
let onset = CfdFlow::march(&nominal)
    .couple(world::corridor_coupling(1.0, 0))
    .trigger(trigger)
    .from_field(world::initial_field())
    .until(|field, _| field.regime().map(|r| r.gnss_denied).unwrap_or(false))?;

// Campaign level: fork the paused onset once per candidate bank command, fly every
// branch concurrently, reduce to scored rows — then refine from the *same* onset with
// 0.5-degree candidates around the coarse winner, and gate the whole two-round result.
let corridor = CfdFlow::study("bank-angle corridor")
    .cases(model::coarse_commands())
    .fork(&onset)                            // the shared flow-resolved fork point
    .branch(model::bank_world)               // one alternated world per command, marked
    .continue_for(constants::BRANCH_STEPS)   // concurrent, copy-on-write
    .reduce_all(model::score_branches)       // aim point from the ballistic branch
    .refine(&onset, model::fine_candidates)  // second round, same paused onset
    .branch(model::bank_world)
    .continue_for(constants::BRANCH_STEPS)
    .reduce_all(model::score_branches)
    .gates(model::corridor_gates())          // steering beats ballistic; fine ≥ coarse
    .verdict()?;
```

The corridor commits the best of the seventeen worlds (six coarse + eleven fine) mid-descent.
Every branch stamps a `!!ContextAlternation!!` marker into its provenance log naming its baseline;
branch fan-outs run concurrently on scoped threads and produce bits identical to the sequential
run. The sibling weather-dispersion table takes the other counterfactual form —
`.baseline(standard_day).alternate(weather_world).ensemble(draws).couple(..).march_for(..)
.reduce_ensemble(..)` — flying six atmospheres alternated from one baseline, each an ensemble of
receiver-noise draws. The gating sequence is a named value the study inserts whole
(`GateSeq<Row>`), the DSL never exits or prints (`verdict()` returns data), and an optional
`save_log(path)` flushes each run's provenance to disk, one file per branch under a fan-out.

## Native Multi Regime

The regime is a classified property of the evolved state, re-decided every step. A vehicle
entering or leaving orbit transitions dynamically through several regimes, and this crate
switches three regime axes independently, each on a measured quantity:

- **Flow regime.** `RegimeClassify` turns the freestream Knudsen number into the governing
  model: continuum Navier-Stokes, slip-corrected continuum, transitional, or free-molecular.
- **Dynamics regime.** The navigation engine switches integrators on the force ratio
  `ε = a_aero/a_grav`: while gravity dominates, the trajectory advances on the exact
  KS-conformal core with aero as a between-step kick (Encke); once aero dominates, it switches
  to direct Cowell integration. This handles orbit entry and exit, where the integrator that is
  exact in orbit loses accuracy in the atmosphere.
- **Link regime.** The evolved electron density sets the plasma frequency, and the plasma
  frequency decides whether the GNSS link exists. The Kalman filter's measurement gating
  follows it.

The regime is read straight off the evolved field, so the DSL turns a regime property into an
**event the run finds** rather than a station it is told to switch at:

```rust
// `field.regime()` -> Option<&RegimeClass> { model, knudsen, plasma_frequency, gnss_denied }.
// Blackout is therefore an interval the run discovers: march to the onset event, fly the
// committed world through the dark, and continue to the recovery event.
let onset = CfdFlow::march(&nominal)
    .couple(world::corridor_coupling(1.0, 0))
    .from_field(world::initial_field())
    .until(|f, _| f.regime().map(|r| r.gnss_denied).unwrap_or(false))?;   // link lost

let exit = CfdFlow::march(&nominal)
    .alternate_context(&committed)
    .couple(world::corridor_coupling(1.0, 0))
    .from(peak.state())
    .until(|f, _| f.regime().map(|r| !r.gnss_denied).unwrap_or(false))?;  // link recovered
```

Every transition lands in the provenance log. From an actual corridor run
([output.txt](../examples/avionics_examples/cfd/plasma_blackout/corridor/output.txt)):

```text
regime -> slip (GNSS-available), Kn=0.07829109848665225
regime -> slip (GNSS-denied),    Kn=0.01705925949914955
regime -> continuum (GNSS-denied), Kn=0.009938308574526865
regime -> continuum (GNSS-available), Kn=0.00025839060489290773
```

One descent moves through orbit-like dynamics, slip flow, continuum flow, comms blackout, and
reacquisition in one uninterrupted program.

## Native Multiphysics

A coupling stack is a static cons-tuple of `PhysicsStage`s stepping one shared `CoupledField`:

```rust
Coupling::between_steps()
    .then(VibrationalLagStage::new(/* Millikan-White bath */))
    .then(FiniteRateIonizationStage::new(n_tot).with_density_field("n_tot"))
    .then(RegimeClassify::new(l_char, trigger))
    .then(BankSteeredLift::new(rho_ref, cda_over_m, l_over_d))
    .then(TrajectoryNav::new(q_diag, gnss_var, optical_var).with_imu(imu))
    .then(CyberneticCorrect::new(SafetyEnvelope::new(q_max, g_max, bank_max)))
    .build()
```

Vibrational relaxation, reacting plasma, regime classification, steered aero force, navigation,
and a bounded-correction gate, in one loop. Stages communicate through named fields on the
evolved state; an `Err` from any stage short-circuits the whole step.

That one stack is the loop body at both levels of the language. A trajectory march couples it
directly; a campaign couples it per case and draw — the ensemble index threads into the stack —
and flies the whole matrix concurrently to one gated table:

```rust
// Trajectory: couple the stack, march a fixed horizon, get one report.
let report = CfdFlow::march(&world).couple(stack).from_field(field0).run_for(steps)?;

// Campaign: six atmospheres alternated from one baseline, each an ensemble of
// receiver-noise draws, every (case, draw) flown concurrently and reduced to a table.
let table = CfdFlow::study("weather-dispersion table")
    .cases(model::weather_cases())
    .baseline(model::standard_day)
    .alternate(model::weather_world)
    .ensemble(constants::MC_DRAWS)
    .couple(|case, draw| world::corridor_coupling(model::bias_departure(case.d_temp), draw))
    .march_for(constants::STEPS, world::initial_field)
    .reduce_ensemble(model::world_row)
    .gates(model::weather_gates())
    .verdict()?;
```

The evolved electron
density gates which measurements the Kalman filter may fold, the Knudsen number selects the
governing model, and the safety gate's clamped bank command is flown by the aero stage,
steering the trajectory that feeds the next step's freestream. CFD, estimation, and control
close one loop in one process.

Two more design decisions carry this. `CfdFlow` composes the run itself: the trajectory march
yields a resumable pause, the campaign study forks it (or alternates whole worlds from a
baseline), continues each branch in copy-on-write O(1), and reduces the outcomes to gated rows —
branch fan-outs run concurrently and bit-identically to the sequential run, and `verdict()`
returns the result as data the caller maps to an exit code. And configuration is separate from
execution: the `flow_config` layer holds owned descriptions (grids, schedules, seeds, stop
conditions, observables, world-published constants) while the `flow` layer materializes runs from
them, so a counterfactual is the same flow handed a different description.

## Everything Self-Verifies

The crate ships its evidence, and CI runs it. `verification/` holds thirteen runnable programs gated
against analytic solutions, published references, or internal invariants;
`.github/workflows/cfd_verification.yml` executes the fast nine on every pull request and the slow
four nightly, failing the build on a non-zero exit. `studies/` holds the empirical probes that settled
design questions before they were committed to specs, findings encoded as gates so the conclusions
stay reproducible. `benches/` pins performance in `PERFORMANCE.md`.

Every gate declares where its bound came from — `[reference]` for an analytic or published value,
`[tripwire]` for one pinned from this code's own prior output — so a `[PASS]` says which of the two it
is. The plasma-blackout examples compare an uncalibrated finite-rate ionization network against the
RAM-C II flight anchor to **order of magnitude** (the earned band is ±0.70 decades, pinned from the
measurement); that is a prediction landing in the right decade, not a per-point accuracy claim.

## Precision as a Parameter

Every theory, solver, stage, and observable is generic over one real scalar (`CfdScalar`,
built on `RealField`). A program fixes a single alias and the entire computation runs at that
precision, from the flux stencils to the Kalman filter, downcasting to `f64` only at the
display boundary. From the `qtt_ramc_stagline` verification:

```rust
/// Working precision.
pub type FloatType = f64; // or f32, or deep_causality_num::Float106

/// Lift an exact `f64` specification into the working precision.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}
```

Specification constants stay exact `f64` literals; `ft` lifts each one into the working
precision, and every derived number is computed in `FloatType`. Changing the alias reruns the
whole program at another precision. The plasma-blackout corridor measured all three: at `f64`,
all gates pass in about 40 seconds; at 106-bit `Float106`, every gate and every discrete event
step is identical, with continuous witnesses agreeing to 15 or 16 significant digits, which
places the corridor's error budget in the model closures and the grid rather than in round-off;
One line change, three precison levels.

## Where Things Live

| Path | Contents |
|---|---|
| `src/theories/` | Fluid theories: the DEC-native `FluidTheory` realization and the pointwise Navier-Stokes regime evaluators with their causal-effect wrappers |
| `src/solvers/` | The DEC Navier-Stokes solver, the QTT incompressible/immersed/linear solvers, the compressible Euler and 2-D/3-D marchers, shock fitting, the Park-2T closure |
| `src/types/flow/` | The `CfdFlow` DSL: the trajectory march (runs, pauses, forks, the named-stage builder) and the campaign study grammar (phase family, `GateSeq`/`Verdict`, the `StudyEffect` carrier, the `save_log` audit sink), plus the coupling stack, physics stages, blackout stages, and reports |
| `src/types/flow_config/` | The configuration layer: owned config containers and type-state builders |
| `src/navigation/` | GNSS-denial navigation: the 17-state error-state Kalman engine, synthetic INS sensors, the integrator regime switch |
| `src/coordinate/` | Body-fitted and blended coordinate maps with metric providers |
| `src/tensor_bridge/` | The CFD to tensor-network bridge: QTT field codecs and finite-difference operator assembly |
| `verification/` | Self-verifying reference checks (see its [README](verification/README.md)) |
| `studies/` | Design-question probes with gated findings (see its [README](studies/README.md)) |
| `benches/` | Criterion benches and the pinned `PERFORMANCE.md` |
| `papers/` | The cited source PDFs behind constants and closures |

The end-to-end examples, the plasma-blackout corridor and its weather-dispersion
table, live in [`examples/avionics_examples/cfd/`](../examples/avionics_examples/cfd/). They
are built entirely from this crate's public API.

## License

MIT. See the workspace [LICENSE](../LICENSE).
