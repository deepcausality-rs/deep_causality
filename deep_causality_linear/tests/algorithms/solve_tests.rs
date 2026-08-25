/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{
    DenseMatrix, DenseVector, LinearError, LinearErrorEnum, Lu, MatrixBuild, MatrixView, inverse,
    solve, solve_lower, solve_upper,
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
    assert!(
        matches!(e, LinearError(LinearErrorEnum::Singular { .. })),
        "got {e:?}"
    );
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
        Err(LinearError(LinearErrorEnum::LengthMismatch { .. }))
    ));
}

#[test]
fn test_a_non_square_system_is_rejected() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0]);
    assert!(matches!(
        solve(&a, &b),
        Err(LinearError(LinearErrorEnum::NotSquare { .. }))
    ));
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
    assert!(matches!(
        Lu::factor(&a),
        Err(LinearError(LinearErrorEnum::Singular { .. }))
    ));
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
fn test_backward_substitution_matches_the_exact_reference() {
    // The reference answer, from exact rational elimination in
    // openspec/notes/linear/reference/reference.py: [1 2 3; 0 1 4; 0 0 1] x = [1; 2; 3]
    // has the exact solution [12, -10, 3].
    //
    // Comparing solve_upper against solve would be circular: both are this crate's, and a shared
    // defect would pass. The reference is computed independently.
    let a = dense(unit_determinant_3x3());
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    let expected = [12.0, -10.0, 3.0];

    let direct = solve_upper(&a, &b).unwrap();
    for (i, want) in expected.iter().enumerate() {
        assert!(
            (direct.get(i).unwrap() - want).abs() < 1e-12,
            "solve_upper[{i}] was {}, reference says {want}",
            direct.get(i).unwrap()
        );
    }

    // And the general path must reach the same reference answer, which is a different claim from
    // the two agreeing with each other.
    let general = solve(&a, &b).unwrap();
    for (i, want) in expected.iter().enumerate() {
        assert!(
            (general.get(i).unwrap() - want).abs() < 1e-12,
            "solve[{i}] was {}, reference says {want}",
            general.get(i).unwrap()
        );
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
        matches!(
            e,
            LinearError(LinearErrorEnum::ZeroDiagonal { at_index: 1 })
        ),
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
        matches!(
            e,
            LinearError(LinearErrorEnum::WrongTriangle { at: (0, 1) })
        ),
        "got {e:?}"
    );
}

// ---- solve against invert-and-multiply -----------------------------------------------------------

#[test]
fn test_solving_is_strictly_more_accurate_than_inverting_then_multiplying() {
    // The Hilbert matrix of order 8, whose condition number is around 1e10.
    //
    // The margin is the point. An earlier version of this test used order 4 and asserted only
    // `solve <= invert * (1 + 1e-9)`, which is satisfied trivially when `solve` IS
    // invert-then-multiply -- the test passed with the defect injected. Measured ratios of the two
    // residuals on this crate:
    //
    //   n= 4      44x        n= 8    3_893x        n=12  5_952_098x
    //
    // Order 8 gives three orders of magnitude of headroom, so a factor of 10 discriminates without
    // being fragile.
    const N: usize = 8;
    let mut d = Vec::new();
    for i in 0..N {
        for j in 0..N {
            d.push(1.0 / ((i + j + 1) as f64));
        }
    }
    let a: DenseMatrix<f64> = DenseMatrix::from_vec(d, N, N).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0; N]);

    let residual = |x: &DenseVector<f64>| -> f64 {
        let mut worst = 0.0f64;
        for i in 0..N {
            let mut acc = 0.0;
            for j in 0..N {
                acc += a.get(i, j).unwrap() * x.get(j).unwrap();
            }
            worst = worst.max((acc - b.get(i).unwrap()).abs());
        }
        worst
    };

    let by_solve = residual(&solve(&a, &b).unwrap());

    // A^-1 b, formed the way estimation.rs:158 forms the Kalman gain.
    let inv = inverse(&a).unwrap();
    let mut xi = Vec::with_capacity(N);
    for i in 0..N {
        let mut acc = 0.0;
        for j in 0..N {
            acc += inv.get(i, j).unwrap() * b.get(j).unwrap();
        }
        xi.push(acc);
    }
    let by_inverting = residual(&DenseVector::from_vec(xi));

    assert!(
        by_solve * 10.0 < by_inverting,
        "solve must be strictly better than inverting and multiplying: solve {by_solve:.3e}, invert {by_inverting:.3e}. \
         If these are equal, solve is implemented as invert-then-multiply."
    );
}
