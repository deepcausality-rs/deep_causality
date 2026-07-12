/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The 3-D `MetricProvider3d` seam — the three-dimensional sibling of
//! [`MetricProvider`](MetricProvider) (design D8). A compressible marcher written over
//! `M: MetricProvider3d<R>` consumes only "there is a `2^Lx × 2^Ly × 2^Lz` computational lattice, a way
//! to sample a field onto it, a physical gradient `(∂/∂x, ∂/∂y, ∂/∂z)`, and a Jacobian volume factor" —
//! so geometry is **data**, not a code path.
//!
//! [`CartesianIdentity3d`](CartesianIdentity3d) is the capture limit (any geometry, but a curved
//! shock costs `χ ~ √side`); `BodyFittedCoordinate3d` (the reentry forebody shell) is the fitted limit
//! (`χ ~ O(10)`, resolution-flat) — the mandatory rank lever measured in `studies/qtt_rank_3d`. Static
//! dispatch only (used as a generic bound; no `dyn`).

use crate::alias::physical_gradient_3_d::PhysicalGradient3d;
use crate::CfdScalar;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensorTrain;

/// A structured curvilinear coordinate over a `2^Lx × 2^Ly × 2^Lz` computational lattice, supplying the
/// pieces a 3-D compressible marcher needs: field sampling, the chain-rule physical gradient, and the
/// Jacobian volume factor — all carried as low-rank tensor trains.
pub trait MetricProvider3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// The computational lattice mode counts `(Lx, Ly, Lz)` (grid is `2^Lx × 2^Ly × 2^Lz`).
    fn dims(&self) -> (usize, usize, usize);

    /// Sample `f(ξ, η, ζ)` on the computational lattice (`ξ_i = i/Nx`, `η_j = j/Ny`, `ζ_k = k/Nz`) and
    /// QTT-encode it.
    ///
    /// # Errors
    /// Propagates sampling / codec errors.
    fn sample<F>(&self, f: F) -> Result<CausalTensorTrain<R>, PhysicsError>
    where
        F: Fn(R, R, R) -> R;

    /// The **physical** gradient `(∂u/∂x, ∂u/∂y, ∂u/∂z)` of a field `u` carried in this coordinate, via
    /// the chain rule and the low-rank metric.
    ///
    /// # Errors
    /// Propagates apply / Hadamard / rounding errors.
    fn physical_gradient(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<PhysicalGradient3d<R>, PhysicsError>;

    /// The Jacobian determinant `|J|` (the conservative volume factor) as a low-rank tensor train.
    fn jacobian(&self) -> &CausalTensorTrain<R>;
}
