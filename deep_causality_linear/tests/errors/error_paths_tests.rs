/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every error variant, reached by calling a real function with input that should be rejected.
//!
//! Formatting a variant proves it renders. It does not prove any code path produces it, and a
//! variant nothing produces is a promise the API does not keep. Each test here drives a public
//! function to the failure and asserts the specific variant, so a function that returned the wrong
//! error — or succeeded where it should have failed — is caught.
//!
//! The rule these enforce: **a rejected input returns a typed error**. Not a panic, not a wrapped
//! number, not a truncated result, not a fabricated zero.

use deep_causality_linear::{
    CsrMatrix, DenseMatrix, DenseVector, LinearError, MatrixBuild, MatrixView, PackedGf2,
    csr_to_packed_gf2_strict, determinant, determinant_exact, solve, solve_lower, solve_upper,
};
use deep_causality_num::Gf2;

// ---- IndexOutOfBounds ---------------------------------------------------------------------------

#[test]
fn test_index_out_of_bounds_from_every_container() {
    let d: DenseMatrix<f64> = DenseMatrix::zeros(2, 2);
    assert!(matches!(
        d.get(2, 0),
        Err(LinearError::IndexOutOfBounds {
            index: (2, 0),
            shape: (2, 2)
        })
    ));

    let c: CsrMatrix<f64> = CsrMatrix::zeros(2, 2);
    assert!(matches!(
        c.get(0, 2),
        Err(LinearError::IndexOutOfBounds { .. })
    ));

    let p: PackedGf2<u64> = PackedGf2::zeros(2, 2);
    assert!(matches!(
        p.get(9, 9),
        Err(LinearError::IndexOutOfBounds { .. })
    ));

    let v: DenseVector<f64> = DenseVector::from_vec(vec![1.0]);
    assert!(matches!(
        v.get(1),
        Err(LinearError::IndexOutOfBounds { .. })
    ));
}

#[test]
fn test_writing_out_of_bounds_is_rejected_and_leaves_the_matrix_unchanged() {
    let mut d: DenseMatrix<f64> = DenseMatrix::zeros(2, 2);
    assert!(d.set(5, 5, 1.0).is_err());
    // The rejected write must not have landed anywhere.
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(
                d.get(i, j).unwrap(),
                0.0,
                "a rejected write modified ({i}, {j})"
            );
        }
    }
}

// ---- ShapeMismatch ------------------------------------------------------------------------------

#[test]
fn test_a_buffer_shorter_than_the_shape_is_rejected() {
    assert!(matches!(
        DenseMatrix::from_vec(vec![1.0, 2.0], 2, 2),
        Err(LinearError::ShapeMismatch { .. })
    ));
}

#[test]
fn test_a_buffer_longer_than_the_shape_is_rejected_too() {
    // The asymmetric case: too much data is as wrong as too little, and a truncating constructor
    // would silently discard the tail.
    assert!(matches!(
        DenseMatrix::from_vec(vec![1.0; 9], 2, 2),
        Err(LinearError::ShapeMismatch { .. })
    ));
    assert!(matches!(
        PackedGf2::<u64>::from_slice(&[Gf2::ONE; 9], 2, 2),
        Err(LinearError::ShapeMismatch { .. })
    ));
}

#[test]
fn test_a_zero_dimension_with_a_non_empty_buffer_is_rejected() {
    assert!(matches!(
        DenseMatrix::from_vec(vec![1.0], 0, 0),
        Err(LinearError::ShapeMismatch {
            left: (0, 0),
            right: (1, 1)
        })
    ));
    assert!(matches!(
        DenseMatrix::from_vec(vec![1.0], 0, 5),
        Err(LinearError::ShapeMismatch {
            left: (0, 5),
            right: (1, 1)
        })
    ));
}

// ---- NotSquare ----------------------------------------------------------------------------------

#[test]
fn test_square_only_operations_reject_both_orientations() {
    for shape in [(2usize, 3usize), (3, 2)] {
        let m: DenseMatrix<f64> = DenseMatrix::zeros(shape.0, shape.1);
        assert!(
            matches!(determinant(&m), Err(LinearError::NotSquare { .. })),
            "determinant accepted {shape:?}"
        );
        let mi: DenseMatrix<i64> = DenseMatrix::zeros(shape.0, shape.1);
        assert!(
            matches!(determinant_exact(&mi), Err(LinearError::NotSquare { .. })),
            "exact determinant accepted {shape:?}"
        );
    }
}

// ---- LengthMismatch -----------------------------------------------------------------------------

#[test]
fn test_a_vector_operation_rejects_a_length_mismatch_in_both_directions() {
    let short: DenseVector<f64> = DenseVector::from_vec(vec![1.0]);
    let long: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    assert!(matches!(
        short.dot(&long),
        Err(LinearError::LengthMismatch {
            expected: 1,
            found: 3
        })
    ));
    assert!(matches!(
        long.dot(&short),
        Err(LinearError::LengthMismatch {
            expected: 3,
            found: 1
        })
    ));
    assert!(short.add(&long).is_err());
    assert!(long.sub(&short).is_err());
    assert!(short.hermitian_inner(&long).is_err());
}

#[test]
fn test_a_solve_rejects_a_right_hand_side_of_the_wrong_length() {
    let a: DenseMatrix<f64> = DenseMatrix::identity(3);
    for len in [0usize, 2, 4] {
        let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0; len]);
        assert!(
            matches!(solve(&a, &b), Err(LinearError::LengthMismatch { .. })),
            "solve accepted a right-hand side of length {len} for a 3x3"
        );
    }
}

// ---- Singular -----------------------------------------------------------------------------------

#[test]
fn test_a_singular_system_is_rejected_rather_than_given_an_arbitrary_answer() {
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 2.0, 2.0, 4.0], 2, 2).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0]);
    // This system is consistent -- b is in the column space -- and still has no unique solution.
    // Returning one of the infinitely many would be worse than failing.
    assert!(matches!(solve(&a, &b), Err(LinearError::Singular { .. })));
}

#[test]
fn test_an_all_zero_matrix_is_singular_rather_than_a_division_by_zero() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(3, 3);
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 1.0, 1.0]);
    assert!(matches!(
        solve(&a, &b),
        Err(LinearError::Singular { at_column: 0 })
    ));
}

// ---- ZeroDiagonal and WrongTriangle -------------------------------------------------------------

#[test]
fn test_triangular_substitution_rejects_a_zero_anywhere_on_the_diagonal() {
    // Not only the first: a zero at the end is just as undividable.
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 2.0, 0.0], 2, 2).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 1.0]);
    assert!(matches!(
        solve_lower(&a, &b),
        Err(LinearError::ZeroDiagonal { at_index: 1 })
    ));

    let c: DenseMatrix<f64> = DenseMatrix::from_vec(vec![0.0, 1.0, 0.0, 1.0], 2, 2).unwrap();
    assert!(matches!(
        solve_upper(&c, &b),
        Err(LinearError::ZeroDiagonal { at_index: 0 })
    ));
}

#[test]
fn test_substitution_rejects_the_wrong_triangle_rather_than_ignoring_it() {
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 1.0]);
    // A non-zero above the diagonal, offered to forward substitution.
    let upper: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 5.0, 2.0, 1.0], 2, 2).unwrap();
    assert!(matches!(
        solve_lower(&upper, &b),
        Err(LinearError::WrongTriangle { at: (0, 1) })
    ));
    // And below, offered to backward substitution.
    let lower: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 5.0, 1.0], 2, 2).unwrap();
    assert!(matches!(
        solve_upper(&lower, &b),
        Err(LinearError::WrongTriangle { at: (1, 0) })
    ));
}

// ---- NotBinary ----------------------------------------------------------------------------------

#[test]
fn test_the_strict_packing_rejects_every_value_outside_zero_and_one() {
    // Both signs, and both magnitudes past the alphabet.
    for bad in [-1i8, 2, -2, 7, i8::MIN, i8::MAX] {
        let m = CsrMatrix::from_triplets(2, 2, &[(1, 1, bad)]).unwrap();
        assert!(
            matches!(
                csr_to_packed_gf2_strict::<u64>(&m),
                Err(LinearError::NotBinary { at: (1, 1) })
            ),
            "the strict packing accepted {bad}"
        );
    }
}

#[test]
fn test_the_strict_packing_accepts_exactly_zero_and_one() {
    for good in [0i8, 1] {
        let m = CsrMatrix::from_triplets(2, 2, &[(1, 1, good)]).unwrap();
        let p = csr_to_packed_gf2_strict::<u64>(&m)
            .unwrap_or_else(|e| panic!("{good} is inside the alphabet, but the packing gave {e}"));
        // Accepting is not enough: the entry has to arrive, and the rest has to stay clear.
        assert_eq!(p.shape(), (2, 2));
        assert_eq!(p.get(1, 1).unwrap(), Gf2::new(good == 1));
        for (i, j) in [(0, 0), (0, 1), (1, 0)] {
            assert_eq!(p.get(i, j).unwrap(), Gf2::ZERO, "at ({i}, {j})");
        }
    }
}

// ---- InnerDimensionMismatch ---------------------------------------------------------------------

#[test]
fn test_a_product_whose_inner_dimensions_do_not_meet_is_rejected() {
    let a: CsrMatrix<f64> = CsrMatrix::zeros(2, 3);
    let b: CsrMatrix<f64> = CsrMatrix::zeros(4, 2);
    assert!(matches!(
        a.mat_mult(&b),
        Err(LinearError::InnerDimensionMismatch {
            left_cols: 3,
            right_rows: 4
        })
    ));
}

#[test]
fn test_a_sparse_matrix_vector_product_rejects_a_wrong_length_vector() {
    let a: CsrMatrix<f64> = CsrMatrix::zeros(2, 3);
    assert!(matches!(
        a.vec_mult(&[1.0, 2.0]),
        Err(LinearError::LengthMismatch { .. })
    ));
}

// ---- construction validation --------------------------------------------------------------------

#[test]
fn test_a_triplet_outside_the_shape_is_rejected() {
    assert!(matches!(
        CsrMatrix::from_triplets(2, 2, &[(2, 0, 1.0)]),
        Err(LinearError::IndexOutOfBounds { .. })
    ));
    assert!(matches!(
        CsrMatrix::from_triplets(2, 2, &[(0, 2, 1.0)]),
        Err(LinearError::IndexOutOfBounds { .. })
    ));
}

#[test]
fn test_an_error_is_returned_rather_than_panicking_for_every_rejected_input() {
    // The blanket property. Each of these is an input the API must refuse, and refusing means a
    // typed error reaching the caller rather than the process ending.
    let d: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    let v: DenseVector<f64> = DenseVector::from_vec(vec![1.0]);
    let p: PackedGf2<u8> = PackedGf2::zeros(1, 1);

    assert!(matches!(
        determinant(&d),
        Err(LinearError::NotSquare { shape: (2, 3) })
    ));
    assert!(matches!(
        d.get(99, 99),
        Err(LinearError::IndexOutOfBounds {
            index: (99, 99),
            shape: (2, 3)
        })
    ));
    assert!(matches!(
        v.get(99),
        Err(LinearError::IndexOutOfBounds {
            index: (99, 0),
            shape: (1, 1)
        })
    ));
    assert!(matches!(
        p.get(99, 99),
        Err(LinearError::IndexOutOfBounds {
            index: (99, 99),
            shape: (1, 1)
        })
    ));
    assert!(matches!(
        DenseMatrix::<f64>::from_vec(vec![1.0], 7, 7),
        Err(LinearError::ShapeMismatch {
            left: (7, 7),
            right: (1, 1)
        })
    ));
    assert!(matches!(
        CsrMatrix::<f64>::from_triplets(1, 1, &[(9, 9, 1.0)]),
        Err(LinearError::IndexOutOfBounds {
            index: (9, 9),
            shape: (1, 1)
        })
    ));
}
