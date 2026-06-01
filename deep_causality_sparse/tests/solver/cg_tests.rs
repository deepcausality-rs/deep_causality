/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

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
