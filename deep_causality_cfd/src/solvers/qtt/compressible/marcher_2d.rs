/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 5 — the 2-D body-fitted compressible Euler marcher.
//!
//! Conservative state `U = (ρ, ρu, ρv, ρE)` carried as four tensor trains, marched in a structured
//! coordinate supplied through the [`MetricProvider`](crate::coordinate::MetricProvider) seam (design D8).
//! The same marcher therefore runs on `CartesianIdentity` (the captured control, `χ ~ √side`) and on a
//! `BodyFittedCoordinate` (the fitted case, `χ ~ O(10)`), so the Stage-5 rank-lever gate is a clean
//! comparison with one solver.
//!
//! The flux divergence `∂ₓF + ∂ᵧG` is assembled from the metric's chain-rule `physical_gradient` and
//! advanced **IMEX**: the convective flux is explicit, while the stabilizing acoustic dissipation is taken
//! **implicitly** through the closed-form 2-D constant-coefficient inverse
//! ([`AcousticCoreInverse2d`](crate::AcousticCoreInverse2d), design D10) — the 2-D system analogue of the
//! Stage-3 `AcousticImex1d`. This removes the explicit acoustic-diffusion stability limit (the step stays
//! bounded where a fully-explicit dissipation control diverges) with **no iterative solve**, and the
//! inverse is **free-stream-exact** (`A₀⁻¹·const = const` to round-off) — the property an AMEn-per-step
//! solve loses to its residual tolerance, which is why the closed-form inverse, not the AMEn prototype, is
//! what lets the implicit step land on a captured curved field. Nonlinear flux / EOS are evaluated
//! pointwise (dequantize → compute → requantize), recompressed each step.

use crate::CfdScalar;
use crate::coordinate::MetricProvider;
use crate::tensor_bridge::{AcousticCoreInverse2d, dequantize_2d, quantize_2d};
use crate::traits::Marcher;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, TensorTrain, Truncation};

/// One 2-D conservative state as four dense buffers `(ρ, ρu, ρv, ρE)`, row-major `2^Lx × 2^Ly`.
pub type EulerState2d<R> = [Vec<R>; 4];

/// One 2-D conservative state as four tensor trains `(ρ, ρu, ρv, ρE)` — the [`Marcher`] state.
pub type EulerStateTt2d<R> = [CausalTensorTrain<R>; 4];

/// A 2-D compressible Euler marcher over a structured coordinate (`M: MetricProvider`).
pub struct CompressibleMarcher2d<R, M>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    M: MetricProvider<R>,
{
    metric: M,
    gamma: R,
    dt: R,
    lx: usize,
    ly: usize,
    trunc: Truncation<R>,
    /// `(I − Δt·ν̄·∇²)⁻¹` with reference dissipation `ν̄ = ½·s_ref·Δx`, the implicit acoustic step (D10).
    acoustic_inv: AcousticCoreInverse2d<R>,
}

impl<R, M> CompressibleMarcher2d<R, M>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    M: MetricProvider<R>,
{
    /// Build the marcher over `metric`, with ratio of specific heats `gamma`, fixed step `dt`, and a
    /// reference wave speed `s_ref` (`max(|u|+c)` the flow will see) that sets the implicit acoustic
    /// dissipation `ν̄ = ½·s_ref·Δx`. The closed-form inverse `A₀⁻¹` for `A₀ = I − Δt·ν̄·∇²` is built once.
    ///
    /// # Errors
    /// [`PhysicsError::NumericalInstability`] if `s_ref` is not finite and positive; propagates
    /// operator-assembly errors.
    pub fn new(
        metric: M,
        gamma: R,
        dt: R,
        s_ref: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let (lx, ly) = metric.dims();
        let nx = 1usize << lx;
        let h = R::one()
            / R::from_usize(nx).ok_or_else(|| {
                PhysicsError::NumericalInstability("R::from_usize(nx) failed".into())
            })?;
        if !s_ref.is_finite() || s_ref <= R::zero() {
            return Err(PhysicsError::NumericalInstability(
                "compressible marcher: reference wave speed s_ref must be finite and positive"
                    .into(),
            ));
        }
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        let beta = dt * half * s_ref * h;
        let acoustic_inv = AcousticCoreInverse2d::new(lx, ly, h, h, beta, trunc)?;
        Ok(Self {
            metric,
            gamma,
            dt,
            lx,
            ly,
            trunc,
            acoustic_inv,
        })
    }

    /// Pointwise `(F, G)` flux components and the global LLF wave speed `s_max = max(|u|+c, |v|+c)`.
    #[allow(clippy::type_complexity)]
    fn flux_and_speed(
        &self,
        u: &[Vec<R>; 4],
    ) -> Result<([Vec<R>; 4], [Vec<R>; 4], R), PhysicsError> {
        let n = u[0].len();
        let mut f = [
            Vec::with_capacity(n),
            Vec::with_capacity(n),
            Vec::with_capacity(n),
            Vec::with_capacity(n),
        ];
        let mut g = [
            Vec::with_capacity(n),
            Vec::with_capacity(n),
            Vec::with_capacity(n),
            Vec::with_capacity(n),
        ];
        let mut s_max = R::zero();
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        for (cell, (((&rho, &mx), &my), &e)) in
            u[0].iter().zip(&u[1]).zip(&u[2]).zip(&u[3]).enumerate()
        {
            if rho <= R::zero() || !rho.is_finite() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "compressible marcher: density must stay positive".into(),
                ));
            }
            let vx = mx / rho;
            let vy = my / rho;
            // p = (γ−1)(E − ½(ρu²+ρv²)/ρ); reuse the 1-D EOS with the combined momentum magnitude.
            let mom2 = mx * mx + my * my;
            let p = (self.gamma - R::one()) * (e - half * mom2 / rho);
            // Reject a non-hyperbolic state before it enters the flux (shared guard, all four
            // marchers). `p` is positive below, so the acoustic speed needs no floor.
            super::require_positive_pressure(p, cell)?;
            let c = (self.gamma * p / rho).sqrt();
            f[0].push(mx);
            f[1].push(mx * vx + p);
            f[2].push(mx * vy);
            f[3].push((e + p) * vx);
            g[0].push(my);
            g[1].push(my * vx);
            g[2].push(my * vy + p);
            g[3].push((e + p) * vy);
            let sx = vx.abs() + c;
            let sy = vy.abs() + c;
            let s = if sx > sy { sx } else { sy };
            if s > s_max {
                s_max = s;
            }
        }
        Ok((f, g, s_max))
    }

    fn encode(&self, v: &[R]) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let nx = 1usize << self.lx;
        let ny = 1usize << self.ly;
        quantize_2d(
            &CausalTensor::new(v.to_vec(), alloc::vec![nx, ny])?,
            &self.trunc,
        )
    }

    /// March `state0` for `steps` fixed-`dt` steps; return the final dense state and the peak `max_bond`
    /// seen over the march (the rank witness for the Stage-5 gate).
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] on a wrong-length input; propagates step errors.
    pub fn run(
        &self,
        state0: &EulerState2d<R>,
        steps: usize,
    ) -> Result<(EulerState2d<R>, usize), PhysicsError> {
        let n = (1usize << self.lx) * (1usize << self.ly);
        for buf in state0.iter() {
            if buf.len() != n {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "state length {} does not match grid 2^{}·2^{}",
                    buf.len(),
                    self.lx,
                    self.ly
                )));
            }
        }
        let mut u: EulerStateTt2d<R> = [
            self.encode(&state0[0])?,
            self.encode(&state0[1])?,
            self.encode(&state0[2])?,
            self.encode(&state0[3])?,
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
            dequantize_2d(&u[0], self.lx, self.ly)?.as_slice().to_vec(),
            dequantize_2d(&u[1], self.lx, self.ly)?.as_slice().to_vec(),
            dequantize_2d(&u[2], self.lx, self.ly)?.as_slice().to_vec(),
            dequantize_2d(&u[3], self.lx, self.ly)?.as_slice().to_vec(),
        ];
        Ok((out, peak))
    }

    /// One IMEX step on the tensor-train conservative state: explicit convective predictor
    /// `U* = Uⁿ − Δt·∇·F`, then the implicit acoustic dissipation in closed form `Uⁿ⁺¹ = A₀⁻¹·U*` — the
    /// 2-D system analogue of `AcousticImex1d`. The flux/EOS are evaluated pointwise (dequantize → compute
    /// → requantize); `∇·F = ∂ₓF + ∂ᵧG` uses the metric's chain-rule `physical_gradient`.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if density leaves the positive cone; propagates flux /
    /// gradient / inverse-apply / rounding errors.
    pub fn step(&self, u: &EulerStateTt2d<R>) -> Result<EulerStateTt2d<R>, PhysicsError> {
        let dense: [Vec<R>; 4] = [
            dequantize_2d(&u[0], self.lx, self.ly)?.as_slice().to_vec(),
            dequantize_2d(&u[1], self.lx, self.ly)?.as_slice().to_vec(),
            dequantize_2d(&u[2], self.lx, self.ly)?.as_slice().to_vec(),
            dequantize_2d(&u[3], self.lx, self.ly)?.as_slice().to_vec(),
        ];
        let (f, g, _s_max) = self.flux_and_speed(&dense)?;
        Ok([
            self.step_component(&u[0], &f[0], &g[0])?,
            self.step_component(&u[1], &f[1], &g[1])?,
            self.step_component(&u[2], &f[2], &g[2])?,
            self.step_component(&u[3], &f[3], &g[3])?,
        ])
    }

    /// Advance one conserved component: explicit `−Δt·∇·F` predictor, then the implicit acoustic inverse.
    fn step_component(
        &self,
        uk: &CausalTensorTrain<R>,
        fk: &[R],
        gk: &[R],
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let neg = R::zero() - R::one();
        let fq = self.encode(fk)?;
        let gq = self.encode(gk)?;
        let (dfx, _) = self.metric.physical_gradient(&fq)?;
        let (_, dgy) = self.metric.physical_gradient(&gq)?;
        let div = dfx.add(&dgy)?;
        let predictor = uk.add(&div.scale(neg * self.dt))?.round(&self.trunc)?;
        Ok(self.acoustic_inv.apply(&predictor)?.round(&self.trunc)?)
    }

    /// The ratio of specific heats.
    pub fn gamma(&self) -> R {
        self.gamma
    }
}

impl<R, M> Marcher<R> for CompressibleMarcher2d<R, M>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    M: MetricProvider<R>,
{
    type State = EulerStateTt2d<R>;
    type Ambient = ();
    type Output = EulerStateTt2d<R>;

    fn advance(
        &self,
        state: &Self::State,
        _ambient: &Self::Ambient,
    ) -> Result<Self::Output, PhysicsError> {
        self.step(state)
    }
}

/// Ideal-gas pressure `p = (γ−1)(E − ½(ρu²+ρv²)/ρ)` from the 2-D conservative state `(ρ, ρu, ρv, ρE)`.
pub fn ideal_gas_pressure_2d<R: CfdScalar>(rho: R, mx: R, my: R, energy: R, gamma: R) -> R {
    let half = R::from_f64(0.5).unwrap_or_else(R::one);
    (gamma - R::one()) * (energy - half * (mx * mx + my * my) / rho)
}
