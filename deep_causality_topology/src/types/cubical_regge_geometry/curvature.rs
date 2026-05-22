/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Curvature, hinge enumeration, deficit angles, and the discrete Regge action for
//! `CubicalReggeGeometry<D, R>` — Phases R2 and R3.
//!
//! ## Dihedral angles on axis-aligned cubical cells
//!
//! Under the project-wide axis-aligned cubical assumption (every edge stored in
//! `EdgeLengths` runs along a coordinate axis, no shear), the two (D−1)-faces of a
//! top D-cube that share any (D−2)-hinge are mutually perpendicular regardless of
//! edge lengths. The dihedral angle is therefore exactly π/2 in every variant:
//! `UnitEdge`, `Uniform`, `PerAxis`, and `PerEdge`.
//!
//! This is a deliberate correction to the original design note
//! `openspec/notes/CubicalReggeCalculus.md` §3.R2, which proposed
//! `arctan2(lengths[j], lengths[i])` for the `PerAxis` case. The proposed formula
//! conflated the dihedral angle between faces with the half-angle of a stretched
//! cube's diagonal; for axis-aligned cubes those are different quantities and only
//! the constant `π/2` is geometrically meaningful as a dihedral.
//!
//! Curvature on a cubical lattice therefore enters only through:
//!
//! 1. **Incidence count of hinges.** Interior hinges in a periodic 2D/3D/4D lattice
//!    have 4 incident top cubes (sum 2π, deficit 0 — flat). Boundary hinges on open
//!    lattices have fewer (e.g. 1 at a 2D corner, sum π/2, deficit 3π/2 — intrinsic
//!    boundary curvature).
//! 2. **Sheared / non-axis-aligned cells**, which the current `EdgeLengths`
//!    representation does not express. A future Gram-matrix-aware extension would
//!    rewrite the dihedral closed forms here.
//!
//! See `openspec/changes/add-cubical-regge-calculus-core/tasks.md` §3 and §4.

use super::CubicalReggeGeometry;
use crate::types::lattice_complex::{LatticeCell, LatticeComplex};
use deep_causality_num::RealField;

impl<const D: usize, R: RealField> CubicalReggeGeometry<D, R> {
    /// Dihedral angle that `top_cube` contributes at `hinge`.
    ///
    /// On an axis-aligned cubical complex this is exactly π/2 = `R::pi() / 2`
    /// regardless of the edge-length variant. See the module documentation for why.
    /// The lattice argument is accepted for API symmetry with future sheared /
    /// Riemannian extensions, where the angle would depend on local Gram-matrix
    /// entries; current code does not read it.
    ///
    /// # Debug assertions
    ///
    /// In debug builds this asserts `top_cube.cell_dim() == D` and
    /// `hinge.cell_dim() == D - 2`. The function does not validate that the cube is
    /// actually incident to the hinge — callers (notably `regge_action` via
    /// `LatticeComplex::hinge_top_cube_neighbors`) are responsible for enumerating
    /// only incident pairs. For axis-aligned cubical cells, non-incident pairs still
    /// produce a geometrically meaningful constant (π/2), so silent misuse degrades
    /// to a constant rather than NaN.
    pub fn dihedral_angle(
        &self,
        _complex: &LatticeComplex<D, R>,
        top_cube: &LatticeCell<D>,
        hinge: &LatticeCell<D>,
    ) -> R {
        debug_assert!(D >= 2, "dihedral_angle requires D >= 2");
        debug_assert_eq!(
            top_cube.cell_dim(),
            D,
            "dihedral_angle: top_cube must be a D-cell (got grade {})",
            top_cube.cell_dim(),
        );
        debug_assert_eq!(
            hinge.cell_dim(),
            D - 2,
            "dihedral_angle: hinge must be a (D-2)-cell (got grade {})",
            hinge.cell_dim(),
        );

        let two = R::one() + R::one();
        R::pi() / two
    }
}
