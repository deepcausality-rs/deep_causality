/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{
    DenseMatrix, DenseVector, LinearError, Lu, MatrixBuild, inverse, solve, solve_lower,
    solve_upper,
};

fn dense(f: (Vec<f64>, usize, usize)) -> DenseMatrix<f64> {
    let (d, r, c) = f;
    DenseMatrix::from_vec(d, r, c).unwrap()
}

#[test]
fn test_a_known_system_is_solved() {
    // [2 1; 1 3] x = [5; 10]  =>  x = [1; 3]
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(vec![2.0, 1.0, 1.0, 3.0], 2, 2).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![5.0, 10.0]);
    let x = solve(&a, &b).unwrap();
    assert!(
        (x.get(0).unwrap() - 1.0).abs() < 1e-12,
        "x0 was {}",
        x.get(0).unwrap()
    );
    assert!(
        (x.get(1).unwrap() - 3.0).abs() < 1e-12,
        "x1 was {}",
        x.get(1).unwrap()
    );
}

#[test]
fn test_a_singular_system_is_rejected_rather_than_answered() {
    let a = dense(singular_2x2());
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0]);
    let e = solve(&a, &b).unwrap_err();
    assert!(matches!(e, LinearError::Singular { .. }), "got {e:?}");
}

#[test]
fn test_a_zero_leading_entry_is_handled_by_pivoting() {
    let a = dense(zero_leading_entry_3x3());
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    let x = solve(&a, &b).unwrap();
    // The matrix swaps rows 0 and 1, so the solution swaps the first two entries of b.
    assert!((x.get(0).unwrap() - 2.0).abs() < 1e-12);
    assert!((x.get(1).unwrap() - 1.0).abs() < 1e-12);
    assert!((x.get(2).unwrap() - 3.0).abs() < 1e-12);
}

#[test]
fn test_a_right_hand_side_of_the_wrong_length_is_rejected() {
    let a: DenseMatrix<f64> = DenseMatrix::identity(3);
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0]);
    assert!(matches!(
        solve(&a, &b),
        Err(LinearError::LengthMismatch { .. })
    ));
}

#[test]
fn test_a_non_square_system_is_rejected() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0]);
    assert!(matches!(solve(&a, &b), Err(LinearError::NotSquare { .. })));
}

// ---- the factorisation as a reusable value ------------------------------------------------------

#[test]
fn test_one_factorisation_serves_three_right_hand_sides() {
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(vec![2.0, 1.0, 1.0, 3.0], 2, 2).unwrap();
    let lu = Lu::factor(&a).unwrap();
    for b in [vec![5.0, 10.0], vec![1.0, 0.0], vec![0.0, 1.0]] {
        let rhs: DenseVector<f64> = DenseVector::from_vec(b);
        let from_lu = lu.apply(&rhs).unwrap();
        let from_solve = solve(&a, &rhs).unwrap();
        for i in 0..2 {
            assert!(
                (from_lu.get(i).unwrap() - from_solve.get(i).unwrap()).abs() < 1e-12,
                "the reused factorisation must agree with a fresh solve"
            );
        }
    }
}

#[test]
fn test_the_factorisation_carries_its_permutation() {
    let a = dense(zero_leading_entry_3x3());
    let lu = Lu::factor(&a).unwrap();
    let p = lu.permutation();
    assert_eq!(p.len(), 3);
    // Pivoting had to move row 0, whose leading entry is zero.
    assert_ne!(p[0], 0, "the permutation must record the swap: {p:?}");
}

#[test]
fn test_a_singular_matrix_fails_at_factorisation_not_at_the_first_application() {
    let a = dense(singular_2x2());
    assert!(matches!(Lu::factor(&a), Err(LinearError::Singular { .. })));
}

#[test]
fn test_the_determinant_falls_out_of_the_factorisation() {
    let a = dense(unit_determinant_3x3());
    let lu = Lu::factor(&a).unwrap();
    assert!((lu.determinant() - UNIT_DETERMINANT_3X3).abs() < 1e-12);
}

#[test]
fn test_the_factorisation_determinant_carries_the_swap_sign() {
    let a = dense(zero_leading_entry_3x3());
    let lu = Lu::factor(&a).unwrap();
    assert!(
        (lu.determinant() - ZERO_LEADING_ENTRY_DETERMINANT).abs() < 1e-12,
        "one row swap makes the determinant negative"
    );
}

// ---- triangular substitution --------------------------------------------------------------------

#[test]
fn test_backward_substitution_agrees_with_the_general_path() {
    // Upper triangular, so solve and solve_upper must give the same answer.
    let a = dense(unit_determinant_3x3());
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    let direct = solve_upper(&a, &b).unwrap();
    let general = solve(&a, &b).unwrap();
    for i in 0..3 {
        assert!((direct.get(i).unwrap() - general.get(i).unwrap()).abs() < 1e-12);
    }
}

#[test]
fn test_forward_substitution_on_a_lower_triangular_system() {
    // [1 0; 2 1] x = [1; 4]  =>  x = [1; 2]
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 2.0, 1.0], 2, 2).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 4.0]);
    let x = solve_lower(&a, &b).unwrap();
    assert!((x.get(0).unwrap() - 1.0).abs() < 1e-12);
    assert!((x.get(1).unwrap() - 2.0).abs() < 1e-12);
}

#[test]
fn test_a_zero_on_the_diagonal_is_rejected_rather_than_divided_by() {
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 2.0, 0.0], 2, 2).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 1.0]);
    let e = solve_lower(&a, &b).unwrap_err();
    assert!(
        matches!(e, LinearError::ZeroDiagonal { at_index: 1 }),
        "got {e:?}"
    );
}

#[test]
fn test_the_wrong_triangle_is_rejected_rather_than_ignored() {
    // A non-zero above the diagonal offered to forward substitution.
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 5.0, 2.0, 1.0], 2, 2).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 1.0]);
    let e = solve_lower(&a, &b).unwrap_err();
    assert!(
        matches!(e, LinearError::WrongTriangle { at: (0, 1) }),
        "got {e:?}"
    );
}

// ---- solve against invert-and-multiply -----------------------------------------------------------

#[test]
fn test_solving_is_at_least_as_accurate_as_inverting_then_multiplying() {
    // An ill-conditioned system: the Hilbert 4x4.
    let mut d = Vec::new();
    for i in 0..4 {
        for j in 0..4 {
            d.push(1.0 / ((i + j + 1) as f64));
        }
    }
    let a = DenseMatrix::from_vec(d, 4, 4).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 1.0, 1.0, 1.0]);

    let x_solve = solve(&a, &b).unwrap();
    let a_inv = inverse(&a).unwrap();

    let residual = |x: &DenseVector<f64>| -> f64 {
        use deep_causality_linear::MatrixView;
        let mut worst = 0.0f64;
        for i in 0..4 {
            let mut acc = 0.0;
            for j in 0..4 {
                acc += a.get(i, j).unwrap() * x.get(j).unwrap();
            }
            worst = worst.max((acc - b.get(i).unwrap()).abs());
        }
        worst
    };

    // A^-1 b, formed the way estimation.rs:158 does it.
    use deep_causality_linear::MatrixView;
    let mut x_invert = Vec::new();
    for i in 0..4 {
        let mut acc = 0.0;
        for j in 0..4 {
            acc += a_inv.get(i, j).unwrap() * b.get(j).unwrap();
        }
        x_invert.push(acc);
    }
    let x_invert: DenseVector<f64> = DenseVector::from_vec(x_invert);

    assert!(
        residual(&x_solve) <= residual(&x_invert) * (1.0 + 1e-9),
        "solve residual {} must not exceed invert-and-multiply {}",
        residual(&x_solve),
        residual(&x_invert)
    );
}
