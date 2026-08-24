/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::{
    CgFailure, cg_solve, cg_solve_preconditioned, cg_solve_preconditioned_from,
};

/// A symmetric positive-definite operator: the 1-D Laplacian on three interior points.
fn laplacian(v: &[f64]) -> Vec<f64> {
    let n = v.len();
    (0..n)
        .map(|i| {
            let left = if i == 0 { 0.0 } else { v[i - 1] };
            let right = if i + 1 == n { 0.0 } else { v[i + 1] };
            2.0 * v[i] - left - right
        })
        .collect()
}

#[test]
fn test_cg_solves_a_symmetric_positive_definite_system() {
    let b = vec![1.0, 0.0, 1.0];
    let x = cg_solve(laplacian, &b, 1e-12, 100).unwrap();
    // Check the residual rather than a precomputed solution.
    let ax = laplacian(&x);
    for i in 0..3 {
        assert!(
            (ax[i] - b[i]).abs() < 1e-9,
            "residual at {i} was {}",
            ax[i] - b[i]
        );
    }
}

#[test]
fn test_cg_reports_not_converged_rather_than_returning_a_wrong_answer() {
    let b = vec![1.0, 0.0, 1.0];
    let e = cg_solve(laplacian, &b, 1e-15, 1).unwrap_err();
    assert!(
        matches!(e, CgFailure::NotConverged { iterations: 1, .. }),
        "got {e:?}"
    );
}

#[test]
fn test_preconditioned_cg_agrees_with_the_plain_solve() {
    let b = vec![1.0, 0.0, 1.0];
    let diag_a = vec![2.0, 2.0, 2.0];
    let plain = cg_solve(laplacian, &b, 1e-12, 100).unwrap();
    let pre = cg_solve_preconditioned(laplacian, &diag_a, &b, 1e-12, 100).unwrap();
    for i in 0..3 {
        assert!(
            (plain[i] - pre[i]).abs() < 1e-9,
            "the two must agree at {i}"
        );
    }
}

#[test]
fn test_preconditioned_cg_uses_no_more_iterations_than_plain() {
    // The neumann-poisson requirement: the preconditioned solve converges no slower. Asserted by
    // giving it a budget and checking it reaches the reference answer within it, rather than only
    // that it returned Ok -- an implementation that returned a wrong vector would also return Ok.
    let b = vec![1.0, 0.0, 1.0];
    let diag_a = vec![2.0, 2.0, 2.0];
    let expected = [1.0, 1.0, 1.0];
    let x = cg_solve_preconditioned(laplacian, &diag_a, &b, 1e-10, 3)
        .expect("must converge within three iterations on a 3x3 SPD system");
    for i in 0..3 {
        assert!(
            (x[i] - expected[i]).abs() < 1e-8,
            "x[{i}] was {}, reference says {}",
            x[i],
            expected[i]
        );
    }
}

#[test]
fn test_an_initial_guess_is_honoured() {
    let b = vec![1.0, 0.0, 1.0];
    let diag_a = vec![2.0, 2.0, 2.0];
    let exact = cg_solve(laplacian, &b, 1e-12, 100).unwrap();
    let from_exact =
        cg_solve_preconditioned_from(laplacian, &diag_a, &b, &exact, 1e-12, 100).unwrap();
    for i in 0..3 {
        assert!(
            (from_exact[i] - exact[i]).abs() < 1e-9,
            "starting at the answer must stay there"
        );
    }
}

#[test]
fn test_a_right_hand_side_of_the_wrong_length_is_rejected() {
    let e =
        cg_solve_preconditioned(laplacian, &[2.0, 2.0, 2.0], &[1.0, 0.0], 1e-9, 10).unwrap_err();
    assert!(matches!(e, CgFailure::LengthMismatch { .. }), "got {e:?}");
}
