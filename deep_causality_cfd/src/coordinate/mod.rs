/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B body-fitted / shock-aligned curvilinear coordinate (design D1).
//!
//! The measured `χ ~ √side` (captured curved shock on Cartesian) vs `χ ~ O(10)` (body-fitted) gap makes
//! a shock-aligned coordinate the mandatory rank lever. This module provides the canonical baseline: a
//! smooth analytic **polar (annular) map** centred at the body nose,
//!
//! ```text
//! x(ξ, η) = r·cos θ,   y(ξ, η) = r·sin θ,   r = r₀ + η·Δr,   θ = θ₀ + ξ·Δθ,   (ξ, η) ∈ [0,1)²
//! ```
//!
//! A curved bow shock standing off the nose at constant physical radius `R` is the line `η = const` in
//! this coordinate — a step in `η`, constant in `ξ` — so its QTT bond dimension is `O(10)` and
//! resolution-independent, exactly the measured precondition for the compressible marcher.
//!
//! The inverse Jacobian (the metric) is computed from the geometry — no hardcoded components — and is
//! itself smooth, so it is carried as low-rank tensor trains. Physical derivatives follow by the chain
//! rule: `∂/∂x = (∂ξ/∂x)∂/∂ξ + (∂η/∂x)∂/∂η`, with the computational `∂/∂ξ`, `∂/∂η` supplied by the §0
//! finite-difference MPOs. The angular direction is a full annulus (`Δθ = 2π`), so the periodic §0
//! operators are exact in `ξ`; non-periodic radial boundary stencils are a Stage-2 refinement (the
//! interior gradient is correct to scheme order).

mod blended;
mod cartesian;
mod cartesian_3d;
mod metric_provider;
mod metric_provider_3d;

pub use blended::{BlendedMap, BlendedMapConfig};
pub use cartesian::CartesianIdentity;
pub use cartesian_3d::CartesianIdentity3d;
pub use metric_provider::MetricProvider;
pub use metric_provider_3d::{MetricProvider3d, PhysicalGradient3d};

use crate::tensor_bridge::{gradient_x, gradient_y, quantize_2d};
use crate::types::CfdScalar;
use alloc::vec;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

/// A body-fitted polar (annular) coordinate over a `2^Lx × 2^Ly` computational lattice (`ξ` × `η`),
/// carrying the low-rank inverse-Jacobian metric and the chain-rule gradient machinery.
pub struct BodyFittedCoordinate<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lx: usize,
    ly: usize,
    r0: R,
    dr: R,
    theta0: R,
    dtheta: R,
    // Computational finite-difference operators (∂/∂ξ on the leading block, ∂/∂η on the trailing).
    g_xi: CausalTensorTrainOperator<R>,
    g_eta: CausalTensorTrainOperator<R>,
    // Inverse-Jacobian metric fields (smooth ⇒ low-rank): ∂ξ/∂x, ∂η/∂x, ∂ξ/∂y, ∂η/∂y.
    dxi_dx: CausalTensorTrain<R>,
    deta_dx: CausalTensorTrain<R>,
    dxi_dy: CausalTensorTrain<R>,
    deta_dy: CausalTensorTrain<R>,
    // The Jacobian determinant |J| = r·Δθ·Δr (the conservative volume factor).
    jacobian: CausalTensorTrain<R>,
    trunc: Truncation<R>,
}

/// Sample `f(ξ, η)` on the `2^lx × 2^ly` computational lattice (`ξ_i = i/Nx`, `η_j = j/Ny`).
pub(crate) fn sample_grid<R, F>(lx: usize, ly: usize, f: F) -> Result<CausalTensor<R>, PhysicsError>
where
    R: CfdScalar,
    F: Fn(R, R) -> R,
{
    let nx = 1usize << lx;
    let ny = 1usize << ly;
    let nxr = R::from_usize(nx)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_usize(nx) failed".into()))?;
    let nyr = R::from_usize(ny)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_usize(ny) failed".into()))?;
    let mut data = vec![R::zero(); nx * ny];
    for i in 0..nx {
        let xi = R::from_usize(i)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_usize(i) failed".into()))?
            / nxr;
        for j in 0..ny {
            let eta = R::from_usize(j)
                .ok_or_else(|| PhysicsError::NumericalInstability("from_usize(j) failed".into()))?
                / nyr;
            data[i * ny + j] = f(xi, eta);
        }
    }
    Ok(CausalTensor::new(data, vec![nx, ny])?)
}

/// Sample `f(ξ, η, ζ)` on the `2^lx × 2^ly × 2^lz` computational lattice
/// (`ξ_i = i/Nx`, `η_j = j/Ny`, `ζ_k = k/Nz`), row-major `[nx, ny, nz]`.
pub(crate) fn sample_grid_3d<R, F>(
    lx: usize,
    ly: usize,
    lz: usize,
    f: F,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: CfdScalar,
    F: Fn(R, R, R) -> R,
{
    let (nx, ny, nz) = (1usize << lx, 1usize << ly, 1usize << lz);
    let lift = |n: usize, what: &str| {
        R::from_usize(n).ok_or_else(|| {
            PhysicsError::NumericalInstability(alloc::format!("R::from_usize({what}) failed"))
        })
    };
    let (nxr, nyr, nzr) = (lift(nx, "nx")?, lift(ny, "ny")?, lift(nz, "nz")?);
    let mut data = vec![R::zero(); nx * ny * nz];
    for i in 0..nx {
        let xi = lift(i, "i")? / nxr;
        for j in 0..ny {
            let eta = lift(j, "j")? / nyr;
            for k in 0..nz {
                let zeta = lift(k, "k")? / nzr;
                data[(i * ny + j) * nz + k] = f(xi, eta, zeta);
            }
        }
    }
    Ok(CausalTensor::new(data, vec![nx, ny, nz])?)
}

impl<R> BodyFittedCoordinate<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the coordinate from the radial range `[r0, r0+dr]`, angular range `[theta0, theta0+dtheta]`,
    /// on a `2^lx × 2^ly` lattice. Use `dtheta = 2π` for a full annulus (periodic `ξ`).
    ///
    /// # Errors
    /// [`PhysicsError`] if `dr`/`dtheta`/`r0` are non-positive, or on codec/operator failures.
    pub fn new(
        lx: usize,
        ly: usize,
        r0: R,
        dr: R,
        theta0: R,
        dtheta: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        if r0 <= R::zero() || dr <= R::zero() || dtheta <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "BodyFittedCoordinate requires r0 > 0, dr > 0, dtheta > 0".into(),
            ));
        }
        let nx = 1usize << lx;
        let ny = 1usize << ly;
        let dxi = R::one()
            / R::from_usize(nx).ok_or_else(|| {
                PhysicsError::NumericalInstability("from_usize(nx) failed".into())
            })?;
        let deta = R::one()
            / R::from_usize(ny).ok_or_else(|| {
                PhysicsError::NumericalInstability("from_usize(ny) failed".into())
            })?;
        let g_xi = gradient_x::<R>(lx, ly, dxi, &trunc)?;
        let g_eta = gradient_y::<R>(lx, ly, deta, &trunc)?;

        // θ(ξ) = θ0 + ξ·Δθ, r(η) = r0 + η·Δr. The inverse-Jacobian metric (from the geometry):
        //   ∂ξ/∂x = −sinθ/(r·Δθ),  ∂η/∂x =  cosθ/Δr,
        //   ∂ξ/∂y =  cosθ/(r·Δθ),  ∂η/∂y =  sinθ/Δr,  |J| = r·Δθ·Δr.
        let theta_at = |xi: R| theta0 + xi * dtheta;
        let radius_at = |eta: R| r0 + eta * dr;

        let dxi_dx = quantize_2d(
            &sample_grid(lx, ly, |xi, eta| {
                -theta_at(xi).sin() / (radius_at(eta) * dtheta)
            })?,
            &trunc,
        )?;
        let deta_dx = quantize_2d(
            &sample_grid(lx, ly, |xi, _eta| theta_at(xi).cos() / dr)?,
            &trunc,
        )?;
        let dxi_dy = quantize_2d(
            &sample_grid(lx, ly, |xi, eta| {
                theta_at(xi).cos() / (radius_at(eta) * dtheta)
            })?,
            &trunc,
        )?;
        let deta_dy = quantize_2d(
            &sample_grid(lx, ly, |xi, _eta| theta_at(xi).sin() / dr)?,
            &trunc,
        )?;
        let jacobian = quantize_2d(
            &sample_grid(lx, ly, |_xi, eta| radius_at(eta) * dtheta * dr)?,
            &trunc,
        )?;

        Ok(Self {
            lx,
            ly,
            r0,
            dr,
            theta0,
            dtheta,
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

    /// The angular coordinate `θ(ξ) = θ0 + ξ·Δθ`.
    fn theta_of(&self, xi: R) -> R {
        self.theta0 + xi * self.dtheta
    }

    /// The physical position `(x, y)` at computational `(ξ, η)`.
    pub fn position(&self, xi: R, eta: R) -> (R, R) {
        let theta = self.theta_of(xi);
        let r = self.r0 + eta * self.dr;
        (r * theta.cos(), r * theta.sin())
    }

    /// The Jacobian determinant `|J| = r·Δθ·Δr` as a (low-rank) tensor train.
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

    /// The **physical** gradient `(∂u/∂x, ∂u/∂y)` of a field `u` (in this coordinate) via the chain
    /// rule and the low-rank metric. Correct to scheme order in the interior; the periodic operators
    /// wrap at the radial boundary (a Stage-2 refinement).
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

impl<R> MetricProvider<R> for BodyFittedCoordinate<R>
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
        BodyFittedCoordinate::sample(self, f)
    }

    fn physical_gradient(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        BodyFittedCoordinate::physical_gradient(self, u)
    }

    fn jacobian(&self) -> &CausalTensorTrain<R> {
        BodyFittedCoordinate::jacobian(self)
    }
}
