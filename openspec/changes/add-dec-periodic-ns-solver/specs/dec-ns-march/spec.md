# dec-ns-march

The time march: `Rk4` over the typed velocity state, the fallible
`leray_project` and `cfl_check` binds, the solver configuration and run
loop, initial-condition seeding, and the causal-monad wrapper.

## ADDED Requirements

### Requirement: The step maps the type-state to the type-state

The crate SHALL provide a solver type
(`DecNsSolver<'m, const D: usize, R>`) whose step accepts a
`&SolenoidalField<R>` and returns
`Result<StepOutput<R>, PhysicsError>` where the output state is again a
`SolenoidalField<R>`. Internally the step SHALL be the chain: extract the
1-form, advance it with `deep_causality_calculus::Rk4` (the only
integrator), re-project through `SolenoidalField::from_leray_projection`,
then apply the CFL guard. There SHALL be no public path that marches an
unprojected `VelocityOneForm<R>`.

#### Scenario: Step output is divergence-free at CG tolerance

- **WHEN** a step succeeds at f32, f64, and Float106
- **THEN** the returned state's divergence residual `‖δu♭‖_∞` is at or
  below the projection tolerance for that precision

#### Scenario: CG failure short-circuits the chain

- **WHEN** the projection's CG is constrained to one iteration so it cannot
  converge
- **THEN** the step returns the wrapped projection error and no state is
  produced

### Requirement: CFL guard with advective and diffusive limits

The step SHALL enforce, after projection, the advective limit
`dt ≤ C_adv · dx_min / max|u|` (skipped while `max|u|` is zero) and the
diffusive limit `dt ≤ C_diff · dx_min² / (2·D·ν)` (skipped when `ν = 0`),
with `max|u|` recovered through `sharp`, `dx_min` taken from the Regge
geometry's edge lengths, and both safety factors configurable with default
`0.9`. A violation SHALL return a dedicated `PhysicsError` carrying the
violated limit and the configured `dt`.

#### Scenario: Advective violation is reported with both numbers

- **WHEN** a solver is configured with `dt` exceeding the advective limit
  for a state of known maximum speed
- **THEN** the step returns the CFL error naming the limit and the actual
  `dt`, and the error message distinguishes the advective from the
  diffusive limit

#### Scenario: Diffusive violation is caught even at rest

- **WHEN** a solver with large `ν` and over-long `dt` steps a zero-velocity
  state
- **THEN** the diffusive limit alone rejects the step

#### Scenario: Zero-velocity inviscid state steps freely

- **WHEN** a solver with `ν = 0` steps a zero-velocity state with no body
  force
- **THEN** the step succeeds and returns the zero state (both guards
  skipped, fixed point preserved)

### Requirement: Run loop over the fallible step

The solver SHALL provide `run_n(state, n)` marching exactly `n` steps and
`run_until(state, predicate, max_steps)` marching until the predicate holds
on a produced state or the bound is reached, both short-circuiting on the
first step error and reporting the step index at which it occurred. The
loop SHALL carry the fallible step as a plain `Result` chain (the
`EndoArrow` combinators are infallible by signature and are not forced over
errors).

#### Scenario: Error carries the failing step index

- **WHEN** a run is configured so the CFL guard trips at a known step
- **THEN** the returned error names that step index

#### Scenario: Predicate stop is honored

- **WHEN** `run_until` is given a predicate on accumulated simulated time
- **THEN** the march stops at the first state satisfying it and reports how
  many steps ran

### Requirement: Initial conditions seed through de Rham and one projection

The solver SHALL provide an initial-condition path that accepts vertex
vectors (or exact per-edge line integrals) and produces the starting
`SolenoidalField<R>` by the Stage 0 de Rham map followed by exactly one
Leray projection, per the gap note: an analytically divergence-free field
is not discretely divergence-free.

#### Scenario: Sampled Taylor–Green seeds to a projected state

- **WHEN** the analytic 2D Taylor–Green velocity is sampled at vertices and
  seeded
- **THEN** the resulting state is divergence-free at CG tolerance and its
  kinetic energy matches the analytic value at the discretization order

### Requirement: Causal-monad wrapper in the kernel-wrapper tradition

The module SHALL expose the step through a `PropagatingEffect`-returning
wrapper in the crate's existing `wrappers.rs` convention (`Ok → pure`,
`Err → from_error`), so the solver composes with causaloids without the
core depending on the causal monad.

#### Scenario: Wrapper propagates success and failure faithfully

- **WHEN** the wrapped step runs once on a valid state and once with a
  CG-starved configuration
- **THEN** the first yields `pure(state)` and the second the converted
  `CausalityError`
