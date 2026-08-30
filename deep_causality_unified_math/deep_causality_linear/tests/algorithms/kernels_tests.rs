/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The dense kernels, reached through the decomposition entry points that wrap them.
//!
//! These drive the guards the kernels carry for inputs the ordinary formulas cannot be written for:
//! a reflector whose pivot entry is zero, and a matrix small enough that a product of two column
//! norms underflows.

use deep_causality_linear::{DenseMatrix, MatrixBuild, MatrixView, qr, singular_values, svd};

#[test]
fn test_qr_of_a_matrix_whose_pivot_entry_is_zero() {
    // The Householder phase is normally taken from the pivot entry, `r[j][j] / |r[j][j]|`, which is
    // undefined when that entry is zero while the column below it is not. The permutation matrix
    // [[0,1],[1,0]] is exactly that case at j = 0, and it is orthogonal, so Q and R are determined:
    // R must come out upper triangular and QR must reproduce A.
    let a = DenseMatrix::from_vec(vec![0.0, 1.0, 1.0, 0.0], 2, 2).unwrap();
    let (q, r) = qr(&a).unwrap();
    assert_eq!(q.shape(), (2, 2));
    assert_eq!(r.shape(), (2, 2));

    assert_eq!(
        r.get(1, 0).unwrap(),
        0.0,
        "R is upper triangular, exactly rather than to rounding"
    );

    // QᵀQ = I: the reflector left orthonormal columns rather than a NaN.
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0_f64;
            for k in 0..2 {
                acc += q.get(k, i).unwrap() * q.get(k, j).unwrap();
            }
            let want = if i == j { 1.0 } else { 0.0 };
            assert!((acc - want).abs() < 1e-12, "QᵀQ at ({i}, {j}) was {acc}");
        }
    }

    // QR = A.
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0_f64;
            for k in 0..2 {
                acc += q.get(i, k).unwrap() * r.get(k, j).unwrap();
            }
            assert!(
                (acc - a.get(i, j).unwrap()).abs() < 1e-12,
                "QR at ({i}, {j}) was {acc}"
            );
        }
    }
}

#[test]
fn test_the_singular_values_of_a_matrix_whose_squared_column_norms_multiply_below_the_exponent_range()
 {
    // The Jacobi sweep measures the off-diagonal relative to `sqrt(αβ)`, where α and β are squared
    // column norms. At entries of 2⁻²⁸³ each column norm squared is 2⁻⁵⁶⁶, and their product 2⁻¹¹³²
    // is below the smallest subnormal double, so that scale collapses to zero and the relative test
    // cannot be formed. The columns of a diagonal matrix are already orthogonal, so the answer is
    // still the moduli of the diagonal, exactly -- 2⁻²⁸³ is a power of two and squares and roots
    // back without rounding.
    let tiny = 2.0_f64.powi(-283);
    assert!(tiny > 0.0, "the entry itself is representable");
    assert_eq!(tiny * tiny * (tiny * tiny), 0.0, "its fourth power is not");

    let a = DenseMatrix::from_vec(vec![tiny, 0.0, 0.0, tiny], 2, 2).unwrap();
    let s = singular_values(&a).unwrap();
    assert_eq!(s.len(), 2);
    assert_eq!(s.get(0).unwrap(), tiny);
    assert_eq!(s.get(1).unwrap(), tiny);

    // And the factors still reconstruct: U = V = I for a positive diagonal matrix.
    let (u, _, vt) = svd(&a).unwrap();
    assert_eq!(u.shape(), (2, 2));
    assert_eq!(vt.shape(), (2, 2));
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0;
            for k in 0..2 {
                acc += u.get(i, k).unwrap() * s.get(k).unwrap() * vt.get(k, j).unwrap();
            }
            assert_eq!(acc, a.get(i, j).unwrap(), "reconstruction at ({i}, {j})");
        }
    }
}

// ---- mutation-driven: the QR noise floor was never exercised -----------------------------------

/// A column at subnormal magnitude is skipped rather than reflected.
///
/// `householder_qr` computes a noise floor from the input's Frobenius norm and skips any
/// sub-column below it. The comment on that floor says reflecting such a column makes `vᴴv`
/// denormal and `β = 2/(vᴴv)` overflow, poisoning the result with NaN.
///
/// Nothing tested it. Mutating the Frobenius accumulation from `+=` to `-=` makes the floor
/// negative, so `norm_sq <= floor` never fires and no column is ever skipped — and every QR case
/// still passed, because the smallest column under test was an ordinary zero, which a later guard
/// catches on its own.
///
/// What this test does and does not establish. It pins that a QR with a subnormal column returns a
/// finite, correct factorisation, which is a property worth holding whatever the floor does. It
/// does NOT kill the mutant: the mutated code passes this test in both of the repo's profiles. A
/// standalone crate built with cargo's default release settings (`lto = false`,
/// `codegen-units = 16`, unlike this workspace's `lto = true, codegen-units = 1`) did return a
/// non-finite `Q R` at `1e-320` under the mutation, so the guard is not obviously dead — but that
/// is a statement about one optimisation configuration, not about this build, and no input has
/// been found that demonstrates the mutation is wrong here.
///
#[test]
fn test_a_subnormal_column_is_skipped_and_the_factorisation_stays_finite() {
    for exponent in [-150_i32, -280, -320] {
        let tiny = 10f64.powi(exponent);
        #[rustfmt::skip]
        let entries = vec![
            1.0, tiny, 0.0,
            2.0, tiny, 0.0,
            3.0, tiny, 1.0,
        ];
        let a = DenseMatrix::from_vec(entries.clone(), 3, 3).unwrap();
        let (q, r) = qr(&a).unwrap();

        let k = r.rows();
        for i in 0..3 {
            for j in 0..3 {
                let mut acc = 0.0_f64;
                for t in 0..k {
                    acc += q.get(i, t).unwrap() * r.get(t, j).unwrap();
                }
                assert!(
                    acc.is_finite(),
                    "(Q R)[{i}][{j}] must be finite with a 1e{exponent} column, got {acc}"
                );
                assert!(
                    (acc - entries[i * 3 + j]).abs() < 1e-12,
                    "(Q R)[{i}][{j}] expected {}, got {acc}",
                    entries[i * 3 + j]
                );
            }
        }
    }
}

// ---- mutation-driven: every SVD input is well scaled and at most 3x3 ---------------------------

/// The SVD is scale-equivariant, checked across 600 binary orders of magnitude.
///
/// `jacobi_svd` brings the matrix to unit scale by an exact power of two before sweeping and
/// divides the singular values back afterwards. For a matrix already in `[1, 4)` neither rescale
/// loop executes, and every SVD case in the suite has entries between 1 and 6. Mutation testing
/// left the whole rescale block unkilled as a result: the loop bodies, both guards, the step
/// counter and the reciprocals.
///
/// `A = [[1, 1], [0, 1]]` has `AᴴA = [[1, 1], [1, 2]]`, whose eigenvalues are `(3 ± √5)/2`. Its
/// singular values are therefore `φ = (1 + √5)/2` and `1/φ = (√5 − 1)/2`, exactly. Scaling `A` by
/// `2^k` scales both by `2^k` and nothing else, because a power of two is exact in binary
/// floating point.
#[test]
fn test_the_singular_values_scale_with_the_matrix() {
    let phi = (1.0 + 5f64.sqrt()) / 2.0;
    let inv_phi = (5f64.sqrt() - 1.0) / 2.0;

    for k in [300_i32, 100, 10, 0, -10, -100, -260, -300] {
        let x = 2f64.powi(k);
        let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![x, x, 0.0, x], 2, 2).unwrap();
        let s = singular_values(&m).unwrap();

        let got_hi = s.as_slice()[0] / x;
        let got_lo = s.as_slice()[1] / x;
        assert!(
            (got_hi - phi).abs() < 1e-12,
            "2^{k}: leading singular value over the scale is {got_hi}, must be φ = {phi}"
        );
        assert!(
            (got_lo - inv_phi).abs() < 1e-12,
            "2^{k}: trailing singular value over the scale is {got_lo}, must be 1/φ = {inv_phi}"
        );
    }
}

/// A dense rectangular decomposition, checked against the three properties that define an SVD.
///
/// The suite's SVD inputs are 2x2 or symmetric 3x3. A tall dense matrix with no zero entries
/// drives many more Jacobi rotations and makes the column-norm scan, the off-diagonal measure and
/// the final normalisation all observable.
#[test]
fn test_a_tall_dense_svd_satisfies_its_defining_properties() {
    const M: usize = 6;
    const N: usize = 4;
    // Entries chosen so no two columns are proportional and none is zero.
    let entries: Vec<f64> = (0..M * N)
        .map(|t| {
            let (i, j) = (t / N, t % N);
            1.0 + ((i * 7 + j * 5) % 9) as f64 + (j as f64) * 0.5
        })
        .collect();
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(entries.clone(), M, N).unwrap();
    let (u, s, vt) = svd(&a).unwrap();

    // 1. Singular values are non-negative and descending.
    for w in s.as_slice().windows(2) {
        assert!(
            w[0] >= w[1],
            "singular values must descend, got {:?}",
            s.as_slice()
        );
    }
    assert!(s.as_slice().iter().all(|v| *v >= 0.0));

    // 2. A = U Σ Vᴴ.
    let k = s.len();
    for i in 0..M {
        for j in 0..N {
            let mut acc = 0.0_f64;
            for t in 0..k {
                acc += u.get(i, t).unwrap() * s.as_slice()[t] * vt.get(t, j).unwrap();
            }
            assert!(
                (acc - entries[i * N + j]).abs() < 1e-10,
                "(U Σ Vᴴ)[{i}][{j}] expected {}, got {acc}",
                entries[i * N + j]
            );
        }
    }

    // 3. The columns of U and the rows of Vᴴ are orthonormal.
    for p in 0..k {
        for q in 0..k {
            let mut du = 0.0_f64;
            let mut dv = 0.0_f64;
            for i in 0..M {
                du += u.get(i, p).unwrap() * u.get(i, q).unwrap();
            }
            for j in 0..N {
                dv += vt.get(p, j).unwrap() * vt.get(q, j).unwrap();
            }
            let want = if p == q { 1.0 } else { 0.0 };
            assert!(
                (du - want).abs() < 1e-10,
                "UᴴU[{p}][{q}] = {du}, want {want}"
            );
            assert!(
                (dv - want).abs() < 1e-10,
                "V Vᴴ[{p}][{q}] = {dv}, want {want}"
            );
        }
    }
}

/// A non-empty all-zero matrix decomposes to all-zero singular values, with no NaN.
///
/// The rescale block is guarded by `max_diag > 0 && max_diag.is_finite()`. Both halves matter and
/// neither was tested: the suite's only zero matrices have a zero *dimension*, which returns early
/// and never reaches the kernel. Weakening the guard to `||` lets a zero matrix enter the rescale,
/// where `max_diag` stays zero however many times it is multiplied, the loop runs to its step
/// bound, and the scale reaches infinity — after which every entry is `0 · ∞`.
#[test]
fn test_an_all_zero_matrix_has_zero_singular_values() {
    for (rows, cols) in [(3_usize, 3_usize), (4, 2), (2, 5)] {
        let m: DenseMatrix<f64> = DenseMatrix::zeros(rows, cols);
        let s = singular_values(&m).unwrap();

        assert_eq!(s.len(), rows.min(cols));
        for (k, v) in s.as_slice().iter().enumerate() {
            assert!(
                v.is_finite(),
                "{rows}x{cols}: singular value {k} must be finite, got {v}"
            );
            assert_eq!(*v, 0.0, "{rows}x{cols}: singular value {k} must be zero");
        }
    }
}

/// A matrix whose squared column norm overflows still decomposes.
///
/// The other half of the same guard. An entry above `√MAX` squares to infinity, so `max_diag` is
/// infinite; the rescale is skipped and the sweep works at the caller's scale rather than trying
/// to bring an infinity into range. Weakening the guard sends it into the loop, where `∞ · ¼` is
/// still `∞`, the step bound is what stops it, and the scale underflows to zero.
#[test]
fn test_a_matrix_whose_squared_norm_overflows_still_decomposes() {
    // 1e200 squares to 1e400, beyond f64's range.
    let big = 1e200_f64;
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![big, 0.0, 0.0, big], 2, 2).unwrap();
    let s = singular_values(&m).unwrap();

    for (k, v) in s.as_slice().iter().enumerate() {
        assert!(v.is_finite(), "singular value {k} must be finite, got {v}");
        assert!(
            (v - big).abs() < big * 1e-12,
            "singular value {k} of diag({big}, {big}) must be {big}, got {v}"
        );
    }
}

/// A column numerically zero against the others is skipped, and its singular value is ~0.
///
/// `jacobi_svd` computes a floor from the largest column norm and skips any pair whose column
/// falls below it, because rotating a near-zero column drives `gmod → 0` and the angle to a
/// pathological value. Every SVD input in the suite has columns of comparable magnitude, so the
/// floor, the scan that feeds it, and the guard that reads it were all unobserved — nine surviving
/// mutants between them.
///
/// The matrix below is rank two by construction: two ordinary columns and one at `1e-170` times
/// their scale, which is below `‖A‖·ε` and therefore numerically zero. Its singular values are the
/// two of the rank-two part and one indistinguishable from zero.
#[test]
fn test_a_column_below_the_noise_floor_is_treated_as_zero() {
    // Columns 0 and 2 are ordinary; column 1 is 1e-170 times their scale.
    let t = 1e-170_f64;
    #[rustfmt::skip]
    let entries = vec![
        3.0, t,   1.0,
        1.0, t,   4.0,
        2.0, t,   2.0,
        4.0, t,   1.0,
    ];
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(entries.clone(), 4, 3).unwrap();
    let (u, s, vt) = svd(&a).unwrap();

    assert_eq!(s.len(), 3);
    for (k, v) in s.as_slice().iter().enumerate() {
        assert!(v.is_finite(), "singular value {k} must be finite, got {v}");
    }
    // Two significant values and one at the noise level.
    assert!(
        s.as_slice()[0] > 1.0 && s.as_slice()[1] > 1.0,
        "the rank-two part has two significant singular values, got {:?}",
        s.as_slice()
    );
    assert!(
        s.as_slice()[2] < 1e-100,
        "the numerically-zero column must give a singular value at the noise level, got {}",
        s.as_slice()[2]
    );

    // The decomposition still reconstructs the significant part of the matrix.
    for i in 0..4 {
        for j in 0..3 {
            let mut acc = 0.0_f64;
            for k in 0..3 {
                acc += u.get(i, k).unwrap() * s.as_slice()[k] * vt.get(k, j).unwrap();
            }
            assert!(
                (acc - entries[i * 3 + j]).abs() < 1e-10,
                "(U Σ Vᴴ)[{i}][{j}] expected {}, got {acc}",
                entries[i * 3 + j]
            );
        }
    }
}

// ---- mutation-driven: the QR sign convention and the QR noise floor ----------------------------
//
// One fact shapes all three tests below. `householder_qr` cleans R's strictly-lower triangle to
// exact zero on the way out, so a caller reading R as triangular is not left deciding whether
// `1e-17` counts. That cleanup is unconditional, and it runs whether or not the reflector for that
// column ran. No assertion on R below the diagonal can therefore detect a reflector that was
// skipped, and the first version of these tests asserted exactly that and could not fail.
//
// The reconstruction is what the cleanup does expose. Zeroing an entry that no reflector
// eliminated discards it, so `Q·R` stops being `A` by the size of the entry thrown away. Every
// test here checks the product.

/// The reflector's sign convention, on the column that needs it.
///
/// `α = −phase(r[j,j])·‖x‖` picks the sign *away* from the pivot, so that `v[j] = x[j] − α` adds
/// two quantities of like sign rather than subtracting two nearly equal ones. The other sign is
/// still a reflector and `H` is still orthogonal, so nothing about orthonormality separates them.
/// Two mutants lived on that: deleting the negation, and turning `v[j] -= alpha` into `+=`.
///
/// A column already nearly along `e_j` separates them. Here `‖x‖` for column 0 rounds to exactly 1
/// and `x[0]` is exactly 1, so the wrong sign gives `v[0] = 0`, the reflector is built from the
/// `1e-8` tail alone, and the entries below the diagonal are left at `1e-8` — which the cleanup
/// then discards, moving `Q·R` away from `A` by that much.
#[test]
fn test_the_qr_sign_convention_survives_a_nearly_aligned_column() {
    #[rustfmt::skip]
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(
        vec![
            1.0,  2.0, 3.0,
            1e-8, 1.0, 4.0,
            1e-8, 0.0, 1.0,
        ],
        3, 3,
    ).unwrap();
    let (q, r) = qr(&a).unwrap();
    assert_reconstructs(&q, &r, &a, 1e-12);
}

/// A sub-column whose squared norm is denormal has to be skipped, not reflected.
///
/// This is the case the floor exists for, and the one no input had reached. A sub-column of
/// `1e-160` has a squared norm near `2e-320`: denormal, but not zero, so it survives the later
/// `v_norm_sq <= 0` guard and reaches `β = 2/(vᴴv)`, which is `3e319` and overflows to infinity.
/// Every entry the reflector touches then comes back non-finite.
///
/// Earlier attempts used `1e-170` and `1e-280`, whose squares underflow to exactly zero; the later
/// guard catches those on its own, which is why they left the floor's mutants alive. The window is
/// narrow — the squared norm has to land between `5e-324` and `2.2e-308` — and `1e-160` is inside
/// it.
#[test]
fn test_a_sub_column_with_a_denormal_squared_norm_is_skipped_rather_than_reflected() {
    #[rustfmt::skip]
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(
        vec![
            1.0, 1.0,    1.0,
            0.0, 1e-160, 1.0,
            0.0, 1e-160, 1.0,
        ],
        3, 3,
    ).unwrap();
    let (q, r) = qr(&a).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                q.get(i, j).unwrap().is_finite(),
                "Q({i}, {j}) was {}",
                q.get(i, j).unwrap()
            );
            assert!(
                r.get(i, j).unwrap().is_finite(),
                "R({i}, {j}) was {}",
                r.get(i, j).unwrap()
            );
        }
    }
}

/// A sub-column can be small and still be real, and the floor has to leave it alone.
///
/// The floor is `‖A‖²_F · ε²`, and both factors of `ε` are needed, because `‖A‖²_F` is a squared
/// quantity and what is compared against it is squared too. Dropping to one factor raises the
/// floor by about `1e16`. No earlier matrix noticed, because every sub-column in the suite was
/// either within a factor of a few of the largest entry or exactly zero.
///
/// Column 1 below the diagonal here is `[1e-9, 1e-9]`, a squared norm of `2e-18`. The correct
/// floor is near `2e-31` and lets it through, so the reflector runs. A floor of `‖A‖²_F · ε` is
/// near `1e-15`, swallows the sub-column and skips the reflector, and the cleanup then discards the
/// `1e-9` the reflector was supposed to eliminate. `Q·R` misses `A` by that much.
#[test]
fn test_a_small_but_genuine_sub_column_is_not_swallowed_by_the_qr_noise_floor() {
    #[rustfmt::skip]
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(
        vec![
            1.0, 1.0,  1.0,
            0.0, 1e-9, 1.0,
            0.0, 1e-9, 1.0,
        ],
        3, 3,
    ).unwrap();
    let (q, r) = qr(&a).unwrap();
    assert_reconstructs(&q, &r, &a, 1e-12);
}

/// `Q·R` against `A`, entry by entry.
fn assert_reconstructs(q: &DenseMatrix<f64>, r: &DenseMatrix<f64>, a: &DenseMatrix<f64>, tol: f64) {
    let (rows, cols) = (a.rows(), a.cols());
    let k = r.rows();
    for i in 0..rows {
        for j in 0..cols {
            let mut acc = 0.0_f64;
            for t in 0..k {
                acc += q.get(i, t).unwrap() * r.get(t, j).unwrap();
            }
            assert!(
                (acc - a.get(i, j).unwrap()).abs() < tol,
                "QR at ({i}, {j}) was {acc}, A has {}",
                a.get(i, j).unwrap()
            );
        }
    }
}
