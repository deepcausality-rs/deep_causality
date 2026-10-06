/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::RealField;

/// The inertia `(positive, negative, zero)` of a real symmetric 4×4 matrix: how many of its
/// eigenvalues are positive, negative and zero. The caller guarantees symmetry.
///
/// Computed by symmetric block elimination. Each step is a congruence `Sᵀ A S` with `S`
/// nonsingular, which leaves the inertia unchanged by Sylvester's law of inertia (Horn & Johnson
/// 2013, Theorem 4.5.8), and splits off one pivot block whose inertia is read directly:
/// - a diagonal entry at least half the largest remaining entry in magnitude: one eigenvalue of
///   its sign;
/// - otherwise the 2×2 block at the largest entry, which is off the diagonal. Both of its diagonal
///   entries are below half that entry in magnitude, so its determinant is negative and it has one
///   positive and one negative eigenvalue.
///
/// Every update is an entry times a bounded ratio, never a product of two entries: a 1×1 pivot
/// `a_pp` subtracts `a_jp · (a_pk / a_pp)` from `a_jk`, with `|a_pk / a_pp| ≤ 2`; a 2×2 pivot at
/// `a_pq = b` subtracts `a_jp · u_p + a_jq · u_q`, with `|u_p|, |u_q| < 2` because `|a_pp / b|` and
/// `|a_qq / b|` are below 1/2 and `|a_pk / b|`, `|a_qk / b|` at most 1. A step therefore changes no
/// entry by more than four times the largest remaining entry.
///
/// When every remaining entry is zero, the remaining eigenvalues are zero.
///
/// Horn, R. A., & Johnson, C. R. (2013). *Matrix Analysis*, 2nd ed. Cambridge University Press.
pub(crate) fn inertia<R: RealField>(m: &[[R; 4]; 4]) -> (usize, usize, usize) {
    let mut a = *m;
    let mut active = [true; 4];
    let (mut positive, mut negative) = (0, 0);

    loop {
        let rows = (0..4).filter(|&i| active[i]);
        // The largest remaining entry, and the largest remaining diagonal entry.
        let (mut big, mut big_at) = (R::zero(), (0, 0));
        let (mut diag, mut diag_at) = (R::zero(), 0);
        for i in rows.clone() {
            for j in (0..4).filter(|&j| active[j]) {
                if a[i][j].abs() > big {
                    big = a[i][j].abs();
                    big_at = (i, j);
                }
            }
            if a[i][i].abs() > diag {
                diag = a[i][i].abs();
                diag_at = i;
            }
        }
        let remaining = rows.count();
        if remaining == 0 || big == R::zero() {
            return (positive, negative, remaining);
        }

        if diag + diag >= big {
            let p = diag_at;
            if a[p][p] > R::zero() {
                positive += 1;
            } else {
                negative += 1;
            }
            active[p] = false;
            for j in (0..4).filter(|&j| active[j]) {
                for k in (0..4).filter(|&k| active[k]) {
                    a[j][k] -= a[j][p] * (a[p][k] / a[p][p]);
                }
            }
        } else {
            let (p, q) = big_at;
            let b = a[p][q];
            let (alpha, beta) = (a[p][p] / b, a[q][q] / b);
            let d = alpha * beta - R::one();
            positive += 1;
            negative += 1;
            active[p] = false;
            active[q] = false;
            for j in (0..4).filter(|&j| active[j]) {
                for k in (0..4).filter(|&k| active[k]) {
                    // a_jk − [a_jp a_jq] B⁻¹ [a_pk a_qk]ᵀ for B = [[a_pp, b], [b, a_qq]].
                    // B⁻¹ = [[a_qq, −b], [−b, a_pp]] / (a_pp a_qq − b²). Dividing numerator and
                    // denominator by b² leaves ratios to b only: B⁻¹ [a_pk a_qk]ᵀ = [u_p u_q]ᵀ.
                    let (r_p, r_q) = (a[p][k] / b, a[q][k] / b);
                    let u_p = (beta * r_p - r_q) / d;
                    let u_q = (alpha * r_q - r_p) / d;
                    a[j][k] -= a[j][p] * u_p + a[j][q] * u_q;
                }
            }
        }
    }
}
