## ADDED Requirements

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
