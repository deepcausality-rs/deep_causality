/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 6 — the **body-fitted** 3-D compressible Euler marcher (design D1 generalised to 3-D).
//!
//! Identical conservative physics to the Cartesian [`CompressibleMarcher3d`](super::CompressibleMarcher3d)
//! — the five-train IMEX state `U = (ρ, ρu, ρv, ρw, ρE)`, the ideal-gas flux, and the closed-form implicit
//! acoustic step ([`AcousticCoreInverse3d`](crate::AcousticCoreInverse3d)) — but the explicit convective
//! flux divergence `∂ₓF + ∂ᵧG + ∂_zH` is taken through a [`MetricProvider3d`], i.e. by the chain-rule
//! **physical** gradient of the chosen curvilinear coordinate rather than raw Cartesian operators. Over
//! [`CartesianIdentity3d`](crate::CartesianIdentity3d) it reproduces the Cartesian marcher exactly; over
//! [`BodyFittedCoordinate3d`](crate::BodyFittedCoordinate3d) a bow shock standing off the nose is a
//! `ζ = const` surface, so the marched rank stays `O(10)` (the §1 lever, measured in `studies/qtt_rank_3d`).
//!
//! The acoustic dissipation is a computational-space regulariser (the same `AcousticCoreInverse3d` used by
//! the Cartesian marcher); a metric-weighted acoustic operator and the geometric conservation law for exact
//! free-stream preservation are the named Stage-2 refinements. Directional physical derivatives (one apply
//! per flux instead of three) are the perf follow-on.

use super::marcher_3d::{EulerState3d, EulerStateTt3d};
use crate::CfdScalar;
use crate::coordinate::{BodyFittedCoordinate3d, MetricProvider3d};
use crate::tensor_bridge::{AcousticCoreInverse3d, dequantize_3d, quantize_3d};
use crate::traits::Marcher;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::{ConjugateScalar, RealField};
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, TensorTrain, Truncation};

/// A body-fitted 3-D compressible Euler marcher, generic over the curvilinear coordinate `M`.
pub struct CompressibleMarcher3dFitted<R, M>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    M: MetricProvider3d<R>,
{
    metric: M,
    gamma: R,
    dt: R,
    lx: usize,
    ly: usize,
    lz: usize,
    acoustic_inv: AcousticCoreInverse3d<R>,
    trunc: Truncation<R>,
}

impl<R, M> CompressibleMarcher3dFitted<R, M>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    M: MetricProvider3d<R>,
{
    /// Build the marcher over coordinate `metric`, with reference computational spacing `dx` (setting the
    /// implicit acoustic dissipation `ν̄ = ½·s_ref·dx`), ratio of specific heats `gamma`, fixed step `dt`,
    /// and reference wave speed `s_ref` (`max(|u|+c)`).
    ///
    /// # Errors
    /// [`PhysicsError::NumericalInstability`] if `s_ref` is not finite and positive; propagates
    /// operator-assembly errors.
    pub fn new(
        metric: M,
        dx: R,
        gamma: R,
        dt: R,
        s_ref: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        if !s_ref.is_finite() || s_ref <= R::zero() {
            return Err(PhysicsError::NumericalInstability(
                "compressible marcher 3d (fitted): s_ref must be finite and positive".into(),
            ));
        }
        let (lx, ly, lz) = metric.dims();
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        let beta = dt * half * s_ref * dx;
        let acoustic_inv = AcousticCoreInverse3d::new((lx, ly, lz), (dx, dx, dx), beta, trunc)?;
        Ok(Self {
            metric,
            gamma,
            dt,
            lx,
            ly,
            lz,
            acoustic_inv,
            trunc,
        })
    }

    /// The ratio of specific heats.
    pub fn gamma(&self) -> R {
        self.gamma
    }

    /// The curvilinear coordinate this marcher runs over.
    pub fn metric(&self) -> &M {
        &self.metric
    }

    /// Swap the coordinate, keeping the (unchanged) acoustic inverse and step parameters. This is the
    /// per-step re-pin move: the lattice `dims`/`dx` and the acoustic dissipation are invariant across a
    /// re-pin, only the fitted metric slides.
    pub fn with_metric(self, metric: M) -> Self {
        Self { metric, ..self }
    }

    fn encode(&self, v: &[R]) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let (nx, ny, nz) = (1usize << self.lx, 1usize << self.ly, 1usize << self.lz);
        quantize_3d(
            &CausalTensor::new(v.to_vec(), alloc::vec![nx, ny, nz])?,
            &self.trunc,
        )
    }

    fn decode(&self, t: &CausalTensorTrain<R>) -> Result<Vec<R>, PhysicsError> {
        Ok(dequantize_3d(t, self.lx, self.ly, self.lz)?
            .as_slice()
            .to_vec())
    }

    /// Pointwise `(F, G, H)` flux components and the global LLF wave speed. Identical to the Cartesian
    /// marcher's flux (conservative ideal-gas Euler).
    #[allow(clippy::type_complexity)]
    fn flux_and_speed(
        &self,
        u: &[Vec<R>; 5],
    ) -> Result<([Vec<R>; 5], [Vec<R>; 5], [Vec<R>; 5], R), PhysicsError> {
        let n = u[0].len();
        let mk = || core::array::from_fn::<_, 5, _>(|_| Vec::with_capacity(n));
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
                    "compressible marcher 3d (fitted): density must stay positive".into(),
                ));
            }
            let (vx, vy, vz) = (mx / rho, my / rho, mz / rho);
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

    /// One IMEX step: explicit convective predictor `U* = Uⁿ − Δt·∇·F` (the divergence via the metric's
    /// **physical** gradient), then the implicit acoustic dissipation in closed form `Uⁿ⁺¹ = A₀⁻¹·U*`.
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

    /// Advance one conserved component: explicit `−Δt·(∂ₓF + ∂ᵧG + ∂_zH)` predictor (each physical
    /// derivative from the metric's chain-rule gradient), then the implicit acoustic inverse.
    fn step_component(
        &self,
        uk: &CausalTensorTrain<R>,
        fk: &[R],
        gk: &[R],
        hk: &[R],
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let neg = R::zero() - R::one();
        let dfx = self.metric.physical_gradient(&self.encode(fk)?)?.0;
        let dgy = self.metric.physical_gradient(&self.encode(gk)?)?.1;
        let dhz = self.metric.physical_gradient(&self.encode(hk)?)?.2;
        let div = dfx.add(&dgy)?.add(&dhz)?.round(&self.trunc)?;
        let predictor = uk.add(&div.scale(neg * self.dt))?.round(&self.trunc)?;
        Ok(self.acoustic_inv.apply(&predictor)?.round(&self.trunc)?)
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
        let out: EulerState3d<R> = [
            self.decode(&u[0])?,
            self.decode(&u[1])?,
            self.decode(&u[2])?,
            self.decode(&u[3])?,
            self.decode(&u[4])?,
        ];
        Ok((out, peak))
    }
}

impl<R, M> Marcher<R> for CompressibleMarcher3dFitted<R, M>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    M: MetricProvider3d<R>,
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

// ── Res-5 / D9 dynamic marched-rank re-pin (body-fitted shell only) ──────────────────────────────

/// Locate the radial front: the interior `ζ` index of steepest `(ξ, η)`-averaged gradient of a scalar
/// (density) train. `None` if the grid is too thin (`Nz < 5`).
fn front_index_zeta<R>(
    density: &CausalTensorTrain<R>,
    lx: usize,
    ly: usize,
    lz: usize,
) -> Result<Option<usize>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let (nx, ny, nz) = (1usize << lx, 1usize << ly, 1usize << lz);
    let dense = dequantize_3d(density, lx, ly, lz)?;
    let s = dense.as_slice();
    let mut prof = vec![R::zero(); nz];
    for i in 0..nx {
        for j in 0..ny {
            for (k, p) in prof.iter_mut().enumerate() {
                *p += s[(i * ny + j) * nz + k];
            }
        }
    }
    if nz < 5 {
        return Ok(None);
    }
    let mut kstar = None;
    let mut best = R::zero() - R::one();
    for k in 2..nz - 2 {
        let g = (prof[k + 1] - prof[k - 1]).abs();
        if g > best {
            best = g;
            kstar = Some(k);
        }
    }
    Ok(kstar)
}

/// Cyclically roll a train by `shift` cells along `ζ` (a rank-preserving relabel) and re-encode — the
/// move that keeps a tracked front coordinate-stationary.
fn roll_zeta<R>(
    u: &CausalTensorTrain<R>,
    lx: usize,
    ly: usize,
    lz: usize,
    shift: isize,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let (nx, ny, nz) = (1usize << lx, 1usize << ly, 1usize << lz);
    let dense = dequantize_3d(u, lx, ly, lz)?;
    let s = dense.as_slice();
    let mut rolled = vec![R::zero(); nx * ny * nz];
    for i in 0..nx {
        for j in 0..ny {
            for k in 0..nz {
                let src = ((k as isize - shift).rem_euclid(nz as isize)) as usize;
                rolled[(i * ny + j) * nz + k] = s[(i * ny + j) * nz + src];
            }
        }
    }
    quantize_3d(&CausalTensor::new(rolled, vec![nx, ny, nz])?, trunc)
}

impl<R> CompressibleMarcher3dFitted<R, BodyFittedCoordinate3d<R>>
where
    R: CfdScalar + ConjugateScalar<Real = R> + RealField,
{
    /// March with **Res-5 / D9 re-pinning**: each step, track the radial (density) front and, if it has
    /// drifted off the fixed computational band `target`, roll the state back to it (a rank-preserving
    /// relabel that keeps the feature coordinate-stationary) and slide the shell's inner radius `r0` so
    /// the front's physical radius maps to that band — rebuilding the fitted metric while reusing the
    /// invariant acoustic inverse. Returns the final dense state, the peak `max_bond`, and the re-pin count.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] on a wrong-length input; propagates step / re-pin errors.
    pub fn run_repinned(
        mut self,
        state0: &EulerState3d<R>,
        steps: usize,
        target: usize,
    ) -> Result<(EulerState3d<R>, usize, usize), PhysicsError> {
        let (lx, ly, lz) = (self.lx, self.ly, self.lz);
        let nz = 1usize << lz;
        let n = (1usize << lx) * (1usize << ly) * nz;
        for buf in state0.iter() {
            if buf.len() != n {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "state length {} does not match grid 2^{lx}·2^{ly}·2^{lz}",
                    buf.len()
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
        let mut n_repin = 0usize;
        for _ in 0..steps {
            u = self.step(&u)?;
            if let Some(kstar) = front_index_zeta(&u[0], lx, ly, lz)? {
                let shift = target as isize - kstar as isize;
                if shift != 0 {
                    for t in u.iter_mut() {
                        *t = roll_zeta(t, lx, ly, lz, shift, &self.trunc)?;
                    }
                    let frac = R::from_f64((kstar as f64 - target as f64) / nz as f64)
                        .unwrap_or_else(R::zero);
                    let new_r0 = self.metric.r0() + frac * self.metric.dr();
                    if new_r0 > R::zero() {
                        let coord = BodyFittedCoordinate3d::new(
                            lx,
                            ly,
                            lz,
                            new_r0,
                            self.metric.dr(),
                            self.metric.theta0(),
                            self.metric.dtheta(),
                            self.metric.phi0(),
                            self.metric.dphi(),
                            self.trunc,
                        )?;
                        self = self.with_metric(coord);
                    }
                    n_repin += 1;
                }
            }
            let step_peak = u.iter().map(|t| t.max_bond()).max().unwrap_or(0);
            if step_peak > peak {
                peak = step_peak;
            }
        }
        let out: EulerState3d<R> = [
            self.decode(&u[0])?,
            self.decode(&u[1])?,
            self.decode(&u[2])?,
            self.decode(&u[3])?,
            self.decode(&u[4])?,
        ];
        Ok((out, peak, n_repin))
    }
}
