/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Dual, Real};
use core::ops::Div;

/// Forward-mode Jacobian of a vector-valued map `f: Rᴺ → Rᴹ`.
///
/// Returns `[[R; N]; M]`: row `k` is the gradient of output component `k`, so
/// `jac[k][i] = ∂f_k/∂x_i`. Computed in `N` forward passes (one seeded input each),
/// then assembled into output-major rows. Allocation-free.
///
/// ```
/// use deep_causality_num::{Dual, jacobian};
///
/// // f(x, y) = [x·y, x + y] → J = [[y, x], [1, 1]] at (2, 3)
/// let j = jacobian::<f64, _, 2, 2>(
///     |p| [p[0] * p[1], p[0] + p[1]],
///     &[2.0, 3.0],
/// );
/// assert_eq!(j, [[3.0, 2.0], [1.0, 1.0]]);
/// # let _ = Dual::variable(1.0_f64);
/// ```
#[inline]
pub fn jacobian<R, F, const N: usize, const M: usize>(f: F, x: &[R; N]) -> [[R; N]; M]
where
    R: Real + Div<Output = R>,
    F: Fn(&[Dual<R>; N]) -> [Dual<R>; M],
{
    // Column `i`: the partial of every output with respect to input `i`.
    let cols: [[R; M]; N] = core::array::from_fn(|i| {
        let seed: [Dual<R>; N] = core::array::from_fn(|j| {
            if j == i {
                Dual::variable(x[j])
            } else {
                Dual::constant(x[j])
            }
        });
        let out = f(&seed);
        core::array::from_fn(|k| out[k].derivative())
    });
    // Transpose into output-major rows: jac[k][i] = ∂f_k/∂x_i.
    core::array::from_fn(|k| core::array::from_fn(|i| cols[i][k]))
}
