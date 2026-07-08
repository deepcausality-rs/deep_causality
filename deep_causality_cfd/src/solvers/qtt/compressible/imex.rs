/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 3 — IMEX time integration with the D10 split-acoustic implicit step.
//!
//! The acoustic (fast pressure) mode is stiff at micrometre cells, and that stiffness is *acoustic*, not
//! source: orthogonal to the Tier-A LER cure. Treat it implicitly. Following design D10 and the
//! `studies/qtt_acoustic_precond` result, the implicit operator is **split**:
//!
//! ```text
//! A = I − Δt·κ·c²(x)·∂²  =  A₀ + A₁,
//!   A₀ = I − Δt·κ·c̄²·∂²   (constant-coefficient core — a known low-rank inverse, solved by AMEn)
//!   A₁ = −Δt·κ·(c²−c̄²)·∂²  (variable remainder — a bounded perturbation, lagged explicitly)
//! ```
//!
//! so the implicit solve is always against the well-conditioned constant-coefficient core. That core is
//! advanced by its **closed-form low-rank inverse** ([`AcousticCoreInverse`], design D10) — `A₀` factors
//! exactly through the cyclic shift and its inverse is applied in `O(l)` shift-applies with no iterative
//! solve, so the step is unconditionally robust and **free-stream-exact** (an AMEn-per-step solve loses
//! free-stream to its residual tolerance). This is the isolated 1-D acoustic operator that task 3.1 gates
//! *first*, before the full system coupling in the marcher. The model equation is
//! `u_t = −a·u_x + κ·c²(x)·u_xx`: explicit advection, split-implicit acoustic/diffusion.

use crate::tensor_bridge::{AcousticCoreInverse, dequantize, gradient, laplacian, quantize};
use crate::types::CfdScalar;
use alloc::format;
use alloc::vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

/// A 1-D IMEX integrator for `u_t = −a·u_x + κ·c²(x)·u_xx` (fixed `Δt`), with the stiff acoustic/diffusion
/// term advanced by the D10 split: constant-coefficient core implicit (closed-form inverse), variable
/// remainder lagged.
pub struct AcousticImex1d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    l: usize,
    dt: R,
    advect: R,
    kappa: R,
    grad: CausalTensorTrainOperator<R>,
    lap: CausalTensorTrainOperator<R>,
    /// `(I − Δt·κ·c̄²·∂²)⁻¹`, the closed-form inverse of the constant-coefficient implicit core.
    a0_inv: AcousticCoreInverse<R>,
    /// `c²(x)`, the full sound-speed-squared field.
    c2: CausalTensorTrain<R>,
    /// `c²(x) − c̄²`, the variable remainder field (lagged each step).
    dc2: CausalTensorTrain<R>,
    trunc: Truncation<R>,
}

impl<R> AcousticImex1d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the integrator on a periodic `2^l`-point grid of spacing `dx`, advection speed `advect`,
    /// stiff coefficient `kappa`, fixed timestep `dt`, and a sound-speed-squared field `c2` (length `2^l`).
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] on a wrong-length `c2`; propagates operator / codec / config
    /// errors.
    pub fn new(
        l: usize,
        dx: R,
        advect: R,
        kappa: R,
        dt: R,
        c2: &[R],
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let n = 1usize << l;
        if c2.len() != n {
            return Err(PhysicsError::DimensionMismatch(format!(
                "c2 length {} does not match grid 2^{l}",
                c2.len()
            )));
        }
        let grad = gradient::<R>(l, dx, &trunc)?;
        let lap = laplacian::<R>(l, dx, &trunc)?;

        let n_r = R::from_usize(n)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_usize(n)".into()))?;
        let cbar2 = c2.iter().fold(R::zero(), |a, &v| a + v) / n_r;
        // A₀ = I − Δt·κ·c̄²·∂² = (1+2s)·I − s·(S₊+S₋) with the dimensionless stiffness s = Δt·κ·c̄²/Δx².
        let s = dt * kappa * cbar2 / (dx * dx);
        let a0_inv = AcousticCoreInverse::new_1d(l, s, trunc)?;

        let c2_field = quantize(&CausalTensor::new(c2.to_vec(), vec![n])?, &trunc)?;
        let dc2_dense: vec::Vec<R> = c2.iter().map(|&v| v - cbar2).collect();
        let dc2 = quantize(&CausalTensor::new(dc2_dense, vec![n])?, &trunc)?;

        Ok(Self {
            l,
            dt,
            advect,
            kappa,
            grad,
            lap,
            a0_inv,
            c2: c2_field,
            dc2,
            trunc,
        })
    }

    /// One IMEX step: explicit advection + lagged variable remainder on the right, the constant-coefficient
    /// acoustic core advanced by its closed-form inverse `A₀⁻¹` (no iterative solve).
    ///
    /// `uⁿ⁺¹ = A₀⁻¹·(uⁿ − Δt·a·∂ₓuⁿ + Δt·κ·(c²−c̄²)·∂²ₓuⁿ)`.
    ///
    /// # Errors
    /// Propagates apply / rounding errors.
    pub fn step(&self, u: &CausalTensorTrain<R>) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let neg_a = R::zero() - self.advect;
        let conv = self.grad.apply(u, &self.trunc)?.scale(neg_a * self.dt);
        let lap_u = self.lap.apply(u, &self.trunc)?;
        let rem = self
            .dc2
            .hadamard_rounded(&lap_u, &self.trunc)?
            .scale(self.kappa * self.dt);
        let rhs = u.add(&conv)?.add(&rem)?.round(&self.trunc)?;
        self.a0_inv.apply(&rhs)
    }

    /// One fully-explicit step `uⁿ⁺¹ = uⁿ + Δt(−a·∂ₓu + κ·c²·∂²ₓu)` — the control that diverges beyond the
    /// explicit acoustic CFL, against which the IMEX step's stability is gated.
    ///
    /// # Errors
    /// Propagates apply / rounding errors.
    pub fn explicit_step(
        &self,
        u: &CausalTensorTrain<R>,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let neg_a = R::zero() - self.advect;
        let conv = self.grad.apply(u, &self.trunc)?.scale(neg_a);
        let lap_u = self.lap.apply(u, &self.trunc)?;
        let diff = self
            .c2
            .hadamard_rounded(&lap_u, &self.trunc)?
            .scale(self.kappa);
        let rate = conv.add(&diff)?;
        Ok(u.add(&rate.scale(self.dt))?.round(&self.trunc)?)
    }

    /// The grid size `2^l`.
    pub fn grid(&self) -> usize {
        1usize << self.l
    }
}

/// Conservation-preserving rounding (design D4): `round` minimizes Frobenius error, not the integral, and
/// the implicit solve carries its own residual, so a marched conservative field drifts its total. Carry the
/// conserved `target` total (the invariant from `t = 0`) and, after rounding, restore it with a **rank-1
/// uniform fixup** (`δ = (target − ∫after)/N` added as a constant field). For a single conserved scalar
/// (mass on a periodic grid) this pins the total to `target` with no secular drift, projecting out both the
/// rounding error and the solver residual each step.
///
/// # Errors
/// Propagates codec / rounding errors.
pub fn conservation_round<R>(
    u: &CausalTensorTrain<R>,
    target: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let rounded = u.round(trunc)?;
    let dense = dequantize(&rounded)?;
    let n = dense.as_slice().len();
    let total_after = dense.as_slice().iter().fold(R::zero(), |a, &v| a + v);
    let n_r = R::from_usize(n)
        .ok_or_else(|| PhysicsError::NumericalInstability("from_usize(n)".into()))?;
    let delta = (target - total_after) / n_r;
    let offset = quantize(&CausalTensor::new(vec![delta; n], vec![n])?, trunc)?;
    Ok(rounded.add(&offset)?.round(trunc)?)
}

/// Positivity limiter (task 3.3): clamp a field to a small positive `floor` (dequantize → `max(·, floor)`
/// → requantize). A pragmatic guard keeping `ρ, p > 0` through a strong rarefaction; the structural
/// upgrade is entropy / log-variable evolution (deferred).
///
/// # Errors
/// Propagates codec errors.
pub fn positivity_floor<R>(
    u: &CausalTensorTrain<R>,
    floor: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let dense = dequantize(u)?;
    let clamped: vec::Vec<R> = dense
        .as_slice()
        .iter()
        .map(|&v| if v > floor { v } else { floor })
        .collect();
    let n = clamped.len();
    quantize(&CausalTensor::new(clamped, vec![n])?, trunc)
}
