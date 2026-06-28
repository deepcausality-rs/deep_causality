## ADDED Requirements

### Requirement: Cache-blocked dense matrix product

The shared dense `matmul` SHALL be implemented with a cache-blocked, right-operand-transposed loop so
both operands are accessed with unit stride in the hot loop — covering the tensor-train contractions
(`to_dense`, MPO `apply`/`compose`, rounding, canonicalization). It SHALL remain dependency-free (no BLAS),
SHALL NOT require `T: Default` (so the dual scalar is still admitted), and SHALL return results
identical to the prior naive implementation.

#### Scenario: Blocked matmul matches the naive product
- **WHEN** the blocked `matmul` and a reference naive product are applied to the same operands
- **THEN** the results are equal to working precision at every supported scalar type

#### Scenario: No new dependency and no `Default` bound
- **WHEN** the crate is built
- **THEN** the blocked `matmul` introduces no external runtime dependency and the `Dual` scalar path
  still compiles and passes

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
