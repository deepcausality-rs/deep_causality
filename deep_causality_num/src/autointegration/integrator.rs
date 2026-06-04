/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::RealField;
use core::ops::{Add, Mul};

/// A fixed-step numeric integrator for an ODE `y' = f(y)` over a module-valued state.
///
/// The state `S` must form a module over the scalar `R` — addition plus scalar
/// multiplication — which is satisfied by `f64`, `Complex`, `Dual`, `CausalTensor`,
/// and `CausalMultiVector`, so the same integrator marches a scalar oscillator, a
/// multivector orientation, or a tensor field with no type-specific code. The rate
/// field `f: Fn(&S) -> S` returns the state's time derivative.
///
/// Implementors differ only in accuracy ([`Euler`](crate::Euler) first-order,
/// [`Rk4`](crate::Rk4) fourth-order); swapping the implementor changes accuracy with
/// no change to the model.
pub trait Integrator {
    /// Advances `state` by one step of size `dt` under the rate field `rate`.
    fn step<S, R, F>(&self, state: &S, dt: R, rate: &F) -> S
    where
        S: Clone + Add<Output = S> + Mul<R, Output = S>,
        R: RealField,
        F: Fn(&S) -> S;

    /// Advances `initial` by `steps` steps of size `dt`, folding [`step`](Integrator::step).
    fn integrate<S, R, F>(&self, initial: S, dt: R, steps: usize, rate: &F) -> S
    where
        S: Clone + Add<Output = S> + Mul<R, Output = S>,
        R: RealField,
        F: Fn(&S) -> S,
    {
        let mut state = initial;
        for _ in 0..steps {
            state = self.step(&state, dt, rate);
        }
        state
    }
}
