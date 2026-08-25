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

// =============================================================================
// The convergence threshold is relative to ‖b‖, not absolute.
// =============================================================================

#[test]
fn test_the_tolerance_is_relative_to_the_norm_of_the_right_hand_side() {
    // Reading `tolerance` as an absolute residual makes the criterion ‖b‖ times stricter than the
    // caller asked for, so a solve with a fixed iteration budget stops converging on systems it
    // used to handle. `deep_causality_topology` documents its default as a *relative* residual, and
    // this is what holds the crate to that.
    //
    // Zero iterations isolates the threshold: the only comparison left is the initial residual —
    // which is ‖b‖, since x₀ = 0 — against the threshold itself.
    let apply = |v: &[f64]| v.to_vec();
    let b = vec![60.0_f64, 80.0];
    assert_eq!(
        b.iter().map(|x| x * x).sum::<f64>().sqrt(),
        100.0,
        "the fixture's norm is what the scaling is read against"
    );

    // tolerance = 1 asks for a residual no larger than ‖b‖, which the initial residual meets
    // exactly. Read as an absolute threshold it would be 100x too tight and this would fail.
    assert_eq!(cg_solve(apply, &b, 1.0, 0).unwrap(), vec![0.0, 0.0]);

    // Just below it, the same residual does not clear the threshold.
    assert!(matches!(
        cg_solve(apply, &b, 0.99, 0),
        Err(CgFailure::NotConverged { iterations: 0, .. })
    ));
}

#[test]
fn test_a_zero_right_hand_side_takes_the_tolerance_unscaled() {
    // ‖b‖ = 0 has no scale to be relative to, so the tolerance is used as given. Scaling by zero
    // would make every threshold zero and turn an exactly-solved system into a failure.
    let apply = |v: &[f64]| v.to_vec();
    let b = vec![0.0_f64, 0.0];
    assert_eq!(cg_solve(apply, &b, 0.0, 0).unwrap(), vec![0.0, 0.0]);
    assert_eq!(cg_solve(apply, &b, 1e-12, 50).unwrap(), vec![0.0, 0.0]);
}

#[test]
fn test_the_relative_threshold_survives_a_large_right_hand_side() {
    // The case that motivated the fix: an ill-conditioned system with ‖b‖ far from 1 and a budget
    // that an absolute threshold would exhaust.
    let n = 200usize;
    let op = |v: &[f64]| -> Vec<f64> {
        (0..v.len())
            .map(|i| {
                let left = if i == 0 { 0.0 } else { v[i - 1] };
                let right = if i + 1 == v.len() { 0.0 } else { v[i + 1] };
                2.0 * v[i] - left - right
            })
            .collect()
    };
    let b: Vec<f64> = (0..n).map(|i| 1e6 * (1.0 + (i % 7) as f64)).collect();
    let x = cg_solve(op, &b, 1e-10, 400).expect("converges against a relative threshold");
    // The answer solves the system: ‖Ax - b‖ is within the relative threshold of ‖b‖.
    let residual = op(&x)
        .iter()
        .zip(&b)
        .map(|(a, c)| (a - c) * (a - c))
        .sum::<f64>()
        .sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!(
        residual <= 1e-10 * norm_b,
        "residual {residual:e} exceeds 1e-10 * {norm_b:e}"
    );
}
