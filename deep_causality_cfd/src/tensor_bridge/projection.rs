/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Divergence-free (Leray) projection for the QTT 2-D incompressible solver.
//!
//! The pressure-Poisson solve is **spectral**, not iterative: on a periodic grid the Laplacian is
//! diagonal in the Fourier basis, so `p̂_k = rhŝ_k / λ_k` with the constant (`k=0`) mode zeroed — which
//! pins the singular Laplacian's null space *by construction*. The projection rests on `∇p` (unique
//! despite the singular operator), so the non-uniqueness of `p` is irrelevant. (ARIZ resolution; see
//! `openspec/notes/plasma-blackout/qtt-incompressible-2d-ariz.md`.) Tier-A solves the spectral step on
//! the dequantized field; a QFT-MPO keeps it in QTT at scale.

use super::codec::{dequantize_2d, quantize_2d};
use super::operators::{gradient_x, gradient_y};
use crate::types::CfdScalar;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_fft::RfftPlanNd;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

/// Periodic 2-D Leray projector: holds the gradient MPOs and the grid metadata, and exposes
/// `divergence`, the spectral `solve_poisson`, and the divergence-free `project`.
pub struct QttProjector2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lx: usize,
    ly: usize,
    dx: R,
    dy: R,
    gx: CausalTensorTrainOperator<R>,
    gy: CausalTensorTrainOperator<R>,
    trunc: Truncation<R>,
}

impl<R> QttProjector2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Builds the projector for a `2^Lx × 2^Ly` periodic grid of spacings `dx`/`dy`.
    ///
    /// # Errors
    /// Propagates operator-assembly errors.
    pub fn new(
        lx: usize,
        ly: usize,
        dx: R,
        dy: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let gx = gradient_x::<R>(lx, ly, dx, &trunc)?;
        let gy = gradient_y::<R>(lx, ly, dy, &trunc)?;
        Ok(Self {
            lx,
            ly,
            dx,
            dy,
            gx,
            gy,
            trunc,
        })
    }

    /// `∇·(u, v) = ∂ₓu + ∂ᵧv` (apply + add + round).
    ///
    /// # Errors
    /// Propagates apply / round errors.
    pub fn divergence(
        &self,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let du = self.gx.apply(u, &self.trunc)?;
        let dv = self.gy.apply(v, &self.trunc)?;
        Ok(du.add(&dv)?.round(&self.trunc)?)
    }

    /// Solves the periodic pressure-Poisson equation `∇²p = rhs` spectrally (null mode zeroed).
    ///
    /// # Errors
    /// Propagates codec and FFT errors.
    pub fn solve_poisson(
        &self,
        rhs: &CausalTensorTrain<R>,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let dense = dequantize_2d(rhs, self.lx, self.ly)?;
        let nx = 1usize << self.lx;
        let ny = 1usize << self.ly;
        let p = spectral_poisson::<R>(dense.as_slice(), nx, ny, self.dx, self.dy)?;
        let field = CausalTensor::new(p, vec![nx, ny])?;
        quantize_2d(&field, &self.trunc)
    }

    /// Leray projection: returns `(u, v)` with `∇p` removed, so the result is discretely
    /// divergence-free. `u ← u* − ∂ₓp`, `v ← v* − ∂ᵧp`.
    ///
    /// # Errors
    /// Propagates divergence / Poisson / apply / round errors.
    pub fn project(
        &self,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
    ) -> Result<(CausalTensorTrain<R>, CausalTensorTrain<R>), PhysicsError> {
        let div = self.divergence(u, v)?;
        let p = self.solve_poisson(&div)?;
        let neg = R::zero() - R::one();
        let un = u
            .add(&self.gx.apply(&p, &self.trunc)?.scale(neg))?
            .round(&self.trunc)?;
        let vn = v
            .add(&self.gy.apply(&p, &self.trunc)?.scale(neg))?
            .round(&self.trunc)?;
        Ok((un, vn))
    }
}

/// Dense spectral Poisson solve on a `Nx × Ny` periodic grid: `∇²p = rhs`, the `k=0` mode zeroed.
fn spectral_poisson<R>(
    rhs: &[R],
    nx: usize,
    ny: usize,
    dx: R,
    dy: R,
) -> Result<Vec<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let plan = RfftPlanNd::<R>::new(&[nx, ny])
        .map_err(|e| PhysicsError::CalculationError(format!("rfft plan: {e:?}")))?;
    let zero = Complex::from_real(R::zero());
    let mut spec = vec![zero; plan.spectrum_len()];
    let mut scratch = vec![zero; plan.scratch_len()];
    plan.execute(rhs, &mut spec, &mut scratch)
        .map_err(|e| PhysicsError::CalculationError(format!("rfft forward: {e:?}")))?;

    let hy = plan.spectrum_shape()[1]; // ny/2 + 1
    let two = R::one() + R::one();
    let tau = R::pi() * two;
    let nxf = from_usize::<R>(nx);
    let nyf = from_usize::<R>(ny);
    let dx2 = dx * dx;
    let dy2 = dy * dy;
    // The projection applies grad-of-grad (centered difference squared), eigenvalue -sin^2(2pik/N)/dx^2
    // (the *consistent* operator, not the compact 5-point Laplacian) so div(project(u)) = 0 exactly. It
    // is singular at k in {0, N/2} per axis (constant + collocated checkerboard/Nyquist), all zeroed.
    let (half_x, half_y) = (nx / 2, ny / 2);
    // λ_k = −(2 − 2cos(2πk/N))/Δ²; the periodic Laplacian eigenvalue (separable in 2-D).
    for kx in 0..nx {
        let sx = (tau * from_usize::<R>(kx) / nxf).sin();
        let lamx = sx * sx / dx2;
        for ky in 0..hy {
            let sy = (tau * from_usize::<R>(ky) / nyf).sin();
            let lamy = sy * sy / dy2;
            let idx = kx * hy + ky;
            let is_null = (kx == 0 || kx == half_x) && (ky == 0 || ky == half_y);
            if is_null {
                spec[idx] = zero;
            } else {
                // ∇²p = rhs with λ = −(lamx+lamy): p̂ = rhŝ / (−λ).
                let inv = R::zero() - R::one() / (lamx + lamy);
                spec[idx] = Complex::new(spec[idx].re * inv, spec[idx].im * inv);
            }
        }
    }

    let mut out = vec![R::zero(); nx * ny];
    plan.execute_inverse(&mut spec, &mut out, &mut scratch)
        .map_err(|e| PhysicsError::CalculationError(format!("rfft inverse: {e:?}")))?;
    Ok(out)
}

fn from_usize<R: FromPrimitive>(n: usize) -> R {
    <R as FromPrimitive>::from_f64(n as f64).unwrap()
}
