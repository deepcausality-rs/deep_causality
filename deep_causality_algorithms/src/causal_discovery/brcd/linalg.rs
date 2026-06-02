/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Small dense SPD linear algebra for the BRCD estimator: an in-place Cholesky
//! factorization and solve. Mirrors the proven routine in
//! `deep_causality_tensor`'s stats extension, kept local to the `brcd` module so
//! neither consumer — the logistic gate's Newton step nor the ridge-Gaussian
//! fit — has to reach into another crate's internals. The systems solved here
//! are tiny and dense (a handful of parameters), so a direct Cholesky is both
//! simpler and more accurate than an iterative solver.

use deep_causality_num::RealField;

/// Solves the symmetric-positive-definite system `A x = b` for `x`, in place.
///
/// `a` is a row-major `k × k` SPD matrix; it is overwritten with its lower
/// Cholesky factor. `b` holds the right-hand side on entry and the solution on
/// return. A non-positive pivot is floored to `T::epsilon()` so the
/// factorization always completes (the callers add ridge regularization, which
/// normally keeps every pivot positive).
pub(crate) fn solve_spd<T: RealField>(a: &mut [T], b: &mut [T], k: usize) {
    cholesky_in_place(a, k);
    cholesky_solve_in_place(a, b, k);
}

/// In-place Cholesky: overwrites the row-major `k × k` SPD matrix `a` with its
/// lower factor `L`. A non-positive pivot is floored to `T::epsilon()`.
fn cholesky_in_place<T: RealField>(a: &mut [T], k: usize) {
    for j in 0..k {
        // Diagonal: L[j,j] = sqrt(a[j,j] − Σ_{p<j} L[j,p]²).
        let mut diag = a[j * k + j];
        for p in 0..j {
            let l_jp = a[j * k + p];
            diag -= l_jp * l_jp;
        }
        let pivot = if diag > T::zero() { diag } else { T::epsilon() };
        let l_jj = pivot.sqrt();
        a[j * k + j] = l_jj;

        // Below-diagonal: L[i,j] = (a[i,j] − Σ_{p<j} L[i,p] L[j,p]) / L[j,j].
        for i in (j + 1)..k {
            let mut s = a[i * k + j];
            for p in 0..j {
                s -= a[i * k + p] * a[j * k + p];
            }
            a[i * k + j] = s / l_jj;
        }
    }
}

/// Solves `(L Lᵀ) x = b` in place, given the lower Cholesky factor `l`
/// (row-major `k × k`). On entry `b` holds the right-hand side; on return it
/// holds the solution `x`.
fn cholesky_solve_in_place<T: RealField>(l: &[T], b: &mut [T], k: usize) {
    // Forward substitution: L y = b.
    for i in 0..k {
        let mut s = b[i];
        for p in 0..i {
            s -= l[i * k + p] * b[p];
        }
        b[i] = s / l[i * k + i];
    }
    // Back substitution: Lᵀ x = y.
    for i in (0..k).rev() {
        let mut s = b[i];
        for p in (i + 1)..k {
            s -= l[p * k + i] * b[p];
        }
        b[i] = s / l[i * k + i];
    }
}

#[cfg(test)]
mod tests {
    use super::solve_spd;

    #[test]
    fn solves_a_known_2x2_system() {
        // [[4, 1], [1, 3]] x = [1, 2]  →  x = [1/11, 7/11].
        let mut a = vec![4.0_f64, 1.0, 1.0, 3.0];
        let mut b = vec![1.0_f64, 2.0];
        solve_spd(&mut a, &mut b, 2);
        assert!((b[0] - 1.0 / 11.0).abs() < 1e-12);
        assert!((b[1] - 7.0 / 11.0).abs() < 1e-12);
    }

    #[test]
    fn solves_identity_to_itself() {
        let mut a = vec![1.0_f64, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let mut b = vec![3.0_f64, -2.0, 5.0];
        solve_spd(&mut a, &mut b, 3);
        assert_eq!(b, vec![3.0, -2.0, 5.0]);
    }
}
