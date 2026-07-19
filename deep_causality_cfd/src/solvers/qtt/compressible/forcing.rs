/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! An optional **masked forcing region** on the compressible march path — the incompressible
//! Brinkman penalization ported to the 4-component conservative state (change
//! `plasma-retropulsion-de-risk`, capability `compressible-forcing-region`).
//!
//! A rank-bounded smoothed mask `χ ∈ [0, 1]` (see [`plume_mask_2d`](crate::plume_mask_2d) /
//! [`body_mask_2d`](crate::body_mask_2d)) selects a region of the grid; each step the conserved
//! state is relaxed toward a **target conserved state** inside it,
//! `Uₖ ← Uₖ − w·χ ⊙ (Uₖ − Tₖ)` per component (fused Hadamard product + round), with the blend
//! weight `w = min(Δt/η, 1)` set by the penalization strength `η` — `η ≤ Δt` is a hard
//! (Dirichlet-like) enforcement, larger `η` a soft relaxation. The exterior (`χ → 0`) is left to
//! evolve freely, so the outer flow forms its own standoff-shock response to the imprinted
//! obstruction; this is the post-step-relaxation insertion point (the carrier's
//! `enforce_inflow` precedent), chosen over a rate-level term for unconditional stability
//! (`w ≤ 1` is a convex blend).
//!
//! The de-risk harness drives a retro-plume through this seam (mask geometry from the Cordell
//! analytic plume-boundary kernel, target from the analytic jet state); the M3 production
//! `PlumeObstruction` stage fills the same seam.

use crate::CfdScalar;
use crate::solvers::qtt::compressible::marcher_2d::EulerStateTt2d;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensorTrain, TensorTrain, Truncation};

/// A masked forcing region over the 2-D compressible conservative state: a smoothed volume
/// fraction mask, the target conserved state `[ρ, ρu, ρv, ρE]` the interior is driven toward,
/// and the penalization strength `η` (time units of the solver step; `η ≤ Δt` enforces hard).
#[derive(Clone)]
pub struct ForcingRegion<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    mask: CausalTensorTrain<R>,
    target: [R; 4],
    eta: R,
}

impl<R> ForcingRegion<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// A forcing region driving the masked interior toward `target = [ρ, ρu, ρv, ρE]` with
    /// penalization strength `eta`.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if `eta` is not finite and positive, if any
    /// target component is not finite, or if the target density is not positive (the marcher
    /// rejects a non-positive density the moment it sees one, so a region must not inject it).
    pub fn new(mask: CausalTensorTrain<R>, target: [R; 4], eta: R) -> Result<Self, PhysicsError> {
        if !eta.is_finite() || eta <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ForcingRegion: penalization strength eta must be finite and positive".into(),
            ));
        }
        if target.iter().any(|t| !t.is_finite()) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ForcingRegion: every target component must be finite".into(),
            ));
        }
        if target[0] <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ForcingRegion: target density must be positive".into(),
            ));
        }
        Ok(Self { mask, target, eta })
    }

    /// The region's smoothed volume-fraction mask.
    pub fn mask(&self) -> &CausalTensorTrain<R> {
        &self.mask
    }

    /// The target conserved state `[ρ, ρu, ρv, ρE]`.
    pub fn target(&self) -> [R; 4] {
        self.target
    }

    /// The penalization strength `η`.
    pub fn eta(&self) -> R {
        self.eta
    }

    /// Apply one post-step relaxation toward the target inside the mask:
    /// `Uₖ ← Uₖ − w·χ ⊙ (Uₖ − Tₖ)` per conserved component, `w = min(dt/η, 1)`.
    ///
    /// # Errors
    /// Propagates the Hadamard / add / round errors of the train arithmetic.
    pub fn apply(
        &self,
        state: &EulerStateTt2d<R>,
        dt: R,
        trunc: &Truncation<R>,
    ) -> Result<EulerStateTt2d<R>, PhysicsError> {
        let ratio = dt / self.eta;
        let w = if ratio < R::one() { ratio } else { R::one() };
        let neg_w = R::zero() - w;
        let force =
            |u: &CausalTensorTrain<R>, t: R| -> Result<CausalTensorTrain<R>, PhysicsError> {
                // U − T (the deficit); for a zero target component this is just `U`.
                let deficit = if t == R::zero() {
                    u.clone()
                } else {
                    u.add_scalar(R::zero() - t)?
                };
                let masked = self.mask.hadamard_rounded(&deficit, trunc)?;
                Ok(u.add(&masked.scale(neg_w))?.round(trunc)?)
            };
        Ok([
            force(&state[0], self.target[0])?,
            force(&state[1], self.target[1])?,
            force(&state[2], self.target[2])?,
            force(&state[3], self.target[3])?,
        ])
    }
}
