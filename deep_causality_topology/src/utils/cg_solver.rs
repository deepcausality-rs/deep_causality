/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Matrix-free Conjugate Gradient solver.
//!
//! Solves `A x = b` for a symmetric positive semi-definite operator `A`
//! provided as a closure `apply: Fn(&[R]) -> Vec<R>` returning `A · v`. The
//! operator itself is never assembled as a matrix; this lets callers reuse
//! existing matrix-free operator-application pipelines (e.g.
//! [`crate::types::manifold::Manifold::laplacian`]).
//!
//! Convergence is measured against the relative residual `‖r‖ / ‖b‖` (with the
//! `‖b‖ = 0` corner case falling back to the absolute residual). The caller
//! supplies a tolerance `R`-derived from the precision backend's machine
//! epsilon; see `deep_causality_topology::HodgeDecomposeOptions` for the
//! recommended default.
//!
//! This solver is private to the crate. If a downstream consumer needs an
//! assembled-matrix CG, the helper can be lifted into `deep_causality_sparse`
//! and exposed publicly there without breaking any current call site.

use deep_causality_num::{FromPrimitive, RealField};

/// Reason a CG solve failed to converge.
///
/// Carried inside the private `HodgeFailReason<R>` enum in
/// `hodge_decomposition_impl.rs`; never crosses the crate boundary.
#[derive(Debug, Clone)]
pub(crate) struct CgFailure<R: RealField> {
    pub iterations: usize,
    pub residual: R,
}

/// Run Conjugate Gradient on a symmetric positive semi-definite operator.
///
/// # Arguments
/// * `apply` — closure computing `A · v` for arbitrary `v`.
/// * `b` — right-hand side vector.
/// * `tolerance` — relative residual threshold for convergence.
/// * `max_iterations` — iteration budget.
///
/// # Returns
/// `Ok(x)` if the residual fell below `tolerance · ‖b‖`, `Err(CgFailure)`
/// if the iteration budget was exhausted or a numerical breakdown (`p^T A p = 0`
/// with a non-zero search direction) occurred.
pub(crate) fn cg_solve<R, Apply>(
    apply: Apply,
    b: &[R],
    tolerance: R,
    max_iterations: usize,
) -> Result<Vec<R>, CgFailure<R>>
where
    R: RealField + FromPrimitive + PartialEq,
    Apply: Fn(&[R]) -> Vec<R>,
{
    let n = b.len();
    let mut x = vec![R::zero(); n];

    // r₀ = b - A·x₀ = b  (since x₀ = 0)
    let mut r: Vec<R> = b.to_vec();
    let mut p = r.clone();
    let mut rsold: R = dot(&r, &r);

    let b_norm = dot(b, b).sqrt();
    let abs_tol = if b_norm == R::zero() {
        tolerance
    } else {
        tolerance * b_norm
    };

    if rsold.sqrt() < abs_tol {
        return Ok(x);
    }

    for iter in 0..max_iterations {
        let ap = apply(&p);
        if ap.len() != n {
            return Err(CgFailure {
                iterations: iter,
                residual: rsold.sqrt(),
            });
        }
        let pap = dot(&p, &ap);
        // Algebraic breakdown only. A relative threshold here would either
        // (a) fire spuriously on the very small `pap` values that arise at
        // Float106 / f32 precision near convergence, or (b) miss true
        // breakdowns at large magnitudes. Exact-zero is the only check that
        // is precision-invariant; non-convergence past `max_iterations`
        // catches every other failure mode.
        if pap == R::zero() {
            return Err(CgFailure {
                iterations: iter,
                residual: rsold.sqrt(),
            });
        }
        let alpha = rsold / pap;
        for i in 0..n {
            x[i] += alpha * p[i];
            r[i] -= alpha * ap[i];
        }
        let rsnew = dot(&r, &r);
        if rsnew.sqrt() < abs_tol {
            return Ok(x);
        }
        let beta = rsnew / rsold;
        for i in 0..n {
            p[i] = r[i] + beta * p[i];
        }
        rsold = rsnew;
    }

    Err(CgFailure {
        iterations: max_iterations,
        residual: rsold.sqrt(),
    })
}

/// Subtract the mean of `v` from every entry, in place.
///
/// Used to fix the gauge of a 0-form potential before computing its
/// exterior derivative — the constant functions are always in the kernel of
/// `Δ_0`, so the solve is degenerate by exactly one dimension and the result
/// is only unique up to an additive constant.
pub(crate) fn subtract_mean_in_place<R>(v: &mut [R])
where
    R: RealField + FromPrimitive,
{
    if v.is_empty() {
        return;
    }
    let n = <R as FromPrimitive>::from_usize(v.len())
        .expect("v.len() is representable in every supported RealField");
    let sum: R = v.iter().copied().fold(R::zero(), |a, b| a + b);
    let mean = sum / n;
    for entry in v.iter_mut() {
        *entry -= mean;
    }
}

#[inline]
fn dot<R>(a: &[R], b: &[R]) -> R
where
    R: RealField,
{
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x * y)
        .fold(R::zero(), |acc, t| acc + t)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!((err.residual - dot(&b, &b).sqrt()).abs() < 1e-14);
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
    fn subtract_mean_in_place_zeros_a_constant_vector() {
        let mut v = vec![5.0_f64; 7];
        subtract_mean_in_place(&mut v);
        for &x in &v {
            assert!(x.abs() < 1e-15);
        }
    }

    #[test]
    fn subtract_mean_in_place_preserves_zero_mean_input() {
        let mut v = vec![-2.0_f64, -1.0, 0.0, 1.0, 2.0];
        let original = v.clone();
        subtract_mean_in_place(&mut v);
        for (a, b) in v.iter().zip(original.iter()) {
            assert!((a - b).abs() < 1e-15);
        }
    }

    #[test]
    fn subtract_mean_in_place_handles_empty_slice() {
        let mut v: Vec<f64> = vec![];
        subtract_mean_in_place(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn subtract_mean_in_place_subtracts_correct_mean_from_arbitrary_input() {
        let mut v = vec![1.0_f64, 2.0, 3.0, 4.0]; // mean = 2.5
        subtract_mean_in_place(&mut v);
        let expected = [-1.5, -0.5, 0.5, 1.5];
        for (a, e) in v.iter().zip(expected.iter()) {
            assert!((a - e).abs() < 1e-15);
        }
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
}
