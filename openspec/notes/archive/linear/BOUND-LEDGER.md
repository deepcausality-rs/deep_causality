<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Bound ledger

Task 1.8b. Every operation whose bound is weaker than the code being replaced, with the number set
that loosening newly admits and the test that instantiates it. A loosened bound nothing exercises is
indistinguishable from an untested one, so phase 2 owes a test for each row.

## Loosened off an ad-hoc operator bundle

The sparse code states bounds as collections of `core::ops` traits. Each is a semiring written
longhand: it makes no algebraic claim, records nothing about why those operators, and leaves a reader
unable to tell whether ℕ is admitted deliberately.

| operation | bound being replaced | new bound | newly admits | phase-2 test |
|---|---|---|---|---|
| `CsrMatrix::mat_mult` | `Copy + Clone + Mul + Zero + PartialEq + Default` | `CommutativeSemiring` | nothing new; states the claim | `u64` and `i64` instantiation |
| `CsrMatrix::transpose` | `Copy + Zero` | `CommutativeSemiring` | nothing new; states the claim | `u64` instantiation |
| `CsrMatrix::vec_mult` | `Copy + Zero + Add + Mul` | `CommutativeSemiring` | nothing new; states the claim | `u64` instantiation |

These three are restatements rather than widenings. The set of admitted types is the same; what
changes is that the bound now says what it means.

## Genuinely loosened

| operation | bound a float-first design would take | new bound | newly admits | phase-2 test |
|---|---|---|---|---|
| `DenseVector::dot` | `RealField` | `CommutativeSemiring` | ℕ, ℤ | dot over `u64` and `i64` |
| `DenseVector::outer` | `RealField` | `CommutativeSemiring` | ℕ, ℤ | outer over `i64` |
| `DenseVector::add` | `RealField` | `CommutativeRing` | ℤ | add over `i64` |
| `DenseVector::sub` | `RealField` | `CommutativeRing` | ℤ | sub over `i64`; must FAIL over `u64` |
| `DenseVector::scale` | `RealField` | `CommutativeRing` | ℤ | scale over `i64` |
| `determinant_exact` | `RealField` via densify-and-SVD | `EuclideanDomain` | ℤ, exactly | `i64` determinant equals the rational answer |
| `rank_exact` | `RealField` with a `1e-5` threshold | `EuclideanDomain` | ℤ, exactly | rank of a matrix whose singular values straddle `1e-5` |
| `rref` / `rank` / `kernel_basis` / `image_basis` | `RealField` | `Field` | 𝔽₂, ℚ | `rref` over `Gf2` and over `Rational<i64>` |
| `matrix_norm_*` | `RealField` | `NormedScalar` | ℂ | Frobenius norm of a `Complex<f64>` matrix |
| `DenseVector::hermitian_inner` | `RealField` | `ConjugateScalar` | ℂ | `⟨v, v⟩` real and non-negative over ℂ |

## Deliberately not loosened

| operation | bound | why it stays |
|---|---|---|
| `rref_stable` / `rank_stable` / `determinant` / `solve` / `Lu` / `inverse` | `NormedScalar` | they pivot by magnitude, which needs a modulus landing in an ordered real |
| `svd` / `svd_decomp` / `svd_truncated` / `qr` / `eigen_hermitian` | `RealField` | iterative, compare magnitudes, take square roots |
| `cg_solve*` | `RealField` | the residual test is a magnitude comparison against a tolerance |

## The cost of the pivot split, recorded

`solve`, `Lu`, `inverse` and the pivoted `determinant` are bounded on `NormedScalar` and therefore
unavailable over `Rational<i64>` and over a dense 𝔽₂ matrix, neither of which is normed. No scenario
in this change needs them there — ℚ appears only in `rref` and in the rank-agreement test — and the
elimination core is already generic over the exact rule, so an exact `solve` is a later entry point
rather than a redesign.
