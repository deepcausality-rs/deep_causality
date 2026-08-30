# linear-integer-algebra Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: The integer determinant is computed without leaving the integers
`deep_causality_linear` SHALL compute the determinant of a matrix over an `EuclideanDomain` by fraction-free elimination, and SHALL NOT divide by a pivot or convert to a floating-point type.

Gaussian elimination divides by its pivot, so over ℤ it leaves the ring immediately: the determinant
of an integer matrix is an integer, but the intermediate quantities are not. Bareiss's fraction-free
elimination keeps every intermediate in the ring — each division it performs is exact, guaranteed by
the integral-domain structure — and reaches the answer in cubic time rather than the factorial time
of Laplace expansion.

`EuclideanDomain` supplies exactly what this needs: `div_euclid` for the exact divisions,
`normalize` for the canonical associate, and `Ord` on `EuclideanValue`.

#### Scenario: An integer determinant is an integer
- **WHEN** the determinant of an `i64` matrix is taken
- **THEN** the result is an `i64` with no rounding
- **AND** it equals the value obtained by exact rational arithmetic

#### Scenario: No floating-point appears
- **WHEN** the integer determinant path is executed
- **THEN** no value is converted to `f32`, `f64` or `Float106`

#### Scenario: Cost is cubic, not factorial
- **WHEN** the determinant of an integer matrix of order 12 is taken
- **THEN** it completes in time consistent with a cubic algorithm

#### Scenario: Agreement with the float path
- **WHEN** the same small matrix is evaluated over `i64` and over `f64`
- **THEN** the integer result equals the float result to within the float path's rounding

### Requirement: Integer rank is exact and carries no tolerance
The crate SHALL compute the rank of a matrix over an `EuclideanDomain` exactly, and SHALL NOT reach that answer through a floating-point decomposition or a threshold.

This is the operation `deep_causality_topology` needs and does not have. Its boundary matrices are
`CsrMatrix<i8>` (`cell_complex/boundary_operator.rs:19`, `cell_complex/mod.rs:94`) — integer matrices
whose entries are `{-1, 0, 1}` — and their rank is currently obtained by densifying to `f64`, running
an SVD, and counting singular values above `1e-5`. The rank of an integer matrix is an exact
question with an exact answer.

#### Scenario: Rank of a known integer matrix
- **WHEN** the exact rank of an integer matrix of known rank is taken
- **THEN** the reported rank equals it, with no tolerance involved

#### Scenario: A matrix the float path gets wrong
- **WHEN** an integer matrix whose singular values straddle the `1e-5` threshold is ranked
- **THEN** the exact path reports the true rank

#### Scenario: No threshold in the signature
- **WHEN** the integer rank function is inspected
- **THEN** it takes no tolerance, epsilon or threshold argument

### Requirement: The exact and numerical rank paths stay distinct
Exact rank over an `EuclideanDomain`, exact rank over 𝔽₂, and numerical rank over a `RealField` SHALL be three separate calls, and no one of them SHALL be silently substituted for another.

There are **two mathematical answers here and one approximation of the first**. Rank over ℤ equals
rank over ℚ — rank is a fraction-field notion, so the integer path computes the characteristic-zero
rank without leaving ℤ. Rank over 𝔽₂ is a genuinely different number. Numerical rank over ℝ is an
approximation of the characteristic-zero one, and `qcl-gaps.md` G-02 records what happens when it is
substituted for either.

The integer path exists because the alternative route to the same number — elimination over
`Rational<i64>` — suffers coefficient growth that overflows a machine integer well before the matrix
gets large. Fraction-free elimination reaches the identical answer while every intermediate stays
in ℤ.

#### Scenario: The integer and rational ranks agree
- **WHEN** the same integer matrix is ranked over `i64` and over `Rational<i64>`
- **THEN** both report the same rank

#### Scenario: The mod-2 rank may differ
- **WHEN** a matrix whose characteristic-zero rank exceeds its 𝔽₂-rank is ranked by both paths
- **THEN** they report different values
- **AND** neither path silently converts the matrix to the other's field

#### Scenario: The integer path survives coefficients a rational path would not
- **WHEN** a matrix is ranked whose rational elimination would overflow `Rational<i64>`
- **THEN** the fraction-free path completes and reports the correct rank

#### Scenario: The caller chooses
- **WHEN** a caller ranks an integer matrix
- **THEN** the choice of exact-integer, exact-mod-2 or numerical rank is made at the call site

### Requirement: Integer matrices support the ring operations without a field bound
Addition, subtraction, negation, scaling, matrix multiplication, matrix–vector multiplication, transpose, trace and the dot product SHALL be available over `CommutativeRing`, and SHALL NOT require `Field`.

None of these divides. Requiring a field for them would exclude ℤ from operations it fully supports,
which is the failure mode the tower exists to prevent — and it is the reason topology's integer
boundary matrices are converted to `f64` before anything is done with them.

#### Scenario: Integer matrix arithmetic
- **WHEN** two `i64` matrices are added, subtracted and multiplied
- **THEN** each operation compiles and returns an `i64` matrix

#### Scenario: An integer boundary operator applies to an integer vector
- **WHEN** a `CsrMatrix<i8>` multiplies an integer vector
- **THEN** the result is integer, with no conversion to a float

#### Scenario: Division is absent from the ring surface
- **WHEN** the operations available over `CommutativeRing` are enumerated
- **THEN** none of them divides

### Requirement: Hermite and Smith normal forms are deferred
The crate SHALL NOT provide Hermite or Smith normal form in this change, and the deferral SHALL be recorded with what it would enable.

They are the integer analogues of RREF and of the singular value decomposition, and Smith normal form
would give integral homology **with torsion** — something no floating-point decomposition can
produce. That makes them valuable rather than optional, and too large to carry alongside everything
else here. Recording the deferral keeps the reason attached to the gap.

#### Scenario: The deferral is documented
- **WHEN** the crate's documentation is read
- **THEN** it records that Hermite and Smith normal forms are absent, and that Smith normal form is
  what integral homology with torsion would require

