/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Euler, Scalar};
use deep_causality_haft::Arrow;
use std::ops::{Add, Mul};

impl<S, R, F> Arrow for Euler<S, R, F>
where
    S: Add<Output = S> + Mul<R, Output = S>,
    R: Scalar,
    F: Fn(&S) -> S,
{
    type In = S;
    type Out = S;

    #[inline]
    fn run(&self, state: S) -> S {
        let rate = (self.rate)(&state);
        state + rate * self.dt
    }
}
