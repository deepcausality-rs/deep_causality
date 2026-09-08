/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fixed-size dense forms moved in from the consumers (`unified-math-next` task 6.7).
//!
//! # Where the expected values come from
//!
//! Four sources, all on the change's allow-list, and every literal names its own:
//!
//! * **Hand-evaluated closed forms.** The cofactor expansion is written out in the comment beside
//!   the literal so the next reader can check the arithmetic rather than trust the digits.
//! * **A demonstrably different algorithm in this crate.** The general [`determinant`] is Gaussian
//!   elimination, [`inverse`] is an LU solve with partial pivoting, and [`eigen_hermitian`] is
//!   cyclic Jacobi. None of them shares a line with the closed forms under test, so agreement
//!   across a family is real evidence.
//! * **Algebraic invariants.** `A·A⁻¹ = I`, `det(AB) = det(A)·det(B)`, a row swap flips the sign,
//!   the eigenvalues reproduce the three characteristic invariants.
//! * **Exact rational arithmetic**, computed away from this code, for the inverse entries — stated
//!   as a fraction beside the decimal so the decimal is auditable.
//!
//! # The coincidence these tests are built around
//!
//! `tr(A²) = Σᵢⱼ aᵢⱼ·aⱼᵢ` and `‖A‖²_F = Σᵢⱼ aᵢⱼ²` are **the same number for every symmetric
//! matrix** and different for almost every other one. So is `A : A` against `tr(A²)`. A test suite
//! whose only `3×3` fixtures are symmetric — which is the natural choice, since the physical
//! tensors these kernels take are symmetric — cannot tell the two implementations apart. The
//! asymmetric fixture `A` below separates them (55 against 109), and the symmetric fixture pins the
//! coincidence itself so that neither is mistaken for the other later.

use deep_causality_linear::{
    DenseMatrix, LinearErrorEnum, determinant, determinant_3x3, determinant_4x4, dot, dot_n,
    double_dot_3x3, eigen_hermitian, eigen_symmetric_3x3, inverse, inverse_3x3, inverse_4x4,
    mat3_vec, trace_of_square_3x3, vector_norm_sq,
};

// =============================================================================
// Fixtures
// =============================================================================

/// A deliberately **asymmetric** `3×3` with small integer entries.
///
/// ```text
///       ⎡ 2  -1   3 ⎤
///   A = ⎢ 0   4  -2 ⎥
///       ⎣ 1   5   7 ⎦
/// ```
const A: [[f64; 3]; 3] = [[2.0, -1.0, 3.0], [0.0, 4.0, -2.0], [1.0, 5.0, 7.0]];

/// A second `3×3`, for the contractions.
///
/// ```text
///       ⎡  1  2   0 ⎤
///   B = ⎢ -3  1   4 ⎥
///       ⎣  2  0  -1 ⎦
/// ```
const B: [[f64; 3]; 3] = [[1.0, 2.0, 0.0], [-3.0, 1.0, 4.0], [2.0, 0.0, -1.0]];

/// A `4×4` with small integer entries and a non-zero determinant.
const M4: [[f64; 4]; 4] = [
    [4.0, -2.0, 1.0, 3.0],
    [0.0, 5.0, -1.0, 2.0],
    [2.0, 1.0, 6.0, -3.0],
    [-1.0, 0.0, 2.0, 4.0],
];

const I3: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// A reproducible 64-bit LCG, so the generated families are the same on every run and a failure can
/// be reproduced from the seed alone. Numerical Recipes' constants (Knuth's MMIX).
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    /// A value in `[-1, 1)`.
    fn next_signed(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    }
    fn mat3(&mut self) -> [[f64; 3]; 3] {
        core::array::from_fn(|_| core::array::from_fn(|_| self.next_signed()))
    }
    fn sym3(&mut self) -> [[f64; 3]; 3] {
        let m = self.mat3();
        core::array::from_fn(|i| core::array::from_fn(|j| 0.5 * (m[i][j] + m[j][i])))
    }
    fn mat4(&mut self) -> [[f64; 4]; 4] {
        core::array::from_fn(|_| core::array::from_fn(|_| self.next_signed()))
    }
}

fn dense3(m: &[[f64; 3]; 3]) -> DenseMatrix<f64> {
    DenseMatrix::from_vec(m.iter().flatten().copied().collect(), 3, 3).unwrap()
}

fn dense4(m: &[[f64; 4]; 4]) -> DenseMatrix<f64> {
    DenseMatrix::from_vec(m.iter().flatten().copied().collect(), 4, 4).unwrap()
}

fn mul3(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    core::array::from_fn(|i| {
        core::array::from_fn(|j| a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j])
    })
}

fn transpose3(a: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    core::array::from_fn(|i| core::array::from_fn(|j| a[j][i]))
}

// =============================================================================
// dot
// =============================================================================

#[test]
fn test_dot_matches_the_hand_evaluated_inner_product() {
    // 1·4 + 2·5 + 3·6 = 4 + 10 + 18 = 32.
    assert_eq!(dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]).unwrap(), 32.0);
}

#[test]
fn test_dot_of_two_empty_slices_is_zero() {
    // The empty sum. Corner class A: a fold that must return its identity here and nowhere else.
    let empty: [f64; 0] = [];
    assert_eq!(dot(&empty, &empty).unwrap(), 0.0);
}

#[test]
fn test_dot_of_single_element_slices_is_the_product() {
    // Corner class B.
    assert_eq!(dot(&[-3.0], &[7.0]).unwrap(), -21.0);
}

#[test]
fn test_dot_with_a_zero_vector_is_zero() {
    // Corner class F.
    assert_eq!(dot(&[0.0, 0.0, 0.0], &[1.0, -2.0, 3.0]).unwrap(), 0.0);
}

#[test]
fn test_dot_refuses_a_length_mismatch_in_both_directions() {
    // Folding over the shorter slice is the silent-truncation class this stage removed from the
    // CSR product; a refusal is the contract. Both orders, because a `zip` truncates either way.
    let short = [1.0, 2.0];
    let long = [1.0, 2.0, 3.0];
    let e = dot(&short, &long).unwrap_err();
    assert!(
        matches!(
            e.kind(),
            LinearErrorEnum::LengthMismatch {
                expected: 2,
                found: 3
            }
        ),
        "got {e}"
    );
    let e = dot(&long, &short).unwrap_err();
    assert!(
        matches!(
            e.kind(),
            LinearErrorEnum::LengthMismatch {
                expected: 3,
                found: 2
            }
        ),
        "got {e}"
    );
}

#[test]
fn test_dot_refuses_an_empty_slice_against_a_non_empty_one() {
    // The boundary of the empty case: zero is the answer for two empties, not for one.
    let empty: [f64; 0] = [];
    assert!(dot(&empty, &[1.0]).is_err());
}

#[test]
fn test_dot_agrees_with_the_squared_euclidean_norm() {
    // `v·v = ‖v‖²`. `vector_norm_sq` is a different function reached by a different fold, so this
    // is cross-algorithm agreement rather than the same expression twice — over 200 vectors of
    // four lengths, including the empty one.
    let mut rng = Lcg::new(0x5EED_0001);
    for len in [0usize, 1, 2, 7] {
        for _ in 0..50 {
            let v: Vec<f64> = (0..len).map(|_| rng.next_signed()).collect();
            let d = dot(&v, &v).unwrap();
            let n = vector_norm_sq(&v);
            assert!(
                (d - n).abs() <= 1e-12 * (1.0 + n.abs()),
                "len {len}: {d} vs {n}"
            );
        }
    }
}

#[test]
fn test_dot_is_symmetric_and_bilinear() {
    // Properties over a generated family: `a·b = b·a`, and `(αa + c)·b = α(a·b) + (c·b)`.
    let mut rng = Lcg::new(0x5EED_0002);
    for _ in 0..100 {
        let a: Vec<f64> = (0..5).map(|_| rng.next_signed()).collect();
        let b: Vec<f64> = (0..5).map(|_| rng.next_signed()).collect();
        let c: Vec<f64> = (0..5).map(|_| rng.next_signed()).collect();
        let alpha = rng.next_signed();

        assert!((dot(&a, &b).unwrap() - dot(&b, &a).unwrap()).abs() < 1e-14);

        let combined: Vec<f64> = a.iter().zip(&c).map(|(x, y)| alpha * x + y).collect();
        let lhs = dot(&combined, &b).unwrap();
        let rhs = alpha * dot(&a, &b).unwrap() + dot(&c, &b).unwrap();
        assert!(
            (lhs - rhs).abs() <= 1e-12 * (1.0 + rhs.abs()),
            "{lhs} vs {rhs}"
        );
    }
}

#[test]
fn test_dot_is_exact_over_the_integers() {
    // The bound is `CommutativeRing`, not `Field`, so ℤ is in the domain and the answer is exact.
    // 3·(-7) + 11·2 + 5·5 = -21 + 22 + 25 = 26.
    assert_eq!(dot::<i64>(&[3, 11, 5], &[-7, 2, 5]).unwrap(), 26);
}

#[test]
fn test_dot_n_agrees_with_the_slice_form_on_every_length_it_can_take() {
    // The two must be the same number, not merely the same idea: `dot_n` exists to drop a check the
    // type already makes, and nothing else.
    let mut rng = Lcg::new(0x5EED_0011);
    for _ in 0..200 {
        let a: [f64; 4] = core::array::from_fn(|_| rng.next_signed());
        let b: [f64; 4] = core::array::from_fn(|_| rng.next_signed());
        assert_eq!(dot_n(&a, &b), dot(&a, &b).unwrap());
    }
    let a17: [f64; 17] = core::array::from_fn(|_| rng.next_signed());
    let b17: [f64; 17] = core::array::from_fn(|_| rng.next_signed());
    assert_eq!(dot_n(&a17, &b17), dot(&a17, &b17).unwrap());
}

#[test]
fn test_dot_n_of_empty_arrays_is_zero() {
    // Corner class A at the type level: `N = 0` is a legal const parameter.
    let e: [f64; 0] = [];
    assert_eq!(dot_n(&e, &e), 0.0);
}

#[test]
fn test_dot_n_matches_the_hand_evaluated_inner_product() {
    // 1·4 + 2·5 + 3·6 = 32.
    assert_eq!(dot_n(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]), 32.0);
}

// =============================================================================
// determinant_3x3
// =============================================================================

#[test]
fn test_determinant_3x3_matches_the_hand_evaluated_expansion() {
    // Cofactor expansion along row 0:
    //   2·(4·7 − (−2)·5) − (−1)·(0·7 − (−2)·1) + 3·(0·5 − 4·1)
    // = 2·(28 + 10) + 1·(0 + 2) + 3·(0 − 4)
    // = 76 + 2 − 12
    // = 66.
    assert_eq!(determinant_3x3(&A), 66.0);
}

#[test]
fn test_determinant_3x3_of_the_identity_is_one() {
    // Corner class C: the identity satisfies almost every wrong formula too, so this is a floor
    // rather than evidence — it is here because a sign slip in the adjugate breaks it.
    assert_eq!(determinant_3x3(&I3), 1.0);
}

#[test]
fn test_determinant_3x3_of_a_singular_matrix_is_zero() {
    // Row 2 is twice row 0.
    let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [2.0, 4.0, 6.0]];
    assert_eq!(determinant_3x3(&m), 0.0);
}

#[test]
fn test_determinant_3x3_agrees_with_gaussian_elimination() {
    // The crate's general `determinant` runs elimination with partial pivoting — a different
    // algorithm, not the same polynomial reordered. 200 matrices.
    let mut rng = Lcg::new(0x5EED_0003);
    for _ in 0..200 {
        let m = rng.mat3();
        let closed = determinant_3x3(&m);
        let eliminated = determinant(&dense3(&m)).unwrap();
        assert!(
            (closed - eliminated).abs() <= 1e-12 * (1.0 + eliminated.abs()),
            "{closed} vs {eliminated} for {m:?}"
        );
    }
}

#[test]
fn test_determinant_3x3_flips_sign_on_a_row_swap() {
    let mut swapped = A;
    swapped.swap(0, 2);
    assert_eq!(determinant_3x3(&swapped), -determinant_3x3(&A));
}

#[test]
fn test_determinant_3x3_is_multiplicative() {
    // det(AB) = det(A)·det(B), an invariant no retyped cofactor expansion can fake.
    let ab = mul3(&A, &B);
    let lhs = determinant_3x3(&ab);
    let rhs = determinant_3x3(&A) * determinant_3x3(&B);
    assert!(
        (lhs - rhs).abs() <= 1e-9 * (1.0 + rhs.abs()),
        "{lhs} vs {rhs}"
    );
}

#[test]
fn test_determinant_3x3_is_exact_over_the_integers() {
    let ai: [[i64; 3]; 3] = [[2, -1, 3], [0, 4, -2], [1, 5, 7]];
    assert_eq!(determinant_3x3(&ai), 66);
}

// =============================================================================
// determinant_4x4
// =============================================================================

#[test]
fn test_determinant_4x4_matches_the_exact_rational_value() {
    // det(M4) = 812, by exact integer cofactor expansion performed away from this code.
    assert_eq!(determinant_4x4(&M4), 812.0);
}

#[test]
fn test_determinant_4x4_of_the_identity_is_one() {
    let i4: [[f64; 4]; 4] =
        core::array::from_fn(|i| core::array::from_fn(|j| if i == j { 1.0 } else { 0.0 }));
    assert_eq!(determinant_4x4(&i4), 1.0);
}

#[test]
fn test_determinant_4x4_of_a_singular_matrix_is_zero() {
    // Row 3 is row 1 + row 2.
    let m = [
        [1.0, 2.0, 3.0, 4.0],
        [5.0, 6.0, 7.0, 8.0],
        [9.0, 1.0, 2.0, 3.0],
        [14.0, 7.0, 9.0, 11.0],
    ];
    assert_eq!(determinant_4x4(&m), 0.0);
}

#[test]
fn test_determinant_4x4_agrees_with_gaussian_elimination() {
    let mut rng = Lcg::new(0x5EED_0004);
    for _ in 0..200 {
        let m = rng.mat4();
        let closed = determinant_4x4(&m);
        let eliminated = determinant(&dense4(&m)).unwrap();
        assert!(
            (closed - eliminated).abs() <= 1e-12 * (1.0 + eliminated.abs()),
            "{closed} vs {eliminated} for {m:?}"
        );
    }
}

#[test]
fn test_determinant_4x4_flips_sign_on_a_row_swap() {
    let mut swapped = M4;
    swapped.swap(1, 3);
    assert_eq!(determinant_4x4(&swapped), -determinant_4x4(&M4));
}

#[test]
fn test_determinant_4x4_of_the_minkowski_metric_is_minus_one() {
    // diag(−1, 1, 1, 1): the signature this function was moved in to serve.
    let g: [[f64; 4]; 4] = core::array::from_fn(|i| {
        core::array::from_fn(|j| {
            if i != j {
                0.0
            } else if i == 0 {
                -1.0
            } else {
                1.0
            }
        })
    });
    assert_eq!(determinant_4x4(&g), -1.0);
}

// =============================================================================
// trace_of_square_3x3
// =============================================================================

#[test]
fn test_trace_of_square_matches_the_hand_evaluated_sum() {
    // tr(A²) = Σᵢⱼ aᵢⱼ·aⱼᵢ
    //   = 2·2 + (−1)·0 + 3·1
    //   + 0·(−1) + 4·4 + (−2)·5
    //   + 1·3 + 5·(−2) + 7·7
    //   = 4 + 0 + 3 + 0 + 16 − 10 + 3 − 10 + 49
    //   = 55.
    assert_eq!(trace_of_square_3x3(&A), 55.0);
}

#[test]
fn test_trace_of_square_differs_from_the_squared_frobenius_norm_on_an_asymmetric_matrix() {
    // Corner class C, and the reason `A` is asymmetric. Σ aᵢⱼ² = 4+1+9+0+16+4+1+25+49 = 109, which
    // is not 55 — so an implementation that dropped the transposition fails here and passes every
    // symmetric fixture.
    let frobenius_sq: f64 = A.iter().flatten().map(|x| x * x).sum();
    assert_eq!(frobenius_sq, 109.0);
    assert_ne!(trace_of_square_3x3(&A), frobenius_sq);
}

#[test]
fn test_trace_of_square_equals_the_squared_frobenius_norm_on_a_symmetric_matrix() {
    // The coincidence itself, pinned so it is not mistaken for the general case.
    let mut rng = Lcg::new(0x5EED_0005);
    for _ in 0..100 {
        let s = rng.sym3();
        let frobenius_sq: f64 = s.iter().flatten().map(|x| x * x).sum();
        let t = trace_of_square_3x3(&s);
        assert!((t - frobenius_sq).abs() <= 1e-12 * (1.0 + t.abs()));
    }
}

#[test]
fn test_trace_of_square_agrees_with_the_diagonal_of_the_explicit_product() {
    // Form A·A and add its diagonal: a different route to the same number.
    let mut rng = Lcg::new(0x5EED_0006);
    for _ in 0..200 {
        let m = rng.mat3();
        let sq = mul3(&m, &m);
        let explicit = sq[0][0] + sq[1][1] + sq[2][2];
        let closed = trace_of_square_3x3(&m);
        assert!(
            (closed - explicit).abs() <= 1e-12 * (1.0 + explicit.abs()),
            "{closed} vs {explicit}"
        );
    }
}

#[test]
fn test_trace_of_square_of_the_identity_is_three() {
    assert_eq!(trace_of_square_3x3(&I3), 3.0);
}

#[test]
fn test_trace_of_square_of_an_antisymmetric_matrix_is_the_negated_frobenius_norm() {
    // For Ωᵀ = −Ω, tr(Ω²) = −Σ ωᵢⱼ². An invariant with the opposite sign to the symmetric case, so
    // it separates the two implementations from the other side.
    let o = [[0.0, 2.0, -3.0], [-2.0, 0.0, 5.0], [3.0, -5.0, 0.0]];
    let frobenius_sq: f64 = o.iter().flatten().map(|x| x * x).sum();
    assert_eq!(trace_of_square_3x3(&o), -frobenius_sq);
}

// =============================================================================
// double_dot_3x3
// =============================================================================

#[test]
fn test_double_dot_matches_the_hand_evaluated_sum() {
    // A : B = 2·1 + (−1)·2 + 3·0 + 0·(−3) + 4·1 + (−2)·4 + 1·2 + 5·0 + 7·(−1)
    //       = 2 − 2 + 0 + 0 + 4 − 8 + 2 + 0 − 7 = −9.
    assert_eq!(double_dot_3x3(&A, &B), -9.0);
}

#[test]
fn test_double_dot_agrees_with_the_trace_of_the_transpose_product() {
    // A : B = tr(AᵀB), formed explicitly. Different route, same number.
    let mut rng = Lcg::new(0x5EED_0007);
    for _ in 0..200 {
        let a = rng.mat3();
        let b = rng.mat3();
        let p = mul3(&transpose3(&a), &b);
        let explicit = p[0][0] + p[1][1] + p[2][2];
        let closed = double_dot_3x3(&a, &b);
        assert!(
            (closed - explicit).abs() <= 1e-12 * (1.0 + explicit.abs()),
            "{closed} vs {explicit}"
        );
    }
}

#[test]
fn test_double_dot_is_symmetric_in_its_arguments() {
    assert_eq!(double_dot_3x3(&A, &B), double_dot_3x3(&B, &A));
}

#[test]
fn test_double_dot_against_the_identity_is_the_trace() {
    assert_eq!(double_dot_3x3(&A, &I3), A[0][0] + A[1][1] + A[2][2]);
}

#[test]
fn test_double_dot_of_a_matrix_with_itself_is_the_squared_frobenius_norm_not_the_trace_of_the_square()
 {
    // Corner class C again, from the other direction: A:A = 109 while tr(A²) = 55. The two
    // contractions differ in exactly one transposition and are the same number for symmetric input.
    assert_eq!(double_dot_3x3(&A, &A), 109.0);
    assert_ne!(double_dot_3x3(&A, &A), trace_of_square_3x3(&A));
}

#[test]
fn test_double_dot_with_a_zero_matrix_is_zero() {
    assert_eq!(double_dot_3x3(&A, &[[0.0; 3]; 3]), 0.0);
}

// =============================================================================
// mat3_vec
// =============================================================================

#[test]
fn test_mat3_vec_matches_the_hand_evaluated_product() {
    // A·(1, −2, 3):
    //   row 0:  2·1 + (−1)·(−2) + 3·3   =  2 + 2 + 9  =  13
    //   row 1:  0·1 +   4·(−2)  + (−2)·3 =  0 − 8 − 6 = −14
    //   row 2:  1·1 +   5·(−2)  + 7·3   =  1 − 10 + 21 =  12
    let got = mat3_vec(&A, &[1.0, -2.0, 3.0]);
    assert_eq!(got, [13.0, -14.0, 12.0]);
}

#[test]
fn test_mat3_vec_with_the_identity_returns_the_vector() {
    assert_eq!(mat3_vec(&I3, &[3.0, -1.0, 4.0]), [3.0, -1.0, 4.0]);
}

#[test]
fn test_mat3_vec_agrees_with_the_dense_matrix_vector_product() {
    // Against the general dense path, which indexes through `MatrixView` rather than the array.
    use deep_causality_linear::MatrixView;
    let mut rng = Lcg::new(0x5EED_0008);
    for _ in 0..200 {
        let m = rng.mat3();
        let v = [rng.next_signed(), rng.next_signed(), rng.next_signed()];
        let dm = dense3(&m);
        let explicit: [f64; 3] =
            core::array::from_fn(|i| (0..3).fold(0.0, |s, k| s + dm.get(i, k).unwrap() * v[k]));
        let closed = mat3_vec(&m, &v);
        for i in 0..3 {
            assert!(
                (closed[i] - explicit[i]).abs() <= 1e-12 * (1.0 + explicit[i].abs()),
                "row {i}: {} vs {}",
                closed[i],
                explicit[i]
            );
        }
    }
}

#[test]
fn test_mat3_vec_is_linear_in_the_vector() {
    let mut rng = Lcg::new(0x5EED_0009);
    for _ in 0..100 {
        let m = rng.mat3();
        let u: [f64; 3] = core::array::from_fn(|_| rng.next_signed());
        let w: [f64; 3] = core::array::from_fn(|_| rng.next_signed());
        let alpha = rng.next_signed();
        let combined: [f64; 3] = core::array::from_fn(|i| alpha * u[i] + w[i]);
        let lhs = mat3_vec(&m, &combined);
        let mu = mat3_vec(&m, &u);
        let mw = mat3_vec(&m, &w);
        for i in 0..3 {
            let rhs = alpha * mu[i] + mw[i];
            assert!((lhs[i] - rhs).abs() <= 1e-12 * (1.0 + rhs.abs()));
        }
    }
}

#[test]
fn test_mat3_vec_reads_rows_not_columns() {
    // The transposition trap: `mat3_vec` must be `M·v`, not `Mᵀ·v`. A single non-zero entry off
    // the diagonal distinguishes them; a symmetric matrix does not.
    let e = [[0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    assert_eq!(mat3_vec(&e, &[0.0, 5.0, 0.0]), [5.0, 0.0, 0.0]);
}

// =============================================================================
// inverse_3x3
// =============================================================================

#[test]
fn test_inverse_3x3_round_trips_to_the_identity() {
    let inv = inverse_3x3(&A).unwrap();
    let p = mul3(&A, &inv);
    for (i, row) in p.iter().enumerate() {
        for (j, entry) in row.iter().enumerate() {
            let want = if i == j { 1.0 } else { 0.0 };
            assert!((entry - want).abs() < 1e-12, "({i},{j}) = {entry}");
        }
    }
}

#[test]
fn test_inverse_3x3_matches_the_exact_rational_adjugate() {
    // Exact rational arithmetic away from this code, det(A) = 66:
    //   ⎡  19/33   1/3   −5/33 ⎤
    //   ⎢  −1/33   1/6    2/33 ⎥
    //   ⎣  −2/33  −1/6    4/33 ⎦
    let want = [
        [19.0 / 33.0, 1.0 / 3.0, -5.0 / 33.0],
        [-1.0 / 33.0, 1.0 / 6.0, 2.0 / 33.0],
        [-2.0 / 33.0, -1.0 / 6.0, 4.0 / 33.0],
    ];
    let got = inverse_3x3(&A).unwrap();
    for (i, (got_row, want_row)) in got.iter().zip(want.iter()).enumerate() {
        for (j, (g, w)) in got_row.iter().zip(want_row.iter()).enumerate() {
            assert!((g - w).abs() < 1e-15, "({i},{j}): {g} vs {w}");
        }
    }
}

#[test]
fn test_inverse_3x3_agrees_with_the_lu_inverse() {
    // The crate's `inverse` is an LU solve with partial pivoting. 200 matrices, conditioned away
    // from singularity so the comparison measures agreement rather than two different amplified
    // roundings.
    let mut rng = Lcg::new(0x5EED_000A);
    let mut compared = 0;
    for _ in 0..300 {
        let m = rng.mat3();
        if determinant_3x3(&m).abs() < 0.05 {
            continue;
        }
        let closed = inverse_3x3(&m).unwrap();
        let lu = inverse(&dense3(&m)).unwrap();
        for (i, row) in closed.iter().enumerate() {
            for (j, entry) in row.iter().enumerate() {
                let want = lu.as_slice()[i * 3 + j];
                assert!(
                    (entry - want).abs() <= 1e-8 * (1.0 + want.abs()),
                    "({i},{j}): {entry} vs {want} for {m:?}"
                );
            }
        }
        compared += 1;
    }
    assert!(compared > 100, "only {compared} well-conditioned draws");
}

#[test]
fn test_inverse_3x3_of_the_identity_is_the_identity() {
    assert_eq!(inverse_3x3(&I3).unwrap(), I3);
}

#[test]
fn test_inverse_3x3_of_a_diagonal_matrix_inverts_each_entry() {
    // Corner class C/D: on a diagonal matrix the adjugate's off-diagonal cofactors all vanish, so
    // this catches a term that should have cancelled and did not.
    let d = [[2.0, 0.0, 0.0], [0.0, -4.0, 0.0], [0.0, 0.0, 0.5]];
    let got = inverse_3x3(&d).unwrap();
    assert_eq!(got[0][0], 0.5);
    assert_eq!(got[1][1], -0.25);
    assert_eq!(got[2][2], 2.0);
    for (i, row) in got.iter().enumerate() {
        for (j, entry) in row.iter().enumerate() {
            if i != j {
                assert_eq!(*entry, 0.0, "({i},{j})");
            }
        }
    }
}

#[test]
fn test_inverse_3x3_refuses_an_exactly_singular_matrix() {
    let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [2.0, 4.0, 6.0]];
    let e = inverse_3x3(&m).unwrap_err();
    assert!(
        matches!(e.kind(), LinearErrorEnum::Singular { .. }),
        "got {e}"
    );
}

#[test]
fn test_inverse_3x3_refuses_the_zero_matrix() {
    // Corner class F.
    let e = inverse_3x3(&[[0.0; 3]; 3]).unwrap_err();
    assert!(
        matches!(e.kind(), LinearErrorEnum::Singular { .. }),
        "got {e}"
    );
}

#[test]
fn test_inverse_3x3_accepts_a_near_singular_matrix_rather_than_applying_a_threshold_of_its_own() {
    // The documented policy: the crate refuses only an exactly zero determinant, because "near
    // singular" is the caller's judgement about its own physics. `deep_causality_physics` keeps its
    // 1e-12 and 1e-14 metric guards in front of these calls for that reason.
    let m: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1e-18]];
    let got = inverse_3x3(&m).expect("a tiny but non-zero determinant is not this crate's refusal");
    // Relative, not exact: `1e-18` is not representable, so its correctly-rounded reciprocal is
    // 9.999999999999999e17 and not the `1e18` the decimal suggests. The claim under test is that
    // the call returns a value rather than an error; asserting the last bit of a reciprocal would
    // be asserting the rounding, not the contract.
    assert!(
        (got[2][2] - 1e18).abs() <= 4.0 * f64::EPSILON * 1e18,
        "got {}",
        got[2][2]
    );
}

// =============================================================================
// inverse_4x4
// =============================================================================

#[test]
fn test_inverse_4x4_round_trips_to_the_identity() {
    let inv = inverse_4x4(&M4).unwrap();
    for (i, m_row) in M4.iter().enumerate() {
        for j in 0..4 {
            let p: f64 = m_row.iter().zip(inv.iter()).map(|(m, r)| m * r[j]).sum();
            let want = if i == j { 1.0 } else { 0.0 };
            assert!((p - want).abs() < 1e-12, "({i},{j}) = {p}");
        }
    }
}

#[test]
fn test_inverse_4x4_matches_the_exact_rational_adjugate() {
    // Exact rational arithmetic away from this code, det(M4) = 812.
    let want = [
        [79.0 / 406.0, 1.0 / 14.0, 13.0 / 406.0, -32.0 / 203.0],
        [-25.0 / 812.0, 5.0 / 28.0, 37.0 / 812.0, -13.0 / 406.0],
        [-23.0 / 812.0, -1.0 / 28.0, 99.0 / 812.0, 53.0 / 406.0],
        [51.0 / 812.0, 1.0 / 28.0, -43.0 / 812.0, 59.0 / 406.0],
    ];
    let got = inverse_4x4(&M4).unwrap();
    for (i, (got_row, want_row)) in got.iter().zip(want.iter()).enumerate() {
        for (j, (g, w)) in got_row.iter().zip(want_row.iter()).enumerate() {
            assert!((g - w).abs() < 1e-15, "({i},{j}): {g} vs {w}");
        }
    }
}

#[test]
fn test_inverse_4x4_agrees_with_the_lu_inverse() {
    let mut rng = Lcg::new(0x5EED_000B);
    let mut compared = 0;
    for _ in 0..300 {
        let m = rng.mat4();
        if determinant_4x4(&m).abs() < 0.02 {
            continue;
        }
        let closed = inverse_4x4(&m).unwrap();
        let lu = inverse(&dense4(&m)).unwrap();
        for (i, row) in closed.iter().enumerate() {
            for (j, entry) in row.iter().enumerate() {
                let want = lu.as_slice()[i * 4 + j];
                assert!(
                    (entry - want).abs() <= 1e-7 * (1.0 + want.abs()),
                    "({i},{j}): {entry} vs {want}"
                );
            }
        }
        compared += 1;
    }
    assert!(compared > 100, "only {compared} well-conditioned draws");
}

#[test]
fn test_inverse_4x4_of_the_minkowski_metric_is_itself() {
    // diag(−1, 1, 1, 1) is an involution, which is the case the general-relativity caller hits on
    // flat space — and an implementation with a sign error in the adjugate fails it while passing
    // the identity.
    let g: [[f64; 4]; 4] = core::array::from_fn(|i| {
        core::array::from_fn(|j| {
            if i != j {
                0.0
            } else if i == 0 {
                -1.0
            } else {
                1.0
            }
        })
    });
    assert_eq!(inverse_4x4(&g).unwrap(), g);
}

#[test]
fn test_inverse_4x4_refuses_an_exactly_singular_matrix() {
    let m = [
        [1.0, 2.0, 3.0, 4.0],
        [5.0, 6.0, 7.0, 8.0],
        [9.0, 1.0, 2.0, 3.0],
        [14.0, 7.0, 9.0, 11.0],
    ];
    let e = inverse_4x4(&m).unwrap_err();
    assert!(
        matches!(e.kind(), LinearErrorEnum::Singular { .. }),
        "got {e}"
    );
}

#[test]
fn test_inverse_4x4_refuses_the_zero_matrix() {
    let e = inverse_4x4(&[[0.0; 4]; 4]).unwrap_err();
    assert!(
        matches!(e.kind(), LinearErrorEnum::Singular { .. }),
        "got {e}"
    );
}

// =============================================================================
// eigen_symmetric_3x3
// =============================================================================

/// The relative accuracy the closed form reaches at a **degenerate** spectrum: about `√ε`, not `ε`.
///
/// This is the algorithm and not slack in the test. The eigenvalues come out of `acos`, whose
/// derivative is unbounded at `±1`, and a repeated root is exactly where the argument sits. A
/// perturbation of `ε` in `det(B)` therefore moves the angle by about `√ε`, and each eigenvalue by
/// `√ε·‖M‖`. Measured here: `v = (−3, −3, −1)` gives a repeated eigenvalue of `5.4e-8` where the
/// exact answer is `0`, against `‖M‖ = 19`.
///
/// The generated-family tests above assert at `1e-9` relative and pass, because a random symmetric
/// matrix almost never has a degenerate spectrum. The limit bites only where two eigenvalues
/// coincide — which for `deep_causality_physics`'s `lambda2_kernel` is precisely the boundary
/// between vortex and non-vortex topology, so it is recorded on the function itself as well.
const DEGENERATE_TOL: f64 = 4.0 * 1.4901161193847656e-8; // 4·√ε

fn assert_desc(e: &[f64; 3]) {
    assert!(e[0] >= e[1] && e[1] >= e[2], "not descending: {e:?}");
}

#[test]
fn test_eigen_symmetric_3x3_of_a_diagonal_matrix_returns_the_sorted_diagonal() {
    // The `p1 == 0` branch, which the normalisation cannot reach because it divides by a quantity
    // that vanishes here. Corner class C and D at once.
    let d: [[f64; 3]; 3] = [[3.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 5.0]];
    let e = eigen_symmetric_3x3(&d).unwrap();
    assert_eq!(e, [5.0, 3.0, -1.0]);
}

#[test]
fn test_eigen_symmetric_3x3_of_the_zero_matrix_is_three_zeros() {
    // Corner class F, and the degenerate end of the diagonal branch.
    let zero: [[f64; 3]; 3] = [[0.0; 3]; 3];
    assert_eq!(eigen_symmetric_3x3(&zero).unwrap(), [0.0, 0.0, 0.0]);
}

#[test]
fn test_eigen_symmetric_3x3_of_the_identity_is_three_ones() {
    assert_eq!(eigen_symmetric_3x3(&I3).unwrap(), [1.0, 1.0, 1.0]);
}

#[test]
fn test_eigen_symmetric_3x3_with_a_repeated_eigenvalue() {
    // M = I + J, where J is all ones. J has eigenvalues 3, 0, 0, so M has 4, 1, 1 — exactly, and
    // by an argument that never touches the cubic. The repeated root drives `acos` to its endpoint,
    // which is where an unclamped implementation returns NaN.
    let m: [[f64; 3]; 3] = [[2.0, 1.0, 1.0], [1.0, 2.0, 1.0], [1.0, 1.0, 2.0]];
    let e = eigen_symmetric_3x3(&m).unwrap();
    // `√ε` again: 1 is a double root, so the same conditioning applies.
    let tol = DEGENERATE_TOL * 4.0;
    assert!((e[0] - 4.0).abs() < tol, "{e:?}");
    assert!((e[1] - 1.0).abs() < tol, "{e:?}");
    assert!((e[2] - 1.0).abs() < tol, "{e:?}");
}

#[test]
fn test_eigen_symmetric_3x3_of_a_block_diagonal_fixture() {
    // diag(2) ⊕ [[3,4],[4,9]]. The 2×2 block has trace 12 and determinant 11, so its eigenvalues
    // solve λ² − 12λ + 11 = 0, giving 11 and 1. The three are therefore 11, 2, 1 exactly.
    let m: [[f64; 3]; 3] = [[2.0, 0.0, 0.0], [0.0, 3.0, 4.0], [0.0, 4.0, 9.0]];
    let e = eigen_symmetric_3x3(&m).unwrap();
    assert!((e[0] - 11.0).abs() < 1e-12, "{e:?}");
    assert!((e[1] - 2.0).abs() < 1e-12, "{e:?}");
    assert!((e[2] - 1.0).abs() < 1e-12, "{e:?}");
}

#[test]
fn test_eigen_symmetric_3x3_handles_a_negative_definite_matrix() {
    // Corner class G: the whole spectrum below zero, where a `.abs()` slipped in for an ordering
    // reverses the sort.
    let m: [[f64; 3]; 3] = [[-2.0, 0.0, 0.0], [0.0, -5.0, 0.0], [0.0, 0.0, -1.0]];
    assert_eq!(eigen_symmetric_3x3(&m).unwrap(), [-1.0, -2.0, -5.0]);
}

#[test]
fn test_eigen_symmetric_3x3_agrees_with_the_jacobi_eigensolver() {
    // `eigen_hermitian` is cyclic Jacobi — an iterative similarity sweep sharing no line with the
    // closed form. 200 symmetric matrices; both spectra sorted descending before comparison,
    // because Jacobi returns its eigenvalues unsorted.
    let mut rng = Lcg::new(0x5EED_000C);
    for _ in 0..200 {
        let s = rng.sym3();
        let closed = eigen_symmetric_3x3(&s).unwrap();
        let (values, _) = eigen_hermitian(&dense3(&s)).unwrap();
        let mut jacobi: Vec<f64> = values.as_slice().to_vec();
        jacobi.sort_by(|a, b| b.partial_cmp(a).unwrap());
        for i in 0..3 {
            assert!(
                (closed[i] - jacobi[i]).abs() <= 1e-9 * (1.0 + jacobi[i].abs()),
                "eigenvalue {i}: {} vs {} for {s:?}",
                closed[i],
                jacobi[i]
            );
        }
    }
}

#[test]
fn test_eigen_symmetric_3x3_reproduces_the_three_characteristic_invariants() {
    // Σλ = tr(M), Σᵢ<ⱼ λᵢλⱼ = the second invariant, Πλ = det(M). All three, over a generated
    // family: the eigenvalues are the roots of the characteristic polynomial, so its coefficients
    // are the strongest available oracle short of a second solver.
    let mut rng = Lcg::new(0x5EED_000D);
    for _ in 0..200 {
        let m = rng.sym3();
        let e = eigen_symmetric_3x3(&m).unwrap();

        let trace = m[0][0] + m[1][1] + m[2][2];
        let sum = e[0] + e[1] + e[2];
        assert!(
            (sum - trace).abs() <= 1e-10 * (1.0 + trace.abs()),
            "{sum} vs {trace}"
        );

        let second = trace_of_square_3x3(&m);
        let sum_sq = e[0] * e[0] + e[1] * e[1] + e[2] * e[2];
        assert!(
            (sum_sq - second).abs() <= 1e-10 * (1.0 + second.abs()),
            "Σλ² {sum_sq} vs tr(M²) {second}"
        );

        let det = determinant_3x3(&m);
        let prod = e[0] * e[1] * e[2];
        assert!(
            (prod - det).abs() <= 1e-10 * (1.0 + det.abs()),
            "{prod} vs {det}"
        );
    }
}

#[test]
fn test_eigen_symmetric_3x3_always_returns_descending_order() {
    let mut rng = Lcg::new(0x5EED_000E);
    for _ in 0..300 {
        assert_desc(&eigen_symmetric_3x3(&rng.sym3()).unwrap());
    }
    // And on the diagonal branch, which returns from a different place.
    for _ in 0..100 {
        let d = [
            [rng.next_signed(), 0.0, 0.0],
            [0.0, rng.next_signed(), 0.0],
            [0.0, 0.0, rng.next_signed()],
        ];
        assert_desc(&eigen_symmetric_3x3(&d).unwrap());
    }
}

#[test]
fn test_eigen_symmetric_3x3_reads_only_the_upper_triangle() {
    // The documented contract. A caller that passes a matrix whose lower triangle disagrees gets
    // the spectrum of the upper triangle's symmetric completion — a stated answer rather than a
    // silently wrong one for the matrix it passed.
    let sym: [[f64; 3]; 3] = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let mut dirty = sym;
    dirty[1][0] = -99.0;
    dirty[2][0] = 77.0;
    dirty[2][1] = 0.5;
    assert_eq!(
        eigen_symmetric_3x3(&sym).unwrap(),
        eigen_symmetric_3x3(&dirty).unwrap()
    );
}

#[test]
fn test_eigen_symmetric_3x3_is_scale_equivariant() {
    // λ(αM) = α·λ(M) for α > 0, over five decades. This is the property the trace-shift and
    // normalisation exist to make true, and it fails if either is applied in the wrong order.
    let mut rng = Lcg::new(0x5EED_000F);
    for _ in 0..50 {
        let m = rng.sym3();
        let base = eigen_symmetric_3x3(&m).unwrap();
        for alpha in [1e-5f64, 1e-2, 1.0, 1e2, 1e5] {
            let scaled: [[f64; 3]; 3] =
                core::array::from_fn(|i| core::array::from_fn(|j| alpha * m[i][j]));
            let e = eigen_symmetric_3x3(&scaled).unwrap();
            for i in 0..3 {
                let want = alpha * base[i];
                assert!(
                    (e[i] - want).abs() <= 1e-9 * (1.0 + want.abs()),
                    "α={alpha}, i={i}: {} vs {want}",
                    e[i]
                );
            }
        }
    }
}

#[test]
fn test_eigen_symmetric_3x3_is_shift_equivariant() {
    // λ(M + cI) = λ(M) + c. Separates a trace shift applied once from one applied twice.
    let mut rng = Lcg::new(0x5EED_0010);
    for _ in 0..100 {
        let m = rng.sym3();
        let base = eigen_symmetric_3x3(&m).unwrap();
        let c = 7.5;
        let shifted: [[f64; 3]; 3] = core::array::from_fn(|i| {
            core::array::from_fn(|j| m[i][j] + if i == j { c } else { 0.0 })
        });
        let e = eigen_symmetric_3x3(&shifted).unwrap();
        for i in 0..3 {
            let want = base[i] + c;
            assert!((e[i] - want).abs() <= 1e-9 * (1.0 + want.abs()));
        }
    }
}

#[test]
fn test_eigen_symmetric_3x3_of_a_rank_one_outer_product_reaches_the_upper_acos_clamp() {
    // `v vᵀ` has eigenvalues `(v·v, 0, 0)` — a closed form that never touches the cubic — and its
    // doubly repeated zero is exactly where `det(B)/2` rounds **past +1**. Over this 342-vector
    // grid it does so 128 times, at values like `1.0000000000000004`. `acos` of that is `NaN`, and
    // the `NaN` propagates into all three eigenvalues, so this is the fixture the clamp exists for.
    //
    // Found by search, not by assumption: the audit removed the clamp and the whole suite still
    // passed, which is what sent me looking for an input that reaches it.
    let mut reached_the_clamp = 0;
    for a in -3..=3 {
        for b in -3..=3 {
            for c in -3..=3 {
                let v = [a as f64, b as f64, c as f64];
                let n = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
                if n == 0.0 {
                    continue;
                }
                let m: [[f64; 3]; 3] =
                    core::array::from_fn(|i| core::array::from_fn(|j| v[i] * v[j]));
                let e = eigen_symmetric_3x3(&m).unwrap();
                // The tolerance is `√ε`, not `ε`, and that is the algorithm rather than slack in
                // the test — see the note above the module's degenerate-spectrum tests.
                let tol = DEGENERATE_TOL * n;
                assert!(e.iter().all(|x| x.is_finite()), "v = {v:?} gave {e:?}");
                assert!((e[0] - n).abs() <= tol, "v = {v:?}: {e:?}, ‖v‖² = {n}");
                assert!(e[1].abs() <= tol, "v = {v:?}: {e:?}");
                assert!(e[2].abs() <= tol, "v = {v:?}: {e:?}");
                if v.iter().filter(|x| **x != 0.0).count() == 3 {
                    reached_the_clamp += 1;
                }
            }
        }
    }
    assert!(
        reached_the_clamp > 100,
        "only {reached_the_clamp} full-rank draws"
    );
}

#[test]
fn test_eigen_symmetric_3x3_of_a_complementary_projector_reaches_the_lower_acos_clamp() {
    // The dual family: `‖v‖²·I − v vᵀ` has eigenvalues `(‖v‖², ‖v‖², 0)`, the repeated root at the
    // top rather than the bottom, and it drives `det(B)/2` **below −1** — 136 times over the same
    // grid, at values like `-1.0000000000000007`. Both arms of the clamp are therefore reachable
    // and neither is defensive.
    for a in -3..=3 {
        for b in -3..=3 {
            for c in -3..=3 {
                let v = [a as f64, b as f64, c as f64];
                let n = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
                if n == 0.0 {
                    continue;
                }
                let m: [[f64; 3]; 3] = core::array::from_fn(|i| {
                    core::array::from_fn(|j| (if i == j { n } else { 0.0 }) - v[i] * v[j])
                });
                let e = eigen_symmetric_3x3(&m).unwrap();
                let tol = DEGENERATE_TOL * n;
                assert!(e.iter().all(|x| x.is_finite()), "v = {v:?} gave {e:?}");
                assert!((e[0] - n).abs() <= tol, "v = {v:?}: {e:?}");
                assert!((e[1] - n).abs() <= tol, "v = {v:?}: {e:?}");
                assert!(e[2].abs() <= tol, "v = {v:?}: {e:?}");
            }
        }
    }
}

#[test]
fn test_eigen_symmetric_3x3_at_the_boundary_where_the_cubic_has_a_triple_root() {
    // A multiple of the identity has p1 = 0 *and* a triple root, so it exercises the diagonal
    // branch at its most degenerate. The off-diagonal-perturbed neighbour goes down the cubic
    // branch with `r` at the clamp, which is where an unclamped `acos` returns NaN.
    let triple: [[f64; 3]; 3] = [[4.0, 0.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 4.0]];
    let e = eigen_symmetric_3x3(&triple).unwrap();
    assert_eq!(e, [4.0, 4.0, 4.0]);

    let nearly: [[f64; 3]; 3] = [[4.0, 1e-9, 0.0], [1e-9, 4.0, 0.0], [0.0, 0.0, 4.0]];
    let e = eigen_symmetric_3x3(&nearly).unwrap();
    for v in e {
        assert!(v.is_finite(), "{e:?}");
        assert!((v - 4.0).abs() < 1e-8, "{e:?}");
    }
}

#[test]
fn test_eigen_symmetric_3x3_survives_both_ends_of_the_scalar_range() {
    // `p1` and `p2` are sums of squares of the entries, so they leave `f64`'s range long before the
    // entries do. Measured before the matrix was normalised, on this fixture:
    //
    //   * at `1e200` the squares overflowed and the answer was `[inf, NaN, -inf]`;
    //   * at `1e-200` they underflowed to zero, `p1 == 0` sent the call down the *diagonal* branch,
    //     and it returned the diagonal `(9, 3, 2)·1e-200` — finite, plausible, and not the
    //     spectrum.
    //
    // The oracle is a closed form, not the implementation: `diag(2) ⊕ [[3,4],[4,9]]` has the
    // isolated eigenvalue 2, and the 2×2 block has trace 12 and determinant `3·9 − 4·4 = 11`, so
    // the block's eigenvalues solve `λ² − 12λ + 11 = 0`, giving 11 and 1. The spectrum is
    // therefore `(11, 2, 1)` exactly, and it is homogeneous of degree one in the matrix, so at
    // scale `s` it is `(11s, 2s, 1s)`. Every scale here keeps all three inside the normal range.
    let m: [[f64; 3]; 3] = [[2.0, 0.0, 0.0], [0.0, 3.0, 4.0], [0.0, 4.0, 9.0]];
    for s in [
        1e-300f64, 1e-200, 1e-160, 1e-8, 1.0, 1e8, 1e160, 1e200, 1e300,
    ] {
        let scaled: [[f64; 3]; 3] = core::array::from_fn(|i| core::array::from_fn(|j| s * m[i][j]));
        let e = eigen_symmetric_3x3(&scaled).unwrap();
        assert!(e.iter().all(|x| x.is_finite()), "s = {s:e} gave {e:?}");
        for (got, want) in e.iter().zip([11.0 * s, 2.0 * s, 1.0 * s]) {
            assert!(
                (got - want).abs() <= 1e-12 * want.abs(),
                "s = {s:e}: got {e:?}, closed form gives (11, 2, 1)·{s:e}"
            );
        }
    }
}

#[test]
fn test_eigen_symmetric_3x3_is_scale_equivariant_over_the_whole_range() {
    // The equivariance the suite already checks over five decades, run over the full exponent
    // range instead. `λ(αM) = α·λ(M)` is exact mathematics, so the only thing that can break it is
    // the arithmetic leaving its range — which is precisely what the normalisation removes.
    let mut rng = Lcg::new(0x5EED_0011);
    for _ in 0..50 {
        let m = rng.sym3();
        let base = eigen_symmetric_3x3(&m).unwrap();
        for alpha in [1e-280f64, 1e-150, 1e-40, 1e40, 1e150, 1e280] {
            let scaled: [[f64; 3]; 3] =
                core::array::from_fn(|i| core::array::from_fn(|j| alpha * m[i][j]));
            let e = eigen_symmetric_3x3(&scaled).unwrap();
            for i in 0..3 {
                let want = alpha * base[i];
                assert!(
                    (e[i] - want).abs() <= 1e-9 * want.abs(),
                    "α={alpha:e}, i={i}: {} vs {want}",
                    e[i]
                );
            }
        }
    }
}
