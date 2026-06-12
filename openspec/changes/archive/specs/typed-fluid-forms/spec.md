## ADDED Requirements

### Requirement: Typed form carriers with invariant-enforcing constructors
`deep_causality_physics` SHALL provide typed wrappers over topology form carriers —
`VelocityOneForm<R>`, `VorticityTwoForm<R>`, `PressureZeroForm<R>`,
`BodyForceOneForm<R>` — each with private fields, constructors that enforce grade
match against the manifold, length match against `num_cells(k)`, and finiteness of
all coefficients, plus read-only accessors. All types SHALL be generic over
`R: RealField` with no `f64` in any public signature.

#### Scenario: Grade mismatch is unconstructible
- **WHEN** `VorticityTwoForm::new` is called with a field whose length equals `num_cells(1)` instead of `num_cells(2)`
- **THEN** construction returns a typed `PhysicsError` and no value is produced

#### Scenario: Non-finite coefficients are rejected
- **WHEN** any constructor receives a NaN or infinite coefficient
- **THEN** construction returns a typed `PhysicsError`

### Requirement: Velocity carrier rides the Rk4 arrow
`VelocityOneForm<R>` SHALL implement `Clone`, `Add<Output = Self>`, and
`Mul<R, Output = Self>` (and nothing further), so the whole-field state satisfies
the `Rk4`/`Euler` arrow bounds and the march requires no raw-tensor access.

#### Scenario: A field state marches through Rk4
- **WHEN** an `Rk4` arrow with a linear-decay rate closure is run over a `VelocityOneForm<R>` state
- **THEN** it compiles via the arrow's generic bounds and reproduces the analytic decay within integrator tolerance at f32, f64, and Float106

### Requirement: SolenoidalField type-state for divergence-freeness
The workspace SHALL contain exactly one divergence-free velocity type,
`SolenoidalField<R>` in `deep_causality_physics` (unifying the gap note's
`ProjectedVelocityOneForm` and `3DCausalFluidDynamics.md` B4's `SolenoidalField`),
with private fields and exactly two construction paths: the Leray projection
(per-step solver path) and `from_hodge_projection` (per-snapshot analysis path).
It SHALL NOT implement `Add` or `Mul` (the sum of two projected fields is not
discretely projected); a read-only `as_one_form()` accessor exposes the underlying
form without permitting re-wrapping.

#### Scenario: Construction only via projection
- **WHEN** code outside the projection modules attempts to construct `SolenoidalField` from a raw field
- **THEN** compilation fails (no public constructor exists)

#### Scenario: Constructed values are discretely divergence-free
- **WHEN** a `SolenoidalField` is produced from an arbitrary smooth velocity sample via either construction path
- **THEN** the discrete divergence (`δ` of the underlying 1-form) has norm at or below the projection's CG tolerance, verified at f32, f64, and Float106

#### Scenario: No arithmetic on the projected type
- **WHEN** code attempts `projected_a + projected_b`
- **THEN** compilation fails (the operators are deliberately unimplemented)
