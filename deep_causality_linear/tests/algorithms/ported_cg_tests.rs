/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The conjugate-gradient suite ported from `deep_causality_sparse::solver::cg`.
//!
//! Covers all three entry points — plain, Jacobi-preconditioned and warm-started — across
//! convergence on small SPD systems, the zero right-hand side, budget exhaustion, three precisions,
//! and the operator-failure and breakdown guards.
//!
//! Four contract changes separate this file from its source, and each ported test carries a note
//! where it is affected:
//!
//! Four divergences were recorded here when the port was written. Three have since been closed, and
//! this header is kept as the record of which:
//!
//! 1. ~~Argument order was `(…, max_iterations, tolerance)`.~~ Restored to the sparse crate's
//!    `(…, tolerance, max_iterations)`.
//! 2. ~~The preconditioner argument was the **reciprocal** of the diagonal, and came after `b`.~~
//!    Restored to the diagonal itself, before `b`.
//! 3. ~~`tolerance` was absolute where the sparse crate scaled it by `‖b‖`.~~ Restored to relative.
//!    This one outlived the other two: it changes no answer, only when the iteration stops, so it
//!    was invisible to every test here — all of them use `‖b‖ ≈ 1`, where the two agree.
//!    `cg_tests.rs` now pins the scaling directly.
//! 4. `CgFailure` is an enum of three named cases, where the sparse crate had one struct carrying
//!    `iterations` and `residual` for every failure mode. **This one stands** — it is the divergence
//!    a re-export shim cannot hide, and the one consumer that destructures the struct is repointed
//!    by hand rather than the type being widened back.
//!
//! A fifth was found while scouting phase 5 and is not a divergence but a defect: the operator's
//! returned length went unchecked, so a short vector indexed past the end of the residual and
//! panicked where the sparse crate returned a typed error. The tests below named that case and
//! asserted the curvature guard's incidental rejection of the *long* direction, which is why the
//! short one went unnoticed. Both directions are now checked.

use deep_causality_linear::{
    CgFailure, cg_solve, cg_solve_preconditioned, cg_solve_preconditioned_from,
};
use deep_causality_num::Float106;
use std::cell::Cell;

/// Euclidean norm of a slice, used to check the reported residual without reaching into the
/// solver's private helpers.
fn norm(v: &[f64]) -> f64 {
    v.iter().map(|&x| x * x).sum::<f64>().sqrt()
}

/// Apply a small dense SPD matrix to `v`. Used as a closure-backed operator.
fn dense_apply(a: &[[f64; 3]; 3], v: &[f64]) -> Vec<f64> {
    (0..3)
        .map(|i| (0..3).map(|j| a[i][j] * v[j]).sum())
        .collect()
}

/// Tridiagonal SPD operator and the reciprocal of its diagonal, shared by the warm-start tests.
///
/// The sparse original returned the diagonal itself; this crate's preconditioner argument is its
/// reciprocal, so the helper inverts once here rather than at every call site.
fn tridiag(n: usize) -> (impl Fn(&[f64]) -> Vec<f64>, Vec<f64>) {
    let apply = move |v: &[f64]| -> Vec<f64> {
        (0..v.len())
            .map(|i| {
                let mut acc = 2.5 * v[i];
                if i > 0 {
                    acc -= v[i - 1];
                }
                if i + 1 < v.len() {
                    acc -= v[i + 1];
                }
                acc
            })
            .collect()
    };
    (apply, vec![1.0 / 2.5_f64; n])
}

#[test]
fn test_cg_solves_2x2_spd_system() {
    // A = [[4, 1], [1, 3]], b = [1, 2], exact solution = [1/11, 7/11]
    let a = [[4.0_f64, 1.0], [1.0, 3.0]];
    let b = vec![1.0_f64, 2.0];
    let apply = |v: &[f64]| -> Vec<f64> {
        (0..2)
            .map(|i| (0..2).map(|j| a[i][j] * v[j]).sum())
            .collect()
    };
    let x = cg_solve(apply, &b, 1e-12_f64, 100).expect("CG converges");
    assert!((x[0] - 1.0 / 11.0).abs() < 1e-10);
    assert!((x[1] - 7.0 / 11.0).abs() < 1e-10);
}

#[test]
fn test_cg_solves_3x3_spd_system() {
    // A = diag(2, 3, 5), b = [4, 9, 25] → x = [2, 3, 5]
    let a = [[2.0_f64, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]];
    let b = vec![4.0_f64, 9.0, 25.0];
    let apply = |v: &[f64]| dense_apply(&a, v);
    let x = cg_solve(apply, &b, 1e-12_f64, 100).expect("CG converges");
    assert!((x[0] - 2.0).abs() < 1e-10);
    assert!((x[1] - 3.0).abs() < 1e-10);
    assert!((x[2] - 5.0).abs() < 1e-10);
}

#[test]
fn test_cg_returns_zero_for_zero_rhs() {
    let a = [[2.0_f64, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]];
    let b = vec![0.0_f64; 3];
    let apply = |v: &[f64]| dense_apply(&a, v);
    let x = cg_solve(apply, &b, 1e-12_f64, 100).expect("CG converges");
    for &xi in &x {
        assert!(xi.abs() < 1e-15);
    }
}

#[test]
fn test_cg_returns_zero_for_zero_rhs_at_zero_tolerance() {
    // b = 0 with tolerance = 0: the initial residual (0) equals the tolerance (0). The convergence
    // test must accept this exact solution rather than entering the loop, where `pᵀ A p = 0` would
    // otherwise report a false breakdown.
    let a = [[2.0_f64, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]];
    let b = vec![0.0_f64; 3];
    let apply = |v: &[f64]| dense_apply(&a, v);
    let x = cg_solve(apply, &b, 0.0_f64, 100).expect("exact zero solution converges");
    assert!(x.iter().all(|&xi| xi == 0.0));
}

#[test]
fn test_cg_reports_nonconvergence_at_iteration_cap() {
    // 100x100 well-conditioned diagonal system; cap to 1 iteration → cannot converge.
    let n = 100;
    let apply = |v: &[f64]| -> Vec<f64> {
        v.iter()
            .enumerate()
            .map(|(i, &vi)| (i as f64 + 1.0) * vi)
            .collect()
    };
    let b: Vec<f64> = (0..n)
        .map(|i| (i as f64 + 1.0) * (i as f64 + 1.0))
        .collect();
    let err = cg_solve(apply, &b, 1e-12_f64, 1).expect_err("CG must fail with iteration cap 1");
    match err {
        CgFailure::NotConverged {
            iterations,
            residual,
        } => {
            assert_eq!(iterations, 1);
            assert!(residual > 0.0);
        }
        other => panic!("expected a budget failure, got {other:?}"),
    }
}

#[test]
fn test_cg_reports_nonconvergence_with_zero_iteration_budget() {
    let apply = |v: &[f64]| v.to_vec();
    let b = vec![1.0_f64, 2.0, 3.0];
    let err = cg_solve(apply, &b, 1e-12_f64, 0).expect_err("CG must fail with iteration cap 0");
    match err {
        CgFailure::NotConverged {
            iterations,
            residual,
        } => {
            assert_eq!(iterations, 0);
            assert!((residual - norm(&b)).abs() < 1e-14);
        }
        other => panic!("expected a budget failure, got {other:?}"),
    }
}

#[test]
fn test_cg_converges_at_f32_precision() {
    let a = [[4.0_f32, 1.0], [1.0, 3.0]];
    let b = vec![1.0_f32, 2.0];
    let apply = |v: &[f32]| -> Vec<f32> {
        (0..2)
            .map(|i| (0..2).map(|j| a[i][j] * v[j]).sum())
            .collect()
    };
    let x = cg_solve(apply, &b, 1e-5_f32, 100).expect("CG converges at f32");
    assert!((x[0] - 1.0 / 11.0).abs() < 1e-4);
    assert!((x[1] - 7.0 / 11.0).abs() < 1e-4);
}

#[test]
fn test_cg_converges_at_float106_precision() {
    // A = [[4, 1], [1, 3]], b = [1, 2], exact solution = [1/11, 7/11].
    let f = Float106::from_f64;
    let a = [[f(4.0), f(1.0)], [f(1.0), f(3.0)]];
    let b = vec![f(1.0), f(2.0)];
    let apply = |v: &[Float106]| -> Vec<Float106> {
        (0..2)
            .map(|i| (0..2).fold(f(0.0), |acc, j| acc + a[i][j] * v[j]))
            .collect()
    };
    let x = cg_solve(apply, &b, f(1e-20), 100).expect("CG converges at Float106");
    assert!((x[0].to_f64() - 1.0 / 11.0).abs() < 1e-12);
    assert!((x[1].to_f64() - 7.0 / 11.0).abs() < 1e-12);
}

#[test]
fn test_cg_failure_is_clonable_and_debug_printable() {
    // The sparse crate's `CgFailure` was a struct with `iterations` and `residual` fields shared by
    // every failure mode. Here the budget case is one variant of an enum, and the fields it carries
    // are the same two.
    let f = CgFailure::NotConverged {
        iterations: 42_usize,
        residual: 1.23_f64,
    };
    let f2 = f.clone();
    match f2 {
        CgFailure::NotConverged {
            iterations,
            residual,
        } => {
            assert_eq!(iterations, 42);
            assert!((residual - 1.23).abs() < 1e-15);
        }
        other => panic!("clone changed the variant: {other:?}"),
    }
    let s = format!("{f:?}");
    assert!(s.contains("42"));
}

#[test]
fn test_preconditioned_cg_solves_diagonally_scaled_system() {
    // Badly scaled SPD diagonal system: Jacobi preconditioning makes it a one-iteration solve;
    // plain CG needs many more.
    let diag = [1.0_f64, 100.0, 10_000.0, 0.01];
    let inv_diag: Vec<f64> = diag.iter().map(|d| 1.0 / d).collect();
    let apply = |v: &[f64]| -> Vec<f64> { v.iter().zip(diag.iter()).map(|(x, d)| x * d).collect() };
    let b = [1.0, 2.0, 3.0, 4.0];
    let x = cg_solve_preconditioned(apply, &inv_diag, &b, 1e-12, 50).unwrap();
    for (i, (xi, (bi, di))) in x.iter().zip(b.iter().zip(diag.iter())).enumerate() {
        assert!((xi - bi / di).abs() < 1e-10, "x[{i}] = {xi}");
    }
}

#[test]
fn test_preconditioned_cg_agrees_with_plain_cg() {
    // Small SPD tridiagonal system.
    let n = 8usize;
    let (apply, diag_a) = tridiag(n);
    let b: Vec<f64> = (0..n).map(|i| ((i as f64) * 0.7).sin()).collect();
    let plain = cg_solve(&apply, &b, 1e-13, 200).unwrap();
    let pre = cg_solve_preconditioned(&apply, &diag_a, &b, 1e-13, 200).unwrap();
    for (a, c) in plain.iter().zip(pre.iter()) {
        assert!((a - c).abs() < 1e-9);
    }
}

#[test]
fn test_preconditioned_cg_absorbs_a_degenerate_diagonal_row() {
    // The sparse original passed the diagonal itself and the solver rewrote every entry at or below
    // zero to 1, leaving that row unpreconditioned. This crate takes the reciprocal and applies it
    // as given, so the caller supplies the 1 — the neutral value in the reciprocal convention.
    let apply = |v: &[f64]| -> Vec<f64> { vec![2.0 * v[0], 2.0 * v[1], 2.0 * v[2]] };
    let b = [2.0_f64, 4.0, 6.0];

    let neutralised = [0.5_f64, 1.0, 0.5];
    let x = cg_solve_preconditioned(apply, &neutralised, &b, 1e-12, 50).unwrap();
    assert!((x[0] - 1.0).abs() < 1e-10);
    assert!((x[1] - 2.0).abs() < 1e-10);
    assert!((x[2] - 3.0).abs() < 1e-10);

    // A degenerate entry is absorbed rather than reaching the caller: an entry at or below zero is
    // treated as `1`, which is no preconditioning on that row and keeps the preconditioner positive
    // definite. That is the documented behaviour of the code this moves from, so the solve still
    // reaches the same answer.
    let carried_through = [0.5_f64, 0.0, 0.5];
    let x = cg_solve_preconditioned(apply, &carried_through, &b, 1e-12, 50)
        .expect("a non-positive diagonal entry is neutralised, not reported");
    assert!((x[0] - 1.0).abs() < 1e-10);
    assert!((x[1] - 2.0).abs() < 1e-10);
    assert!((x[2] - 3.0).abs() < 1e-10);
}

#[test]
fn test_preconditioned_cg_surfaces_budget_exhaustion() {
    let n = 16usize;
    let apply = |v: &[f64]| -> Vec<f64> {
        (0..n)
            .map(|i| {
                let mut acc = 2.0 * v[i];
                if i > 0 {
                    acc -= v[i - 1];
                }
                if i + 1 < n {
                    acc -= v[i + 1];
                }
                acc
            })
            .collect()
    };
    let inv_diag = vec![0.5_f64; n];
    let b = vec![1.0_f64; n];
    let err = cg_solve_preconditioned(apply, &inv_diag, &b, 1e-15, 1).unwrap_err();
    assert!(
        matches!(err, CgFailure::NotConverged { iterations: 1, .. }),
        "got {err:?}"
    );
}

#[test]
fn test_warm_started_cg_agrees_with_a_cold_solve() {
    let n = 8usize;
    let (apply, diag_a) = tridiag(n);
    let b: Vec<f64> = (0..n).map(|i| ((i as f64) * 0.7).sin()).collect();

    let cold = cg_solve_preconditioned(&apply, &diag_a, &b, 1e-13, 200).unwrap();
    // A guess near the solution: the cold answer nudged. The warm solve must land on the same
    // solution to tolerance, independent of the guess.
    let x0: Vec<f64> = cold.iter().map(|c| c + 0.3).collect();
    let warm = cg_solve_preconditioned_from(&apply, &diag_a, &b, &x0, 1e-13, 200).unwrap();
    for (c, w) in cold.iter().zip(warm.iter()) {
        assert!((c - w).abs() < 1e-9, "warm {w} disagrees with cold {c}");
    }
}

#[test]
fn test_warm_started_cg_returns_immediately_from_the_exact_solution() {
    let n = 6usize;
    let (apply, diag_a) = tridiag(n);
    let b: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();
    let exact = cg_solve_preconditioned(&apply, &diag_a, &b, 1e-14, 200).unwrap();
    // Seeding with the exact solution: the initial residual is already below tolerance, so the
    // solver returns the guess without iterating.
    let warm = cg_solve_preconditioned_from(&apply, &diag_a, &b, &exact, 1e-10, 200).unwrap();
    for (e, w) in exact.iter().zip(warm.iter()) {
        assert!((e - w).abs() < 1e-12);
    }
}

#[test]
fn test_warm_started_cg_rejects_a_mismatched_initial_guess() {
    // The sparse original reported this as a failure at iteration 0 with the residual `‖b‖`. Here
    // the length check is its own variant and names both lengths.
    let (apply, diag_a) = tridiag(4);
    let b = vec![1.0_f64; 4];
    let x0 = vec![0.0_f64; 3]; // wrong length
    let err = cg_solve_preconditioned_from(&apply, &diag_a, &b, &x0, 1e-12, 50).unwrap_err();
    assert_eq!(
        err,
        CgFailure::LengthMismatch {
            expected: 4,
            found: 3
        }
    );
}

#[test]
fn test_warm_started_cg_surfaces_budget_exhaustion() {
    let n = 16usize;
    let (apply, diag_a) = tridiag(n);
    let b = vec![1.0_f64; n];
    let x0 = vec![0.0_f64; n];
    let err = cg_solve_preconditioned_from(&apply, &diag_a, &b, &x0, 1e-15, 1).unwrap_err();
    assert!(
        matches!(err, CgFailure::NotConverged { iterations: 1, .. }),
        "got {err:?}"
    );
}

// =============================================================================
// Operator-failure and algebraic-breakdown guards across the three CG variants.
// =============================================================================

#[test]
fn test_cg_solve_rejects_a_wrong_length_operator() {
    // The operator returns a vector of the wrong length. Compared against `b.len()` and reported
    // as a mismatch, as the sparse solver did.
    //
    // An earlier version of this crate had no such check: `zip` truncated the longer case against
    // `b` and the curvature guard rejected the solve one step later for the wrong reason, while the
    // shorter case indexed past the end of the residual and panicked.
    let b = vec![1.0_f64, 2.0];
    let long = |_v: &[f64]| -> Vec<f64> { vec![0.0; 5] };
    assert_eq!(
        cg_solve(long, &b, 1e-12, 50).unwrap_err(),
        CgFailure::LengthMismatch {
            expected: 2,
            found: 5
        }
    );
    // The direction that used to panic rather than truncate.
    let short = |_v: &[f64]| -> Vec<f64> { vec![0.0; 1] };
    assert_eq!(
        cg_solve(short, &b, 1e-12, 50).unwrap_err(),
        CgFailure::LengthMismatch {
            expected: 2,
            found: 1
        }
    );
}

#[test]
fn test_cg_solve_breaks_down_on_a_singular_operator() {
    // A·p = 0 for a non-zero search direction makes pᵀAp = 0 — the algebraic-breakdown guard.
    let b = vec![1.0_f64, 1.0];
    let apply = |_v: &[f64]| -> Vec<f64> { vec![0.0, 0.0] };
    let err = cg_solve(apply, &b, 1e-12, 50).unwrap_err();
    assert_eq!(
        err,
        CgFailure::NotPositiveDefinite { iteration: 0 },
        "breakdown fires on the first iteration"
    );
}

#[test]
fn test_preconditioned_cg_returns_zero_for_zero_rhs() {
    // b = 0 leaves the initial residual at 0, which clears any tolerance and converges immediately.
    let apply = |v: &[f64]| v.to_vec();
    let inv_diag = vec![1.0_f64, 1.0];
    let b = vec![0.0_f64, 0.0];
    let x = cg_solve_preconditioned(apply, &inv_diag, &b, 1e-12, 50).unwrap();
    assert_eq!(x, vec![0.0, 0.0]);
}

#[test]
fn test_preconditioned_cg_rejects_a_wrong_length_operator() {
    let inv_diag = vec![1.0_f64, 1.0];
    let b = vec![1.0_f64, 2.0];
    let apply = |_v: &[f64]| -> Vec<f64> { vec![0.0; 7] };
    let err = cg_solve_preconditioned(apply, &inv_diag, &b, 1e-12, 50).unwrap_err();
    assert_eq!(
        err,
        CgFailure::LengthMismatch {
            expected: 2,
            found: 7
        }
    );
    let short = |_v: &[f64]| -> Vec<f64> { vec![0.0; 1] };
    assert_eq!(
        cg_solve_preconditioned(short, &inv_diag, &b, 1e-12, 50).unwrap_err(),
        CgFailure::LengthMismatch {
            expected: 2,
            found: 1
        }
    );
}

#[test]
fn test_preconditioned_cg_breaks_down_on_a_singular_operator() {
    let inv_diag = vec![1.0_f64, 1.0];
    let b = vec![1.0_f64, 1.0];
    let apply = |_v: &[f64]| -> Vec<f64> { vec![0.0, 0.0] };
    let err = cg_solve_preconditioned(apply, &inv_diag, &b, 1e-12, 50).unwrap_err();
    assert_eq!(err, CgFailure::NotPositiveDefinite { iteration: 0 });
}

#[test]
fn test_warm_started_cg_returns_zero_for_zero_rhs() {
    // b = 0, x0 = 0: r₀ = 0 clears the tolerance and converges immediately.
    let apply = |v: &[f64]| v.to_vec();
    let inv_diag = vec![1.0_f64, 1.0];
    let b = vec![0.0_f64, 0.0];
    let x0 = vec![0.0_f64, 0.0];
    let x = cg_solve_preconditioned_from(apply, &inv_diag, &b, &x0, 1e-12, 50).unwrap();
    assert_eq!(x, vec![0.0, 0.0]);
}

#[test]
fn test_warm_started_cg_rejects_a_wrong_length_operator_on_the_initial_residual() {
    // The very first application (A·x₀) returns the wrong length, and is caught there rather than
    // being carried into the residual.
    let inv_diag = vec![1.0_f64, 1.0];
    let b = vec![1.0_f64, 2.0];
    let x0 = vec![0.0_f64, 0.0];
    let apply = |_v: &[f64]| -> Vec<f64> { vec![0.0; 9] };
    let err = cg_solve_preconditioned_from(apply, &inv_diag, &b, &x0, 1e-12, 50).unwrap_err();
    assert_eq!(
        err,
        CgFailure::LengthMismatch {
            expected: 2,
            found: 9
        }
    );
}

#[test]
fn test_warm_started_cg_rejects_a_wrong_length_operator_inside_the_loop() {
    // Correct length for the initial residual (A·x₀), wrong length once the loop applies A·p.
    let calls = Cell::new(0usize);
    let apply = |v: &[f64]| -> Vec<f64> {
        let n = calls.get();
        calls.set(n + 1);
        if n == 0 { v.to_vec() } else { vec![0.0; 9] }
    };
    let inv_diag = vec![1.0_f64, 1.0];
    let b = vec![1.0_f64, 2.0];
    let x0 = vec![0.0_f64, 0.0];
    let err = cg_solve_preconditioned_from(apply, &inv_diag, &b, &x0, 1e-12, 50).unwrap_err();
    assert_eq!(
        err,
        CgFailure::LengthMismatch {
            expected: 2,
            found: 9
        },
        "the loop's operator application is checked, not only the initial one"
    );
}

#[test]
fn test_warm_started_cg_breaks_down_on_a_singular_operator() {
    // A·x₀ = 0 (x₀ = 0) gives a non-zero residual, then A·p = 0 trips the breakdown guard.
    let apply = |_v: &[f64]| -> Vec<f64> { vec![0.0, 0.0] };
    let inv_diag = vec![1.0_f64, 1.0];
    let b = vec![1.0_f64, 1.0];
    let x0 = vec![0.0_f64, 0.0];
    let err = cg_solve_preconditioned_from(apply, &inv_diag, &b, &x0, 1e-12, 50).unwrap_err();
    assert_eq!(err, CgFailure::NotPositiveDefinite { iteration: 0 });
}
