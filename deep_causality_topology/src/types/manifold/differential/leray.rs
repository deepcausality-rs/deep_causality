/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Leray projection: the divergence-free component of a 1-form via the
//! *half* Hodge decomposition.
//!
//! `P(ω) = ω − dφ` with `Δ₀ φ = δω`, gauge-fixed by mean subtraction — the
//! grade-0 Poisson solve only. The β-step of the full decomposition never
//! runs: the projector costs **one** CG solve per evaluation, and the
//! harmonic-kernel question of `Δ_{k+1}` on periodic lattices does not arise
//! on this path (see `cfd-gap.md` §2 of the `add-dec-solver-foundations`
//! change). On a torus the harmonic component (the mean flow) is
//! divergence-free and is retained unchanged by `ω − dφ`.
//!
//! The grade-0 potential is returned alongside the projected field
//! ([`crate::LerayProjection`]); the opt-in pressure-recovery diagnostic
//! consumes it downstream.

use core::fmt::{Debug, Display};

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

use crate::errors::topology_error::TopologyError;
use crate::traits::chain_complex::ChainComplex;
use crate::traits::has_hodge_star::HasHodgeStar;
use crate::types::hodge_decomposition::HodgeDecomposition;
use crate::types::leray_projection::LerayProjection;
use crate::types::manifold::Manifold;
use crate::types::manifold::differential::HodgeDecomposeOptions;
use crate::types::manifold::differential::hodge_decomposition_impl::{
    pad_or_truncate, resolve_cg_tolerance, solve_laplacian,
};
use crate::utils::cg_solver::subtract_mean_in_place;

impl<K, R> Manifold<K, R>
where
    K: ChainComplex + Clone,
    K::Metric: HasHodgeStar<R, Complex = K> + Clone,
    R: RealField + FromPrimitive + Default + PartialEq + Debug + Display,
{
    /// Leray projection of a 1-form with default tolerance and iteration
    /// budget: removes the gradient component, returning the divergence-free
    /// field and the grade-0 potential. See the module doc.
    ///
    /// # Errors
    /// * `TopologyError::DimensionMismatch` when `field` is not of length
    ///   `num_cells(1)`.
    /// * `TopologyError::InvalidInput` when the manifold has no metric.
    /// * `TopologyError::HodgeDecompositionFailed` when the grade-0 CG solve
    ///   does not converge within the iteration budget (or the caller
    ///   supplies a non-positive tolerance via the `_opts` variant).
    pub fn leray_project(
        &self,
        field: &CausalTensor<R>,
    ) -> Result<LerayProjection<R>, TopologyError> {
        self.leray_project_opts(field, &HodgeDecomposeOptions::default())
    }

    /// Leray projection with caller-supplied tolerance / iteration budget.
    /// See [`Self::leray_project`].
    pub fn leray_project_opts(
        &self,
        field: &CausalTensor<R>,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<LerayProjection<R>, TopologyError> {
        let n1 = self.complex.num_cells(1);
        if field.len() != n1 {
            return Err(TopologyError::DimensionMismatch(format!(
                "leray_project: expected {} grade-1 coefficients, got {}",
                n1,
                field.len()
            )));
        }
        if self.metric.is_none() {
            return Err(TopologyError::InvalidInput(
                "leray_project requires a metric; construct the manifold with a metric attached"
                    .to_string(),
            ));
        }

        let tolerance = resolve_cg_tolerance(opts.tolerance)?;
        let max_iter = opts.max_iterations.unwrap_or(1000);
        let n0 = self.complex.num_cells(0);

        // δω — the divergence source for the grade-0 Poisson solve.
        let omega_tensor = CausalTensor::new(field.as_slice().to_vec(), vec![n1])
            .expect("1-D tensor allocation cannot fail");
        let temp_omega = self.create_temp_manifold(1, omega_tensor);
        let delta_omega = temp_omega.codifferential(1);
        let mut rhs = delta_omega.as_slice().to_vec();
        pad_or_truncate(&mut rhs, n0);
        // Grade-0 gauge: constants are always harmonic (β₀ = 1); fix the
        // gauge by mean subtraction on both the RHS and the solution.
        subtract_mean_in_place(&mut rhs);

        let mut phi = solve_laplacian(self, 0, &rhs, tolerance, max_iter)?;
        subtract_mean_in_place(&mut phi);

        // P(ω) = ω − dφ.
        let phi_tensor =
            CausalTensor::new(phi.clone(), vec![n0]).expect("1-D tensor allocation cannot fail");
        let temp_phi = self.create_temp_manifold(0, phi_tensor);
        let d_phi = temp_phi.exterior_derivative(0);
        let mut grad = d_phi.as_slice().to_vec();
        pad_or_truncate(&mut grad, n1);

        let projected: Vec<R> = field
            .as_slice()
            .iter()
            .zip(grad.iter())
            .map(|(w, g)| *w - *g)
            .collect();

        let projected_t =
            CausalTensor::new(projected, vec![n1]).expect("1-D tensor allocation cannot fail");
        let potential_t =
            CausalTensor::new(phi, vec![n0]).expect("1-D tensor allocation cannot fail");

        Ok(LerayProjection::new(projected_t, potential_t))
    }

    /// Convenience agreement check used by the verification suite: the
    /// gradient part removed by [`Self::leray_project`] must match the exact
    /// (`dα`) component of the full [`Self::hodge_decompose`] on the same
    /// field. Exposed as a method so downstream crates can assert the
    /// half-vs-full consistency without reimplementing either side.
    ///
    /// Returns the maximum absolute difference between `ω − P(ω)` and the
    /// full decomposition's exact component.
    pub fn leray_vs_hodge_gradient_gap(
        &self,
        field: &CausalTensor<R>,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<R, TopologyError> {
        let projection = self.leray_project_opts(field, opts)?;
        let full: HodgeDecomposition<R> = self.hodge_decompose_opts(field, 1, opts)?;

        let gap = field
            .as_slice()
            .iter()
            .zip(projection.projected().as_slice().iter())
            .zip(full.exact().as_slice().iter())
            .map(|((w, p), a)| ((*w - *p) - *a).abs())
            .fold(R::zero(), |m, v| if v > m { v } else { m });

        Ok(gap)
    }
}
