/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The no-slip wall constraint (the no-slip-viscous capability of
//! add-walls-and-dec-stencils).
//!
//! No-slip = zero tangential velocity at a wall. In DEC terms the velocity
//! 1-form's coefficients on **wall-tangential edges** are constrained to
//! zero. An edge running along axis `a` is tangential to a wall perpendicular
//! to a *different* non-periodic axis `w ≠ a` when it lies on that wall
//! (`position[w] ∈ {0, N_w − 1}`). Edges along a wall axis are *normal* to
//! that wall and are never constrained here — wall-normal flux is the
//! projection's Neumann condition (Group B), and boundary-crossing normal
//! edges do not exist on an open lattice (the complex already trims them).
//!
//! The constraint set feeds the **constrained Leray projector**
//! (`Manifold::leray_project_constrained_opts`): the M-orthogonal projection
//! onto the intersection of the divergence-free subspace with the no-slip
//! subspace `S`. The plain projector and the coordinate projector onto `S`
//! do not commute, so a post-hoc zeroing stage cannot keep both invariants;
//! the intersection projection keeps both exactly (tangential edges zero
//! *and* divergence at the solve's exactness — design D8, realized as a
//! constraint-aware projector rather than ghost-coefficient surgery).
//! Because the Hodge mass is diagonal and `P_S` only zeroes coordinates, the
//! restricted viscous operator `P_S Δ₁ P_S` is exactly M-symmetric — the
//! symmetry the CG / energy arguments need. The marching state stays in the
//! subspace: the seed, every projected stage rate, and the re-entry
//! projection all run constrained, and the step's explicit constraint stage
//! re-asserts exact zeros last.
//!
//! On a fully periodic lattice the constrained-edge set is empty and every
//! entry point below is a bit-exact no-op, preserving the periodic path.

use alloc::collections::BTreeSet;
use deep_causality_algebra::RealField;
use deep_causality_topology::{
    CutCellRegistry, CutConstraintKind, CutFaceConstraint, LatticeComplex,
};

/// The no-slip constraint set: binary edge pins (in `iter_cells(1)` /
/// cochain-coefficient order) plus, for an aperture-resolved immersed body,
/// the weighted cut-face rows. Empty on fully periodic lattices with no body.
#[derive(Debug, Clone)]
pub(in crate::solvers::dec) struct NoSlipConstraint<R: RealField> {
    edges: alloc::vec::Vec<usize>,
    rows: alloc::vec::Vec<CutFaceConstraint<R>>,
}

impl<R: RealField> NoSlipConstraint<R> {
    /// Enumerate the no-slip constraint of `complex`: the axis-aligned wall-tangential edges
    /// (binary pins), plus the immersed body's no-slip when a cut-cell registry is attached.
    ///
    /// When `aperture_resolved` and the registry has `Cut` cells, the body is aperture-resolved: the
    /// wall condition becomes the weighted cut-face rows ([`CutCellRegistry::cut_face_constraints`]),
    /// and the binary pins keep only the body's *interior* solid edges (the `solid_incident` set minus
    /// every edge a cut-face row already governs) so the body interior stays at rest without
    /// double-constraining the wetted cut face. With `aperture_resolved = false` (the validation
    /// comparison / fallback), or a registry with no `Cut` cells (an axis-aligned `Solid` layer), or
    /// no registry, the body falls back to the **staircase** set
    /// ([`CutCellRegistry::solid_incident_edges`]), so the axis-aligned and periodic paths are
    /// bit-unchanged.
    pub(in crate::solvers::dec) fn new<const D: usize>(
        complex: &LatticeComplex<D, R>,
        cut_registry: Option<&CutCellRegistry<D, R>>,
        aperture_resolved: bool,
    ) -> Self {
        let periodic = complex.periodic();
        let mut edges: alloc::vec::Vec<usize> = alloc::vec::Vec::new();

        // Axis-aligned wall-tangential edges (unchanged from Stage 3).
        if !periodic.iter().all(|&p| p) {
            let shape = complex.shape();
            for (idx, cell) in complex.iter_cells(1).enumerate() {
                // A grade-1 cell has exactly one active orientation bit: its axis.
                let axis = cell.orientation().trailing_zeros() as usize;
                let pos = cell.position();
                // Tangential to a wall ⟺ it sits on a boundary perpendicular to
                // some *other* (non-periodic) axis.
                let tangential = (0..D)
                    .any(|w| w != axis && !periodic[w] && (pos[w] == 0 || pos[w] + 1 == shape[w]));
                if tangential {
                    edges.push(idx);
                }
            }
        }

        // Immersed-body no-slip.
        let mut rows: alloc::vec::Vec<CutFaceConstraint<R>> = alloc::vec::Vec::new();
        if let Some(registry) = cut_registry {
            let all_rows = if aperture_resolved {
                registry.cut_face_constraints(complex)
            } else {
                // Staircase fallback / validation comparison: no weighted rows, pin the full
                // solid-incident edge ring (the Stage-4 B4 set).
                alloc::vec::Vec::new()
            };
            if all_rows.is_empty() {
                // No cut cells (axis-aligned solid layer / empty body): the staircase set, unchanged.
                edges.extend(registry.solid_incident_edges(complex));
            } else {
                // Aperture-resolved body. Keep only the **tangential** no-slip rows: the
                // no-penetration rows of a closed body are linearly dependent (the discrete
                // divergence-theorem identity `∮ n·u dA = 0`), which gives the KKT system a tiny
                // eigenvalue and floors the projection CG. They are also redundant — the body
                // interior is pinned to zero (the solid edges below) and the projection is
                // divergence-free, so the net flux through the body surface already vanishes. The
                // tangential rows (independent, well-conditioned) carry the wall condition that
                // sets separation (design open question 4.3: no-penetration row off).
                let row_edges: BTreeSet<usize> = all_rows
                    .iter()
                    .flat_map(|r| r.entries().iter().map(|&(e, _)| e))
                    .collect();
                rows = all_rows
                    .into_iter()
                    .filter(|r| r.kind() == CutConstraintKind::Tangential)
                    .collect();
                // Keep the solid pins that no cut-face row governs (the body interior); the wetted
                // cut face is carried by the tangential rows.
                edges.extend(
                    registry
                        .solid_incident_edges(complex)
                        .into_iter()
                        .filter(|e| !row_edges.contains(e)),
                );
            }
        }

        // Sort + dedup so the union is canonical and the masked-CG constraint set is unique;
        // a no-op on the wall-only path (already ascending, distinct) and the empty path.
        edges.sort_unstable();
        edges.dedup();
        Self { edges, rows }
    }

    /// The constrained edge indices (cochain-coefficient order). Empty ⟺
    /// fully periodic; every consumer (the constrained projector, the
    /// constraint stage) is a no-op on an empty slice, keeping the periodic
    /// path bit-unchanged.
    pub(in crate::solvers::dec) fn edges(&self) -> &[usize] {
        &self.edges
    }

    /// The aperture-resolved weighted cut-face rows. Empty unless the geometry carries a cut-cell
    /// registry with `Cut` cells; every consumer is a no-op (the binary path) on an empty slice.
    pub(in crate::solvers::dec) fn rows(&self) -> &[CutFaceConstraint<R>] {
        &self.rows
    }

    /// Remove a set of edges from the constraint (the free-slip **un-pin** seam): a free-slip face
    /// frees its wall-tangential edges, so they drop out of the no-slip set. A no-op when `slip` is
    /// empty (no slip face declared), preserving the no-slip / periodic paths bit-for-bit.
    pub(in crate::solvers::dec) fn remove_edges(&mut self, slip: &[usize]) {
        if slip.is_empty() {
            return;
        }
        self.edges.retain(|e| !slip.contains(e));
    }
}
