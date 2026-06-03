/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Small dense linear solver for the BRCD estimator: Gaussian elimination with
//! **partial pivoting** (an LU solve), kept local to the `brcd` module so its
//! consumers — the logistic gate's Newton step and the ridge-Gaussian fit — do
//! not reach into another crate's internals.
//!
//! Partial pivoting (not Cholesky) is deliberate: the reference `_fit_ridge`
//! solves `XᵀX + λI` with `numpy.linalg.solve`, which is LAPACK `gesv` (LU with
//! partial pivoting). On well-scaled data a Cholesky agrees, but real-world
//! metric matrices span many orders of magnitude; a Cholesky that floors a
//! computed-non-positive pivot then corrupts the solve and the rankings drift
//! from the reference. Pivoting on the largest available element matches `numpy`
//! and stays accurate without any flooring.

use deep_causality_num::RealField;

/// Solves the dense linear system `A x = b` for `x`, in place, by Gaussian
/// elimination with partial pivoting.
///
/// `a` is a row-major `n × n` matrix (overwritten during elimination); `b` holds
/// the right-hand side on entry and the solution on return. The matrix is assumed
/// non-singular — the callers add ridge regularization (`+ λI`), which guarantees
/// it.
pub(crate) fn solve_linear<T: RealField>(a: &mut [T], b: &mut [T], n: usize) {
    // Forward elimination with partial pivoting.
    for col in 0..n {
        // Pick the row (≥ col) with the largest-magnitude entry in this column.
        let mut pivot_row = col;
        let mut best = a[col * n + col].abs();
        for row in (col + 1)..n {
            let mag = a[row * n + col].abs();
            if mag > best {
                best = mag;
                pivot_row = row;
            }
        }
        if pivot_row != col {
            for j in 0..n {
                a.swap(col * n + j, pivot_row * n + j);
            }
            b.swap(col, pivot_row);
        }

        let pivot = a[col * n + col];
        for row in (col + 1)..n {
            let factor = a[row * n + col] / pivot;
            for j in (col + 1)..n {
                let above = a[col * n + j];
                a[row * n + j] -= factor * above;
            }
            let b_col = b[col];
            b[row] -= factor * b_col;
        }
    }

    // Back substitution over the upper-triangular factor.
    for i in (0..n).rev() {
        let mut s = b[i];
        for j in (i + 1)..n {
            s -= a[i * n + j] * b[j];
        }
        b[i] = s / a[i * n + i];
    }
}

#[cfg(test)]
mod tests {
    use super::solve_linear;

    #[test]
    fn solves_a_known_2x2_system() {
        // [[4, 1], [1, 3]] x = [1, 2]  →  x = [1/11, 7/11].
        let mut a = vec![4.0_f64, 1.0, 1.0, 3.0];
        let mut b = vec![1.0_f64, 2.0];
        solve_linear(&mut a, &mut b, 2);
        assert!((b[0] - 1.0 / 11.0).abs() < 1e-12);
        assert!((b[1] - 7.0 / 11.0).abs() < 1e-12);
    }

    #[test]
    fn solves_identity_to_itself() {
        let mut a = vec![1.0_f64, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let mut b = vec![3.0_f64, -2.0, 5.0];
        solve_linear(&mut a, &mut b, 3);
        assert_eq!(b, vec![3.0, -2.0, 5.0]);
    }

    #[test]
    fn partial_pivoting_handles_a_zero_leading_pivot() {
        // [[0, 1], [1, 0]] x = [2, 3] needs a row swap; Cholesky would fail here.
        // 0·x0 + 1·x1 = 2 → x1 = 2; 1·x0 + 0·x1 = 3 → x0 = 3.
        let mut a = vec![0.0_f64, 1.0, 1.0, 0.0];
        let mut b = vec![2.0_f64, 3.0];
        solve_linear(&mut a, &mut b, 2);
        assert!((b[0] - 3.0).abs() < 1e-12);
        assert!((b[1] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn solves_an_extreme_scale_system() {
        // Diagonal spanning 12 orders of magnitude — the pathological case that
        // corrupts a pivot-flooring Cholesky. A x = b with A = diag(1e8, 1e-4):
        // x = [3/1e8, 5/1e-4] = [3e-8, 5e4].
        let mut a = vec![1e8_f64, 0.0, 0.0, 1e-4];
        let mut b = vec![3.0_f64, 5.0];
        solve_linear(&mut a, &mut b, 2);
        assert!((b[0] - 3e-8).abs() < 1e-20);
        assert!((b[1] - 5e4).abs() < 1e-6);
    }
}
