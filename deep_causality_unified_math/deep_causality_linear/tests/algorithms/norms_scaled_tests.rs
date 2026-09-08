/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The scaled Euclidean norm, and the sparse row-major conversion behind the dense kernels.
//!
//! # The oracle
//!
//! Every magnitude assertion here rests on one closed form evaluated by hand: the `3-4-5`
//! Pythagorean triple, scaled by a power of two. `‖(3·2ᵏ, 4·2ᵏ)‖ = 5·2ᵏ` holds exactly in binary
//! floating point at every precision, because `3`, `4` and `5` are exact significands and `2ᵏ` only
//! moves an exponent. The intermediate the scaled form takes is exact too — `1 + (3/4)² = 1.5625`
//! and `√1.5625 = 1.25`, both of which have terminating binary expansions short enough for
//! `BFloat16`'s eight-bit significand — so the tests assert **equality**, not a tolerance.
//!
//! Choosing `k` per type is what makes the same closed form probe each type's own extremes:
//! `(3·2ᵏ)²` must overflow while `5·2ᵏ` stays representable, and `(3·2⁻ᵏ)²` must underflow to zero
//! while `5·2⁻ᵏ` stays normal. `k = 100` does that for the eight-bit exponents (`f32`, `BFloat16`)
//! and `k = 600` for the eleven-bit ones (`f64`, `Float106`).
//!
//! # Corner cases, enumerated before the suite
//!
//! Vector norm: the empty slice; an all-zero vector; one component; the ordinary range against a
//! hand closed form; the ordinary range against the naive form it replaces; a component near the
//! maximum; components near the minimum positive value; a huge and a tiny component together;
//! complex components; a `NaN`; one infinity; two infinities; negative components.
//!
//! Frobenius norm: a known matrix; a large entry; the identity with the flattened two-norm, held at
//! the extremes rather than only in the middle.
//!
//! Sparse conversion: agreement with a position-by-position read; unstored positions; the three
//! empty shapes; a row storing nothing; a shape whose entry count overflows a `usize`; and the four
//! dense kernels reached from a sparse matrix.

use deep_causality_algebra::RealField;
use deep_causality_linear::{
    CsrMatrix, DenseMatrix, LinearErrorEnum, MatrixBuild, MatrixView, cholesky, eigen_hermitian,
    matrix_norm_frobenius, qr, svd, vector_norm_l2, vector_norm_sq,
};
use deep_causality_num::{BFloat16, Float106, FromPrimitive};
use deep_causality_num_complex::Complex;

/// `k` for the eight-bit exponents: `(3·2¹⁰⁰)²` is `9·2²⁰⁰`, past `f32`'s `2¹²⁸`, and `5·2¹⁰⁰`
/// is well inside it.
const NARROW_K: i32 = 100;
/// `k` for the eleven-bit exponents: `(3·2⁶⁰⁰)²` is `9·2¹²⁰⁰`, past `f64`'s `2¹⁰²⁴`.
const WIDE_K: i32 = 600;

/// The triple `(3·2ᵏ, 4·2ᵏ)` and its hand-evaluated norm `5·2ᵏ`, in the target type.
///
/// Built through `f64` because every value involved is exact there for the `k` in use, so the
/// conversion rounds nothing.
fn triple<T: RealField + FromPrimitive>(k: i32) -> ([T; 2], T) {
    let p = 2f64.powi(k);
    let at = |m: f64| T::from_f64(m * p).expect("the scaled triple is representable");
    ([at(3.0), at(4.0)], at(5.0))
}

// =============================================================================
// vector_norm_l2 — the defect
// =============================================================================

/// The overflow case, generic so the same closed form runs at each precision.
fn large_component_stays_finite<T: RealField + FromPrimitive + core::fmt::Debug>(k: i32) {
    let (v, expected) = triple::<T>(k);
    // The defect this replaces: the squared norm reaches infinity for a norm that is representable.
    assert!(
        !vector_norm_sq(&v).is_finite(),
        "the premise of this test is that the naive form overflows here"
    );
    let n = vector_norm_l2(&v);
    assert!(
        n.is_finite(),
        "the norm is representable and must be finite"
    );
    assert_eq!(n, expected);
}

/// The underflow case, at the same closed form scaled down instead of up.
fn small_components_stay_nonzero<T: RealField + FromPrimitive + core::fmt::Debug>(k: i32) {
    let (v, expected) = triple::<T>(-k);
    // The defect at the other end: the squares fall below the smallest subnormal and vanish.
    assert_eq!(
        vector_norm_sq(&v),
        T::zero(),
        "the premise of this test is that the naive form underflows here"
    );
    let n = vector_norm_l2(&v);
    assert!(
        n > T::zero(),
        "the norm is representable and must be non-zero"
    );
    assert_eq!(n, expected);
}

#[test]
fn test_a_large_component_does_not_overflow_at_f32() {
    large_component_stays_finite::<f32>(NARROW_K);
}

#[test]
fn test_a_large_component_does_not_overflow_at_f64() {
    large_component_stays_finite::<f64>(WIDE_K);
}

#[test]
fn test_a_large_component_does_not_overflow_at_float106() {
    large_component_stays_finite::<Float106>(WIDE_K);
}

#[test]
fn test_a_large_component_does_not_overflow_at_bfloat16() {
    large_component_stays_finite::<BFloat16>(NARROW_K);
}

#[test]
fn test_small_components_do_not_underflow_at_f32() {
    small_components_stay_nonzero::<f32>(NARROW_K);
}

#[test]
fn test_small_components_do_not_underflow_at_f64() {
    small_components_stay_nonzero::<f64>(WIDE_K);
}

#[test]
fn test_small_components_do_not_underflow_at_float106() {
    small_components_stay_nonzero::<Float106>(WIDE_K);
}

#[test]
fn test_small_components_do_not_underflow_at_bfloat16() {
    small_components_stay_nonzero::<BFloat16>(NARROW_K);
}

// =============================================================================
// vector_norm_l2 — the ordinary range and the degenerate inputs
// =============================================================================

#[test]
fn test_the_empty_vector_has_zero_norm() {
    let v: [f64; 0] = [];
    assert_eq!(vector_norm_l2(&v), 0.0);
}

#[test]
fn test_an_all_zero_vector_has_zero_norm_and_no_nan() {
    // The scaled form divides by the largest modulus; at an all-zero vector there is none, and a
    // division would produce the NaN this asserts against.
    let n = vector_norm_l2(&[0.0f64, 0.0, 0.0]);
    assert_eq!(n, 0.0);
    assert!(!n.is_nan());
}

#[test]
fn test_a_single_component_gives_its_modulus() {
    assert_eq!(vector_norm_l2(&[-7.5f64]), 7.5);
}

#[test]
fn test_the_pythagorean_triple_in_the_ordinary_range() {
    // 3² + 4² = 25, so the norm is exactly 5.
    assert_eq!(vector_norm_l2(&[3.0f64, 4.0]), 5.0);
}

#[test]
fn test_a_four_component_norm_against_its_closed_form() {
    // 1 + 4 + 9 + 16 = 30.
    let n = vector_norm_l2(&[1.0f64, 2.0, 3.0, 4.0]);
    assert!((n - 30.0f64.sqrt()).abs() < 1e-15);
}

#[test]
fn test_negative_components_match_their_absolute_values() {
    assert_eq!(
        vector_norm_l2(&[-3.0f64, 4.0]),
        vector_norm_l2(&[3.0f64, -4.0])
    );
    assert_eq!(vector_norm_l2(&[-3.0f64, -4.0]), 5.0);
}

#[test]
fn test_the_ordinary_range_agrees_with_the_form_it_replaces() {
    // The naive `sqrt(Σ|x|²)` is a demonstrably different algorithm and is correct wherever it does
    // not overflow, which makes it a legitimate oracle in the middle of the range.
    for v in [
        vec![1.0f64, 2.0, 3.0],
        vec![-0.5, 0.25, 100.0, -3.75],
        vec![1e-8, 2e-8],
        vec![1e8, 2e8, 3e8],
        vec![42.0],
    ] {
        let scaled = vector_norm_l2(&v);
        let naive = vector_norm_sq(&v).sqrt();
        assert!(
            (scaled - naive).abs() <= naive * 1e-15,
            "scaled {scaled} against naive {naive}"
        );
    }
}

#[test]
fn test_a_huge_and_a_tiny_component_give_the_huge_one() {
    // The tiny component is below half a unit in the last place of the huge one, so it contributes
    // nothing and the norm is the huge component itself.
    let n = vector_norm_l2(&[1e300f64, 1e-300]);
    assert!(n.is_finite());
    assert_eq!(n, 1e300);
}

#[test]
fn test_complex_components_use_the_modulus() {
    // |3+4i| = 5 and |0+12i| = 12, so the norm is sqrt(25 + 144) = 13.
    let v = [Complex::new(3.0f64, 4.0), Complex::new(0.0, 12.0)];
    assert!((vector_norm_l2(&v) - 13.0).abs() < 1e-13);
}

#[test]
fn test_a_complex_component_near_the_maximum_does_not_overflow() {
    // |(3·2⁶⁰⁰) + (4·2⁶⁰⁰)i| = 5·2⁶⁰⁰, and the vector of that one entry has the same norm.
    let p = 2f64.powi(WIDE_K);
    let v = [Complex::new(3.0 * p, 4.0 * p)];
    let n = vector_norm_l2(&v);
    assert!(n.is_finite());
    assert_eq!(n, 5.0 * p);
}

#[test]
fn test_a_nan_component_gives_nan() {
    assert!(vector_norm_l2(&[1.0f64, f64::NAN, 2.0]).is_nan());
}

#[test]
fn test_an_infinite_component_gives_infinity() {
    let n = vector_norm_l2(&[1.0f64, f64::INFINITY]);
    assert!(n.is_infinite() && n > 0.0);
}

#[test]
fn test_two_infinite_components_give_infinity_rather_than_nan() {
    // The scaled form divides by the largest modulus. With two infinities that ratio is inf/inf,
    // which is where a form without an explicit guard would return NaN.
    let n = vector_norm_l2(&[f64::INFINITY, f64::NEG_INFINITY]);
    assert!(n.is_infinite() && n > 0.0, "got {n}");
}

#[test]
fn test_an_infinity_with_a_nan_gives_nan() {
    assert!(vector_norm_l2(&[f64::INFINITY, f64::NAN]).is_nan());
}

// =============================================================================
// matrix_norm_frobenius — the same quantity, the same scaling
// =============================================================================

#[test]
fn test_the_frobenius_norm_on_a_known_matrix() {
    // 1 + 4 + 9 + 16 = 30.
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, -2.0, -3.0, 4.0], 2, 2).unwrap();
    assert!((matrix_norm_frobenius(&m).unwrap() - 30.0f64.sqrt()).abs() < 1e-15);
}

#[test]
fn test_the_frobenius_norm_of_the_zero_matrix_is_zero() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(3, 3);
    let n = matrix_norm_frobenius(&m).unwrap();
    assert_eq!(n, 0.0);
    assert!(!n.is_nan());
}

#[test]
fn test_the_frobenius_norm_does_not_overflow_on_a_large_entry() {
    // The 3-4-5 triple laid into a 1x2 matrix: the norm is 5·2⁶⁰⁰ and the squares overflow.
    let p = 2f64.powi(WIDE_K);
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![3.0 * p, 4.0 * p], 1, 2).unwrap();
    let n = matrix_norm_frobenius(&m).unwrap();
    assert!(n.is_finite());
    assert_eq!(n, 5.0 * p);
}

#[test]
fn test_the_frobenius_norm_equals_the_two_norm_of_the_entries_at_the_extremes() {
    // The identity the crate already asserts in the ordinary range, held where it previously failed:
    // both sides were infinity before, which is an equality that says nothing.
    let p = 2f64.powi(WIDE_K);
    let entries = vec![3.0 * p, 4.0 * p, 0.0, 0.0];
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(entries.clone(), 2, 2).unwrap();
    let a = matrix_norm_frobenius(&m).unwrap();
    let b = vector_norm_l2(&entries);
    assert!(a.is_finite() && b.is_finite());
    assert_eq!(a, b);
}

// =============================================================================
// CsrMatrix::to_row_major — one pass over the stored entries
// =============================================================================

/// The shape and triplets of a small sparse matrix with a stored zero, an empty row, and gaps.
fn sample_csr() -> CsrMatrix<f64> {
    // 3x4, row 1 stores nothing:
    //   [ 1  0  0  2 ]
    //   [ 0  0  0  0 ]
    //   [ 0  3  0  0 ]
    CsrMatrix::from_triplets(3, 4, &[(0, 0, 1.0), (0, 3, 2.0), (2, 1, 3.0)]).unwrap()
}

/// Reads the matrix position by position, which is what the trait default does. An independent
/// route to the same buffer, so the override is checked against something other than itself.
fn by_position<M: MatrixView>(m: &M) -> Vec<M::Scalar> {
    let (r, c) = (m.rows(), m.cols());
    let mut out = Vec::with_capacity(r * c);
    for i in 0..r {
        for j in 0..c {
            out.push(m.get(i, j).unwrap());
        }
    }
    out
}

#[test]
fn test_csr_to_row_major_matches_a_position_by_position_read() {
    let m = sample_csr();
    assert_eq!(m.to_row_major().unwrap(), by_position(&m));
}

#[test]
fn test_csr_to_row_major_writes_the_hand_written_buffer() {
    // The buffer above, written out by hand rather than read from the matrix.
    let expected = vec![1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0];
    assert_eq!(sample_csr().to_row_major().unwrap(), expected);
}

#[test]
fn test_csr_to_row_major_fills_unstored_positions_with_zero() {
    let flat = sample_csr().to_row_major().unwrap();
    assert_eq!(flat.len(), 12);
    // Row 1 stores nothing at all and must still occupy four zeros.
    assert_eq!(&flat[4..8], &[0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn test_csr_to_row_major_handles_the_three_empty_shapes() {
    for (r, c) in [(0usize, 0usize), (0, 4), (3, 0)] {
        let m: CsrMatrix<f64> = CsrMatrix::zeros(r, c);
        assert_eq!(m.to_row_major().unwrap(), Vec::<f64>::new());
    }
}

#[test]
fn test_csr_to_row_major_refuses_a_shape_whose_entry_count_overflows() {
    // Three words hold this shape; the dense buffer it describes cannot exist.
    let m: CsrMatrix<f64> = CsrMatrix::with_capacity(2, usize::MAX, 0);
    let err = m.to_row_major().unwrap_err();
    assert!(
        matches!(err.kind(), LinearErrorEnum::Overflow { .. }),
        "got {err:?}"
    );
}

// =============================================================================
// The dense kernels reached from a sparse matrix
// =============================================================================

/// A symmetric 3x3 with known eigenvalues, as triplets and as a dense buffer.
///
/// `diag(2, 3, 5)` with a single off-diagonal pair `(0,1) = (1,0) = 1`. Its leading 2x2 block
/// `[[2,1],[1,3]]` has characteristic polynomial `λ² − 5λ + 5`, so the eigenvalues are
/// `(5 ± √5)/2` — hand-evaluated, not read from the solver — and `5` comes through untouched.
fn symmetric_triplets() -> [(usize, usize, f64); 5] {
    [
        (0, 0, 2.0),
        (0, 1, 1.0),
        (1, 0, 1.0),
        (1, 1, 3.0),
        (2, 2, 5.0),
    ]
}

fn symmetric_dense() -> DenseMatrix<f64> {
    DenseMatrix::from_vec(vec![2.0, 1.0, 0.0, 1.0, 3.0, 0.0, 0.0, 0.0, 5.0], 3, 3).unwrap()
}

#[test]
fn test_eigen_hermitian_agrees_between_the_sparse_and_dense_forms() {
    let sparse = CsrMatrix::from_triplets(3, 3, &symmetric_triplets()).unwrap();
    let (sv, _) = eigen_hermitian(&sparse).unwrap();
    let (dv, _) = eigen_hermitian(&symmetric_dense()).unwrap();
    let mut s: Vec<f64> = sv.as_slice().to_vec();
    let mut d: Vec<f64> = dv.as_slice().to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(s.len(), 3);
    for (a, b) in s.iter().zip(d.iter()) {
        assert!((a - b).abs() < 1e-12, "sparse {a} against dense {b}");
    }
    // And against the closed form: (5 − √5)/2, (5 + √5)/2, 5.
    let root5 = 5.0f64.sqrt();
    for (got, want) in s
        .iter()
        .zip([(5.0 - root5) / 2.0, (5.0 + root5) / 2.0, 5.0])
    {
        assert!((got - want).abs() < 1e-12, "got {got}, want {want}");
    }
}

#[test]
fn test_qr_svd_and_cholesky_reach_their_dense_paths_from_a_sparse_matrix() {
    let sparse = CsrMatrix::from_triplets(3, 3, &symmetric_triplets()).unwrap();
    // The matrix is symmetric positive definite — its eigenvalues above are all positive — so all
    // three decompositions exist.
    let (q, r) = qr(&sparse).unwrap();
    assert_eq!((q.rows(), r.cols()), (3, 3));
    let (u, s, vt) = svd(&sparse).unwrap();
    assert_eq!((u.rows(), s.as_slice().len(), vt.cols()), (3, 3, 3));
    let l = cholesky(&sparse).unwrap();
    assert_eq!((l.rows(), l.cols()), (3, 3));
}

#[test]
fn test_the_frobenius_norm_of_a_sparse_matrix_counts_only_stored_entries() {
    // 1 + 4 + 9 = 14 over the sample matrix's three stored entries; the nine zeros add nothing.
    let n = matrix_norm_frobenius(&sample_csr()).unwrap();
    assert!((n - 14.0f64.sqrt()).abs() < 1e-15);
}
