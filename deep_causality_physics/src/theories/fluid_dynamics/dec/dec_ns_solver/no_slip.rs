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

use deep_causality_num::RealField;
use deep_causality_topology::LatticeComplex;

/// The set of wall-tangential edge indices (in `iter_cells(1)` /
/// cochain-coefficient order) that no-slip pins to zero. Empty on fully
/// periodic lattices.
#[derive(Debug, Clone)]
pub(in crate::theories::fluid_dynamics::dec) struct NoSlipConstraint {
    edges: alloc::vec::Vec<usize>,
}

impl NoSlipConstraint {
    /// Enumerate the wall-tangential edges of `complex`. Returns an empty
    /// constraint when no axis is walled.
    pub(in crate::theories::fluid_dynamics::dec) fn new<const D: usize, R: RealField>(
        complex: &LatticeComplex<D, R>,
    ) -> Self {
        let periodic = complex.periodic();
        if periodic.iter().all(|&p| p) {
            return Self {
                edges: alloc::vec::Vec::new(),
            };
        }
        let shape = complex.shape();
        let mut edges = alloc::vec::Vec::new();
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
        Self { edges }
    }

    /// The constrained edge indices (cochain-coefficient order). Empty ⟺
    /// fully periodic; every consumer (the constrained projector, the
    /// constraint stage) is a no-op on an empty slice, keeping the periodic
    /// path bit-unchanged.
    pub(in crate::theories::fluid_dynamics::dec) fn edges(&self) -> &[usize] {
        &self.edges
    }
}
