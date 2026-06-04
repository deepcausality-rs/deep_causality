/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::RealField;
use crate::autointegration::integrator::Integrator;
use core::ops::{Add, Mul};

/// Forward (explicit) Euler — first-order: `yₙ₊₁ = yₙ + f(yₙ)·dt`.
///
/// ```
/// use deep_causality_num::{Euler, Integrator};
///
/// // y' = y, y(0) = 1, integrated to t = 1 → approaches e.
/// let y = Euler.integrate(1.0_f64, 0.001, 1000, &|y: &f64| *y);
/// assert!((y - std::f64::consts::E).abs() < 1e-2);
/// ```
pub struct Euler;

impl Integrator for Euler {
    #[inline]
    fn step<S, R, F>(&self, state: &S, dt: R, rate: &F) -> S
    where
        S: Clone + Add<Output = S> + Mul<R, Output = S>,
        R: RealField,
        F: Fn(&S) -> S,
    {
        state.clone() + rate(state) * dt
    }
}
