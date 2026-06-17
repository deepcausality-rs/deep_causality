/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;
use core::ops::{Add, Mul};
use deep_causality_physics::PhysicsError;

/// A Navier–Stokes regime expressed as a **field-level marching rate**, abstracted
/// above both the DEC-native rate and the pointwise regime evaluators (the latter
/// realized by sampling the state and calling the classical kernels for MMS /
/// analytic verification).
///
/// The manifold borrow lives in the *implementor* (the theory is materialized bound
/// to the manifold at run time), not in this trait, so the trait stays `'m`-free.
///
/// CPU parallelism rides the `R: CfdScalar` bound (which carries `MaybeParallel`): the
/// topology operator loops inside `rate` fan out over the scalar under `--features
/// parallel`. The theory and its state are *not* required to be `Send + Sync` — the
/// rate uses single-threaded `RefCell` workspaces, and the fan-out is intra-operator.
pub trait FluidTheory<R: CfdScalar> {
    /// The marching state this regime advances. Its algebra bounds are exactly what
    /// `deep_causality_calculus::Rk4` needs, so any theory drops into the integrator.
    type State: Clone + Add<Output = Self::State> + Mul<R, Output = Self::State>;

    /// The ambient this regime reads each step. The incompressible regime uses
    /// [`crate::Ambient`]; compressible/thermal regimes supply their own type,
    /// extending the ambient without changing this trait.
    type Ambient;

    /// The time-derivative `∂state/∂t` under the given ambient. Fallible: the
    /// projection CG / pressure solve can fail. The solver adapts this to `Rk4`'s
    /// infallible `Fn(&S) -> S` via a deferred-error cell, so the rate stays pure
    /// across the four stages.
    fn rate(
        &self,
        state: &Self::State,
        ambient: &Self::Ambient,
    ) -> Result<Self::State, PhysicsError>;
}
