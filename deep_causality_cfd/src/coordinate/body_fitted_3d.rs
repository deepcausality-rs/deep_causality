/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `BodyFittedCoordinate3d` — the fitted-limit 3-D [`MetricProvider3d`] (design D1, three dimensions):
//! a **spherical-shell** coordinate for a reentry forebody, the 3-D sibling of the 2-D polar
//! [`BodyFittedCoordinate`](super::BodyFittedCoordinate).
//!
//! ```text
//! x = r·sinθ·cosφ,  y = r·sinθ·sinφ,  z = r·cosθ
//! φ(ξ) = φ0 + ξ·Δφ   (azimuth, ξ-lattice; Δφ = 2π ⇒ periodic ξ)
//! θ(η) = θ0 + η·Δθ   (polar,   η-lattice)
//! r(ζ) = r0 + ζ·Δr   (radial,  ζ-lattice; the shock-normal / stand-off direction)
//! ```
//!
//! A curved bow shock standing off the nose at constant physical radius `R` is the surface `ζ = const`
//! in this coordinate — a step in `ζ`, constant in `(ξ, η)` — so its QTT bond dimension is `O(10)` and
//! resolution-independent, the measured precondition for the 3-D compressible marcher
//! (`studies/qtt_rank_3d`), versus `χ ~ √side` for the same shock captured on a Cartesian grid.
//!
//! The inverse-Jacobian metric (the nine `∂(ξ,η,ζ)/∂(x,y,z)` components) is computed from the geometry —
//! no hardcoded numbers — and is smooth away from the poles, so it is carried as low-rank tensor trains.
//! The shell is restricted to `0 < θ0` and `θ0 + Δθ < π` to stay off the spherical-coordinate poles
//! (the `1/sinθ` singularity of the azimuthal metric). Interior gradients are correct to scheme order;
//! the periodic radial/polar boundary stencils are the same Stage-2 refinement noted for the 2-D chart.

use super::sample_grid_3d;
use crate::CfdScalar;
use crate::alias::physical_gradient_3_d::PhysicalGradient3d;
use crate::tensor_bridge::{gradient_x_3d, gradient_y_3d, gradient_z_3d, quantize_3d};
use crate::traits::MetricProvider3d;
use deep_causality_algebra::{ConjugateScalar, RealField};
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator, Truncation,
};

/// A spherical-shell body-fitted coordinate over a `2^Lx × 2^Ly × 2^Lz` lattice.
pub struct BodyFittedCoordinate3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lx: usize,
    ly: usize,
    lz: usize,
    r0: R,
    dr: R,
    theta0: R,
    dtheta: R,
    phi0: R,
    dphi: R,
    g_xi: CausalTensorTrainOperator<R>,
    g_eta: CausalTensorTrainOperator<R>,
    g_zeta: CausalTensorTrainOperator<R>,
    // Inverse-Jacobian metric components (∂ξ/∂z ≡ 0 for this chart, so it is omitted).
    dxi_dx: CausalTensorTrain<R>,
    dxi_dy: CausalTensorTrain<R>,
    deta_dx: CausalTensorTrain<R>,
    deta_dy: CausalTensorTrain<R>,
    deta_dz: CausalTensorTrain<R>,
    dzeta_dx: CausalTensorTrain<R>,
    dzeta_dy: CausalTensorTrain<R>,
    dzeta_dz: CausalTensorTrain<R>,
    jacobian: CausalTensorTrain<R>,
    trunc: Truncation<R>,
}

impl<R> BodyFittedCoordinate3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R> + RealField,
{
    /// Build the shell from the radial range `[r0, r0+dr]`, polar range `[θ0, θ0+dθ]`, and azimuthal
    /// range `[φ0, φ0+dφ]`, on a `2^lx × 2^ly × 2^lz` lattice. Use `dφ = 2π` for a full revolution.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if `r0`/`dr`/`dθ`/`dφ` are non-positive, or the polar
    /// range is not strictly inside `(0, π)` (the pole singularity); codec/operator failures.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lx: usize,
        ly: usize,
        lz: usize,
        r0: R,
        dr: R,
        theta0: R,
        dtheta: R,
        phi0: R,
        dphi: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        if r0 <= R::zero() || dr <= R::zero() || dtheta <= R::zero() || dphi <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "BodyFittedCoordinate3d requires r0 > 0, dr > 0, dθ > 0, dφ > 0".into(),
            ));
        }
        if theta0 <= R::zero() || theta0 + dtheta >= R::pi() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "polar range must lie strictly inside (0, π) to avoid the spherical poles".into(),
            ));
        }
        let lift = |n: usize| {
            R::from_usize(n)
                .ok_or_else(|| PhysicsError::NumericalInstability("from_usize failed".into()))
        };
        let dxi = R::one() / lift(1usize << lx)?;
        let deta = R::one() / lift(1usize << ly)?;
        let dzeta = R::one() / lift(1usize << lz)?;
        let g_xi = gradient_x_3d::<R>(lx, ly, lz, dxi, &trunc)?;
        let g_eta = gradient_y_3d::<R>(lx, ly, lz, deta, &trunc)?;
        let g_zeta = gradient_z_3d::<R>(lx, ly, lz, dzeta, &trunc)?;

        let phi_at = |xi: R| phi0 + xi * dphi;
        let theta_at = |eta: R| theta0 + eta * dtheta;
        let radius_at = |zeta: R| r0 + zeta * dr;

        // ∇r = r̂, ∇θ = θ̂/r, ∇φ = φ̂/(r sinθ), scaled by 1/Δ for the computational coordinates.
        let dxi_dx = encode(lx, ly, lz, &trunc, |xi, eta, zeta| {
            -phi_at(xi).sin() / (radius_at(zeta) * theta_at(eta).sin() * dphi)
        })?;
        let dxi_dy = encode(lx, ly, lz, &trunc, |xi, eta, zeta| {
            phi_at(xi).cos() / (radius_at(zeta) * theta_at(eta).sin() * dphi)
        })?;
        let deta_dx = encode(lx, ly, lz, &trunc, |xi, eta, zeta| {
            theta_at(eta).cos() * phi_at(xi).cos() / (radius_at(zeta) * dtheta)
        })?;
        let deta_dy = encode(lx, ly, lz, &trunc, |xi, eta, zeta| {
            theta_at(eta).cos() * phi_at(xi).sin() / (radius_at(zeta) * dtheta)
        })?;
        let deta_dz = encode(lx, ly, lz, &trunc, |_xi, eta, zeta| {
            -theta_at(eta).sin() / (radius_at(zeta) * dtheta)
        })?;
        let dzeta_dx = encode(lx, ly, lz, &trunc, |xi, eta, _zeta| {
            theta_at(eta).sin() * phi_at(xi).cos() / dr
        })?;
        let dzeta_dy = encode(lx, ly, lz, &trunc, |xi, eta, _zeta| {
            theta_at(eta).sin() * phi_at(xi).sin() / dr
        })?;
        let dzeta_dz = encode(lx, ly, lz, &trunc, |_xi, eta, _zeta| {
            theta_at(eta).cos() / dr
        })?;
        // |J| = r²·sinθ·Δr·Δθ·Δφ (the spherical volume element in computational coordinates).
        let jacobian = encode(lx, ly, lz, &trunc, |_xi, eta, zeta| {
            let r = radius_at(zeta);
            r * r * theta_at(eta).sin() * dr * dtheta * dphi
        })?;

        Ok(Self {
            lx,
            ly,
            lz,
            r0,
            dr,
            theta0,
            dtheta,
            phi0,
            dphi,
            g_xi,
            g_eta,
            g_zeta,
            dxi_dx,
            dxi_dy,
            deta_dx,
            deta_dy,
            deta_dz,
            dzeta_dx,
            dzeta_dy,
            dzeta_dz,
            jacobian,
            trunc,
        })
    }

    /// The inner radius `r0` of the radial (ζ) shell.
    pub fn r0(&self) -> R {
        self.r0
    }
    /// The radial span `Δr`.
    pub fn dr(&self) -> R {
        self.dr
    }
    /// The polar start `θ0`.
    pub fn theta0(&self) -> R {
        self.theta0
    }
    /// The polar span `Δθ`.
    pub fn dtheta(&self) -> R {
        self.dtheta
    }
    /// The azimuth start `φ0`.
    pub fn phi0(&self) -> R {
        self.phi0
    }
    /// The azimuth span `Δφ`.
    pub fn dphi(&self) -> R {
        self.dphi
    }

    /// The physical position `(x, y, z)` at computational `(ξ, η, ζ)`.
    pub fn position(&self, xi: R, eta: R, zeta: R) -> (R, R, R) {
        let phi = self.phi0 + xi * self.dphi;
        let theta = self.theta0 + eta * self.dtheta;
        let r = self.r0 + zeta * self.dr;
        (
            r * theta.sin() * phi.cos(),
            r * theta.sin() * phi.sin(),
            r * theta.cos(),
        )
    }
}

impl<R> MetricProvider3d<R> for BodyFittedCoordinate3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R> + RealField,
{
    fn dims(&self) -> (usize, usize, usize) {
        (self.lx, self.ly, self.lz)
    }

    fn sample<F>(&self, f: F) -> Result<CausalTensorTrain<R>, PhysicsError>
    where
        F: Fn(R, R, R) -> R,
    {
        quantize_3d(&sample_grid_3d(self.lx, self.ly, self.lz, f)?, &self.trunc)
    }

    fn physical_gradient(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<PhysicalGradient3d<R>, PhysicsError> {
        let t = &self.trunc;
        let du_dxi = self.g_xi.apply(u, t)?;
        let du_deta = self.g_eta.apply(u, t)?;
        let du_dzeta = self.g_zeta.apply(u, t)?;

        // ∂u/∂x = (∂ξ/∂x)uξ + (∂η/∂x)uη + (∂ζ/∂x)uζ, and likewise for y; ∂ξ/∂z ≡ 0 drops the ξ term in z.
        let du_dx = self
            .dxi_dx
            .hadamard_rounded(&du_dxi, t)?
            .add(&self.deta_dx.hadamard_rounded(&du_deta, t)?)?
            .add(&self.dzeta_dx.hadamard_rounded(&du_dzeta, t)?)?
            .round(t)?;
        let du_dy = self
            .dxi_dy
            .hadamard_rounded(&du_dxi, t)?
            .add(&self.deta_dy.hadamard_rounded(&du_deta, t)?)?
            .add(&self.dzeta_dy.hadamard_rounded(&du_dzeta, t)?)?
            .round(t)?;
        let du_dz = self
            .deta_dz
            .hadamard_rounded(&du_deta, t)?
            .add(&self.dzeta_dz.hadamard_rounded(&du_dzeta, t)?)?
            .round(t)?;
        Ok((du_dx, du_dy, du_dz))
    }

    fn jacobian(&self) -> &CausalTensorTrain<R> {
        &self.jacobian
    }
}

/// Sample a metric component `f(ξ, η, ζ)` over the lattice and QTT-encode it (a static-dispatch helper
/// so each metric train is one readable line without a `dyn` closure).
fn encode<R, F>(
    lx: usize,
    ly: usize,
    lz: usize,
    trunc: &Truncation<R>,
    f: F,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    F: Fn(R, R, R) -> R,
{
    quantize_3d(&sample_grid_3d(lx, ly, lz, f)?, trunc)
}
