# dec-ns-diagnostics

DEC-native solver diagnostics: the integral invariants the validation
ladder gates on, the per-step cheap observables, and the opt-in
two-convention pressure recovery.

## ADDED Requirements

### Requirement: Integral invariants from the existing operators

The module SHALL provide, for a velocity state on a metric-bearing periodic
manifold: kinetic energy `E = ½ Σ_e u_e (⋆u)_e`, enstrophy
`Z = ½ Σ_f ω_f (⋆ω)_f` with `ω = du♭`, and — in three dimensions only —
helicity `H = Σ_c (u♭ ∧ du♭)_c` through the Stage 0 wedge. Requesting
helicity on a manifold whose dimension is not 3 SHALL return a
`PhysicsError` rather than a meaningless number. All three SHALL be
precision-generic.

#### Scenario: Energy matches the analytic Taylor–Green value

- **WHEN** kinetic energy is computed on the seeded 2D Taylor–Green state
  over the refinement ladder
- **THEN** it converges to the analytic `E(0)` at the discretization order

#### Scenario: Helicity is rejected in 2D

- **WHEN** helicity is requested on a `square_torus` manifold
- **THEN** the call returns a `PhysicsError` naming the dimension
  requirement

#### Scenario: Enstrophy of a constant field is zero

- **WHEN** enstrophy is computed on a constant (harmonic, mean-flow) state
- **THEN** it is exactly zero (`du♭ = 0` for constants on a torus)

### Requirement: Per-step observables ride the step output

The step output SHALL carry, without recomputation by the caller, the
maximum pointwise speed (the `sharp`-based value the CFL guard already
computed) and the post-projection divergence residual `‖δu♭‖_∞`. These two
SHALL also be available as standalone functions on a state.

#### Scenario: Reported residual agrees with direct evaluation

- **WHEN** a step succeeds and the caller independently evaluates
  `‖δu♭‖_∞` on the returned state
- **THEN** the two values are identical

### Requirement: Opt-in pressure recovery emits both conventions

The module SHALL provide `pressure_diagnostic` on a solver and a state,
performing one Leray projection of the unprojected RHS at that
state, and returning **both** the Bernoulli pressure 0-form (the grade-0
potential — the true dynamics is `∂u/∂t = rhs_unproj − ∇B`, so
`(I − P)rhs = +∇B` and `B = p + ½|u|²` at `ρ = 1`) and the static pressure
0-form (Bernoulli minus the kinetic 0-form assembled from `sharp`
magnitudes), as
`PressureZeroForm<R>` values. Documentation SHALL state the extra-solve
cost and the `ρ = 1` convention. The diagnostic SHALL NOT run inside the
step.

#### Scenario: Taylor–Green pressure field is recovered

- **WHEN** the diagnostic runs on the seeded 2D Taylor–Green state
  (`u = (sin kx cos ky, −cos kx sin ky)`)
- **THEN** the static pressure agrees with the analytic
  `p = +¼(cos 2kx + cos 2ky)` of that phase convention, up to its mean
  (the gauge), at the discretization order over the refinement ladder

#### Scenario: The two conventions differ by the kinetic 0-form

- **WHEN** both pressures are returned for any state
- **THEN** their pointwise difference equals `½|u|²` from the same `sharp`
  evaluation, to machine rounding

#### Scenario: Projection failure surfaces, not panics

- **WHEN** the diagnostic's CG is starved of iterations
- **THEN** the call returns the wrapped error
