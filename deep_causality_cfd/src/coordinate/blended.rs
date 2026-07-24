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
//! Both charts share the `(ξ, η)` patch with compatible orientation. That is *not* on its own enough to
//! keep `det J_λ` one-signed: a linear blend of two orientation-compatible Jacobians can still fold, and
//! configurations this constructor accepts do — a wide fan at low blend (`r0 = 0.1`, `dr = 5`, `dθ = 5`,
//! `λ = 0.2`) reverses the sign of `det`. One sign is therefore a property to be **checked**, not derived.
//!
//! So [`BlendedMap::new`] scans `det J_λ` over the closed `(ξ, η)` domain and refuses the map if it is
//! non-finite, changes sign, or falls below [`DET_FLOOR_FRACTION`] of the geometric scale `dr × span_y`.
//! The shipped blend sweep passes with margin: `qtt_blend_metric` (gate BM-A) measures `min|det J| ≈ 1.5`
//! against a scale of the same order, about six orders above the floor.
//!
//! The marcher consumes only the **inverse** metric `∂(ξ,η)/∂(x,y)` and the volume factor,
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
use crate::CfdScalar;
use crate::tensor_bridge::{gradient_x, gradient_y, quantize_2d};
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator, Truncation,
};

/// Floor on `|det J|` as a fraction of the geometric scale `dr x span_y`, below which the map is
/// rejected as near-singular.
///
/// Relative, not absolute: `det` is an area ratio, `-dr*span_y` in the Cartesian limit and
/// `-r*dr*dtheta` in the fitted limit, so an absolute floor would mean different things at different
/// geometries. The value leaves wide margin against the shipped blend sweep, whose `qtt_blend_metric`
/// measurement records `min|det J| ~ 1.5` against a scale of the same order — i.e. the sweep sits
/// about six orders above this floor.
const DET_FLOOR_FRACTION: f64 = 1.0e-6;

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
    // The validity scan's measured minimum |det J_λ| over the closed domain, and the floor it was
    // accepted against. Retained so a consumer reads the shipped scan's number rather than
    // recomputing the determinant algebra — see `min_abs_det`.
    min_abs_det: R,
    det_floor: R,
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

        // Enforce invertibility before building anything from `1/det`.
        //
        // The marcher consumes the **inverse** metric, whose entries are `cofactor / det`, so a sign
        // change or a near-zero determinant yields values no downstream consumer can distinguish
        // from a valid map. This scan was previously absent: the module doc claimed the constructor
        // "rejects a fold" and that one-signed `det J_λ` holds "by construction", but the support
        // offered was the `qtt_blend_metric` measurement for one geometry — evidence for that case,
        // not a property of the inputs this constructor accepts.
        //
        // The scan covers the **closed** domain `[0,1]²`, one row and column more than the metric
        // trains sample. `sample_grid` forms `ξ = i / nx` for `i` in `0..nx`, so it never evaluates
        // the far edges `ξ = 1` or `η = 1` — but the chart is claimed invertible over the whole
        // computational domain, and the fan's outer boundary is part of it. Scanning only the
        // sampled points admits maps that degenerate exactly on that boundary: for `r0 = 0.1`,
        // `dr = 5`, `dθ = 5`, `λ` just under the fold threshold, `det` approaches zero at the
        // `(ξ, η) = (1, 1)` corner and the sampled minimum falls off only as `~1/nx`, so the floor
        // is not reached until `lx ≈ 20`. Closing the domain costs `nx + ny + 1` extra evaluations.
        //
        // This divides (`i / nxr`) the same way `sample_grid` does rather than multiplying by
        // `dxi = 1/nx`. The two agree bit-for-bit only because `nx` is a power of two; relying on
        // that would make the scan's agreement an accident of the lattice rather than a property of
        // the code.
        let nxr = R::from_usize(nx)
            .ok_or_else(|| PhysicsError::NumericalInstability("from_usize(nx) failed".into()))?;
        let nyr = R::from_usize(ny)
            .ok_or_else(|| PhysicsError::NumericalInstability("from_usize(ny) failed".into()))?;
        // `det` is an area ratio — `−dr·span_y` in the Cartesian limit, `−r·dr·dθ` in the fitted
        // limit — so the floor is relative to that scale. An absolute floor would mean different
        // things at different geometries.
        let det_scale = dr * span_y;
        let det_floor = R::from_f64(DET_FLOOR_FRACTION).unwrap_or_else(R::zero) * det_scale;
        let (mut det_min_abs, mut det_sign_positive) = (None::<R>, None::<bool>);
        for i in 0..=nx {
            let xi = R::from_usize(i)
                .ok_or_else(|| PhysicsError::NumericalInstability("from_usize(i) failed".into()))?
                / nxr;
            for j in 0..=ny {
                let eta = R::from_usize(j).ok_or_else(|| {
                    PhysicsError::NumericalInstability("from_usize(j) failed".into())
                })? / nyr;
                let det = det_at(xi, eta);
                if !det.is_finite() {
                    return Err(PhysicsError::PhysicalInvariantBroken(format!(
                        "BlendedMap: non-finite Jacobian determinant at the (xi, eta) sample \
                         ({i}, {j})"
                    )));
                }
                let positive = det > R::zero();
                match det_sign_positive {
                    None => det_sign_positive = Some(positive),
                    Some(first) if first != positive => {
                        return Err(PhysicsError::PhysicalInvariantBroken(format!(
                            "BlendedMap: the map folds — the Jacobian determinant changes sign by \
                             the (xi, eta) sample ({i}, {j}), so the chart is not invertible over \
                             the computational domain"
                        )));
                    }
                    Some(_) => {}
                }
                let mag = det.abs();
                if det_min_abs.is_none_or(|m| mag < m) {
                    det_min_abs = Some(mag);
                }
            }
        }
        if let Some(min_abs) = det_min_abs
            && min_abs < det_floor
        {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "BlendedMap: the map is near-singular — min |det J| is below {DET_FLOOR_FRACTION} \
                 x the geometric scale (dr x span_y); the inverse metric (cofactor / det) would be \
                 unbounded"
            )));
        }

        // The blended inverse metric = cofactor / det, sampled analytically (both charts are
        // analytic). Every division below is safe: the scan above established `|det| >= det_floor`
        // with one sign over the closed domain, which contains every point sampled here.
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
            min_abs_det: det_min_abs.unwrap_or_else(R::zero),
            det_floor,
            trunc,
        })
    }

    /// The validity scan's measured `min |det J_λ|` over the closed `(ξ, η)` domain, and the floor
    /// it was accepted against ([`DET_FLOOR_FRACTION`] of the geometric scale `dr × span_y`).
    ///
    /// Exposed so a study reporting the blend's validity margin reads the number the **shipped**
    /// scan measured, rather than recomputing the determinant algebra alongside it. A study that
    /// keeps its own copy of the Jacobian can agree with this one today and drift from it tomorrow,
    /// which makes its gate a measurement of the copy.
    pub fn det_margin(&self) -> (R, R) {
        (self.min_abs_det, self.det_floor)
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
