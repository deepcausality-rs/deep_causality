/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quantized-tensor-train (QTT) flow solvers — the first solvers that evolve a flowfield as a tensor
//! train (the CFD ↔ tensor-network bridge): a quasi-1D linear advection–diffusion marcher and a 2-D
//! incompressible Navier–Stokes marcher.

mod compressible;
mod immersed_2d;
mod incompressible_2d;
mod observe;

pub use compressible::{
    AcousticImex1d, CompressibleEuler1d, CompressibleMarcher2d, CompressibleMarcher3d,
    CompressibleMarcher3dFitted, EulerState, EulerState2d, EulerState3d, EulerStateTt2d,
    EulerStateTt3d, FittedNormalShock, ForcingRegion, Park2tClosure, PostShockState,
    StagnationOutcome, conservation_round, ideal_gas_pressure, ideal_gas_pressure_2d,
    positivity_floor,
};
pub use immersed_2d::QttImmersed2d;
pub use incompressible_2d::QttIncompressible2d;
pub use observe::{
    divergence_residual, drag_lift, kinetic_energy, max_bond, max_speed,
    penalization_heat_integral, preserved_drag_fraction, strip_pressure_force,
};

use crate::CfdScalar;
use crate::tensor_bridge::{dequantize, gradient, laplacian, quantize};
use crate::traits::Marcher;
use alloc::format;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

/// Marches the periodic linear advection–diffusion equation `∂u/∂t = −c·∂ₓu + ν·∂²ₓu` on a `2^L`
/// grid in compressed tensor-train form.
///
/// Each step applies the assembled `gradient` / `laplacian` MPOs to the quantized field and
/// **recompresses** with the round policy, so the field stays low-rank with bounded bond dimension.
/// It implements [`Marcher`] directly (one explicit Euler step with an explicit round) rather than
/// riding the Rk4-based `FluidTheory` path, because tensor-train stages must round between operations
/// to keep the rank from growing.
pub struct QttLinear1d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    l: usize,
    dt: R,
    c: R,
    nu: R,
    grad: CausalTensorTrainOperator<R>,
    lap: CausalTensorTrainOperator<R>,
    trunc: Truncation<R>,
}

impl<R> QttLinear1d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Builds the marcher for a periodic `2^L`-point grid of spacing `dx`, advection speed `c`,
    /// diffusivity `nu`, time step `dt`, and the per-step round policy `trunc`.
    ///
    /// # Errors
    /// Propagates operator-assembly errors.
    pub fn new(
        l: usize,
        dx: R,
        dt: R,
        c: R,
        nu: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let grad = gradient::<R>(l, dx, &trunc)?;
        let lap = laplacian::<R>(l, dx, &trunc)?;
        Ok(Self {
            l,
            dt,
            c,
            nu,
            grad,
            lap,
            trunc,
        })
    }

    /// Encodes `u0`, marches `steps` steps, and decodes the result — the end-to-end driver.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] if `u0`'s length is not the grid size `2^L`; propagates
    /// step errors.
    pub fn run(&self, u0: &CausalTensor<R>, steps: usize) -> Result<CausalTensor<R>, PhysicsError> {
        let n = u0.as_slice().len();
        if n != (1usize << self.l) {
            return Err(PhysicsError::DimensionMismatch(format!(
                "field length {n} does not match the grid size 2^{}",
                self.l
            )));
        }
        let mut state = quantize(u0, &self.trunc)?;
        for _ in 0..steps {
            state = self.advance(&state, &())?;
        }
        dequantize(&state)
    }
}

impl<R> Marcher<R> for QttLinear1d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    type State = CausalTensorTrain<R>;
    type Ambient = ();
    type Output = CausalTensorTrain<R>;

    /// One explicit Euler step with per-step recompression:
    /// `u ← round(u + Δt·(−c·∂ₓu + ν·∂²ₓu))`.
    fn advance(
        &self,
        state: &Self::State,
        _ambient: &Self::Ambient,
    ) -> Result<Self::Output, PhysicsError> {
        let neg_c = R::zero() - self.c;
        let grad_u = self.grad.apply(state, &self.trunc)?;
        let lap_u = self.lap.apply(state, &self.trunc)?;
        let rate = grad_u.scale(neg_c).add(&lap_u.scale(self.nu))?;
        let next = state.add(&rate.scale(self.dt))?.round(&self.trunc)?;
        Ok(next)
    }
}
