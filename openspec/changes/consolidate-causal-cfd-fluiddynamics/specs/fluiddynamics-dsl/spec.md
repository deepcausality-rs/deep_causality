## ADDED Requirements

### Requirement: The `Flow` DSL composes fluid simulations statically
The `deep_causality_cfd` crate SHALL provide a `Flow` domain-specific language, peer to `CausalFlow`
and `CausalDiscovery`, that composes a simulation from a theory (a Navier–Stokes regime reused across
solvers), a solver (a theory and/or physics kernels solving one designated case), and a set of
boundary zones (the archived `BoundaryZone` terms). Composition SHALL use **static dispatch** built
on the HKT/algebra foundation (`deep_causality_haft`, `deep_causality_num`) — no `dyn`, trait
objects, or dynamic dispatch — and adding a new solver SHALL be a small trait implementation, not a
change to the DSL core. Every solver and theory SHALL be generic over `R: RealField` with no
downcasting to `f64`.

#### Scenario: A case is composed and run
- **WHEN** a `Flow` case is assembled from a theory, a solver, and boundary zones and is run
- **THEN** it produces the simulation result with all composition resolved at compile time (no dynamic dispatch)

#### Scenario: Precision is a parameter
- **WHEN** an example sets its `FloatType` alias to `f32`, `f64`, or `Float106`
- **THEN** the same solver and theory run natively at that precision with no `f64` downcast in the solver

### Requirement: A `Flow` case is an owned description materialized at run time
A `Flow` case SHALL be a fully-owned declarative description holding no borrows of the mesh, geometry,
manifold, or solver. The mesh, manifold, and solver SHALL be materialized at run time, the march
executed, and only an owned result returned. A case SHALL be reusable (cheap to clone) so the same
case can seed a factual run and one or more counterfactual runs.

#### Scenario: The same case seeds factual and counterfactual runs
- **WHEN** a case is run factually and then reused to derive a counterfactual
- **THEN** both runs share the same owned description and no solver borrow escapes the run boundary

### Requirement: A theory is a first-class trait solvers are generic over
A theory (a Navier–Stokes regime reused across solvers) SHALL be a first-class trait
(`FluidTheory<R>`) that abstracts the marching rate, generic over `R: RealField`. The DEC-native
incompressible rate SHALL implement it, and the pointwise regime evaluators (incompressible,
compressible, Euler, Stokes) SHALL be reachable through it for verification solvers. Adding a theory
or a solver SHALL be a small trait implementation using static dispatch, not a change to the DSL core.

#### Scenario: A solver is generic over its theory
- **WHEN** a solver is constructed with a theory that implements `FluidTheory<R>`
- **THEN** the solver marches using that theory with all dispatch resolved at compile time

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

### Requirement: Case orchestration is lifted into reusable generic solvers
Per-case orchestration SHALL be lifted out of the avionics example `main`s into standalone solvers
and a generic diagnostics/observe layer, generic over `R: RealField` with no `f64` downcast. This
covers seeding, the marching loop, and case diagnostics such as the Strouhal number, drag and lift
coefficients, Ghia comparison, vortex detection, and dissipation sampling. Each migrated example SHALL
drive such a solver through the Flow DSL rather than re-implementing the orchestration inline.

#### Scenario: A migrated example drives a reusable solver
- **WHEN** a migrated validation example is inspected
- **THEN** its seeding, marching loop, and diagnostics come from a reusable solver and the generic observe layer, not from inline `main` orchestration

### Requirement: The DSL wraps CausalFlow for the march, control flow, and counterfactuals
The `Flow` DSL SHALL lower its march, control flow, and intervention onto the `CausalFlow` monad: a
multi-step march via the flow's arrow-algebra iterator (`iterate_n` / `iterate_until`),
branching/looping via its control-flow combinators (`branch_with` / `either`), and counterfactual
interventions (e.g. on material, mesh, temperature, or a dynamic law) via `.intervene`. The DSL SHALL
integrate with `CausalFlow` for between-step / pre-processing physics and with `CausalDiscovery`
(e.g. a SURD tap) on solver output.

#### Scenario: The march lowers onto the arrow-algebra iterator
- **WHEN** a case is marched for a fixed horizon or until a developed/steady predicate holds
- **THEN** the march lowers onto the `CausalFlow` `iterate_n` / `iterate_until` combinators

#### Scenario: Multi-physics composition
- **WHEN** two physics stages are composed in a `Flow` coupling
- **THEN** they compose through the `CausalFlow` bind passthrough, as in the multi-physics pipeline example

### Requirement: `.couple` wires modular physics stages into a between-step pipeline
The `Flow` DSL SHALL provide a `.couple` seam that registers a between-step physics pipeline,
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
The `Flow` DSL SHALL support counterfactual interventions through a closed `Intervene` vocabulary
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
The crate's examples SHALL be written in the `Flow` DSL. Each example SHALL separate configuration
from wiring: a `config.rs` SHALL own every solver/mesh/zone/seed configuration (built with type-state
builders) and the `FloatType` alias, and a `main.rs` SHALL plug the imported configuration into the
Flow pipeline. Larger multi-physics examples SHALL further decompose into per-physics sub-process
modules wired into one holistic coupling.

#### Scenario: Configuration is imported, not inlined
- **WHEN** an example's `main.rs` is inspected
- **THEN** it plugs in configuration imported from `config.rs` rather than constructing solver/mesh/zone/seed configuration inline
