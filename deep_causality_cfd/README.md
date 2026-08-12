<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# DeepCausality CFD: Counterfactual Fluid Dynamics

DeepCausality CFD provides Counterfactual Fluid Dynamics and multidisciplinary analysis and optimization (MDAO) by coupling fluid dynamics, multiple physics, navigation, and control, in one typed dynamic process.
DeepCausality CFD couples several disciplines' analyses, optimizes over the coupled result,
and keeps track of the uncertainty along the way: the plasma-blackout
example marches a compressible flow, reacts its plasma chemistry, gates a Kalman filter on the
result, flies the control command it selects, and picks that command by forking the running
simulation into counterfactual worlds. Multiple solvers, multi-physics, multi-regime,
counterfactual dynamics, and precision as a parameter, in one crate.

## Usage

The crate is unpublished, but you can add it [as a git dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories):

You can pin the repo by:
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

## Counterfactual Dynamics

`CfdFlow` is a two-level language. At the **trajectory** level, `CfdFlow::march` marches a coupled
run until a predicate fires and yields a resumable pause; at the **campaign** level,
`CfdFlow::study` runs a family of counterfactual cases forked from that pause to a `Verdict`. A
fork shares the marched tensor state in O(1) and never copies it. The `CoupledField` is
copy-on-write: each branch takes its single clone at its first field write, which `continue_march`
always performs, so the per-branch field cost is O(cells), not O(1). That clone covers the per-cell
scalar vectors, the navigation engine, and the provenance log. Each branch then continues in its
own alternated world. From the plasma-blackout corridor, condensed:

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
(`GateSeq<Row>`), and the DSL never exits or prints (`verdict()` returns data). The optional
campaign-level `save_log(path)` writes provenance to disk on that coupled `march_for` path: one
file per branch under the fan-out, plus a `<base>.main.log` naming every spawn and rejoin. The
`fork` / `branch` / `continue_for` chain shown above does not thread the audit sink, so it writes
no files. Each of its branches still carries its full provenance in the returned report's effect
log.

## Dynamic Regime Change

The regime is classified dynamically from the evolved state at each step. A vehicle entering or
leaving orbit transitions through several regimes, and the governing regime switches on the
physics:

- **Flow regime.** `RegimeClassify` classifies the freestream Knudsen number into the governing
  model — continuum Navier-Stokes, slip-corrected continuum, transitional, or free-molecular —
  and logs each transition. The classification is a diagnostic carried on the evolved state: the
  crate does not switch closures on it, and no slip, transitional, or free-molecular closure is
  implemented.
- **Dynamics regime.** `RegimeSwitch` and `aero_gravity_ratio` express the integrator switch on the
  force ratio `ε = a_aero/a_grav`: while gravity dominates, a trajectory advances on the exact
  KS-conformal core with aero as a between-step kick (Encke); once aero dominates, direct Cowell
  integration is the appropriate choice. This is the criterion for orbit entry and exit, where the
  integrator that is exact in orbit loses accuracy in the atmosphere. Both are public API; the
  shipped navigation engine does not call them, so the switch is the caller's to apply.
- **Link regime.** The evolved electron density sets the plasma frequency, and the plasma
  frequency decides whether the GNSS link exists. The Kalman filter's measurement gating
  follows it.

The regime is read straight off the evolved field, so the DSL turns a regime property into an
**event the run finds** rather than a station it is told to switch at:

```rust
// `field.regime()` -> Option<RegimeClass<R>> { model, knudsen, plasma_frequency, gnss_denied,
// mach_regime, thrust_state, touchdown }. The last three are the powered-descent axes and stay
// neutral unless `RegimeClassify::with_flight_axes` is called.
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
regime -> slip (GNSS-denied), Kn=0.012690837165407727
regime -> continuum (GNSS-denied), Kn=0.00993838892165156
regime -> continuum (GNSS-available), Kn=0.0002551442196046344
```

One descent moves through orbit-like dynamics, slip flow, continuum flow, comms blackout, and
reacquisition in one uninterrupted program.

## Dynamic Multiphysics

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
baseline), continues each branch copy-on-write from the shared state, and reduces the outcomes to
gated rows —
branch fan-outs run concurrently and bit-identically to the sequential run, and `verdict()`
returns the result as data the caller maps to an exit code. And configuration is separate from
execution: the `flow_config` layer holds owned descriptions (grids, schedules, seeds, stop
conditions, observables, world-published constants) while the `flow` layer materializes runs from
them, so a counterfactual is the same flow handed a different description.

## Multiple Solver Paradigms

**Calculus-based: the DEC-native Navier-Stokes solver.** Velocity lives as an edge 1-form on a
discrete exterior calculus. Each time step marches the Leray-projected rate, so the field stays
divergence-free at every step. The `SolenoidalField` type-state encodes that: the carrier is a
private field, so there is no public constructor, every constructing path is a projection, and the type
implements no arithmetic, so two projected fields cannot be added into an unprojected one. Two
wall-bounded escape hatches are public. `constrain_edges` and `with_lift` re-wrap a modified tensor
without re-projecting; they exist because the DEC solver re-enters them at the end of each step, on
the output of the constrained projection that already pinned those edges. Off that path the caller
carries the invariant. Validated against Taylor-Green decay, exact Couette and
Poiseuille states, the Ghia et al. (1982) lid-driven cavity tables, and cylinder wake
references.

**Quantized Tensor Trains: the QTT marchers.** The compressible Euler marchers (1-D through 3-D,
including a body-fitted variant) run on quantized tensor trains, where a `2^L` grid *stores* order
`chi^2 * L`: logarithmic in point count for a bounded bond dimension `chi`, with sharp structure
paid for in `chi`. Storage is not runtime: the incompressible immersed harness measured per-step
wall-clock rising far faster than `chi^2 * L` while the achieved bond stayed flat, so a
non-compression bottleneck dominates that path (see the QTT envelope note in
[`verification/README.md`](verification/README.md)). Whether `chi` stays bounded is the design
question the rank studies in `studies/` answered, and they measured the decisive caveat: the rank
driver is coordinate alignment, not sharpness. For the descent-schedule case the compressible
carrier answers it with a shock-fitted inflow strip, imposing the exact Rankine-Hugoniot state as
the boundary of the marched layer, so that shock is never captured at all.

**Analytic and pointwise: fitted closures.** Exact Rankine-Hugoniot jumps, the Park
two-temperature relaxation closures, the finite-rate ionization network, and the pointwise
Navier-Stokes regime evaluators with their causal-effect wrappers. A stagnation line with a
fitted shock runs entirely on these, with no grid.

All three solver families sit behind the same `CfdFlow` language and the same scalar type, so you
can pick the best fit for your problem: the DEC solver for an incompressible cavity, the QTT
marcher for a reentry layer, a fitted closure for the stagnation line.

## Provenance for Comparison Across Boundaries

The append-only effect log continues across regimes and physics. When a counterfactual fan-out
occurs, each branch writes its own scenario effect log, so you can compare why one variant failed
and others succeeded. Provenance is preserved under counterfactual intervention: when you inject a
failure to stress-test a simulation, the effect log records the intervention, the replaced value,
and every subsequent derived step, so you can read the causal chain from its inception to its
completion.

Because the log continues across boundaries and records regime changes, it allows precise
comparative dissection across transitions. For example, you can compare flow parameters from the
subsonic regime before and after the vehicle enters the transonic regime. This supports efficient
structured diffs over causal event sequences, with precise attribution.

Generate parameter tables, for example for weather conditions, from a single flow simulation where
each scenario brings its own append-only effect log for end-to-end provenance.

Ingest trajectory logs from an existing 6-DOF simulation generated with your own tooling, run
counterfactuals across the parameter space to find the failure threshold at which the simulation
breaks down, and use the provenance log to compare how the safety envelope evolves across regimes.

## Precision as a Parameter

Every theory, solver, stage, and observable is generic over one real scalar. A program fixes a
single alias and the entire computation runs at that precision: `f32` for speed, `f64` for
industry-standard precision, or `Float106` for high-fidelity, reference-grade results with up to 30
significant digits. One line change, three precision levels. Precision as a parameter also makes
every solver in this project future-proof for the upcoming IEEE f16 and f128 standards.

```rust
/// Working precision.
pub type FloatType = f64; // or f32, or deep_causality_num::Float106

```

Specification constants stay exact `f64` literals; `ft` lifts each one into the working
precision, and every derived number is computed in `FloatType`. Changing the alias reruns the
whole program at another precision.


## Selected Capabilities

- **One entry per configuration.** `CfdConfigBuilder` starts every owned config: the DEC solver
  (`dec_ns`), the four marching cases (`march`, `qtt_march`, `compressible_march`, `duct`), the
  MMS verification (`verify`), and the sensor-fed uncertain march (`uncertain_march`). Each case
  entry takes the case name, each builder validates at `build()`, and no config family has a
  second public constructor.

  ```rust,ignore
  let config = CfdConfigBuilder::duct::<f64>("nozzle-pr-0.7")
      .profile(DuctAreaProfile::ConvergingDiverging { inlet_area, throat_area, exit_area, length })
      .inlet(p0, t0)
      .gamma(1.4)
      .back_pressure(p0 * 0.7)
      .cells(128)
      .stop(10_000, 1e-8)
      .build()?;
  let report = CfdFlow::march(&config).run()?;
  ```

- **Suspend and resume a march.** `save_resume_state` / `load_resume_state` (with `pack_resume` /
  `unpack_resume`) checkpoint a running `CoupledField` to disk and restore it. A world fingerprint guards
  the seam, so a snapshot taken under different constants is refused rather than silently resumed.
- **Duct marching.** `DuctMarchRun`, composed by `CfdFlow::march`, runs an internal-flow duct case over a
  borrowed `DuctConfig` and returns an owned `Report`.
- **Ignition corridor.** `IgnitionCorridor`, committed through by `ThrottleGuidance`, expresses the
  four-condition corridor a powered-descent throttle must satisfy; the margin is supplied by the caller.
- **Closed-form acoustic core inverse.** `AcousticCoreInverse` (with its 2-D and 3-D forms) inverts the
  constant-coefficient acoustic core `A₀ = I − β·∂²` on a periodic grid, without an iterative solve.

## Verification

The crate ships its evidence, and CI runs it. `verification/` holds thirteen runnable programs gated
against analytic solutions, published references, or internal invariants;
`.github/workflows/cfd_verification.yml` executes the fast nine on every pull request and the slow
four nightly, failing the build on a non-zero exit. `studies/` holds the empirical probes that settled
design questions before they were committed to specs, findings encoded as gates so the conclusions
stay reproducible. `benches/` pins performance in `PERFORMANCE.md`.

Every gate declares where its bound came from — `[reference]` for an analytic or published value,
`[tripwire]` for one pinned from this code's own prior output — so a `[PASS]` says which of the two it
is. The plasma-blackout examples gate an uncalibrated finite-rate ionization network against the
RAM-C II flight anchor to **order of magnitude**: the earned band is ±0.70 decades, pinned from the
measurement. That is a prediction landing in the right decade, not a per-point accuracy claim, and it
holds after the `fix-ramc-vibrational-relaxation-pair` reduced-mass correction, which moved the
stagnation-line closed-form Park-2T controller to 1.27 decades below the anchor (reported as an offset,
not re-admitted).

## Where Things Live

| Path | Contents |
|---|---|
| `src/theories/` | Fluid theories: the DEC-native `FluidTheory` realization and the pointwise Navier-Stokes regime evaluators with their causal-effect wrappers |
| `src/solvers/` | The DEC Navier-Stokes solver, the QTT incompressible/immersed/linear solvers, the compressible Euler and 2-D/3-D marchers, shock fitting, the Park-2T closure |
| `src/types/flow/` | The `CfdFlow` DSL: the trajectory march (runs, pauses, forks, the named-stage builder) and the campaign study grammar (phase family, `GateSeq`/`Verdict`, the `StudyEffect` carrier, the `save_log` audit sink), plus the coupling stack, physics stages, blackout stages, and reports |
| `src/types/flow_config/` | The configuration layer: owned config containers, fluent builders validated at `build()`, and type-state phase transitions for the zone and coupling tuples |
| `src/navigation/` | GNSS-denial navigation: the 17-state error-state Kalman engine, synthetic INS sensors, the integrator regime switch |
| `src/coordinate/` | Body-fitted and blended coordinate maps with metric providers |
| `src/tensor_bridge/` | The CFD to tensor-network bridge: QTT field codecs and finite-difference operator assembly |
| `verification/` | Self-verifying reference checks (see its [README](verification/README.md)) |
| `studies/` | Design-question probes with gated findings (see its [README](studies/README.md)) |
| `benches/` | Criterion benches and the pinned `PERFORMANCE.md` |
| `papers/` | Source PDFs behind constants and closures, indexed by [`papers/README.md`](papers/README.md), which maps each PDF to its citing code and lists the references cited in code whose PDF is not yet present |

The end-to-end examples, the plasma-blackout corridor and its weather-dispersion
table, live in [`examples/avionics_examples/cfd/`](../examples/avionics_examples/cfd/). Every
run is driven through this crate's `CfdFlow` API. The examples also depend on five workspace
crates directly, because this crate's signatures expose their types without re-exporting them:

- `deep_causality_tensor` for `CausalTensor`, `CausalTensorTrain`, and `Truncation`, which
  appear in the QTT configs and the field accessors.
- `deep_causality_core` for `AlternatableContext` and `EffectLog`, the fork and provenance seams.
- `deep_causality_haft` for `LogAddEntry` and `LogSize`, the traits that read that log.
- `deep_causality_algebra` and `deep_causality_num` for `Real` and `FromPrimitive`, two of the
  traits behind the `CfdScalar` bound, needed to call scalar methods in generic code.

The plasma-blackout examples additionally use `deep_causality_physics` for advanced physics.

## License

MIT. See the workspace [LICENSE](../LICENSE).
