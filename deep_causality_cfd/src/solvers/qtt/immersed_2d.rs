/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Periodic 2-D incompressible Navier–Stokes with an **immersed body** by Brinkman volume penalization,
//! in quantized-tensor-train form.
//!
//! The body enters as a forcing term `−(1/η)·χ_body ⊙ (u − u_body)` added to the velocity rate each step
//! (the fused Hadamard product, then rounded), driving the velocity toward the body velocity `u_body`
//! (zero for a static wall) inside the solid — **no cut cells**, so everything stays on the uniform
//! power-of-two lattice the codec and operators assume. The convection, diffusion, recompression, and
//! divergence-free projection are reused unchanged from [`QttIncompressible2d`]; the penalization is
//! applied **before** the projection so the projection cleans the divergence the forcing introduces.

use crate::CfdScalar;
use crate::solvers::qtt::QttIncompressible2d;
use crate::tensor_bridge::{dequantize_2d, quantize_2d};
use crate::traits::Marcher;
use alloc::format;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, TensorTrain, Truncation};

/// Marches the periodic 2-D incompressible Navier–Stokes equations with an immersed body enforced by
/// Brinkman volume penalization. Wraps a [`QttIncompressible2d`] (convection + diffusion + projection)
/// and adds the penalization forcing each step. State is the `(u, v)` velocity train pair.
pub struct QttImmersed2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    inner: QttIncompressible2d<R>,
    mask: CausalTensorTrain<R>,
    ubx: R,
    uby: R,
    eta: R,
}

impl<R> QttImmersed2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Builds the immersed solver for a `2^Lx × 2^Ly` periodic grid: the base incompressible solver
    /// (spacings `dx`/`dy`, step `dt`, viscosity `nu`, round policy `trunc`) plus the body `mask`
    /// (a `[0, 1]` volume fraction, see [`body_mask_2d`](crate::body_mask_2d)), the body velocity
    /// `(ubx, uby)` (zero for a static wall), and the penalization parameter `eta` (small → hard wall;
    /// explicit stepping needs `Δt ≲ η`).
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] when the configuration is outside the numerical
    /// envelope: a non-positive or non-finite `eta`, a `dt` beyond the penalization
    /// explicit-stability limit `2η` (forward Euler on `du/dt = −u/η`), or any base-solver violation
    /// from [`QttIncompressible2d::new`] (spacings, `dt`, `nu`, the diffusive limit). The diagnostic
    /// names the limit and both the configured and the limiting value, as the DEC family's
    /// `cfl_check` does. Propagates operator-assembly errors.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lx: usize,
        ly: usize,
        dx: R,
        dy: R,
        dt: R,
        nu: R,
        mask: CausalTensorTrain<R>,
        ubx: R,
        uby: R,
        eta: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        // Penalization envelope, checked before the shared base envelope so an `eta` fault is named
        // as one. With `η = 0` the forcing `−1/η` is infinite; explicit Euler on `du/dt = −u/η` is
        // unstable for `dt > 2η`.
        if !eta.is_finite() || eta <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
                "QttImmersed2d: penalization parameter eta must be positive and finite, got {eta:?}"
            )));
        }
        let two = R::one() + R::one();
        let pen_limit = two * eta;
        if dt > pen_limit {
            return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
                "QttImmersed2d: dt {dt:?} exceeds the penalization explicit-stability limit 2·eta \
                 = {pen_limit:?} (eta {eta:?}); forward Euler on du/dt = −u/eta is unstable beyond it"
            )));
        }
        let inner = QttIncompressible2d::new(lx, ly, dx, dy, dt, nu, trunc)?;
        Ok(Self {
            inner,
            mask,
            ubx,
            uby,
            eta,
        })
    }

    /// The body mask (a `[0, 1]` volume fraction), exposed for the surface-force observables.
    pub fn mask(&self) -> &CausalTensorTrain<R> {
        &self.mask
    }

    /// The penalization parameter `η`.
    pub fn eta(&self) -> R {
        self.eta
    }

    /// The body velocity `(ubx, uby)`.
    pub fn body_velocity(&self) -> (R, R) {
        (self.ubx, self.uby)
    }

    /// The grid mode counts `(Lx, Ly)`.
    pub fn modes(&self) -> (usize, usize) {
        self.inner.modes()
    }

    /// The Leray projector (delegated to the base solver) — for the divergence observable.
    pub fn projector(&self) -> &crate::tensor_bridge::QttProjector2d<R> {
        self.inner.projector()
    }

    /// The Brinkman penalization forcing `−(1/η)·χ_body ⊙ (a − u_body)` for one velocity component.
    fn penalize(
        &self,
        a: &CausalTensorTrain<R>,
        ub: R,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let t = self.inner.trunc();
        let neg_inv_eta = (R::zero() - R::one()) / self.eta;
        // a − u_body (the velocity deficit); for a static wall (u_body = 0) this is just `a`.
        let deficit = if ub == R::zero() {
            a.clone()
        } else {
            a.add_scalar(R::zero() - ub)?
        };
        Ok(self.mask.hadamard_rounded(&deficit, &t)?.scale(neg_inv_eta))
    }

    /// One penalized explicit-Euler step followed by the divergence-free projection.
    fn step(
        &self,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        let t = self.inner.trunc();
        let dt = self.inner.dt();
        let (ru, rv) = self.inner.rate_pair(u, v)?;
        // Add the penalization forcing to the convection + diffusion rate.
        let ru = ru.add(&self.penalize(u, self.ubx)?)?.round(&t)?;
        let rv = rv.add(&self.penalize(v, self.uby)?)?.round(&t)?;
        let ustar = u.add(&ru.scale(dt))?.round(&t)?;
        let vstar = v.add(&rv.scale(dt))?.round(&t)?;
        self.inner.projector().project(&ustar, &vstar)
    }

    /// Advances a **passive scalar** `temp` one step on the same rollout: advection–diffusion by the
    /// velocity `(u, v)` (diffusivity `kappa`) plus penalization to the wall temperature `t_wall` inside
    /// the body (`−(1/η)·χ_body ⊙ (T − T_w)`). No projection (a scalar has no incompressibility
    /// constraint). This is the **neutral** thermal seam the Gap-2 reacting energy equation replaces.
    ///
    /// # Errors
    /// Propagates operator-apply / round errors.
    pub fn advance_scalar(
        &self,
        temp: &CausalTensorTrain<R>,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
        t_wall: R,
        kappa: R,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let t = self.inner.trunc();
        let dt = self.inner.dt();
        let transport = self.inner.scalar_rate(temp, u, v, kappa)?;
        let rate = transport.add(&self.penalize(temp, t_wall)?)?.round(&t)?;
        Ok(temp.add(&rate.scale(dt))?.round(&t)?)
    }

    /// Encodes `(u0, v0)`, marches `steps` penalized steps, and decodes — the end-to-end driver.
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
        let (lx, ly) = self.inner.modes();
        let want = [1usize << lx, 1usize << ly];
        for f in [u0, v0] {
            if f.shape() != want {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "field shape {:?} does not match the grid {want:?}",
                    f.shape()
                )));
            }
        }
        let t = self.inner.trunc();
        let mut u = quantize_2d(u0, &t)?;
        let mut v = quantize_2d(v0, &t)?;
        for _ in 0..steps {
            let (un, vn) = self.step(&u, &v)?;
            u = un;
            v = vn;
        }
        Ok((dequantize_2d(&u, lx, ly)?, dequantize_2d(&v, lx, ly)?))
    }
}

impl<R> Marcher<R> for QttImmersed2d<R>
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
