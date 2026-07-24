/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Periodic 2-D incompressible Navier–Stokes in quantized-tensor-train form.

use crate::CfdScalar;
use crate::tensor_bridge::{
    QttProjector2d, dequantize_2d, gradient_x, gradient_y, laplacian_2d, quantize_2d,
};
use crate::traits::Marcher;
use alloc::format;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

/// Marches the periodic 2-D incompressible Navier–Stokes equations with the velocity pair `(u, v)` held
/// as tensor trains. Each step forms the nonlinear convection `u·∇u` (via the fused Hadamard product,
/// so the `r²` intermediate is never materialized) plus viscous diffusion, advances by explicit Euler,
/// recompresses, and applies the [Leray projection](crate::QttProjector2d) — so the field stays
/// divergence-free and low-rank. Implements [`Marcher`] directly (the tensor-train stages must round
/// between operations).
pub struct QttIncompressible2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lx: usize,
    ly: usize,
    dt: R,
    nu: R,
    gx: CausalTensorTrainOperator<R>,
    gy: CausalTensorTrainOperator<R>,
    lap: CausalTensorTrainOperator<R>,
    projector: QttProjector2d<R>,
    trunc: Truncation<R>,
}

impl<R> QttIncompressible2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Builds the solver for a `2^Lx × 2^Ly` periodic grid of spacings `dx`/`dy`, time step `dt`,
    /// kinematic viscosity `nu`, and per-step round policy `trunc`.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] when the configuration is outside the solver's
    /// numerical envelope: non-finite or non-positive `dx`/`dy`/`dt`, negative or non-finite `nu`, or
    /// a `dt` beyond the diffusive explicit-stability limit `min(dx, dy)² / (4ν)` (this solver
    /// advances by explicit Euler). The diagnostic names the limit and both the configured and the
    /// limiting value, matching the DEC family's `cfl_check`. Propagates operator-assembly errors.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lx: usize,
        ly: usize,
        dx: R,
        dy: R,
        dt: R,
        nu: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        validate_qtt_envelope(dx, dy, dt, nu)?;
        Ok(Self {
            lx,
            ly,
            dt,
            nu,
            gx: gradient_x::<R>(lx, ly, dx, &trunc)?,
            gy: gradient_y::<R>(lx, ly, dy, &trunc)?,
            lap: laplacian_2d::<R>(lx, ly, dx, dy, &trunc)?,
            projector: QttProjector2d::new(lx, ly, dx, dy, trunc)?,
            trunc,
        })
    }

    /// The grid mode counts `(Lx, Ly)` (the grid is `2^Lx × 2^Ly`).
    pub fn modes(&self) -> (usize, usize) {
        (self.lx, self.ly)
    }

    /// The Leray projector — exposed so a driver can read the divergence residual off the same
    /// gradient MPOs the solver projects with (no operator rebuild).
    pub fn projector(&self) -> &QttProjector2d<R> {
        &self.projector
    }

    /// The time step.
    pub fn dt(&self) -> R {
        self.dt
    }

    /// The per-step round policy.
    pub fn trunc(&self) -> Truncation<R> {
        self.trunc
    }

    /// The convection + diffusion rate `−(u·∇)a + ν·∇²a` for both velocity components — exposed so a
    /// sibling solver (e.g. the immersed-body marcher) can add its own forcing before the projection.
    ///
    /// # Errors
    /// Propagates operator-apply / round errors.
    pub fn rate_pair(
        &self,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        Ok((self.rate(u, u, v)?, self.rate(v, u, v)?))
    }

    /// The advection–diffusion rate `−(u·∇)s + κ·∇²s` of a **passive scalar** `s` carried by the
    /// velocity `(u, v)` with diffusivity `kappa` — the same operators a velocity component uses,
    /// exposed so the immersed-body marcher can transport a temperature field on this rollout.
    ///
    /// # Errors
    /// Propagates operator-apply / round errors.
    pub fn scalar_rate(
        &self,
        s: &CausalTensorTrain<R>,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
        kappa: R,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let t = &self.trunc;
        let neg = R::zero() - R::one();
        let dsx = self.gx.apply(s, t)?;
        let dsy = self.gy.apply(s, t)?;
        let conv = u
            .hadamard_rounded(&dsx, t)?
            .add(&v.hadamard_rounded(&dsy, t)?)?
            .round(t)?;
        let diff = self.lap.apply(s, t)?.scale(kappa);
        Ok(diff.add(&conv.scale(neg))?.round(t)?)
    }

    /// `−(u·∇)a + ν·∇²a` for one velocity component `a`, with `(u, v)` the advecting velocity.
    fn rate(
        &self,
        a: &CausalTensorTrain<R>,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let t = &self.trunc;
        let neg = R::zero() - R::one();
        // Convection u·∇a = u ⊙ ∂ₓa + v ⊙ ∂ᵧa (fused Hadamard, then summed and rounded).
        let dax = self.gx.apply(a, t)?;
        let day = self.gy.apply(a, t)?;
        let conv = u
            .hadamard_rounded(&dax, t)?
            .add(&v.hadamard_rounded(&day, t)?)?
            .round(t)?;
        // ν·∇²a − convection.
        let diff = self.lap.apply(a, t)?.scale(self.nu);
        Ok(diff.add(&conv.scale(neg))?.round(t)?)
    }

    /// One explicit Euler step followed by the divergence-free projection.
    fn step(
        &self,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        let t = &self.trunc;
        let ru = self.rate(u, u, v)?;
        let rv = self.rate(v, u, v)?;
        let ustar = u.add(&ru.scale(self.dt))?.round(t)?;
        let vstar = v.add(&rv.scale(self.dt))?.round(t)?;
        self.projector.project(&ustar, &vstar)
    }

    /// Encodes `(u0, v0)`, marches `steps` steps, and decodes — the end-to-end driver.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] if a field's shape is not the grid `[2^Lx, 2^Ly]`;
    /// propagates step errors.
    pub fn run(
        &self,
        u0: &CausalTensor<R>,
        v0: &CausalTensor<R>,
        steps: usize,
    ) -> Result<(CausalTensor<R>, CausalTensor<R>), PhysicsError> {
        let want = [1usize << self.lx, 1usize << self.ly];
        for f in [u0, v0] {
            if f.shape() != want {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "field shape {:?} does not match the grid {want:?}",
                    f.shape()
                )));
            }
        }
        let mut u = quantize_2d(u0, &self.trunc)?;
        let mut v = quantize_2d(v0, &self.trunc)?;
        for _ in 0..steps {
            let (un, vn) = self.step(&u, &v)?;
            u = un;
            v = vn;
        }
        Ok((
            dequantize_2d(&u, self.lx, self.ly)?,
            dequantize_2d(&v, self.lx, self.ly)?,
        ))
    }
}

impl<R> Marcher<R> for QttIncompressible2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    type State = (CausalTensorTrain<R>, CausalTensorTrain<R>);
    type Ambient = ();
    type Output = (CausalTensorTrain<R>, CausalTensorTrain<R>);

    fn advance(
        &self,
        state: &Self::State,
        _ambient: &Self::Ambient,
    ) -> Result<Self::Output, PhysicsError> {
        self.step(&state.0, &state.1)
    }
}

/// Validate the QTT solver's shared numerical envelope: grid spacings, time step, viscosity, and the
/// diffusive explicit-stability limit. Shared by [`QttIncompressible2d::new`] and, through it,
/// [`QttImmersed2d::new`](super::QttImmersed2d) — the contract the DEC family enforces in
/// `dec_ns_solver::cfl_check`, which this crate's QTT family previously lacked at any layer.
///
/// The diagnostic names the violated limit and reports both the configured value and the limiting
/// value, so a caller can act without reading the solver source.
///
/// # Errors
/// [`PhysicsError::PhysicalInvariantBroken`] on any envelope violation.
pub(crate) fn validate_qtt_envelope<R: CfdScalar>(
    dx: R,
    dy: R,
    dt: R,
    nu: R,
) -> Result<(), PhysicsError> {
    if !dx.is_finite() || dx <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "QTT solver: dx must be positive and finite, got {dx:?}"
        )));
    }
    if !dy.is_finite() || dy <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "QTT solver: dy must be positive and finite, got {dy:?}"
        )));
    }
    if !dt.is_finite() || dt <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "QTT solver: dt must be positive and finite, got {dt:?}"
        )));
    }
    if !nu.is_finite() || nu < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "QTT solver: kinematic viscosity nu must be non-negative and finite, got {nu:?}"
        )));
    }
    // Diffusive explicit-stability limit (FTCS, explicit Euler): dt ≤ min(dx, dy)² / (4ν). Only
    // binds for a viscous flow; an inviscid solve (ν = 0) has no diffusive limit.
    if nu > R::zero() {
        let four = R::from_f64(4.0).expect("4 lifts into every real field");
        let dmin = if dx < dy { dx } else { dy };
        let diffusive_limit = dmin * dmin / (four * nu);
        if dt > diffusive_limit {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "QTT solver: dt {dt:?} exceeds the diffusive explicit-stability limit {diffusive_limit:?} \
                 (min(dx, dy)² {:?} / (4·ν {nu:?})); explicit Euler is unstable beyond it",
                dmin * dmin
            )));
        }
    }
    Ok(())
}
