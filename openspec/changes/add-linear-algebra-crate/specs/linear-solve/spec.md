## ADDED Requirements

### Requirement: A dense linear solve is part of the public surface
`deep_causality_linear` SHALL provide `solve(A, b)` for a dense square system, implemented by LU factorisation with partial pivoting.

The workspace has no public dense solve. Its only implementation is `pub(crate) fn solve_linear`
(`algorithms/src/causal_discovery/brcd/brcd_linalg.rs:28`), deliberately kept private to one
estimator, whose own header records why partial pivoting rather than Cholesky: it matches LAPACK
`gesv`, and a Cholesky that floors a computed-non-positive pivot corrupts the solve on
badly-scaled matrices.

#### Scenario: A known system is solved
- **WHEN** a non-singular system with a known solution is solved
- **THEN** the returned vector matches the known solution to the scalar's precision

#### Scenario: A singular system is rejected
- **WHEN** a singular system is solved
- **THEN** the call fails with a typed error rather than returning a value or dividing by zero

#### Scenario: Pivoting handles a zero leading entry
- **WHEN** a non-singular system whose `(0,0)` entry is zero is solved
- **THEN** it succeeds

#### Scenario: Dimension mismatch is rejected
- **WHEN** the right-hand side's length differs from the matrix order
- **THEN** the call fails with a typed error

### Requirement: Solving is preferred over inverting, and the documentation says so
The documentation of `inverse` SHALL direct callers computing `A⁻¹b` to `solve` instead.

`deep_causality_physics` computes the Kalman gain as `K = P Hᵀ S⁻¹` by explicitly inverting `S`
and multiplying (`kernels/dynamics/estimation.rs:158-164`). Explicit inversion is both less accurate
and more work than a solve, and it is there because no solve existed to call. The API should not
let the next caller make the same choice without seeing the alternative.

#### Scenario: The inverse documents the alternative
- **WHEN** `inverse`'s documentation is read
- **THEN** it names `solve` as the operation to use when the inverse is only wanted to multiply by

#### Scenario: A solve is at least as accurate
- **WHEN** an ill-conditioned system is answered by `solve` and by inverting and multiplying
- **THEN** the solve's residual is no larger

### Requirement: The factorisation is reusable
The LU factorisation SHALL be exposed as a value that can be computed once and applied to several right-hand sides.

Factorising costs `O(n³)` and each application costs `O(n²)`. A solve-only API forces a caller with
`k` right-hand sides to pay the cubic cost `k` times. Kalman filtering and the ridge fits in
`brcd` are both repeated-solve workloads.

#### Scenario: One factorisation, several right-hand sides
- **WHEN** a matrix is factorised once and applied to three right-hand sides
- **THEN** each result matches the corresponding single `solve`
- **AND** the matrix is factorised once

#### Scenario: The factorisation carries its permutation
- **WHEN** a factorisation is inspected
- **THEN** the row permutation chosen by pivoting is part of it

#### Scenario: A singular matrix fails at factorisation
- **WHEN** a singular matrix is factorised
- **THEN** the failure is reported at factorisation rather than at the first application

### Requirement: Triangular systems are solved directly
The crate SHALL provide forward and backward substitution for lower- and upper-triangular systems, without factorising.

A triangular system is already factorised; running an LU over it would be quadratically wasteful and
would lose the structure. Substitution is also the operation the LU applications are built from, so
exposing it costs nothing and it is independently useful — Cholesky and QR both produce triangular
factors.

#### Scenario: Backward substitution on an upper-triangular system
- **WHEN** an upper-triangular system is solved by backward substitution
- **THEN** the result matches the same system solved by the general path

#### Scenario: A zero on the diagonal is rejected
- **WHEN** a triangular system with a zero diagonal entry is solved
- **THEN** the call fails with a typed error rather than dividing by zero

#### Scenario: The wrong triangle is rejected
- **WHEN** a matrix with non-zero entries above the diagonal is offered to forward substitution
- **THEN** the call fails rather than silently ignoring them

### Requirement: The sparse and dense solve paths stay distinct
A sparse system SHALL be solved by the iterative solvers, and the dense LU path SHALL NOT be offered for a sparse matrix without an explicit densification by the caller.

LU on a sparse matrix fills in: the factors are dense even when the matrix is not, so applying the
dense path to a large sparse system silently allocates the square. Sparse direct solving needs a
fill-reducing ordering and a symbolic factorisation, which is a separate proposal; the conjugate
gradient solvers already cover the symmetric positive-definite case that the workspace actually uses.

#### Scenario: Densification is explicit
- **WHEN** a caller solves a sparse system by the dense path
- **THEN** the conversion appears at the call site

#### Scenario: The iterative path is available for sparse
- **WHEN** a symmetric positive-definite sparse system is solved
- **THEN** the conjugate gradient solvers apply without densifying
