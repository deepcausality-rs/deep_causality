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
        // The constrained projection is the closed-domain special case of the open-boundary
        // projection: constrained edges fixed to zero, no prescribed (inflow) edges, no outflow
        // reference. Because the fixed edges carry value zero, the full-mass divergence RHS agrees
        // with the masked-mass one, so this is bit-identical to the former standalone path.
        self.leray_project_open_opts(field, constrained_edges, &[], &[], opts)
    }

    /// The **open-boundary** Leray projection: the constrained projection generalized to admit a
    /// prescribed inflow (nonzero net boundary flux) under mixed boundary conditions.
    ///
    /// Three roles partition the edges/vertices:
    /// * `zeroed_edges` — fixed to zero (no-slip / wall-tangential); excluded from the free solve.
    /// * `prescribed_edges` — fixed to their current `field` value (a Dirichlet inflow); excluded
    ///   from the free solve, but their flux **is counted** in the divergence right-hand side.
    /// * `reference_vertices` — the outflow pressure reference (`φ = 0`): it fixes the Poisson
    ///   nullspace and lets the complementary (free) outflow flux adjust so the net flux leaves
    ///   there and mass is conserved (`∮ u·n = 0` over the divergence-free interior).
    ///
    /// Masking an inflow edge disconnects its outer (ghostless) inlet vertex from the interior in
    /// the free-edge graph; the divergence is therefore enforced only on the component reachable
    /// from the reference (a flood fill over the free edges). The disconnected inlet ring carries
    /// the prescribed velocity but its divergence is not constrained — exactly the open-boundary
    /// condition. A prescribed inflow with **no** reference is rejected (the net flux has nowhere
    /// to leave under the all-Neumann gauge).
    ///
    /// With all three empty this is the plain projection; with only `zeroed_edges` it is the
    /// constrained projection, bit-identically.
    ///
    /// # Errors
    /// * `TopologyError::DimensionMismatch` on a field-length mismatch.
    /// * `TopologyError::InvalidInput` for a missing metric, an out-of-range edge/vertex, a
    ///   fully-constrained domain, or prescribed (inflow) edges without a reference.
    /// * `TopologyError::HodgeDecompositionFailed` when the CG solve does not converge.
    pub fn leray_project_open_opts(
        &self,
        field: &CausalTensor<R>,
        zeroed_edges: &[usize],
        prescribed_edges: &[usize],
        reference_vertices: &[usize],
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<LerayProjection<R>, TopologyError> {
        if zeroed_edges.is_empty() && prescribed_edges.is_empty() && reference_vertices.is_empty() {
            return self.leray_project_opts(field, opts);
        }
        let n1 = self.complex.num_cells(1);
        if field.len() != n1 {
            return Err(TopologyError::DimensionMismatch(format!(
                "leray_project_open: expected {} grade-1 coefficients, got {}",
                n1,
                field.len()
            )));
        }
        let metric = self.metric.as_ref().ok_or_else(|| {
            TopologyError::InvalidInput(
                "leray_project_open requires a metric; construct the manifold \
                 with a metric attached"
                    .to_string(),
            )
        })?;
        if !prescribed_edges.is_empty() && reference_vertices.is_empty() {
            return Err(TopologyError::InvalidInput(
                "leray_project_open: a prescribed inflow requires a reference (outflow) face to \
                 balance the net flux"
                    .to_string(),
            ));
        }
        let tolerance = resolve_cg_tolerance(opts.tolerance)?;
        let max_iter = opts.max_iterations.unwrap_or(1000);
        let n0 = self.complex.num_cells(0);

        // Full grade-1 masses (for the divergence RHS, so prescribed flux counts) and the free
        // masses (the operator excludes every fixed edge — zeroed ∪ prescribed).
        let star1 = metric
            .hodge_star_matrix(self.complex(), 1)
            .map_err(|e| TopologyError::InvalidInput(format!("hodge star (grade 1): {e}")))?;
        let mass_full = super::stencil::build::star_diag(star1.as_ref(), n1);
        let mut mass_free = mass_full.clone();
        for &e in zeroed_edges.iter().chain(prescribed_edges.iter()) {
            if e >= n1 {
                return Err(TopologyError::InvalidInput(format!(
                    "leray_project_open: edge index {e} out of range ({n1} edges)"
                )));
            }
            mass_free[e] = R::zero();
        }

        // v: zero the zeroed edges; keep the prescribed (inflow) edges at their field value.
        let mut v: Vec<R> = field.as_slice().to_vec();
        for &e in zeroed_edges {
            v[e] = R::zero();
        }

        // Weighted divergence ∂₁(mass ⊙ ·) off the boundary CSR (mass selects free vs full).
        let boundary = self.complex.boundary_matrix(1);
        let ptr = boundary.row_indices().to_vec();
        let cols = boundary.col_indices().to_vec();
        let vals = boundary.values().to_vec();
        drop(boundary);
        let div_with = |mass: &[R], w: &[R]| -> Vec<R> {
            let mut out = vec![R::zero(); n0];
            for (i, o) in out.iter_mut().enumerate() {
                for e in ptr[i]..ptr[i + 1] {
                    let c = cols[e];
                    let term = mass[c] * w[c];
                    if vals[e] >= 0 {
                        *o += term;
                    } else {
                        *o -= term;
                    }
                }
            }
            out
        };
        // Jacobi diagonal from the free masses.
        let mut diag = vec![R::zero(); n0];
        for (i, d) in diag.iter_mut().enumerate() {
            for e in ptr[i]..ptr[i + 1] {
                *d += mass_free[cols[e]];
            }
        }

        // Reference mask and the free-edge connectivity. With a reference, only the component
        // reachable from it through free edges enforces the divergence; the disconnected inlet
        // ring (its sole interior link is the masked inflow edge) is left unconstrained.
        let mut is_ref = vec![false; n0];
        for &r in reference_vertices {
            if r >= n0 {
                return Err(TopologyError::InvalidInput(format!(
                    "leray_project_open: reference vertex {r} out of range ({n0} vertices)"
                )));
            }
            is_ref[r] = true;
        }
        let mut live = vec![reference_vertices.is_empty(); n0];
        if !reference_vertices.is_empty() {
            // Vertex adjacency over free edges, inverted from the vertex→edge CSR.
            let mut incident: Vec<Vec<usize>> = vec![Vec::new(); n1];
            for (i, inc) in ptr.windows(2).enumerate() {
                for e in inc[0]..inc[1] {
                    incident[cols[e]].push(i);
                }
            }
            let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n0];
            for (e, ends) in incident.iter().enumerate() {
                if mass_free[e] > R::zero() && ends.len() == 2 {
                    adj[ends[0]].push(ends[1]);
                    adj[ends[1]].push(ends[0]);
                }
            }
            let mut stack: Vec<usize> = reference_vertices.to_vec();
            for &r in reference_vertices {
                live[r] = true;
            }
            while let Some(u) = stack.pop() {
                for &w in &adj[u] {
                    if !live[w] {
                        live[w] = true;
                        stack.push(w);
                    }
                }
            }
        }

        // RHS = ∂ M₁ v (full masses ⇒ prescribed flux counts).
        let mut wrhs = div_with(&mass_full, &v);
        if reference_vertices.is_empty() {
            // Closed/constrained gauge: zero null rows, subtract the mean over the connected block.
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
                    "leray_project_open: every edge is constrained".to_string(),
                ));
            }
            let block_mean =
                block_sum / <R as FromPrimitive>::from_usize(block).expect("count lifts");
            for (r, d) in wrhs.iter_mut().zip(diag.iter()) {
                if *d != R::zero() {
                    *r -= block_mean;
                }
            }
        } else {
            // Open gauge: enforce divergence only on the live (reference-reachable) component; the
            // reference itself is pinned (φ = 0), so no mean subtraction.
            for (i, (r, d)) in wrhs.iter_mut().zip(diag.iter()).enumerate() {
                if !live[i] || is_ref[i] || *d == R::zero() {
                    *r = R::zero();
                }
            }
        }

        // A φ = ∂₁(M₁|_F ⊙ d₀ φ). With a reference, the pinned and non-live DOFs are symmetrically
        // eliminated (input and output rows zeroed ⇒ they stay at φ = 0 and never couple in).
        let eliminate = !reference_vertices.is_empty();
        let apply = |phi: &[R]| -> Vec<R> {
            let mut p = phi.to_vec();
            if eliminate {
                for (i, pv) in p.iter_mut().enumerate() {
                    if is_ref[i] || !live[i] {
                        *pv = R::zero();
                    }
                }
            }
            let d_phi = self.exterior_derivative_of(&p, 0);
            let mut grad = d_phi.into_vec();
            pad_or_truncate(&mut grad, n1);
            let mut out = div_with(&mass_free, &grad);
            if eliminate {
                for (i, o) in out.iter_mut().enumerate() {
                    if is_ref[i] || !live[i] {
                        *o = R::zero();
                    }
                }
            }
            out
        };
        let mut phi = deep_causality_sparse::cg_solve_preconditioned(
            apply, &diag, &wrhs, tolerance, max_iter,
        )
        .map_err(|f| {
            TopologyError::HodgeDecompositionFailed(format!(
                "open projection solve did not converge in {} iterations (final residual {})",
                f.iterations, f.residual
            ))
        })?;
        if reference_vertices.is_empty() {
            subtract_mean_in_place(&mut phi);
        } else {
            for (i, pv) in phi.iter_mut().enumerate() {
                if is_ref[i] || !live[i] {
                    *pv = R::zero();
                }
            }
        }

        // u = v − P_F dφ: gradient correction masked off every fixed edge.
        let d_phi = self.exterior_derivative_of(&phi, 0);
        let mut grad = d_phi.into_vec();
        pad_or_truncate(&mut grad, n1);
        for &e in zeroed_edges.iter().chain(prescribed_edges.iter()) {
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
