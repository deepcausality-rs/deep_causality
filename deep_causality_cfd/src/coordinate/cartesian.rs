/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CartesianIdentity` — the capture-limit `MetricProvider` (design D8, `λ = 0`).
//!
//! The identity chart `x = ξ·Δx·Nx`, `y = η·Δy·Ny`: physical derivatives are the computational
//! finite-difference operators directly, and the Jacobian is the constant cell volume `Δx·Δy`. It works
//! for any geometry (a captured shock is high rank, but representable), and is the control the fitted
//! coordinate is measured against.

use super::sample_grid;
use crate::CfdScalar;
use crate::tensor_bridge::{gradient_x, gradient_y, quantize_2d};
use crate::traits::MetricProvider;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensorTrain, CausalTensorTrainOperator, TensorTrainOperator, Truncation,
};

/// The Cartesian identity coordinate over a `2^Lx × 2^Ly` lattice with physical spacing `(dx, dy)`.
pub struct CartesianIdentity<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lx: usize,
    ly: usize,
    gx: CausalTensorTrainOperator<R>,
    gy: CausalTensorTrainOperator<R>,
    jacobian: CausalTensorTrain<R>,
    trunc: Truncation<R>,
}

impl<R> CartesianIdentity<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the identity chart on a `2^lx × 2^ly` lattice with physical spacing `(dx, dy)`.
    ///
    /// # Errors
    /// Propagates operator-assembly / codec errors.
    pub fn new(
        lx: usize,
        ly: usize,
        dx: R,
        dy: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let gx = gradient_x::<R>(lx, ly, dx, &trunc)?;
        let gy = gradient_y::<R>(lx, ly, dy, &trunc)?;
        let cell = dx * dy;
        let jacobian = quantize_2d(&sample_grid(lx, ly, |_xi, _eta| cell)?, &trunc)?;
        Ok(Self {
            lx,
            ly,
            gx,
            gy,
            jacobian,
            trunc,
        })
    }
}

impl<R> MetricProvider<R> for CartesianIdentity<R>
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
        quantize_2d(&sample_grid(self.lx, self.ly, f)?, &self.trunc)
    }

    fn physical_gradient(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        Ok((
            self.gx.apply(u, &self.trunc)?,
            self.gy.apply(u, &self.trunc)?,
        ))
    }

    fn jacobian(&self) -> &CausalTensorTrain<R> {
        &self.jacobian
    }
}
