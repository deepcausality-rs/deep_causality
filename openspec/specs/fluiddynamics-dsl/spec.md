# fluiddynamics-dsl Specification

## Purpose
TBD - created by archiving change consolidate-causal-cfd-fluiddynamics. Update Purpose after archive.
## Requirements
### Requirement: The `CfdFlow` DSL composes fluid simulations statically
The `deep_causality_cfd` crate SHALL provide a `CfdFlow` domain-specific language, peer to `CausalFlow`
and `CausalDiscovery`, that composes a simulation from a theory (a Navier–Stokes regime reused across
solvers), a solver (a theory and/or physics kernels solving one designated case), and a set of
boundary zones (the archived `BoundaryZone` terms). Composition SHALL use **static dispatch** built
on the HKT/algebra foundation (`deep_causality_haft`, `deep_causality_num`) — no `dyn`, trait
objects, or dynamic dispatch — and adding a new solver SHALL be a small trait implementation, not a
change to the DSL core. Every solver and theory SHALL be generic over `R: RealField` with no
downcasting to `f64`.

#### Scenario: A case is composed and run
- **WHEN** a `CfdFlow` case is assembled from a theory, a solver, and boundary zones and is run
- **THEN** it produces the simulation result with all composition resolved at compile time (no dynamic dispatch)

#### Scenario: Precision is a parameter
- **WHEN** an example sets its `FloatType` alias to `f32`, `f64`, or `Float106`
- **THEN** the same solver and theory run natively at that precision with no `f64` downcast in the solver

### Requirement: Configuration is separated from workflow composition
The crate SHALL separate **configuration** (the "what") from **workflow composition** (the "how"),
mirroring the Discovery `CdlConfigBuilder` → `CdlBuilder` split. A single `CfdConfigBuilder` entry
SHALL start each owned, validated configuration — a solver config (`dec_ns`), a marching-case
container (`march`), or an MMS-verification config (`verify`) — and the `CfdFlow` facade SHALL compose
those configs onto a caller-owned geometry and run them. Configuration objects SHALL hold no geometry
borrow; the geometry SHALL be lent to the run via `.on(&manifold)` (the B1 borrow model) and SHALL NOT
escape the run.

#### Scenario: A marching case is configured, then composed and run
- **WHEN** a marching case is built with `CfdConfigBuilder::march(...)` and run with `CfdFlow::march(&config).on(&manifold)`
- **THEN** the configuration carries no manifold borrow, the geometry is lent for the run only, and a `Report` is returned

### Requirement: A `CfdFlow` case is an owned description materialized at run time
A `CfdFlow` case SHALL be a fully-owned declarative description holding no borrows of the mesh, geometry,
manifold, or solver. The mesh, manifold, and solver SHALL be materialized at run time, the march
executed, and only an owned result returned. A case SHALL be reusable (cheap to clone) so the same
case can seed a factual run and one or more counterfactual runs.

#### Scenario: The same case seeds factual and counterfactual runs
- **WHEN** a case is run factually and then reused to derive a counterfactual
- **THEN** both runs share the same owned description and no solver borrow escapes the run boundary

### Requirement: A theory is a first-class trait solvers are generic over
A theory (a Navier–Stokes regime reused across solvers) SHALL be a first-class trait
(`FluidTheory<R>`) that abstracts the field-level marching rate, generic over `R: CfdScalar` (a bound
of `RealField + FromPrimitive + … + MaybeParallel`). The trait SHALL expose an associated marching
`State` carrying the algebra bounds the RK4 integrator requires (`Clone + Add + Mul<R>`), an
associated `Ambient` type the rate reads each step (so compressible/thermal regimes extend the ambient
without changing the trait), and a fallible `rate(&state, &ambient)` method. The DEC-native
incompressible rate SHALL implement it, and the pointwise regime evaluators (incompressible,
compressible, Euler, Stokes) SHALL be reachable through it for verification solvers. The manifold
borrow SHALL live in the implementor, not the trait. Adding a theory or a solver SHALL be a small
trait implementation using static dispatch, not a change to the DSL core.

#### Scenario: A solver is generic over its theory
- **WHEN** a solver is constructed with a theory that implements `FluidTheory<R>`
- **THEN** the solver marches using that theory with all dispatch resolved at compile time

#### Scenario: A regime extends the ambient without changing the trait
- **WHEN** a compressible or thermal regime needs additional ambient state (temperature, density, EOS)
- **THEN** it supplies its own associated `Ambient` type and the `FluidTheory` trait is unchanged

### Requirement: Parallelism is an opt-in parameter threaded through the solver bounds
The crate SHALL carry an opt-in `parallel` feature that forwards to `deep_causality_topology/parallel`
and `deep_causality_par/parallel` (and pulls Rayon directly only where the crate fans out itself). The
`CfdScalar` bound and the theory/solver trait bounds SHALL include `MaybeParallel`, so the inner
topology operator loops fan out under the feature with no solver code change. Serial execution SHALL
be the default. The crate SHALL parallelize at a single granularity per run — coarse-grained over
independent cases (counterfactual branches, ensembles, sweeps) or fine-grained per cell — and SHALL
NOT nest the two, to avoid oversubscription.

#### Scenario: The feature is off by default and results are unchanged
- **WHEN** the crate is built without the `parallel` feature
- **THEN** the solver runs serially and reproduces the same results as a parallel build to tolerance

#### Scenario: Independent cases fan out under the feature
- **WHEN** several independent counterfactual branches or ensemble members run under `--features parallel`
- **THEN** they execute concurrently and each result matches its serial counterpart

### Requirement: Solvers separate owned configuration from the manifold-bound marcher
Each solver SHALL expose an owned configuration struct that holds no borrow of the manifold, built via
a type-state builder (modeled on the Discovery `CdlBuilder`). The manifold-bound marcher SHALL be
materialized from the configuration, the boundary zones, and the manifold at run time, so the same
owned configuration can be reused across runs.

#### Scenario: Configuration is owned and reusable
- **WHEN** a solver configuration is built and then used to materialize a marcher
- **THEN** the configuration carries no manifold borrow and can be reused to materialize another marcher

### Requirement: The solver ambient is a per-step context channel
The solver marching rate SHALL read its ambient — kinematic viscosity, freestream inflow, and body
force — from a per-step context channel rather than from values fixed at construction, so that a
coupling stage or a dynamic-law intervention can drive the ambient each step. When no coupling is
present the ambient SHALL be constant, and the result SHALL reproduce the construction-fixed numerics
of the migrated solver.

#### Scenario: Coupling drives the ambient each step
- **WHEN** a coupling writes a per-step viscosity (e.g. `ν(T)`) or freestream speed into the ambient channel
- **THEN** the marcher uses the updated ambient on the next step

#### Scenario: No coupling reproduces the fixed-ambient result
- **WHEN** a solver runs with a constant ambient and no coupling
- **THEN** it reproduces the pre-refactor construction-fixed validation result to the same tolerance

### Requirement: The DSL owns the march and bespoke analysis plugs in by typed seams
The DSL SHALL own the generic march — seeding, the per-step projected DEC loop, the stop condition,
and the common `Observe` diagnostics — through a single `MarchConfig` container and the `CfdFlow`
marching run, generic over `R` with no `f64` downcast; it SHALL NOT be re-implemented per case nor
specialized into per-case solver types. An example's bespoke analysis (e.g. the cavity
centerline / Ghia compare / vortex detection, or a wake probe) SHALL plug in through **typed seams**
rather than by forking the march: a per-step `run_with(hook)` receiving a read-only `StepView`, the
final field via `Report::final_field`, and the `Observe` set for common series. A solution that recurs
across cases (rule of three) SHALL be **promoted into the DSL corpus** rather than copied; a genuinely
unique experiment SHALL stay in its example behind a seam.

#### Scenario: An example streams a per-step probe without forking the march
- **WHEN** an example needs a per-step diagnostic (a progress line, a wake probe)
- **THEN** it supplies a `run_with` hook over the DSL's march and reads `StepView` / `Report::final_field`, rather than re-implementing the marching loop

#### Scenario: A recurring solution is promoted to the corpus
- **WHEN** a capability (e.g. a graded metric, a manufactured solution, autodiff derivatives) is needed by more than one case
- **THEN** it is promoted into the `deep_causality_cfd` corpus and reused, not copied into each example

### Requirement: The DSL covers marching, MMS-verification, and operator-accuracy solvers
The `CfdFlow` DSL SHALL provide three solver kinds, all producing a common `Report` and sharing a static
`Solver` seam so that adding a kind is a trait implementation, not a change to the DSL core: a
**marching** solver (a DEC regime marched over a mesh with boundary zones, a seed, an observe set, and
optional `.couple` multiphysics); an **MMS-verification** solver (a manufactured solution checked
against a pointwise `FluidTheory` regime kernel, with no DEC march), generic over the regime so every
regime (incompressible, Euler, Stokes, compressible) is reachable through the DSL; and an
**operator-accuracy** solver (DEC operators on a possibly graded mesh, swept over resolutions for
convergence orders, with no march). The marching solver SHALL support fixed-step, until-developed /
steady, and uncertain causal-monad march styles. The mesh abstraction SHALL cover periodic / wall-
bounded / open lattices, uniform and graded metrics, and immersed cut-cell bodies.

#### Scenario: A marching case runs to a Report
- **WHEN** a marching case (mesh + zones + seed + march + observe) is assembled and run
- **THEN** it produces a `Report` with all composition resolved at compile time

#### Scenario: Every regime is reachable through MMS verification
- **WHEN** an MMS-verification solver is run for the incompressible, Euler, Stokes, and compressible regimes
- **THEN** each regime's pointwise kernel is verified against its manufactured solution

#### Scenario: An operator-accuracy study reports convergence orders
- **WHEN** an operator-accuracy solver sweeps a graded mesh over a set of resolutions
- **THEN** it reports the observed convergence orders without marching a field

### Requirement: The DSL wraps CausalFlow for the march, control flow, and counterfactuals
The `CfdFlow` DSL SHALL lower its march, control flow, and intervention onto the `CausalFlow` monad: a
multi-step march via the flow's arrow-algebra iterator (`iterate_n` / `iterate_until`),
branching/looping via its control-flow combinators (`branch_with` / `either`), and counterfactual
interventions (e.g. on material, mesh, temperature, or a dynamic law) via `.intervene`. The DSL SHALL
integrate with `CausalFlow` for between-step / pre-processing physics and with `CausalDiscovery`
(e.g. a SURD tap) on solver output.

#### Scenario: The march lowers onto the arrow-algebra iterator
- **WHEN** a case is marched for a fixed horizon or until a developed/steady predicate holds
- **THEN** the march lowers onto the `CausalFlow` `iterate_n` / `iterate_until` combinators

#### Scenario: Multi-physics composition
- **WHEN** two physics stages are composed in a `CfdFlow` coupling
- **THEN** they compose through the `CausalFlow` bind passthrough, as in the multi-physics pipeline example

### Requirement: `.couple` wires modular physics stages into a between-step pipeline
The `CfdFlow` DSL SHALL provide a `.couple` seam that registers a between-step physics pipeline,
evaluated once per timestep around the CFD step. A coupling SHALL be a statically-composed pipeline of
physics stages over the error algebra (`PropagatingEffect`/`bind`), such that a stage, a sub-process,
and a whole coupling are first-class values that can be built and tested independently, stored in
variables, and wired together with `then` / `compose`. Adding a new coupled physics SHALL be a small
trait implementation. Errors SHALL propagate across the whole holistic coupling automatically.

#### Scenario: Modular sub-processes wire into a holistic coupling
- **WHEN** independently-built physics sub-processes (e.g. thermal, structural, trajectory) are composed via `.couple`
- **THEN** they form one holistic between-step pipeline with automatic error propagation, and a stage failure short-circuits the chain

#### Scenario: Coupled physics changes the flow dynamics over time
- **WHEN** a coupling writes a temperature-dependent property (e.g. `ν(T)`) back into the solver each step
- **THEN** the fluid dynamics evolve with the coupled context (dynamic causality), not with a fixed parameter

### Requirement: Counterfactuals intervene on dynamics and reuse the developed solve
The `CfdFlow` DSL SHALL support counterfactual interventions through a closed `Intervene` vocabulary
spanning static terms (e.g. material, Reynolds number, mesh) and dynamic laws (e.g. a thrust
schedule, heat gradient, wall temperature, Prandtl number). It SHALL provide two flavors: a
shared-seed counterfactual that derives a sibling case sharing the background (mesh/zones/seed) with a
single `do(·)` applied, and a continuation counterfactual that abducts a developed result so the
expensive solve runs once and multiple scenarios branch on top.

#### Scenario: A counterfactual intervention re-runs a case
- **WHEN** a composed case is intervened on (e.g. the material or a dynamic law is substituted) through the DSL
- **THEN** the intervention is applied via the underlying `CausalFlow` `.intervene` and the case re-runs with the substituted term

#### Scenario: Compute once, branch many scenarios
- **WHEN** a developed result is continued with several distinct interventions
- **THEN** the expensive solve is reused as the shared background and each scenario marches only its incremental relaxation

### Requirement: Examples are written in the DSL with a config/main split
The crate's examples SHALL be written in the `CfdFlow` DSL. Each example SHALL separate configuration
from wiring: a `config.rs` SHALL own every solver/mesh/zone/seed configuration (built with type-state
builders) and the `FloatType` alias, and a `main.rs` SHALL plug the imported configuration into the
CfdFlow pipeline. Larger multi-physics examples SHALL further decompose into per-physics sub-process
modules wired into one holistic coupling.

#### Scenario: Configuration is imported, not inlined
- **WHEN** an example's `main.rs` is inspected
- **THEN** it plugs in configuration imported from `config.rs` rather than constructing solver/mesh/zone/seed configuration inline

### Requirement: The corpus supplies graded geometry and manufactured solutions
The DSL corpus SHALL supply reusable building blocks beyond the uniform march. A `Mesh` SHALL cover
periodic / wall-bounded / open lattices (`periodic_cube`/`torus`, `box_domain`, `channel`), uniform
and graded metrics (`Grading::cosine` → a `PerEdge` Regge metric that keeps `d`, the discrete Stokes
theorem, and divergence-freeness exact), an immersed cut-cell `Body`, and a geometry-only `manifold()`
for operator studies. A `Manufactured<R>` seam SHALL admit any analytic solution for MMS verification,
with `TaylorGreen` as the corpus solution whose exact spatial derivatives come from the tangent functor
(`deep_causality_calculus` autodiff), not finite differences.

#### Scenario: A graded operator study runs without a march
- **WHEN** a graded mesh is built with `Mesh::torus(n).graded(Grading::cosine(axis, amp))` and materialized via `manifold()`
- **THEN** the DEC operators are studied on the graded metric with no field marched

#### Scenario: A manufactured solution verifies a kernel with exact derivatives
- **WHEN** an MMS verification runs a `Manufactured` solution (e.g. `TaylorGreen`)
- **THEN** the kernel residual uses exact autodiff derivatives, not finite differences

### Requirement: A configured run is deterministically reproducible
A `CfdFlow` run with a fixed configuration SHALL be bit-for-bit reproducible across invocations. Where a
run draws Monte-Carlo samples (e.g. a sensor-fed uncertain inflow), it SHALL be made reproducible by
seeding the sampler; where it iterates an immersed cut-cell registry, the iteration order SHALL be made
deterministic (ascending cell id) so the constrained projection's floating-point reduction order does
not vary per process.

#### Scenario: A stochastic, cut-cell case reproduces byte-for-byte
- **WHEN** a sensor-fed cut-cell case fixes its sampler seed and enables deterministic cut-cell order
- **THEN** repeated runs produce byte-for-byte identical output

