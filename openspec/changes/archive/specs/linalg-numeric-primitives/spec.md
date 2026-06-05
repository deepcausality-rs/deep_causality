# linalg-numeric-primitives Specification

## Purpose
TBD - created by archiving change brcd-prep-foundations. Update Purpose after archive.
## Requirements
### Requirement: Numeric primitives are generic over RealField precision

Every numeric primitive in this capability SHALL be generic over a precision type `T: RealField` (with `FromPrimitive` added where integer-to-real conversion is required), so it composes with the `real-field-discovery` crates and the wider numerical stack. Each SHALL run at any supported precision without code change.

#### Scenario: A primitive runs at multiple precisions
- **WHEN** a primitive is instantiated at `f32`, `f64`, and `Float106`
- **THEN** it compiles and produces correct results at each precision

#### Scenario: No f64 is hardwired in the generic path
- **WHEN** a primitive's source is compiled
- **THEN** it uses `RealField` operations and constructed constants only, with no `f64`-typed intermediate in the generic code path

### Requirement: Public SPD conjugate-gradient solver

The system SHALL provide a publicly accessible conjugate-gradient solver that solves `A x = b` for a symmetric positive semi-definite operator `A`, supplied as a matrix-free apply closure, to a caller-specified relative-residual tolerance and iteration budget. The solver SHALL report non-convergence rather than returning an unconverged result silently.

#### Scenario: Solves a known SPD system within tolerance
- **WHEN** the solver is given an SPD matrix `A`, a right-hand side `b`, a tolerance, and a sufficient iteration budget
- **THEN** it returns `x` whose residual `‖A x − b‖ / ‖b‖` is below the tolerance

#### Scenario: Reports non-convergence at the iteration cap
- **WHEN** the iteration budget is too small to reach the tolerance
- **THEN** the solver returns a non-convergence error carrying the iteration count and final residual

#### Scenario: Zero right-hand side yields the zero solution
- **WHEN** `b` is the zero vector
- **THEN** the solver returns the zero vector

### Requirement: Sample mean and covariance over CausalTensor

The system SHALL compute the column means and the sample covariance matrix (`ddof = 1`) of a two-dimensional `CausalTensor` whose rows are observations and whose columns are variables.

#### Scenario: Covariance of known data matches the closed form
- **WHEN** a tensor of observations is supplied
- **THEN** the returned covariance matrix equals the sample covariance computed by definition within numerical tolerance

#### Scenario: Single observation is rejected or floored
- **WHEN** the tensor has fewer than two rows
- **THEN** the function returns a documented error or a defined fallback rather than dividing by zero

### Requirement: Numerically stable logsumexp

The system SHALL compute `log(Σ_i exp(x_i))` using the max-shift formulation so that the result does not overflow for large inputs.

#### Scenario: Matches the naive computation for small inputs
- **WHEN** the inputs are small enough that naive evaluation is safe
- **THEN** `logsumexp` agrees with `log(sum(exp(x)))` within numerical tolerance

#### Scenario: Does not overflow for large inputs
- **WHEN** the inputs include large positive values
- **THEN** `logsumexp` returns a finite, correct result

### Requirement: Gaussian log-density

The system SHALL compute the one-dimensional normal log-density given a value, a mean, and a variance, flooring non-positive variance to a small positive constant.

#### Scenario: Matches the closed-form normal log-density
- **WHEN** value, mean, and a positive variance are supplied
- **THEN** the result equals `-0.5 (log(2π var) + (x − μ)² / var)` within numerical tolerance

#### Scenario: Non-positive variance is floored
- **WHEN** the supplied variance is zero or negative
- **THEN** the variance is replaced by a small positive constant and a finite density is returned

### Requirement: Conditional variance via covariance Schur complement

The system SHALL compute the conditional variance of a target variable given a parent set as the Schur complement `Σ_yy − Σ_yP Σ_PP⁻¹ Σ_Py` of the covariance matrix, with ridge regularization on `Σ_PP` for numerical stability.

#### Scenario: Empty parent set returns the marginal variance
- **WHEN** the parent set is empty
- **THEN** the conditional variance equals the target's marginal variance

#### Scenario: Known multivariate-normal block returns the expected conditional variance
- **WHEN** a covariance matrix with a known conditional structure is supplied
- **THEN** the computed conditional variance matches the analytic value within numerical tolerance

#### Scenario: Singular parent block is stabilized by ridge
- **WHEN** `Σ_PP` is singular or near-singular
- **THEN** ridge regularization keeps the solve finite and the result well-defined

