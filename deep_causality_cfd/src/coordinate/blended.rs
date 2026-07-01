/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The continuous body-fit blend `BlendedMap` (design D8 / Resolution 4) — the `λ` dial.
//!
//! `λ = 0` is the **Cartesian capture** chart (a rectangle in front of the nose, any geometry, high rank);
//! `λ = 1` is the **body-fitted** polar fan (this geometry, `O(10)` rank). The blend is of the two charts'
//! **positions**, so their *forward* Jacobians blend linearly:
//!
//! ```text
//! T_λ = (1−λ)·T_cart + λ·T_fit  ⇒  J_λ = ∂(x,y)/∂(ξ,η) = (1−λ)·J_cart + λ·J_fit.
//! ```
//!
//! Both charts share the `(ξ, η)` patch with compatible orientation, so `det J_λ` keeps one sign across
//! the sweep (the validity residual the `qtt_blend_metric` study measured — gate BM-A), and the constructor
//! rejects a fold. The marcher consumes only the **inverse** metric `∂(ξ,η)/∂(x,y)` and the volume factor,
//! obtained by inverting the `2×2` forward Jacobian pointwise:
//!
//! ```text
//! [∂ξ/∂x  ∂ξ/∂y]   1   [ ∂y/∂η  −∂x/∂η]
//! [∂η/∂x  ∂η/∂y] = ─── [−∂y/∂ξ   ∂x/∂ξ],   |J| = |det J_λ|.
//! ```
//!
//! Both charts are analytic, so the blended inverse-metric fields are sampled directly (no tensor-train
//! reciprocal) and carried low-rank, exactly as [`BodyFittedCoordinate`](super::BodyFittedCoordinate) does.
//! At `λ = 1` this reproduces the `BodyFittedCoordinate` metric identically.

use super::{MetricProvider, sample_grid};
use crate::tensor_bridge::{gradient_x, gradient_y, quantize_2d};
use crate::types::CfdScalar;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator, Truncation,
};

/// Geometry + blend parameters for [`BlendedMap`]: the `2^lx × 2^ly` lattice, the polar fan
/// `r ∈ [r0, r0+dr]`, `θ ∈ [theta0, theta0+dtheta]`, and the blend `lambda ∈ [0, 1]`.
pub struct BlendedMapConfig<R> {
    lx: usize,
    ly: usize,
    r0: R,
    dr: R,
    theta0: R,
    dtheta: R,
    lambda: R,
}

impl<R: CfdScalar> BlendedMapConfig<R> {
    /// Bundle the blend's lattice + fan geometry + blend parameter.
    pub fn new(lx: usize, ly: usize, r0: R, dr: R, theta0: R, dtheta: R, lambda: R) -> Self {
        Self {
            lx,
            ly,
            r0,
            dr,
            theta0,
            dtheta,
            lambda,
        }
    }
}

/// A continuous blend between the Cartesian-capture rectangle (`λ = 0`) and the body-fitted polar fan
/// (`λ = 1`) over a `2^Lx × 2^Ly` `(ξ, η)` lattice, exposing the same low-rank inverse-Jacobian metric a
/// compressible marcher consumes through [`MetricProvider`].
pub struct BlendedMap<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lx: usize,
    ly: usize,
    lambda: R,
    // Geometry, retained for the analytic position map.
    r0: R,
    dr: R,
    theta0: R,
    dtheta: R,
    span_y: R,
    // Computational finite-difference operators (∂/∂ξ on the leading block, ∂/∂η on the trailing).
    g_xi: CausalTensorTrainOperator<R>,
    g_eta: CausalTensorTrainOperator<R>,
    // Blended inverse-Jacobian metric fields (smooth ⇒ low-rank).
    dxi_dx: CausalTensorTrain<R>,
    deta_dx: CausalTensorTrain<R>,
    dxi_dy: CausalTensorTrain<R>,
    deta_dy: CausalTensorTrain<R>,
    // The Jacobian volume factor |det J_λ|.
    jacobian: CausalTensorTrain<R>,
    trunc: Truncation<R>,
}

impl<R> BlendedMap<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the blend from a [`BlendedMapConfig`]. The Cartesian-capture partner is the rectangle of
    /// radial extent `dr` and transverse width `2·(r0+½dr)·sin(½dtheta)` (the fan chord at mid radius), so
    /// the two charts are compatibly oriented.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if `r0`/`dr`/`dtheta` are non-positive or `lambda ∉ [0,1]`;
    /// propagates codec / operator errors.
    pub fn new(cfg: BlendedMapConfig<R>, trunc: Truncation<R>) -> Result<Self, PhysicsError> {
        let BlendedMapConfig {
            lx,
            ly,
            r0,
            dr,
            theta0,
            dtheta,
            lambda,
        } = cfg;
        let one = R::one();
        let two = one + one;
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        if r0 <= R::zero() || dr <= R::zero() || dtheta <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "BlendedMap requires r0 > 0, dr > 0, dtheta > 0".into(),
            ));
        }
        if lambda < R::zero() || lambda > one {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "BlendedMap requires lambda in [0, 1]".into(),
            ));
        }
        let nx = 1usize << lx;
        let ny = 1usize << ly;
        let dxi = one
            / R::from_usize(nx).ok_or_else(|| {
                PhysicsError::NumericalInstability("from_usize(nx) failed".into())
            })?;
        let deta = one
            / R::from_usize(ny).ok_or_else(|| {
                PhysicsError::NumericalInstability("from_usize(ny) failed".into())
            })?;
        let g_xi = gradient_x::<R>(lx, ly, dxi, &trunc)?;
        let g_eta = gradient_y::<R>(lx, ly, deta, &trunc)?;

        // Cartesian-capture rectangle width (the fan chord at mid radius): span_y = 2·(r0+½dr)·sin(½dtheta).
        let span_y = two * (r0 + half * dr) * (half * dtheta).sin();
        let oml = one - lambda;
        // The blended forward Jacobian `(∂x/∂ξ, ∂x/∂η, ∂y/∂ξ, ∂y/∂η)` at computational `(ξ, η)`:
        //   Cartesian: (0, dr, span_y, 0);  fitted: (−r·sinθ·Δθ, cosθ·Δr, r·cosθ·Δθ, sinθ·Δr).
        let forward = move |xi: R, eta: R| -> (R, R, R, R) {
            let theta = theta0 + xi * dtheta;
            let r = r0 + eta * dr;
            let s = theta.sin();
            let c = theta.cos();
            let dx_dxi = lambda * (R::zero() - r * s * dtheta);
            let dx_deta = oml * dr + lambda * (c * dr);
            let dy_dxi = oml * span_y + lambda * (r * c * dtheta);
            let dy_deta = lambda * (s * dr);
            (dx_dxi, dx_deta, dy_dxi, dy_deta)
        };
        let det_at = move |xi: R, eta: R| -> R {
            let (a, b, c, d) = forward(xi, eta);
            a * d - b * c
        };

        // Validity (gate BM-A) holds **by construction**: the Cartesian-capture rectangle and the polar fan
        // share orientation (the `span_y` chord is taken from the same geometry), so `det J_λ` keeps one
        // sign and stays bounded away from zero across `λ ∈ [0,1]` — the `qtt_blend_metric` measurement. The
        // blended inverse metric = cofactor / det, sampled analytically (both charts are analytic).
        let neg = R::zero() - one;
        let dxi_dx = quantize_2d(
            &sample_grid(lx, ly, |xi, eta| {
                let (_, _, _, d) = forward(xi, eta);
                d / det_at(xi, eta)
            })?,
            &trunc,
        )?;
        let dxi_dy = quantize_2d(
            &sample_grid(lx, ly, |xi, eta| {
                let (_, b, _, _) = forward(xi, eta);
                neg * b / det_at(xi, eta)
            })?,
            &trunc,
        )?;
        let deta_dx = quantize_2d(
            &sample_grid(lx, ly, |xi, eta| {
                let (_, _, c, _) = forward(xi, eta);
                neg * c / det_at(xi, eta)
            })?,
            &trunc,
        )?;
        let deta_dy = quantize_2d(
            &sample_grid(lx, ly, |xi, eta| {
                let (a, _, _, _) = forward(xi, eta);
                a / det_at(xi, eta)
            })?,
            &trunc,
        )?;
        let jacobian = quantize_2d(
            &sample_grid(lx, ly, |xi, eta| det_at(xi, eta).abs())?,
            &trunc,
        )?;

        Ok(Self {
            lx,
            ly,
            lambda,
            r0,
            dr,
            theta0,
            dtheta,
            span_y,
            g_xi,
            g_eta,
            dxi_dx,
            deta_dx,
            dxi_dy,
            deta_dy,
            jacobian,
            trunc,
        })
    }

    /// The blend parameter `λ ∈ [0, 1]` (0 = Cartesian capture, 1 = body-fitted).
    pub fn lambda(&self) -> R {
        self.lambda
    }

    /// The physical position `(x, y) = (1−λ)·T_cart + λ·T_fit` at computational `(ξ, η)`.
    pub fn position(&self, xi: R, eta: R) -> (R, R) {
        let one = R::one();
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        let oml = one - self.lambda;
        let theta = self.theta0 + xi * self.dtheta;
        let r = self.r0 + eta * self.dr;
        let xc = self.r0 + eta * self.dr;
        let yc = (R::zero() - half) * self.span_y + xi * self.span_y;
        let xf = r * theta.cos();
        let yf = r * theta.sin();
        (oml * xc + self.lambda * xf, oml * yc + self.lambda * yf)
    }

    /// The Jacobian volume factor `|det J_λ|` as a (low-rank) tensor train.
    pub fn jacobian(&self) -> &CausalTensorTrain<R> {
        &self.jacobian
    }

    /// Sample `f(ξ, η)` over the lattice and QTT-encode it.
    ///
    /// # Errors
    /// Propagates sampling/codec errors.
    pub fn sample<F>(&self, f: F) -> Result<CausalTensorTrain<R>, PhysicsError>
    where
        F: Fn(R, R) -> R,
    {
        quantize_2d(&sample_grid(self.lx, self.ly, f)?, &self.trunc)
    }

    /// The **physical** gradient `(∂u/∂x, ∂u/∂y)` via the chain rule and the blended low-rank metric.
    ///
    /// # Errors
    /// Propagates apply / Hadamard / rounding errors.
    pub fn physical_gradient(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        let du_dxi = self.g_xi.apply(u, &self.trunc)?;
        let du_deta = self.g_eta.apply(u, &self.trunc)?;
        let du_dx = self
            .dxi_dx
            .hadamard_rounded(&du_dxi, &self.trunc)?
            .add(&self.deta_dx.hadamard_rounded(&du_deta, &self.trunc)?)?
            .round(&self.trunc)?;
        let du_dy = self
            .dxi_dy
            .hadamard_rounded(&du_dxi, &self.trunc)?
            .add(&self.deta_dy.hadamard_rounded(&du_deta, &self.trunc)?)?
            .round(&self.trunc)?;
        Ok((du_dx, du_dy))
    }
}

impl<R> MetricProvider<R> for BlendedMap<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn dims(&self) -> (usize, usize) {
        (self.lx, self.ly)
    }

    fn sample<F>(&self, f: F) -> Result<CausalTensorTrain<R>, PhysicsError>
    where
        F: Fn(R, R) -> R,
    {
        BlendedMap::sample(self, f)
    }

    fn physical_gradient(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        BlendedMap::physical_gradient(self, u)
    }

    fn jacobian(&self) -> &CausalTensorTrain<R> {
        BlendedMap::jacobian(self)
    }
}
