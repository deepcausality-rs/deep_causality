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

