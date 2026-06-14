/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The outflow boundary zone: a pressure-reference open face that balances the inflow net flux.

use alloc::format;
use alloc::vec::Vec;

use deep_causality_topology::{LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use crate::theories::fluid_dynamics::dec::DecNsScalar;

use super::boundary_zone::BoundaryZone;

/// An outflow boundary: the face perpendicular to `wall_axis` (the `max_side` face when true, the
/// zero face otherwise) is the open-boundary **pressure reference**. Its vertices pin `φ = 0` in
/// the projection, so the outflow velocity is free and adjusts to balance the inflow flux (mass
/// conservation). It carries no prescribed velocity — the outflow is determined by incompressibility
/// (a zero-gradient / convective outflow; the boundary time-update lands with the outflow group).
#[derive(Debug, Clone, Copy)]
pub struct Outflow<const D: usize> {
    wall_axis: usize,
    max_side: bool,
}

impl<const D: usize> Outflow<D> {
    /// An outflow reference on `wall_axis` (`max_side` face).
    ///
    /// # Errors
    /// `PhysicsError::DimensionMismatch` when `wall_axis >= D`.
    pub fn new(wall_axis: usize, max_side: bool) -> Result<Self, PhysicsError> {
        if wall_axis >= D {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Outflow: wall axis {wall_axis} out of range for D = {D}"
            )));
        }
        Ok(Self {
            wall_axis,
            max_side,
        })
    }
}

impl<const D: usize, R: DecNsScalar> BoundaryZone<D, R> for Outflow<D> {
    fn collect_reference_vertices(
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
        for (idx, cell) in complex.iter_cells(0).enumerate() {
            if cell.position()[self.wall_axis] == pos {
                out.push(idx);
            }
        }
    }
}
