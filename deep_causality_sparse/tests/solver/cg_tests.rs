/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Float106;
use deep_causality_sparse::{CgFailure, cg_solve};

/// Euclidean norm of a slice, used to check the reported residual without
/// reaching into the solver's private helpers.
fn norm(v: &[f64]) -> f64 {
    v.iter().map(|&x| x * x).sum::<f64>().sqrt()
}

/// Apply a small dense SPD matrix to `v`. Used as a closure-backed operator.
fn dense_apply(a: &[[f64; 3]; 3], v: &[f64]) -> Vec<f64> {
    (0..3)
        .map(|i| (0..3).map(|j| a[i][j] * v[j]).sum())
        .collect()
}

#[test]
fn cg_solves_2x2_spd_system() {
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
fn cg_solves_3x3_spd_system() {
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
fn cg_returns_zero_for_zero_rhs() {
    let a = [[2.0_f64, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]];
    let b = vec![0.0_f64; 3];
    let apply = |v: &[f64]| dense_apply(&a, v);
    let x = cg_solve(apply, &b, 1e-12_f64, 100).expect("CG converges");
    for &xi in &x {
        assert!(xi.abs() < 1e-15);
    }
}

#[test]
fn cg_returns_zero_for_zero_rhs_at_zero_tolerance() {
    // b = 0 with tolerance = 0: the initial residual (0) equals the absolute
    // tolerance (0). The convergence test must accept this exact solution rather
    // than entering the loop, where `pᵀ A p = 0` would otherwise report a false
    // breakdown.
    let a = [[2.0_f64, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]];
    let b = vec![0.0_f64; 3];
    let apply = |v: &[f64]| dense_apply(&a, v);
    let x = cg_solve(apply, &b, 0.0_f64, 100).expect("exact zero solution converges");
    assert!(x.iter().all(|&xi| xi == 0.0));
}

#[test]
fn cg_reports_nonconvergence_at_iteration_cap() {
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
    let result = cg_solve(apply, &b, 1e-12_f64, 1);
    let err = result.expect_err("CG must fail with iteration cap 1");
    assert_eq!(err.iterations, 1);
    assert!(err.residual > 0.0);
}

#[test]
fn cg_reports_nonconvergence_with_zero_iteration_budget() {
    let apply = |v: &[f64]| v.to_vec();
    let b = vec![1.0_f64, 2.0, 3.0];
    let result = cg_solve(apply, &b, 1e-12_f64, 0);
    let err = result.expect_err("CG must fail with iteration cap 0");
    assert_eq!(err.iterations, 0);
    assert!((err.residual - norm(&b)).abs() < 1e-14);
}

#[test]
fn cg_converges_at_f32_precision() {
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
fn cg_converges_at_float106_precision() {
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
fn cg_failure_struct_is_clonable_and_debug_printable() {
    let f = CgFailure {
        iterations: 42_usize,
        residual: 1.23_f64,
    };
    let f2 = f.clone();
    assert_eq!(f2.iterations, 42);
    assert!((f2.residual - 1.23).abs() < 1e-15);
    let s = format!("{:?}", f);
    assert!(s.contains("42"));
}

#[test]
fn preconditioned_cg_solves_diagonally_scaled_system() {
    use deep_causality_sparse::cg_solve_preconditioned;
    // Badly scaled SPD diagonal system: Jacobi preconditioning makes it
    // a one-iteration solve; plain CG needs many more.
    let diag = [1.0_f64, 100.0, 10_000.0, 0.01];
    let apply = |v: &[f64]| -> Vec<f64> { v.iter().zip(diag.iter()).map(|(x, d)| x * d).collect() };
    let b = [1.0, 2.0, 3.0, 4.0];
    let x = cg_solve_preconditioned(apply, &diag, &b, 1e-12, 50).unwrap();
    for (i, (xi, (bi, di))) in x.iter().zip(b.iter().zip(diag.iter())).enumerate() {
        assert!((xi - bi / di).abs() < 1e-10, "x[{i}] = {xi}");
    }
}

#[test]
fn preconditioned_cg_agrees_with_plain_cg() {
    use deep_causality_sparse::{cg_solve, cg_solve_preconditioned};
    // Small SPD tridiagonal system.
    let n = 8usize;
    let apply = |v: &[f64]| -> Vec<f64> {
        (0..n)
            .map(|i| {
                let mut acc = 2.5 * v[i];
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
    let diag = vec![2.5_f64; n];
    let b: Vec<f64> = (0..n).map(|i| ((i as f64) * 0.7).sin()).collect();
    let plain = cg_solve(apply, &b, 1e-13, 200).unwrap();
    let pre = cg_solve_preconditioned(apply, &diag, &b, 1e-13, 200).unwrap();
    for (a, c) in plain.iter().zip(pre.iter()) {
        assert!((a - c).abs() < 1e-9);
    }
}

#[test]
fn preconditioned_cg_handles_nonpositive_diagonal_entries() {
    use deep_causality_sparse::cg_solve_preconditioned;
    // Zero diagonal entries fall back to identity preconditioning rows.
    let diag = [2.0_f64, 0.0, 2.0];
    let apply = |v: &[f64]| -> Vec<f64> { vec![2.0 * v[0], 2.0 * v[1], 2.0 * v[2]] };
    let b = [2.0, 4.0, 6.0];
    let x = cg_solve_preconditioned(apply, &diag, &b, 1e-12, 50).unwrap();
    assert!((x[0] - 1.0).abs() < 1e-10);
    assert!((x[1] - 2.0).abs() < 1e-10);
    assert!((x[2] - 3.0).abs() < 1e-10);
}

#[test]
fn preconditioned_cg_surfaces_budget_exhaustion() {
    use deep_causality_sparse::cg_solve_preconditioned;
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
    let diag = vec![2.0_f64; n];
    let b = vec![1.0_f64; n];
    let err = cg_solve_preconditioned(apply, &diag, &b, 1e-15, 1).unwrap_err();
    assert_eq!(err.iterations, 1);
}

/// Tridiagonal SPD operator and its Jacobi diagonal, shared by the warm-start tests.
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
    (apply, vec![2.5_f64; n])
}

#[test]
fn warm_started_cg_agrees_with_a_cold_solve() {
    use deep_causality_sparse::{cg_solve_preconditioned, cg_solve_preconditioned_from};
    let n = 8usize;
    let (apply, diag) = tridiag(n);
    let b: Vec<f64> = (0..n).map(|i| ((i as f64) * 0.7).sin()).collect();

    let cold = cg_solve_preconditioned(&apply, &diag, &b, 1e-13, 200).unwrap();
    // A guess near the solution: the cold answer nudged. The warm solve must land on the same
    // solution to tolerance, independent of the guess.
    let x0: Vec<f64> = cold.iter().map(|c| c + 0.3).collect();
    let warm = cg_solve_preconditioned_from(&apply, &diag, &b, &x0, 1e-13, 200).unwrap();
    for (c, w) in cold.iter().zip(warm.iter()) {
        assert!((c - w).abs() < 1e-9, "warm {w} disagrees with cold {c}");
    }
}

#[test]
fn warm_started_cg_returns_immediately_from_the_exact_solution() {
    use deep_causality_sparse::{cg_solve_preconditioned, cg_solve_preconditioned_from};
    let n = 6usize;
    let (apply, diag) = tridiag(n);
    let b: Vec<f64> = (0..n).map(|i| (i as f64 + 1.0)).collect();
    let exact = cg_solve_preconditioned(&apply, &diag, &b, 1e-14, 200).unwrap();
    // Seeding with the exact solution: the initial residual is already below tolerance, so the
    // solver returns the guess without iterating.
    let warm = cg_solve_preconditioned_from(&apply, &diag, &b, &exact, 1e-10, 200).unwrap();
    for (e, w) in exact.iter().zip(warm.iter()) {
        assert!((e - w).abs() < 1e-12);
    }
}

#[test]
fn warm_started_cg_rejects_a_mismatched_initial_guess() {
    use deep_causality_sparse::cg_solve_preconditioned_from;
    let (apply, diag) = tridiag(4);
    let b = vec![1.0_f64; 4];
    let x0 = vec![0.0_f64; 3]; // wrong length
    let err = cg_solve_preconditioned_from(&apply, &diag, &b, &x0, 1e-12, 50).unwrap_err();
    assert_eq!(err.iterations, 0);
}

#[test]
fn warm_started_cg_surfaces_budget_exhaustion() {
    use deep_causality_sparse::cg_solve_preconditioned_from;
    let n = 16usize;
    let (apply, diag) = tridiag(n);
    let b = vec![1.0_f64; n];
    let x0 = vec![0.0_f64; n];
    let err = cg_solve_preconditioned_from(&apply, &diag, &b, &x0, 1e-15, 1).unwrap_err();
    assert_eq!(err.iterations, 1);
}
