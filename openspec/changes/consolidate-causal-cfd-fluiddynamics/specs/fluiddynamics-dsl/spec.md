## ADDED Requirements

### Requirement: The `FluidDynamics` DSL composes fluid simulations statically
The `causal_cfd` crate SHALL provide a `FluidDynamics` domain-specific language, peer to
`CausalFlow` and `CausalDiscovery`, that composes a simulation from a theory (a Navier–Stokes
regime reused across solvers), a solver (a theory and/or physics kernels solving one designated
case), and a set of boundary zones (the `add-boundary-zone-abstraction` terms). Composition SHALL
use **static dispatch** built on the HKT/algebra foundation (`deep_causality_haft`,
`deep_causality_num`) — no `dyn`, trait objects, or dynamic dispatch — and adding a new solver
SHALL be a small trait implementation, not a change to the DSL core. Every solver and theory SHALL
be generic over `R: RealField` with no downcasting to `f64`.

#### Scenario: A case is composed and run
- **WHEN** a `FluidDynamics` case is assembled from a theory, a solver, and boundary zones and is run
- **THEN** it produces the simulation result with all composition resolved at compile time (no dynamic dispatch)

#### Scenario: Precision is a parameter
- **WHEN** an example sets its `FloatType` alias to `f32`, `f64`, or `Float106`
- **THEN** the same solver and theory run natively at that precision with no `f64` downcast in the solver

### Requirement: The DSL wraps CausalFlow for steps, control flow, and counterfactuals
The `FluidDynamics` DSL SHALL lower its march, control flow, and intervention onto the `CausalFlow`
monad: a multi-step march via the flow's iteration, branching/looping via its control-flow
combinators, and counterfactual interventions (e.g. on material, mesh, or temperature) via
`.intervene`. The DSL SHALL integrate with `CausalFlow` for between-step / pre-processing physics
and with `CausalDiscovery` (e.g. a SURD tap) on solver output.

#### Scenario: A counterfactual intervention re-runs a case
- **WHEN** a composed case is intervened on (e.g. the material or mesh is substituted) through the DSL
- **THEN** the intervention is applied via the underlying `CausalFlow` `.intervene` and the case re-runs with the substituted value

#### Scenario: Multi-physics composition
- **WHEN** two physics stages are composed in a `FluidDynamics` pipeline
- **THEN** they compose through the `CausalFlow` bind passthrough, as in the multi-physics pipeline example
