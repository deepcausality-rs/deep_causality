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
use deep_causality_par::MaybeParallel;

impl<K, R> Manifold<K, R>
where
    K: ChainComplex + Clone,
    K::Metric: HasHodgeStar<R, Complex = K> + Clone,
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug + Display,
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

        // δω — the divergence source for the grade-0 Poisson solve,
        // evaluated directly on the field (no temporary manifold).
        let delta_omega = self.codifferential_of(field.as_slice(), 1);
        let mut rhs = delta_omega.into_vec();
        pad_or_truncate(&mut rhs, n0);
        // Grade-0 gauge: constants are always harmonic (β₀ = 1). The RHS
        // kernel projection happens inside `solve_laplacian` on the
        // mass-weighted system (Euclidean subtraction here would break
        // M-consistency on boundary-clipped lattices); the solution's
        // gauge is fixed below.

        let mut phi = solve_laplacian(self, 0, &rhs, tolerance, max_iter)?;
        subtract_mean_in_place(&mut phi);

        // P(ω) = ω − dφ, with dφ evaluated directly on the potential.
        let d_phi = self.exterior_derivative_of(&phi, 0);
        let mut grad = d_phi.into_vec();
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

    /// The **constrained** Leray projection: the M-orthogonal projection
    /// onto the intersection of the divergence-free subspace with the
    /// essential-constraint subspace `S = {u : u|_E = 0}` for a caller-given
    /// edge set `E` (the no-slip wall-tangential edges of the
    /// no-slip-viscous capability).
    ///
    /// The plain projector and the coordinate projector onto `S` do not
    /// commute, so composing them post hoc cannot produce a field that is
    /// simultaneously divergence-free and constrained. The intersection
    /// projection follows from the KKT conditions: with `v = P_S(field)`,
    ///
    /// ```text
    /// u = v − P_F dφ,   (∂ M₁|_F d) φ = ∂ M₁ v
    /// ```
    ///
    /// where `M₁|_F` zeroes the constrained edges' masses and `P_F` masks
    /// the gradient correction to the free edges. Then `δu = 0` **and**
    /// `u|_E = 0`, both exactly: the constrained edges carry zero values, so
    /// the full (unmasked) divergence agrees with the masked one. The
    /// masked operator loses the per-axis separability of the spectral
    /// paths (wall rows drop their tangential coupling), so the solve runs
    /// Jacobi-preconditioned CG; the direct DCT/DFT solve remains the
    /// unconstrained dispatch (and the natural future preconditioner).
    ///
    /// Vertices whose every incident edge is constrained (all-walls box
    /// corners and, in 3D, lattice-edge vertices) yield structurally null
    /// rows: their rhs entries are zeroed and the gauge projection runs
    /// over the connected block, so CG never excites those coordinates.
    ///
    /// An empty `constrained_edges` delegates to
    /// [`Self::leray_project_opts`] (bit-identical periodic path).
    ///
    /// # Errors
    /// * `TopologyError::DimensionMismatch` on a field-length mismatch.
    /// * `TopologyError::InvalidInput` when the manifold has no metric or
    ///   an edge index is out of range.
    /// * `TopologyError::HodgeDecompositionFailed` when the CG solve does
    ///   not converge (or a non-positive tolerance is supplied).
    pub fn leray_project_constrained_opts(
        &self,
        field: &CausalTensor<R>,
        constrained_edges: &[usize],
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<LerayProjection<R>, TopologyError> {
        if constrained_edges.is_empty() {
            return self.leray_project_opts(field, opts);
        }
        let n1 = self.complex.num_cells(1);
        if field.len() != n1 {
            return Err(TopologyError::DimensionMismatch(format!(
                "leray_project_constrained: expected {} grade-1 coefficients, got {}",
                n1,
                field.len()
            )));
        }
        let metric = self.metric.as_ref().ok_or_else(|| {
            TopologyError::InvalidInput(
                "leray_project_constrained requires a metric; construct the manifold \
                 with a metric attached"
                    .to_string(),
            )
        })?;
        let tolerance = resolve_cg_tolerance(opts.tolerance)?;
        let max_iter = opts.max_iterations.unwrap_or(1000);
        let n0 = self.complex.num_cells(0);

        // Masked grade-1 masses: constrained edges drop out of the operator.
        let star1 = metric
            .hodge_star_matrix(self.complex(), 1)
            .map_err(|e| TopologyError::InvalidInput(format!("hodge star (grade 1): {e}")))?;
        let mut mass1 = super::stencil::build::star_diag(star1.as_ref(), n1);
        for &e in constrained_edges {
            if e >= n1 {
                return Err(TopologyError::InvalidInput(format!(
                    "leray_project_constrained: edge index {e} out of range ({n1} edges)"
                )));
            }
            mass1[e] = R::zero();
        }

        // v = P_S(field): the constraint applied to the input.
        let mut v: Vec<R> = field.as_slice().to_vec();
        for &e in constrained_edges {
            v[e] = R::zero();
        }

        // The masked weighted operator A = ∂₁ (M₁|_F ⊙ d₀ ·) and its
        // Jacobi diagonal diag[i] = Σ_{e∋i} M₁|_F[e] (incidence signs are
        // ±1), both off the boundary CSR.
        let boundary = self.complex.boundary_matrix(1);
        let ptr = boundary.row_indices().to_vec();
        let cols = boundary.col_indices().to_vec();
        let vals = boundary.values().to_vec();
        drop(boundary);
        let weighted_div = |w: &[R]| -> Vec<R> {
            let mut out = vec![R::zero(); n0];
            for (i, o) in out.iter_mut().enumerate() {
                for e in ptr[i]..ptr[i + 1] {
                    let c = cols[e];
                    let term = mass1[c] * w[c];
                    if vals[e] >= 0 {
                        *o += term;
                    } else {
                        *o -= term;
                    }
                }
            }
            out
        };
        let mut diag = vec![R::zero(); n0];
        for (i, d) in diag.iter_mut().enumerate() {
            for e in ptr[i]..ptr[i + 1] {
                *d += mass1[cols[e]];
            }
        }

        // RHS = ∂ M₁|_F v, with null rows (every incident edge masked)
        // zeroed and the consistency gauge taken over the connected block.
        let mut wrhs = weighted_div(&v);
        let mut block = 0usize;
        let mut block_sum = R::zero();
        for (r, d) in wrhs.iter_mut().zip(diag.iter()) {
            if *d == R::zero() {
                *r = R::zero();
            } else {
                block += 1;
                block_sum += *r;
            }
        }
        if block == 0 {
            return Err(TopologyError::InvalidInput(
                "leray_project_constrained: every edge is constrained".to_string(),
            ));
        }
        let block_mean = block_sum / <R as FromPrimitive>::from_usize(block).expect("count lifts");
        for (r, d) in wrhs.iter_mut().zip(diag.iter()) {
            if *d != R::zero() {
                *r -= block_mean;
            }
        }

        let apply = |phi: &[R]| -> Vec<R> {
            let d_phi = self.exterior_derivative_of(phi, 0);
            let mut grad = d_phi.into_vec();
            pad_or_truncate(&mut grad, n1);
            weighted_div(&grad)
        };
        let mut phi = deep_causality_sparse::cg_solve_preconditioned(
            apply, &diag, &wrhs, tolerance, max_iter,
        )
        .map_err(|f| {
            TopologyError::HodgeDecompositionFailed(format!(
                "constrained projection solve did not converge in {} iterations \
                 (final residual {})",
                f.iterations, f.residual
            ))
        })?;
        subtract_mean_in_place(&mut phi);

        // u = v − P_F dφ: gradient correction masked to the free edges.
        let d_phi = self.exterior_derivative_of(&phi, 0);
        let mut grad = d_phi.into_vec();
        pad_or_truncate(&mut grad, n1);
        for &e in constrained_edges {
            grad[e] = R::zero();
        }
        let projected: Vec<R> = v.iter().zip(grad.iter()).map(|(w, g)| *w - *g).collect();

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
