/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 6 — the 3-D compressible Euler marcher (the forebody-sheath machinery).
//!
//! Conservative state `U = (ρ, ρu, ρv, ρw, ρE)` carried as five tensor trains on a periodic
//! `2^Lx × 2^Ly × 2^Lz` Cartesian lattice (the §0 3-D operators). The time step is **IMEX** (design D10):
//! the convective flux divergence `∂ₓF + ∂ᵧG + ∂_zH` is explicit, the stabilizing acoustic dissipation is
//! taken **implicitly** through the closed-form 3-D ADI inverse
//! ([`AcousticCoreInverse3d`](crate::AcousticCoreInverse3d)) — the 3-D analogue of the Stage-5 marcher,
//! free-stream-exact and bounded beyond the explicit acoustic-diffusion limit, with no iterative solve.
//!
//! This is the Cartesian-capture 3-D marcher (the §0 operators + §2–4 flux/EOS machinery in 3-D). The
//! body-fitted 3-D coordinate that bounds the forebody bond (§1 generalized to 3-D) rides the same
//! `MetricProvider` seam and is the remaining Stage-6 piece; the **wake** is out of scope (turbulence).

use crate::CfdScalar;
use crate::tensor_bridge::{
    AcousticCoreInverse3d, dequantize_3d, gradient_x_3d, gradient_y_3d, gradient_z_3d, quantize_3d,
};
use crate::traits::Marcher;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

/// One 3-D conservative state as five dense buffers `(ρ, ρu, ρv, ρw, ρE)`, row-major `2^Lx × 2^Ly × 2^Lz`.
pub type EulerState3d<R> = [Vec<R>; 5];

/// One 3-D conservative state as five tensor trains `(ρ, ρu, ρv, ρw, ρE)` — the [`Marcher`] state.
pub type EulerStateTt3d<R> = [CausalTensorTrain<R>; 5];

/// A 3-D compressible Euler marcher on a periodic Cartesian lattice.
pub struct CompressibleMarcher3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    gamma: R,
    dt: R,
    lx: usize,
    ly: usize,
    lz: usize,
    grad_x: CausalTensorTrainOperator<R>,
    grad_y: CausalTensorTrainOperator<R>,
    grad_z: CausalTensorTrainOperator<R>,
    /// `(I − Δt·ν̄·∇²)⁻¹` with reference dissipation `ν̄ = ½·s_ref·Δx`, the implicit acoustic step (D10).
    acoustic_inv: AcousticCoreInverse3d<R>,
    trunc: Truncation<R>,
}

impl<R> CompressibleMarcher3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the marcher on a periodic `2^lx × 2^ly × 2^lz` lattice `dims` of (cubic) cell size `dx`, with
    /// ratio of specific heats `gamma`, fixed step `dt`, and a reference wave speed `s_ref` (`max(|u|+c)`)
    /// that sets the implicit acoustic dissipation `ν̄ = ½·s_ref·dx`.
    ///
    /// # Errors
    /// [`PhysicsError::NumericalInstability`] if `s_ref` is not finite and positive; propagates
    /// operator-assembly errors.
    pub fn new(
        dims: (usize, usize, usize),
        dx: R,
        gamma: R,
        dt: R,
        s_ref: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let (lx, ly, lz) = dims;
        if !s_ref.is_finite() || s_ref <= R::zero() {
            return Err(PhysicsError::NumericalInstability(
                "compressible marcher 3d: reference wave speed s_ref must be finite and positive"
                    .into(),
            ));
        }
        let grad_x = gradient_x_3d::<R>(lx, ly, lz, dx, &trunc)?;
        let grad_y = gradient_y_3d::<R>(lx, ly, lz, dx, &trunc)?;
        let grad_z = gradient_z_3d::<R>(lx, ly, lz, dx, &trunc)?;
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        let beta = dt * half * s_ref * dx;
        let acoustic_inv = AcousticCoreInverse3d::new((lx, ly, lz), (dx, dx, dx), beta, trunc)?;
        Ok(Self {
            gamma,
            dt,
            lx,
            ly,
            lz,
            grad_x,
            grad_y,
            grad_z,
            acoustic_inv,
            trunc,
        })
    }

    /// Pointwise `(F, G, H)` flux components and the global LLF wave speed `s_max = max(|u|+c,|v|+c,|w|+c)`.
    #[allow(clippy::type_complexity)]
    fn flux_and_speed(
        &self,
        u: &[Vec<R>; 5],
    ) -> Result<([Vec<R>; 5], [Vec<R>; 5], [Vec<R>; 5], R), PhysicsError> {
        let n = u[0].len();
        let mk = || {
            [
                Vec::with_capacity(n),
                Vec::with_capacity(n),
                Vec::with_capacity(n),
                Vec::with_capacity(n),
                Vec::with_capacity(n),
            ]
        };
        let mut f = mk();
        let mut g = mk();
        let mut h = mk();
        let mut s_max = R::zero();
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        for (cell, ((((&rho, &mx), &my), &mz), &e)) in u[0]
            .iter()
            .zip(&u[1])
            .zip(&u[2])
            .zip(&u[3])
            .zip(&u[4])
            .enumerate()
        {
            if rho <= R::zero() || !rho.is_finite() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "compressible marcher 3d: density must stay positive".into(),
                ));
            }
            let vx = mx / rho;
            let vy = my / rho;
            let vz = mz / rho;
            let mom2 = mx * mx + my * my + mz * mz;
            let p = (self.gamma - R::one()) * (e - half * mom2 / rho);
            // Reject a non-hyperbolic state before it enters the flux (shared guard, all four
            // marchers). `p` is positive below, so the acoustic speed needs no floor.
            super::require_positive_pressure(p, cell)?;
            let c = (self.gamma * p / rho).sqrt();
            f[0].push(mx);
            f[1].push(mx * vx + p);
            f[2].push(mx * vy);
            f[3].push(mx * vz);
            f[4].push((e + p) * vx);
            g[0].push(my);
            g[1].push(my * vx);
            g[2].push(my * vy + p);
            g[3].push(my * vz);
            g[4].push((e + p) * vy);
            h[0].push(mz);
            h[1].push(mz * vx);
            h[2].push(mz * vy);
            h[3].push(mz * vz + p);
            h[4].push((e + p) * vz);
            let (ax, ay, az) = (vx.abs(), vy.abs(), vz.abs());
            let axy = if ax > ay { ax } else { ay };
            let amax = if axy > az { axy } else { az };
            let s = amax + c;
            if s > s_max {
                s_max = s;
            }
        }
        Ok((f, g, h, s_max))
    }

    fn encode(&self, v: &[R]) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let (nx, ny, nz) = (1usize << self.lx, 1usize << self.ly, 1usize << self.lz);
        quantize_3d(
            &CausalTensor::new(v.to_vec(), alloc::vec![nx, ny, nz])?,
            &self.trunc,
        )
    }

    /// March `state0` for `steps` fixed-`dt` steps; return the final dense state and the peak `max_bond`
    /// seen over the march (the rank witness for the forebody gate).
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] on a wrong-length input; propagates step errors.
    pub fn run(
        &self,
        state0: &EulerState3d<R>,
        steps: usize,
    ) -> Result<(EulerState3d<R>, usize), PhysicsError> {
        let n = (1usize << self.lx) * (1usize << self.ly) * (1usize << self.lz);
        for buf in state0.iter() {
            if buf.len() != n {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "state length {} does not match grid 2^{}·2^{}·2^{}",
                    buf.len(),
                    self.lx,
                    self.ly,
                    self.lz
                )));
            }
        }
        let mut u: EulerStateTt3d<R> = [
            self.encode(&state0[0])?,
            self.encode(&state0[1])?,
            self.encode(&state0[2])?,
            self.encode(&state0[3])?,
            self.encode(&state0[4])?,
        ];
        let mut peak = u.iter().map(|t| t.max_bond()).max().unwrap_or(0);
        for _ in 0..steps {
            u = self.step(&u)?;
            let step_peak = u.iter().map(|t| t.max_bond()).max().unwrap_or(0);
            if step_peak > peak {
                peak = step_peak;
            }
        }
        let out = [
            self.decode(&u[0])?,
            self.decode(&u[1])?,
            self.decode(&u[2])?,
            self.decode(&u[3])?,
            self.decode(&u[4])?,
        ];
        Ok((out, peak))
    }

    fn decode(&self, t: &CausalTensorTrain<R>) -> Result<Vec<R>, PhysicsError> {
        Ok(dequantize_3d(t, self.lx, self.ly, self.lz)?
            .as_slice()
            .to_vec())
    }

    /// One IMEX step on the tensor-train state: explicit convective predictor `U* = Uⁿ − Δt·∇·F`, then the
    /// implicit acoustic dissipation in closed form `Uⁿ⁺¹ = A₀⁻¹·U*`.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if density leaves the positive cone; propagates flux /
    /// gradient / inverse-apply / rounding errors.
    pub fn step(&self, u: &EulerStateTt3d<R>) -> Result<EulerStateTt3d<R>, PhysicsError> {
        let dense: [Vec<R>; 5] = [
            self.decode(&u[0])?,
            self.decode(&u[1])?,
            self.decode(&u[2])?,
            self.decode(&u[3])?,
            self.decode(&u[4])?,
        ];
        let (f, g, h, _s_max) = self.flux_and_speed(&dense)?;
        Ok([
            self.step_component(&u[0], &f[0], &g[0], &h[0])?,
            self.step_component(&u[1], &f[1], &g[1], &h[1])?,
            self.step_component(&u[2], &f[2], &g[2], &h[2])?,
            self.step_component(&u[3], &f[3], &g[3], &h[3])?,
            self.step_component(&u[4], &f[4], &g[4], &h[4])?,
        ])
    }

    /// Advance one conserved component: explicit `−Δt·(∂ₓF+∂ᵧG+∂_zH)` predictor, then the implicit inverse.
    fn step_component(
        &self,
        uk: &CausalTensorTrain<R>,
        fk: &[R],
        gk: &[R],
        hk: &[R],
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let neg = R::zero() - R::one();
        let dfx = self.grad_x.apply(&self.encode(fk)?, &self.trunc)?;
        let dgy = self.grad_y.apply(&self.encode(gk)?, &self.trunc)?;
        let dhz = self.grad_z.apply(&self.encode(hk)?, &self.trunc)?;
        let div = dfx.add(&dgy)?.add(&dhz)?.round(&self.trunc)?;
        let predictor = uk.add(&div.scale(neg * self.dt))?.round(&self.trunc)?;
        Ok(self.acoustic_inv.apply(&predictor)?.round(&self.trunc)?)
    }

    /// The ratio of specific heats.
    pub fn gamma(&self) -> R {
        self.gamma
    }
}

impl<R> Marcher<R> for CompressibleMarcher3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    type State = EulerStateTt3d<R>;
    type Ambient = ();
    type Output = EulerStateTt3d<R>;

    fn advance(
        &self,
        state: &Self::State,
        _ambient: &Self::Ambient,
    ) -> Result<Self::Output, PhysicsError> {
        self.step(state)
    }
}
