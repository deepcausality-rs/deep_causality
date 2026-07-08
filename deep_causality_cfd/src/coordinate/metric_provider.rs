/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `MetricProvider` seam (design D8): a static-dispatch trait that lets the compressible marcher run
//! generically over *any* structured coordinate, so geometry is **data**, not a code path.
//!
//! A marcher written over `M: MetricProvider<R>` consumes only "there is a computational lattice, a way
//! to sample a field onto it, a physical gradient, and a Jacobian volume factor". `CartesianIdentity` is
//! the capture limit (any geometry, high rank); [`BodyFittedCoordinate`](super::BodyFittedCoordinate) is
//! the fitted limit (this geometry, `O(10)` rank). Body-fittedness becomes a choice of impl, recovering
//! generality at zero asymptotic rank cost — the result the `qtt_blend_metric` study measured.
//!
//! The continuous body-fit blend parameter `λ` (a `BlendedMap` over two providers) is a follow-on: a
//! correct blended metric needs the *forward* Jacobians of both charts, which the present impls do not
//! expose. The blend itself is already validated numerically (`studies/qtt_blend_metric`).

use crate::types::CfdScalar;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensorTrain;

/// A structured curvilinear coordinate over a `2^Lx × 2^Ly` computational lattice, supplying the pieces a
/// compressible marcher needs: field sampling, the chain-rule physical gradient, and the Jacobian volume
/// factor — all carried as low-rank tensor trains. Static dispatch only (used as a generic bound).
pub trait MetricProvider<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// The computational lattice mode counts `(Lx, Ly)` (grid is `2^Lx × 2^Ly`).
    fn dims(&self) -> (usize, usize);

    /// Sample `f(ξ, η)` on the computational lattice (`ξ_i = i/Nx`, `η_j = j/Ny`) and QTT-encode it.
    ///
    /// # Errors
    /// Propagates sampling / codec errors.
    fn sample<F>(&self, f: F) -> Result<CausalTensorTrain<R>, PhysicsError>
    where
        F: Fn(R, R) -> R;

    /// The **physical** gradient `(∂u/∂x, ∂u/∂y)` of a field `u` carried in this coordinate, via the
    /// chain rule and the low-rank metric.
    ///
    /// # Errors
    /// Propagates apply / Hadamard / rounding errors.
    fn physical_gradient(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError>;

    /// The Jacobian determinant `|J|` (the conservative volume factor) as a low-rank tensor train.
    fn jacobian(&self) -> &CausalTensorTrain<R>;
}
