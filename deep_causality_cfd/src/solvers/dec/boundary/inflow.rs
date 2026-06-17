/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The inflow boundary zone: a prescribed wall-normal Dirichlet velocity on an open face.

use alloc::format;
use alloc::vec::Vec;

use deep_causality_topology::{LatticeComplex, Manifold};

use crate::solvers::dec::DecNsScalar;
use deep_causality_physics::PhysicsError;

use super::boundary_zone::BoundaryZone;

/// An inflow boundary: the face perpendicular to `wall_axis` (the `max_side` face when true, the
/// zero face otherwise) carries a prescribed **wall-normal** velocity `speed`. It contributes the
/// face's normal edges as the prescribed (inflow) set — held at their lifted value with their flux
/// counted in the open-boundary projection — and the lift that sets that value. Requires a
/// matching [`Outflow`](super::outflow::Outflow) reference to balance the net flux.
#[derive(Debug, Clone, Copy)]
pub struct Inflow<const D: usize, R: DecNsScalar> {
    wall_axis: usize,
    max_side: bool,
    speed: R,
}

impl<const D: usize, R: DecNsScalar> Inflow<D, R> {
    /// An inflow on `wall_axis` (`max_side` face) at wall-normal `speed`.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` when `wall_axis >= D`.
    /// * `PhysicsError::NumericalInstability` when `speed` is not finite.
    pub fn new(wall_axis: usize, max_side: bool, speed: R) -> Result<Self, PhysicsError> {
        if wall_axis >= D {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Inflow: wall axis {wall_axis} out of range for D = {D}"
            )));
        }
        if !speed.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Inflow: speed must be finite".into(),
            ));
        }
        Ok(Self {
            wall_axis,
            max_side,
            speed,
        })
    }

    /// The position index of the face's normal-edge column.
    fn edge_column(&self, complex: &LatticeComplex<D, R>) -> usize {
        if self.max_side {
            complex.shape()[self.wall_axis] - 2
        } else {
            0
        }
    }
}

impl<const D: usize, R: DecNsScalar> BoundaryZone<D, R> for Inflow<D, R> {
    fn collect_lift(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        _step: usize,
        out: &mut Vec<(usize, R)>,
    ) {
        let complex = manifold.complex();
        if complex.periodic()[self.wall_axis] {
            return;
        }
        let Some(metric) = manifold.metric() else {
            return;
        };
        let col = self.edge_column(complex);
        for (idx, cell) in complex.iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            if axis != self.wall_axis || cell.position()[self.wall_axis] != col {
                continue;
            }
            let length = metric.cell_volume(complex, &cell);
            out.push((idx, self.speed * length));
        }
    }

    fn collect_prescribed_edges(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        out: &mut Vec<usize>,
    ) {
        let complex = manifold.complex();
        if complex.periodic()[self.wall_axis] {
            return;
        }
        let col = self.edge_column(complex);
        for (idx, cell) in complex.iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            if axis == self.wall_axis && cell.position()[self.wall_axis] == col {
                out.push(idx);
            }
        }
    }
}
