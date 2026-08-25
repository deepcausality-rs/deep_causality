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

// =============================================================================
// What the failures say. A caller that logs the message and nothing else still has to be able to
// tell the three apart and to see the numbers that name the failure.
// =============================================================================

#[test]
fn test_the_breakdown_message_names_the_iteration_and_the_curvature_it_found() {
    // Negating the identity gives pᵀAp = -‖p‖², which is negative on the first search direction.
    let negate = |v: &[f64]| -> Vec<f64> { v.iter().map(|x| -x).collect() };
    let e = cg_solve(negate, &[1.0, 2.0], 1e-12, 10).unwrap_err();
    assert_eq!(e, CgFailure::NotPositiveDefinite { iteration: 0 });
    let s = format!("{e}");
    assert!(s.contains('0'), "must name the iteration it broke at: {s}");
    assert!(
        s.contains("not positive definite"),
        "must name the cause: {s}"
    );
    assert!(
        s.contains("curvature"),
        "must say what was found rather than only that it stopped: {s}"
    );
}

#[test]
fn test_the_length_mismatch_message_names_the_length_produced_and_the_one_expected() {
    // The operator is the caller's closure and can return any length. Both numbers have to appear,
    // and on the right sides of the sentence: 3 is what was asked for, 5 is what came back.
    let too_long = |v: &[f64]| -> Vec<f64> {
        let mut out = v.to_vec();
        out.extend([0.0, 0.0]);
        out
    };
    let e = cg_solve(too_long, &[1.0, 1.0, 1.0], 1e-12, 10).unwrap_err();
    assert_eq!(
        e,
        CgFailure::LengthMismatch {
            expected: 3,
            found: 5
        }
    );
    let s = format!("{e}");
    assert!(s.contains("length mismatch"), "{s}");
    assert!(
        s.contains("produced 5"),
        "the operator's length is the one it produced: {s}"
    );
    assert!(
        s.contains("right-hand side of 3"),
        "the expected length is the right-hand side's: {s}"
    );
}

// ---- mutation-driven: the preconditioner's effect was never asserted ---------------------------

/// Jacobi-preconditioned CG solves a diagonal system in one iteration.
///
/// For `A = diag(d)` the Jacobi preconditioner is `M = A`, so `M⁻¹A = I` and the first search
/// direction already points at the solution. That makes a one-iteration budget a sharp test of the
/// preconditioner: it succeeds only if the preconditioning step is `x / dᵢ` applied to positive
/// diagonal entries.
///
/// Mutation testing found both halves of that step unasserted. Replacing `dᵢ > 0` with `dᵢ < 0`
/// and replacing `x / dᵢ` with `x * dᵢ` each survived the whole suite: the existing cases used a
/// Laplacian with a constant diagonal and a generous iteration budget, where preconditioning
/// changes the iteration count but never the answer.
#[test]
fn test_the_jacobi_preconditioner_solves_a_diagonal_system_in_one_iteration() {
    let d = [100.0_f64, 4.0, 0.25, 9.0];
    let apply = |v: &[f64]| -> Vec<f64> { v.iter().zip(d.iter()).map(|(x, di)| x * di).collect() };
    let b = [100.0_f64, 8.0, 0.5, 27.0];
    // A x = b has the exact solution (1, 2, 2, 3).
    let expected = [1.0_f64, 2.0, 2.0, 3.0];

    let x = cg_solve_preconditioned(apply, &d, &b, 1e-12, 1)
        .expect("preconditioned CG solves a diagonal system in one iteration");
    for (got, want) in x.iter().zip(expected.iter()) {
        assert!(
            (got - want).abs() < 1e-9,
            "expected {want}, got {got}; the preconditioner did not reduce the system to identity"
        );
    }
}

/// The same system without preconditioning does not converge in one iteration.
///
/// This is the control. It shows the test above is about the preconditioner rather than about the
/// system being easy: the diagonal spans four orders of magnitude, so plain CG needs its full
/// Krylov sequence.
#[test]
fn test_the_same_diagonal_system_needs_more_than_one_iteration_unpreconditioned() {
    let d = [100.0_f64, 4.0, 0.25, 9.0];
    let apply = |v: &[f64]| -> Vec<f64> { v.iter().zip(d.iter()).map(|(x, di)| x * di).collect() };
    let b = [100.0_f64, 8.0, 0.5, 27.0];

    let outcome = cg_solve(apply, &b, 1e-12, 1);
    assert!(
        outcome.is_err(),
        "plain CG on a system with a spread diagonal must not converge in one iteration"
    );
}

/// A non-positive diagonal entry is left unpreconditioned rather than inverted.
///
/// The guard is `dᵢ > 0`. A zero entry must pass the residual through unchanged; inverting it
/// would divide by zero. A negative entry must also pass through, because dividing by it would
/// make the preconditioner indefinite.
#[test]
fn test_a_non_positive_diagonal_entry_passes_through_unpreconditioned() {
    // A is diag(4, 1) but the caller supplies a diagonal with a zero and a negative entry, which
    // is what a clipped or partially-degenerate diagonal looks like. The solve must still reach
    // the right answer, which it can only do if those rows are left alone.
    let a = [4.0_f64, 1.0];
    let apply = |v: &[f64]| -> Vec<f64> { v.iter().zip(a.iter()).map(|(x, ai)| x * ai).collect() };
    let b = [8.0_f64, 3.0];

    for supplied in [[0.0_f64, 1.0], [-4.0_f64, 1.0]] {
        let x = cg_solve_preconditioned(apply, &supplied, &b, 1e-12, 50)
            .unwrap_or_else(|e| panic!("diagonal {supplied:?} must still solve: {e}"));
        assert!((x[0] - 2.0).abs() < 1e-9, "x0 was {}", x[0]);
        assert!((x[1] - 3.0).abs() < 1e-9, "x1 was {}", x[1]);
    }
}
