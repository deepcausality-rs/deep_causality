/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Periodic 2-D incompressible Navier–Stokes in quantized-tensor-train form.

use crate::tensor_bridge::{
    QttProjector2d, dequantize_2d, gradient_x, gradient_y, laplacian_2d, quantize_2d,
};
use crate::traits::Marcher;
use crate::types::CfdScalar;
use alloc::format;
use deep_causality_num::ConjugateScalar;
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
    /// Propagates operator-assembly errors.
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
