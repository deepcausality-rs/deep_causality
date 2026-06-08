/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Rk4, Scalar};
use deep_causality_haft::Arrow;
use std::ops::{Add, Mul};

impl<S, R, F> Arrow for Rk4<S, R, F>
where
    S: Clone + Add<Output = S> + Mul<R, Output = S>,
    R: Scalar,
    F: Fn(&S) -> S,
{
    type In = S;
    type Out = S;

    #[inline]
    fn run(&self, state: S) -> S {
        // Small integer scalars; the conversions are total for any float/dual scalar.
        let two = R::from_u8(2).expect("scalar from 2");
        let six = R::from_u8(6).expect("scalar from 6");
        let dt_half = self.dt / two;
        let dt_sixth = self.dt / six;

        let k1 = (self.rate)(&state);
        let k2 = (self.rate)(&(state.clone() + k1.clone() * dt_half));
        let k3 = (self.rate)(&(state.clone() + k2.clone() * dt_half));
        let k4 = (self.rate)(&(state.clone() + k3.clone() * self.dt));

        // k1 + 2·k2 + 2·k3 + k4
        let weighted = k1 + k2 * two + k3 * two + k4;
        state + weighted * dt_sixth
    }
}
