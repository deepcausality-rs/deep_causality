## ADDED Requirements

### Requirement: Finite-rate network rate kernels extend the pointwise set

The `kernels/hypersonic/` pointwise set SHALL gain the finite-rate network kernels: the
dissociative-recombination rate (the reverse of the associative-ionization channel, derived from the forward
rate and the Park equilibrium-constant fit), the thresholded electron-impact ionization rates for N and O,
and the Park equilibrium-constant curve-fit kernel itself. Each SHALL follow the shipped kernel contract: a
pure free `fn name_kernel<R: RealField>(...) -> Result<Quantity<R>, PhysicsError>`, no captured state, no
spatial discretization, a `PropagatingEffect` wrapper, registration in `kernels/hypersonic/mod.rs`, and
flattening at `lib.rs`.

#### Scenario: New kernels are pure and cited
- **WHEN** the new kernel sources are inspected
- **THEN** each carries a full citation to the Park (1990) source in its docstring, holds no state, and its
  only float literals are cited coefficients defined in `constants/`

### Requirement: Per-reaction validation extends to two-way properties

The pointwise validation SHALL extend to the two-way network properties, in `deep_causality_physics` and in
isolation: equilibrium recovery from both sides per reaction, detailed balance (`k_f / k_b` reproduces the
equilibrium-constant fit across the tabulated temperature range), and frozen limits at low temperature for
every channel including the reverse ones.

#### Scenario: Detailed balance across the tabulated range
- **WHEN** the forward and derived backward rates are evaluated across the Park fit's stated temperature
  range
- **THEN** their ratio reproduces the equilibrium-constant kernel at every sampled temperature within
  rounding tolerance
