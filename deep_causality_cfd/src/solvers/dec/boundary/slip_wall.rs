/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The free-slip / far-field boundary zone: no penetration, zero tangential shear.

use alloc::format;
use alloc::vec::Vec;

use deep_causality_topology::{LatticeComplex, Manifold};

use crate::solvers::dec::DecNsScalar;
use deep_causality_physics::PhysicsError;

use super::boundary_zone::BoundaryZone;

/// A free-slip (far-field) wall on the face perpendicular to `wall_axis` (the `max_side` face when
/// true, the zero face otherwise): **no penetration** (zero wall-normal flux — already the
/// projection's Neumann condition at a closed face) with a **free tangential** velocity (zero
/// shear). It un-pins the face's wall-tangential edges from the auto-derived no-slip set, so the
/// boundary-clipped viscous operator gives the zero-shear condition. It is the lateral boundary an
/// isolated body needs (a confining no-slip wall would impose a spurious boundary layer).
#[derive(Debug, Clone, Copy)]
pub struct SlipWall<const D: usize> {
    wall_axis: usize,
    max_side: bool,
}

impl<const D: usize> SlipWall<D> {
    /// A free-slip wall on `wall_axis` (`max_side` face).
    ///
    /// # Errors
    /// `PhysicsError::DimensionMismatch` when `wall_axis >= D`.
    pub fn new(wall_axis: usize, max_side: bool) -> Result<Self, PhysicsError> {
        if wall_axis >= D {
            return Err(PhysicsError::DimensionMismatch(format!(
                "SlipWall: wall axis {wall_axis} out of range for D = {D}"
            )));
        }
        Ok(Self {
            wall_axis,
            max_side,
        })
    }
}

impl<const D: usize, R: DecNsScalar> BoundaryZone<D, R> for SlipWall<D> {
    fn collect_slip_edges(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        out: &mut Vec<usize>,
    ) {
        let complex = manifold.complex();
        let pos = if self.max_side {
            complex.shape()[self.wall_axis] - 1
        } else {
            0
        };
        // The face's wall-tangential edges: oriented along an axis other than `wall_axis`, sitting
        // on this face. These are exactly the edges the auto no-slip would pin; free-slip frees them.
        for (idx, cell) in complex.iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            if axis != self.wall_axis && cell.position()[self.wall_axis] == pos {
                out.push(idx);
            }
        }
    }
}
