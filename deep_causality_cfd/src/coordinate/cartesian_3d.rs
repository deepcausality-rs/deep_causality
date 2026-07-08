/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CartesianIdentity3d` — the 3-D capture-limit [`MetricProvider3d`].
//!
//! The identity chart `x = ξ·Δx·Nx`, `y = η·Δy·Ny`, `z = ζ·Δz·Nz`: physical derivatives are the
//! computational finite-difference operators directly, and the Jacobian is the constant cell volume
//! `Δx·Δy·Δz`. It represents any geometry (a captured curved shock is high rank — `χ ~ √side` — but
//! representable), and is the control the fitted 3-D coordinate is measured against.

use super::metric_provider_3d::{MetricProvider3d, PhysicalGradient3d};
use super::sample_grid_3d;
use crate::tensor_bridge::{gradient_x_3d, gradient_y_3d, gradient_z_3d, quantize_3d};
use crate::types::CfdScalar;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensorTrain, CausalTensorTrainOperator, TensorTrainOperator, Truncation,
};

/// The Cartesian identity coordinate over a `2^Lx × 2^Ly × 2^Lz` lattice with physical spacing
/// `(dx, dy, dz)`.
pub struct CartesianIdentity3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lx: usize,
    ly: usize,
    lz: usize,
    gx: CausalTensorTrainOperator<R>,
    gy: CausalTensorTrainOperator<R>,
    gz: CausalTensorTrainOperator<R>,
    jacobian: CausalTensorTrain<R>,
    trunc: Truncation<R>,
}

impl<R> CartesianIdentity3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the identity chart on a `2^lx × 2^ly × 2^lz` lattice with physical spacing `(dx, dy, dz)`.
    ///
    /// # Errors
    /// Propagates operator-assembly / codec errors.
    pub fn new(
        lx: usize,
        ly: usize,
        lz: usize,
        dx: R,
        dy: R,
        dz: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let gx = gradient_x_3d::<R>(lx, ly, lz, dx, &trunc)?;
        let gy = gradient_y_3d::<R>(lx, ly, lz, dy, &trunc)?;
        let gz = gradient_z_3d::<R>(lx, ly, lz, dz, &trunc)?;
        let cell = dx * dy * dz;
        let jacobian = quantize_3d(&sample_grid_3d(lx, ly, lz, |_x, _y, _z| cell)?, &trunc)?;
        Ok(Self {
            lx,
            ly,
            lz,
            gx,
            gy,
            gz,
            jacobian,
            trunc,
        })
    }
}

impl<R> MetricProvider3d<R> for CartesianIdentity3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
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
        Ok((
            self.gx.apply(u, &self.trunc)?,
            self.gy.apply(u, &self.trunc)?,
            self.gz.apply(u, &self.trunc)?,
        ))
    }

    fn jacobian(&self) -> &CausalTensorTrain<R> {
        &self.jacobian
    }
}
