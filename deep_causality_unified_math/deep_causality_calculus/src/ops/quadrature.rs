/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Scalar;

/// Definite integral `∫ₐᵇ f` by composite Simpson's rule over `n` panels — a fold over a
/// closed-form integrand.
///
/// This is a free function, not a type extension: its subject is a *closure* (`Fn(R) -> R`),
/// and blanket-extending `Fn` is inference-fragile (the closure type cannot resolve before
/// method lookup). Differentiation, by contrast, acts on a *named* model and so reaches the
/// user as the [`DifferentiateExt`](crate::DifferentiateExt) methods.
///
/// `n` is normalised to an even value `≥ 2`. The rule is exact through cubics and converges for
/// smooth integrands. Being generic over [`Scalar`], it runs over `Dual` too: seed a parameter
/// as `Dual::variable` and the `ε` part of the result is `d/dθ ∫ f(x, θ) dx` — the Leibniz rule,
/// which is the naturality of the tangent functor through this fold.
#[inline]
pub fn quadrature<R, F>(f: F, a: R, b: R, n: usize) -> R
where
    R: Scalar,
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

    let n_r = R::from_usize(n).expect("scalar from panel count");
    let two = R::from_u8(2).expect("scalar from 2");
    let three = R::from_u8(3).expect("scalar from 3");
    let four = R::from_u8(4).expect("scalar from 4");
    let h = (b - a) / n_r;

    // (h/3)·[f₀ + 4f₁ + 2f₂ + … + 4f_{n-1} + fₙ]
    let mut sum = f(a) + f(b);
    let mut x = a + h;
    for i in 1..n {
        let weight = if i % 2 == 1 { four } else { two };
        sum += weight * f(x);
        x += h;
    }
    sum * h / three
}
