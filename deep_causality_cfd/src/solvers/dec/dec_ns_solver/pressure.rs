/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Opt-in pressure recovery, both conventions.
//!
//! The diagnostic evaluates the unprojected RHS at the given state and
//! Leray-projects it — one CG solve, paid only when called, never inside
//! the step. With `ρ = 1`, the true dynamics is
//! `∂u/∂t = rhs_unproj − ∇(p + ½|u|²)`, and `∂u/∂t = P(rhs_unproj)` —
//! so the discarded gradient part satisfies `(I − P)rhs = +∇B`: the
//! Bernoulli pressure **is** the grade-0 potential, and the static
//! pressure subtracts the kinetic 0-form. Both come back mean-zero-gauged through the projection's gauge
//! fixing; absolute pressure on a torus is defined only up to a constant.

use alloc::format;
use alloc::vec::Vec;

use deep_causality_physics::PhysicsError;
use deep_causality_physics::PressureZeroForm;
use deep_causality_physics::SolenoidalField;
use deep_causality_physics::VelocityOneForm;

use super::DecNsSolver;
use crate::solvers::dec::DecNsScalar;

impl<const D: usize, R: DecNsScalar> DecNsSolver<'_, D, R> {
    /// Recovers `(bernoulli, static)` pressure 0-forms at the given state.
    ///
    /// `bernoulli` is `p + ½|u|²` (at `ρ = 1`) and `static` is `p`, both
    /// up to the additive gauge constant fixed by the projection's
    /// mean-zero convention. Costs one CG solve.
    ///
    /// # Errors
    /// * `PhysicsError::TopologyError` when the diagnostic projection's CG
    ///   does not converge, or `sharp` rejects the state.
    pub fn pressure_diagnostic(
        &self,
        state: &SolenoidalField<R>,
    ) -> Result<(PressureZeroForm<R>, PressureZeroForm<R>), PhysicsError> {
        // The gradient potential of the unprojected RHS at the current
        // state: rhs = dφ + solenoidal, with dφ = +∇B (see module doc).
        let u = VelocityOneForm::from_raw(state.as_one_form().clone());
        let (_projected_rhs, potential) = self
            .rate
            .eval_projected_with_potential(&u, &self.cg_options)
            .map_err(|e| {
                PhysicsError::TopologyError(format!("pressure diagnostic projection failed: {e}"))
            })?;

        // Bernoulli: B = φ.
        let bernoulli: Vec<R> = potential.as_slice().to_vec();

        // Kinetic 0-form ½|u|² from the sharp-recovered vertex vectors.
        let vertex_vectors = self
            .manifold
            .sharp(state.as_one_form())
            .map_err(|e| PhysicsError::TopologyError(format!("sharp failed: {e}")))?;
        let half = R::from_f64(0.5)
            // Coverage exemption: 0.5 lifts into every real field.
            .expect("0.5 lifts into R");
        let kinetic: Vec<R> = vertex_vectors
            .as_slice()
            .chunks_exact(D)
            .map(|v| v.iter().fold(R::zero(), |acc, x| acc + *x * *x) * half)
            .collect();

        // Static: p = B − ½|u|².
        let stat: Vec<R> = bernoulli
            .iter()
            .zip(kinetic.iter())
            .map(|(b, k)| *b - *k)
            .collect();

        let n0 = bernoulli.len();
        let bernoulli_tensor = deep_causality_tensor::CausalTensor::new(bernoulli, alloc::vec![n0])
            // Coverage exemption: a 1-D tensor of the potential's length
            // cannot fail to allocate.
            .expect("1-D tensor allocation cannot fail");
        let static_tensor = deep_causality_tensor::CausalTensor::new(stat, alloc::vec![n0])
            // Coverage exemption: as above.
            .expect("1-D tensor allocation cannot fail");

        let bernoulli_form = PressureZeroForm::new(bernoulli_tensor, self.manifold)?;
        let static_form = PressureZeroForm::new(static_tensor, self.manifold)?;
        Ok((bernoulli_form, static_form))
    }
}
