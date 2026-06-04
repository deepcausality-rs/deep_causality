/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Real;
use core::ops::Div;

/// Definite integral `∫ₐᵇ f` by composite Simpson's rule over `n` panels.
///
/// `n` is forced to an even value `≥ 2` (Simpson needs an even panel count). The rule
/// is exact through cubics and converges for smooth integrands. Bounded on
/// `R: Real + Div`, so it runs over `f32`/`f64`/`Float106` and — being generic over
/// [`Real`](Real) — over `Dual` too, giving the Leibniz bridge: evaluate with a
/// parameter seeded as `Dual::variable` and the `ε` part is the integral's derivative
/// with respect to that parameter.
///
/// ```
/// use deep_causality_num::quadrature;
///
/// // ∫₀¹ x³ dx = 1/4 (Simpson is exact on cubics)
/// let i = quadrature(|x: f64| x * x * x, 0.0, 1.0, 8);
/// assert!((i - 0.25).abs() < 1e-12);
/// ```
#[inline]
pub fn quadrature<R, F>(f: F, a: R, b: R, n: usize) -> R
where
    R: Real + Div<Output = R>,
    F: Fn(R) -> R,
{
    // Simpson requires an even panel count of at least 2.
    let n = if n < 2 {
        2
    } else if n % 2 == 1 {
        n + 1
    } else {
        n
    };

    let one = R::one();
    let two = one + one;
    let three = two + one;
    let four = two + two;

    // `Real` has no integer conversion; build the panel count by accumulation (O(n),
    // the same order as the n function evaluations below).
    let mut n_r = R::zero();
    for _ in 0..n {
        n_r += one;
    }
    let h = (b - a) / n_r;

    // Composite Simpson: (h/3)·[f₀ + 4f₁ + 2f₂ + … + 4f_{n-1} + fₙ].
    let mut sum = f(a) + f(b);
    let mut x = a + h;
    for i in 1..n {
        let weight = if i % 2 == 1 { four } else { two };
        sum += weight * f(x);
        x += h;
    }
    sum * h / three
}
