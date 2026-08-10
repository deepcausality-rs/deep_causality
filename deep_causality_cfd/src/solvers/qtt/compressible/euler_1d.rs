/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 2: a 1-D conservative compressible Euler marcher in quantized-tensor-train form, with
//! an ideal-gas EOS and a global Lax–Friedrichs flux (Rusanov with a global wave-speed estimate).
//!
//! The conservative state `U = (ρ, ρu, ρE)` is carried as three tensor trains. The flux update
//! `Uⁿ⁺¹ = Uⁿ − (Δt/Δx)(F_{i+½} − F_{i−½})` rearranges to a **conservative central flux difference plus
//! a scalar artificial viscosity**,
//!
//! ```text
//! dU/dt = −½(F_{i+1} − F_{i−1})/Δx + ½·s_max·(U_{i+1} − 2U_i + U_{i−1})/Δx
//!       = −∂ₓF  +  ½·s_max·Δx·∂²ₓU
//! ```
//!
//! so it is assembled from the §0 `gradient` / `laplacian` MPOs (conservative, telescoping) applied to
//! the flux and the state, recompressed each step. `s_max = max(|u| + c)` is one maximum over the whole
//! state, applied uniformly to every cell; it is not a per-interface estimate, so the dissipation is
//! that of a global Lax–Friedrichs flux. The nonlinear flux / EOS are evaluated pointwise (dequantize →
//! compute → requantize) — exact for the Sod gate; the rank-preserving TT-cross (`apply_nonlinear`)
//! form is the large-`L` upgrade.

use crate::CfdScalar;
use crate::tensor_bridge::{dequantize, gradient, laplacian, quantize};
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

/// Ideal-gas pressure `p = (γ−1)(E − ½ρu²) = (γ−1)(E − ½m²/ρ)` from the conservative state.
pub fn ideal_gas_pressure<R: CfdScalar>(rho: R, mom: R, energy: R, gamma: R) -> R {
    let half = R::from_f64(0.5).unwrap_or_else(R::one);
    (gamma - R::one()) * (energy - half * mom * mom / rho)
}

/// The 1-D conservative compressible Euler marcher (ideal gas + global Lax–Friedrichs flux) in QTT
/// form.
pub struct CompressibleEuler1d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    l: usize,
    dx: R,
    gamma: R,
    cfl: R,
    grad: CausalTensorTrainOperator<R>,
    lap: CausalTensorTrainOperator<R>,
    trunc: Truncation<R>,
}

/// One conservative state as three dense component buffers `(ρ, ρu, ρE)`.
pub type EulerState<R> = (Vec<R>, Vec<R>, Vec<R>);

impl<R> CompressibleEuler1d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the marcher for a periodic `2^L`-point grid of spacing `dx`, ratio of specific heats
    /// `gamma`, and CFL number `cfl` (≤ 1).
    ///
    /// The numerical envelope is validated here, matching the pattern the shock-fitting constructor
    /// and the QTT incompressible/immersed constructors already use: an out-of-envelope configuration
    /// is refused at construction rather than producing numbers that look like a solve.
    ///
    /// # Errors
    /// * [`PhysicsError::PhysicalInvariantBroken`] when `dx` is not finite and positive, `gamma` is not
    ///   finite and greater than 1, or `cfl` is not finite and in `(0, 1]`.
    /// * Propagates operator-assembly errors.
    pub fn new(
        l: usize,
        dx: R,
        gamma: R,
        cfl: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        if !dx.is_finite() || dx <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "CompressibleEuler1d: grid spacing dx must be finite and positive, got {dx:?}"
            )));
        }
        if !gamma.is_finite() || gamma <= R::one() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "CompressibleEuler1d: ratio of specific heats gamma must be finite and > 1, got {gamma:?}"
            )));
        }
        if !cfl.is_finite() || cfl <= R::zero() || cfl > R::one() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "CompressibleEuler1d: CFL number must be finite and in (0, 1], got {cfl:?}"
            )));
        }
        let grad = gradient::<R>(l, dx, &trunc)?;
        let lap = laplacian::<R>(l, dx, &trunc)?;
        Ok(Self {
            l,
            dx,
            gamma,
            cfl,
            grad,
            lap,
            trunc,
        })
    }

    /// The pointwise flux components `(ρu, ρu²+p, (E+p)u)` from the dense conservative state, and the
    /// global wave speed `s_max = max(|u| + c)`: one maximum over the whole grid.
    fn flux_and_speed(
        &self,
        rho: &[R],
        mom: &[R],
        energy: &[R],
    ) -> Result<(EulerState<R>, R), PhysicsError> {
        let n = rho.len();
        let mut f1 = Vec::with_capacity(n);
        let mut f2 = Vec::with_capacity(n);
        let mut f3 = Vec::with_capacity(n);
        let mut s_max = R::zero();
        for i in 0..n {
            let r = rho[i];
            if r <= R::zero() || !r.is_finite() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "compressible Euler: density must stay positive".into(),
                ));
            }
            let u = mom[i] / r;
            let p = ideal_gas_pressure(r, mom[i], energy[i], self.gamma);
            // Reject a non-hyperbolic state before it enters the flux (shared guard, all four
            // marchers). `p` is positive below, so the acoustic speed needs no floor.
            super::require_positive_pressure(p, i)?;
            let c = (self.gamma * p / r).sqrt();
            f1.push(mom[i]);
            f2.push(mom[i] * u + p);
            f3.push((energy[i] + p) * u);
            let speed = u.abs() + c;
            if speed > s_max {
                s_max = speed;
            }
        }
        Ok(((f1, f2, f3), s_max))
    }

    /// One Lax–Friedrichs component update `U ← round(U + Δt·(−∂ₓF + ½·s·Δx·∂²ₓU))`.
    fn update_component(
        &self,
        u: &CausalTensorTrain<R>,
        f: &CausalTensorTrain<R>,
        dt: R,
        diss_coeff: R,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let df = self.grad.apply(f, &self.trunc)?;
        let lap_u = self.lap.apply(u, &self.trunc)?;
        let neg = R::zero() - R::one();
        let rate = df.scale(neg).add(&lap_u.scale(diss_coeff))?;
        Ok(u.add(&rate.scale(dt))?.round(&self.trunc)?)
    }

    /// March the Sod-style conservative state to physical time `t_final` (adaptive dt under the CFL).
    /// Returns the dense `(ρ, ρu, ρE)`.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] on a wrong-length input; propagates step errors; positivity
    /// failure if the density goes non-positive.
    pub fn run(&self, state0: &EulerState<R>, t_final: R) -> Result<EulerState<R>, PhysicsError> {
        let n = 1usize << self.l;
        for buf in [&state0.0, &state0.1, &state0.2] {
            if buf.len() != n {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "state length {} does not match grid 2^{}",
                    buf.len(),
                    self.l
                )));
            }
        }
        let to_tt = |v: &[R]| -> Result<CausalTensorTrain<R>, PhysicsError> {
            quantize(&CausalTensor::new(v.to_vec(), alloc::vec![n])?, &self.trunc)
        };
        let mut rho = to_tt(&state0.0)?;
        let mut mom = to_tt(&state0.1)?;
        let mut energy = to_tt(&state0.2)?;

        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        let mut t = R::zero();
        let mut guard = 0usize;
        let max_steps = 1_000_000usize;
        while t < t_final && guard < max_steps {
            guard += 1;
            let rd = dequantize(&rho)?;
            let md = dequantize(&mom)?;
            let ed = dequantize(&energy)?;
            let ((f1, f2, f3), s_max) =
                self.flux_and_speed(rd.as_slice(), md.as_slice(), ed.as_slice())?;
            if s_max <= R::zero() || !s_max.is_finite() {
                return Err(PhysicsError::NumericalInstability(
                    "compressible Euler: non-physical wave speed".into(),
                ));
            }
            let mut dt = self.cfl * self.dx / s_max;
            if t + dt > t_final {
                dt = t_final - t;
            }
            let diss = half * s_max * self.dx;
            let f1t = to_tt(&f1)?;
            let f2t = to_tt(&f2)?;
            let f3t = to_tt(&f3)?;
            rho = self.update_component(&rho, &f1t, dt, diss)?;
            mom = self.update_component(&mom, &f2t, dt, diss)?;
            energy = self.update_component(&energy, &f3t, dt, diss)?;
            t += dt;
        }
        Ok((
            dequantize(&rho)?.as_slice().to_vec(),
            dequantize(&mom)?.as_slice().to_vec(),
            dequantize(&energy)?.as_slice().to_vec(),
        ))
    }

    /// The ratio of specific heats.
    pub fn gamma(&self) -> R {
        self.gamma
    }
}
