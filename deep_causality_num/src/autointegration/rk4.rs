/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::RealField;
use crate::autointegration::integrator::Integrator;
use core::ops::{Add, Mul};

/// Classical fourth-order Runge–Kutta:
/// `yₙ₊₁ = yₙ + (k₁ + 2k₂ + 2k₃ + k₄)·dt/6`, with the standard half-step stages.
///
/// A drop-in replacement for [`Euler`](crate::Euler) — same model, far higher accuracy.
///
/// ```
/// use deep_causality_num::{Integrator, Rk4};
///
/// // y' = y, y(0) = 1, integrated to t = 1 → very close to e even with few steps.
/// let y = Rk4.integrate(1.0_f64, 0.1, 10, &|y: &f64| *y);
/// assert!((y - std::f64::consts::E).abs() < 1e-4);
/// ```
pub struct Rk4;

impl Integrator for Rk4 {
    #[inline]
    fn step<S, R, F>(&self, state: &S, dt: R, rate: &F) -> S
    where
        S: Clone + Add<Output = S> + Mul<R, Output = S>,
        R: RealField,
        F: Fn(&S) -> S,
    {
        let one = R::one();
        let two = one + one;
        let six = two + two + two;
        let dt_half = dt / two;
        let dt_sixth = dt / six;

        let k1 = rate(state);
        let k2 = rate(&(state.clone() + k1.clone() * dt_half));
        let k3 = rate(&(state.clone() + k2.clone() * dt_half));
        let k4 = rate(&(state.clone() + k3.clone() * dt));

        // k1 + 2·k2 + 2·k3 + k4
        let weighted = k1 + k2 * two + k3 * two + k4;
        state.clone() + weighted * dt_sixth
    }
}
