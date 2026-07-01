# tensor-network-numerics Specification

## Purpose
TBD - created by archiving change add-tensor-network. Update Purpose after archive.
## Requirements
### Requirement: Robust truncated thin-SVD

The crate SHALL provide a truncated thin-SVD on `CausalTensor<T>` that, given a `Truncation<T>` policy,
returns factors `(U, S, Vt)` retaining only the selected rank, with `U` and `Vt` orthonormal to working
precision. It SHALL be an addition to `ops/tensor_svd*` and MUST NOT alter the behavior of the existing
public `svd`.

#### Scenario: Rank truncation by bond cap
- **WHEN** `svd_truncated` is called with `Truncation::by_bond(k)` on a matrix of rank `≥ k`
- **THEN** `S` has exactly `k` singular values in non-increasing order and `U`,`Vt` have the matching
  `k` columns/rows

#### Scenario: Rank truncation by tolerance
- **WHEN** `svd_truncated` is called with a relative tolerance `rel_tol`
- **THEN** every retained singular value satisfies `σ_i / σ_0 ≥ rel_tol` and all smaller ones are dropped

#### Scenario: Orthonormal factors on clustered spectra
- **WHEN** the input has clustered or repeated singular values
- **THEN** the returned factors satisfy `‖Uᵀ U − I‖ ≤ k·ε` and `‖Vt Vtᵀ − I‖ ≤ k·ε`

### Requirement: Householder QR

The crate SHALL provide a Householder QR decomposition on `CausalTensor<T>` returning `(Q, R)` with `Q`
orthonormal and `R` upper-triangular, used as the canonicalization primitive for tensor trains.

#### Scenario: Reconstruction and orthonormality
- **WHEN** `qr` is called on a real or complex matrix `A`
- **THEN** `‖Q R − A‖ ≤ k·ε` and `‖Qᴴ Q − I‖ ≤ k·ε`

### Requirement: Truncation policy type

The crate SHALL provide a `Truncation<T>` value type carrying a maximum bond cap and relative/absolute
tolerance gates, threaded explicitly into every lossy operation. A singular value SHALL be kept iff its
index is below the bond cap AND it passes both tolerance gates.

#### Scenario: Construction helpers
- **WHEN** a `Truncation` is built via `by_bond`, `by_tol`, or `new`
- **THEN** the resulting policy applies the documented keep rule

#### Scenario: Invalid policy rejected
- **WHEN** a `Truncation` is constructed with a zero bond cap or a negative tolerance
- **THEN** construction returns `CausalTensorError::InvalidParameter`

### Requirement: Layered precision-generic scalar bounds

All Stage-0 routines SHALL be generic over the scalar and SHALL bound on the crate-native traits, not on a
flat `RealField`: the norm/orthogonality layer on `Normed` (using `Normed::Real` for magnitudes and
tolerances), the analytic/division layer on `Scalar` (`Real + Div + FromPrimitive`), and complex
conjugation via `ComplexField`. No concrete `f32`/`f64` literal SHALL appear in lib code; constants come
from the scalar's own API (`T::zero/one/epsilon/pi`, `FromPrimitive`).

#### Scenario: Magnitude comparisons via the real subfield
- **WHEN** a routine compares magnitudes to choose a pivot or truncate a singular value
- **THEN** the comparison is performed on `Normed::Real` (ordered), never on an assumed ordering of `T`

#### Scenario: Tolerances derived from precision
- **WHEN** a routine needs a default tolerance
- **THEN** it is derived from `T::epsilon()` so it scales with the working precision

### Requirement: Reference-checked numerics across precisions

The Stage-0 SVD and QR SHALL be validated against checked-in full-precision reference fixtures and SHALL
pass at `f32`, `f64`, and `Float106` with precision-appropriate tolerances.

#### Scenario: Cross-precision fixture validation
- **WHEN** the SVD/QR test suite runs at each of `f32`, `f64`, `Float106`
- **THEN** singular values and orthogonality residuals match the reference fixtures within the
  precision-appropriate tolerance

### Requirement: Numerically robust SVD/QR on rank-deficient input

The truncated SVD (one-sided Jacobi) and the Householder QR SHALL return **finite** factors on
rank-deficient matrices at every supported precision, including the double-double `Float106`. A
near-zero column (which arises whenever a low-rank train is canonicalized or rounded) MUST NOT produce
non-finite singular values or factors: the kernels SHALL compute `sqrt(1 + ζ²)` without overflow and
SHALL skip the rotation/reflector for any column whose norm falls below the `‖A‖·ε` noise floor (such a
column is numerically zero and its singular value is ~0). The rank gate (`retained_rank`) consequently
sees only finite singular values and compresses to the true numerical rank.

#### Scenario: Rank-deficient SVD stays finite and ranks correctly
- **WHEN** a rank-deficient matrix (a near-zero column from a canonicalized low-rank train) is
  factored by the truncated SVD at `f64` or `Float106`
- **THEN** all singular values are finite, and the negligible ones are below the relative tolerance so
  the retained rank equals the true numerical rank

#### Scenario: Householder QR stays finite on a near-zero column
- **WHEN** the QR factorizes a matrix with a numerically-zero column at `Float106`
- **THEN** `Q` and `R` are finite (no `β = 2/(vᴴv)` overflow), and `Q·R` reconstructs the input

#### Scenario: Deterministic round compresses an exactly-low-rank train at every precision
- **WHEN** an exactly-rank-`r` train with an inflated bond is rounded by tolerance at `f64` or `Float106`
- **THEN** the round compresses the interior bonds back to `r` (it does not keep the inflated bond)

### Requirement: Optional randomized range-finder truncated SVD

The numerics layer SHALL provide an optional **randomized range-finder** variant of the truncated SVD,
selectable through the `Truncation` policy, that trades the one-sided Jacobi kernel's high relative
accuracy for speed when the caller's tolerance permits. When selected, it SHALL produce factors
`U`/`S`/`Vᴴ` whose reconstruction matches the input to the requested tolerance, and the deterministic
Jacobi SVD SHALL remain the default.

#### Scenario: Randomized SVD reconstructs to tolerance
- **WHEN** the randomized range-finder SVD factors a matrix at relative tolerance `τ`
- **THEN** `U · diag(S) · Vᴴ` reproduces the input to within `τ`, with orthonormal `U`/`V`

#### Scenario: Default SVD is unchanged
- **WHEN** `svd_truncated` is called without selecting the randomized variant
- **THEN** the deterministic one-sided Jacobi SVD is used unchanged

