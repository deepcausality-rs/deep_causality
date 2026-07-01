<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Closed-form integrated-increment kernel contract

A stiff relaxation source SHALL be exposed as a kernel that returns the **analytically integrated increment
over the timestep `Δt`**, not an instantaneous rate. For a first-order relaxation toward a target `x_eq` with
timescale `τ`, the kernel SHALL compute the closed-form exponential update
`x(t+Δt) = x_eq − (x_eq − x(t))·exp(−Δt/τ)`; a nonlinear source SHALL use a linearly-implicit one-step form of
the same shape. `Δt` is supplied by the caller (the `StepContext`). The kernel SHALL remain a pure pointwise
`fn<R: RealField>` with no global solve and no iteration.

#### Scenario: Exactness on linear relaxation
- **WHEN** the relaxation kernel advances a state toward a fixed target over `Δt`
- **THEN** the result equals `x_eq − (x_eq − x₀)·exp(−Δt/τ)` to round-off (an equality, not a tolerance)

#### Scenario: Equilibrium limit
- **WHEN** the relaxation timescale `τ → 0`
- **THEN** the kernel returns the target `x_eq` (the increment jumps the state exactly to equilibrium)

### Requirement: Unconditional stability under stiffness

The LER update SHALL be unconditionally stable: for `τ ≪ Δt` the integrated state SHALL stay bounded and
monotone toward the target, with no overshoot and no oscillation, where an explicit Euler rate-step would
diverge. The stiffness SHALL be confined inside the stage — the marcher timestep and the `PhysicsStage` seam are
not constrained by the source timescale.

#### Scenario: Stable at extreme stiffness
- **WHEN** a relaxation stage is run with `τ = Δt / 1000` over many steps
- **THEN** the carried state stays bounded and approaches the target monotonically (an explicit Euler update on
  the same problem diverges)

### Requirement: LER between-step stage over a state-derived target

The LER stage SHALL be a between-step `PhysicsStage` (the existing `Coupling` seam) that carries one or more
extra scalar states, each relaxing toward an **equilibrium target computed from the current flow state** via the
closed-form kernel above, and SHALL compose statically by cons-tuple with other stages (no `dyn`). The
equilibrium target SHALL be a function of state and/or config — never a hardcoded schedule. The split is
first-order (Lie); a second-order Strang composition SHALL be available as the documented upgrade when
blackout-onset timing requires it.

#### Scenario: Target tracks state
- **WHEN** the flow state driving the equilibrium target changes between two steps
- **THEN** the relaxation target changes accordingly and the carried state relaxes toward the new target

#### Scenario: Static composition preserved
- **WHEN** an LER stage is composed with another stage via `Coupling::between_steps().then(…)`
- **THEN** the composition type-checks with static dispatch and no `dyn`, exactly like the existing
  `ThermalRelax → ViscosityArrhenius` template
