# ks-conformal-propagator Specification

## Purpose
TBD - created by archiving change add-plasma-blackout-corridor. Update Purpose after archive.
## Requirements

### Requirement: 3-D KS regularized two-body propagator

`deep_causality_physics` SHALL provide a 3-D Kustaanheimo–Stiefel (KS) regularized two-body propagator that
advances a Cartesian position/velocity state under a central inverse-square field `μ = GM` as a
**constant-generator matrix exponential** in the KS fictitious time `s` (with `dt = r·ds`), generalizing the
shipped planar `TwoBodyPropagator`. It SHALL be generic over the scalar (tested `f32`/`f64`/`Float106`), use
`from_f64` for all literals, use static dispatch, be singularity-free as `r → 0`, and cite Stiefel & Scheifele
(1971) / Battin (1999) in the docstring with the PDF in `papers/`. It SHALL expose `from_state` →
`propagate(dt)` matching the shipped planar API shape.

#### Scenario: Coast exactness against analytic Kepler
- **WHEN** a bound state is propagated over one full period with monopole gravity only
- **THEN** the state matches an independent analytic Kepler reference to round-off (‖error‖ ≲ 1e-12·a) and beats
  a first-order Euler step measurably

#### Scenario: Exact periodicity and semigroup
- **WHEN** the constant generator is exponentiated over a full period and over two consecutive substeps
- **THEN** the one-period map equals the identity to round-off and `e^{Ωs₁}·e^{Ωs₂} = e^{Ω(s₁+s₂)}` holds to
  round-off

#### Scenario: Conservation invariants
- **WHEN** a bound state is propagated over arbitrary `dt`
- **THEN** specific energy, angular momentum, and the period are conserved to working precision, and
  hyperbolic/degenerate inputs are rejected with a typed `PhysicsError`

### Requirement: Between-step non-conformal perturbation hook

The propagator SHALL provide a between-step perturbation hook applying a caller-supplied non-conformal
acceleration (in physical Cartesian velocity) as a 2nd-order Strang split around the exact KS drift
(half-kick → exact drift → half-kick), never expressing the perturbation inside the KS algebra. The hook's
closure is the consumer of the Stage-0 coupling-interface aero-force channel.

#### Scenario: Second-order split accuracy
- **WHEN** a bound orbit is propagated with a non-conformal mock perturbation via the hook at `H` and `H/2`
- **THEN** the observed order is ≈ 2 (`log₂(err_H/err_{H/2}) ∈ [1.8, 2.2]`) against an RK4 reference of the full
  perturbed EOM

#### Scenario: Error vanishes with the perturbation ratio
- **WHEN** the perturbation magnitude is reduced 10× (`ε → ε/10`)
- **THEN** the split error decreases approximately linearly with `ε`, and the exact core is bit-unchanged when
  the perturbation is zero
