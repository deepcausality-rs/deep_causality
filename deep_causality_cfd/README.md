<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# DeepCausality CFD 

DeepCausality CFD provides Multidisciplinary analysis and optimization (MDAO) by coupling fluid dynamics, 
multiple physics, navigation, and control, in one typed dynamic process. DeepCausality CFD couples 
several disciplines' analyses, optimizes over the coupled result, 
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

`CfdFlow` marches a simulation until a predicate fires, forks the paused state in O(1) through
copy-on-write (tensor fields, navigation engine, and provenance log included), and continues
each fork in its own alternated world. Branches share the paused state and differ only in
their world description. From the plasma-blackout corridor, condensed:

```rust
// March until the evolved sheath's n_e crosses the GPS L1 cutoff.
let onset = CfdFlow::compressible_march(&nominal).run_until(
    world::corridor_coupling(1.0, 0),
    world::initial_field(),
    trigger, ft(0.0),
    |field, _| field.regime().map(|r| r.gnss_denied).unwrap_or(false),
)?;

// Fork the paused onset once per candidate bank command; fly all six
// concurrently; score each branch by its trajectory-derived miss.
let reports = onset.continue_branches(&branch_configs, BRANCH_STEPS)?;
let committed = model::pick_committed(&branches);
```

The corridor runs this twice from the same paused state: a coarse sweep over six bank commands,
then eleven 0.5-degree candidates around the coarse winner, and commits the best of the
seventeen worlds mid-descent. Every branch stamps a `!!ContextAlternation!!` marker into its
provenance log naming its baseline. Branch fan-outs run concurrently on scoped threads and
produce bits identical to the sequential run.

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

Every transition lands in the provenance log. From an actual corridor run
([output.txt](../examples/avionics_examples/cfd/plasma_blackout_corridor/output.txt)):

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
evolved state; an `Err` from any stage short-circuits the whole step. The evolved electron
density gates which measurements the Kalman filter may fold, the Knudsen number selects the
governing model, and the safety gate's clamped bank command is flown by the aero stage,
steering the trajectory that feeds the next step's freestream. CFD, estimation, and control
close one loop in one process.

Two more design decisions carry this. `CfdFlow` composes the run itself: march until a
predicate fires, fork the paused state in O(1) through copy-on-write, continue each fork in its
own alternated world, and score the outcomes, with branch fan-outs running concurrently and
bit-identically to the sequential run. And configuration is separate from execution: the
`flow_config` layer holds owned descriptions (grids, schedules, seeds, stop conditions,
observables, world-published constants) while the `flow` layer materializes runs from them, so
a counterfactual is the same flow handed a different description.

## Everything Self-Verifies

The crate ships its evidence. `verification/` holds thirteen runnable programs gated against
analytic solutions, published references, or internal invariants. `studies/` holds the
empirical probes that settled design questions before they were committed to specs, findings
encoded as gates so the conclusions stay reproducible. `benches/` pins performance in
`PERFORMANCE.md`. The plasma-blackout examples validate an uncalibrated finite-rate ionization network
against RAM-C II flight data.

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
| `src/types/flow/` | The `CfdFlow` DSL: coupling stack, physics stages, march runs, pauses and forks, blackout stages, reports |
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
