/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The moving-wall boundary zone: a prescribed tangential wall velocity (the inhomogeneous
//! no-slip lift of Couette flow and the lid-driven cavity).

use alloc::format;
use alloc::vec::Vec;

use deep_causality_topology::{LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use crate::theories::fluid_dynamics::dec::DecNsScalar;

use super::boundary_zone::BoundaryZone;

/// A moving wall: the wall perpendicular to `wall_axis` (the `max_side` face when true, the zero
/// face otherwise) carries the tangential `velocity`. It contributes the prescribed lift (edge
/// integral `velocity[a]·edge length`) on that wall's tangential edges; those edges are already in
/// the wall's auto-derived no-slip set, so the projection holds the value exactly each step.
#[derive(Debug, Clone, Copy)]
pub struct MovingWall<const D: usize, R: DecNsScalar> {
    wall_axis: usize,
    max_side: bool,
    velocity: [R; D],
}

impl<const D: usize, R: DecNsScalar> MovingWall<D, R> {
    /// A moving wall on `wall_axis` (`max_side` face) with tangential `velocity`.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` when `wall_axis >= D`.
    /// * `PhysicsError::NumericalInstability` when the velocity is not finite.
    /// * `PhysicsError::PhysicalInvariantBroken` when the wall-normal velocity component is
    ///   non-zero (wall-normal flux is the projection's Neumann condition).
    pub fn new(wall_axis: usize, max_side: bool, velocity: [R; D]) -> Result<Self, PhysicsError> {
        if wall_axis >= D {
            return Err(PhysicsError::DimensionMismatch(format!(
                "MovingWall: wall axis {wall_axis} out of range for D = {D}"
            )));
        }
        if velocity.iter().any(|v| !v.is_finite()) {
            return Err(PhysicsError::NumericalInstability(
                "MovingWall: velocity must be finite".into(),
            ));
        }
        if velocity[wall_axis] != R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "MovingWall: the wall-normal velocity component (axis {wall_axis}) must be zero"
            )));
        }
        Ok(Self {
            wall_axis,
            max_side,
            velocity,
        })
    }
}

impl<const D: usize, R: DecNsScalar> BoundaryZone<D, R> for MovingWall<D, R> {
    fn collect_lift(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        _step: usize,
        out: &mut Vec<(usize, R)>,
    ) {
        let complex = manifold.complex();
        // A periodic axis has no wall to move; a metric is required for the edge lengths.
        if complex.periodic()[self.wall_axis] {
            return;
        }
        let Some(metric) = manifold.metric() else {
            return;
        };
        let shape = complex.shape();
        let wall_pos = if self.max_side {
            shape[self.wall_axis] - 1
        } else {
            0
        };
        for (idx, cell) in complex.iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            if axis == self.wall_axis
                || self.velocity[axis] == R::zero()
                || cell.position()[self.wall_axis] != wall_pos
            {
                continue;
            }
            let length = metric.cell_volume(complex, &cell);
            out.push((idx, self.velocity[axis] * length));
        }
    }
}
