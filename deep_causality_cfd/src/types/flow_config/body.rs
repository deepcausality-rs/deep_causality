/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CfdScalar;

/// An immersed cut-cell body. The mesh clips the lattice against this primitive
/// (exact clipped volumes + wetted-face apertures) and merges sliver cut cells whose
/// fluid fraction falls below `merge_floor` (stabilization). Coordinates are in the
/// mesh's spacing units.
#[derive(Debug, Clone, Copy)]
pub struct Body<const D: usize, R: CfdScalar> {
    center: [R; D],
    radius: R,
    merge_floor: R,
}

impl<const D: usize, R: CfdScalar> Body<D, R> {
    /// A ball (a disk in 2D) of the given center and radius.
    pub fn disk(center: [R; D], radius: R) -> Self {
        Self {
            center,
            radius,
            merge_floor: R::zero(),
        }
    }

    /// Merge cut cells whose fluid fraction is below `fraction` into a neighbour
    /// (sliver-cell stabilization). Default zero (no merging).
    pub fn merge_floor(mut self, fraction: R) -> Self {
        self.merge_floor = fraction;
        self
    }

    pub(crate) fn center(&self) -> [R; D] {
        self.center
    }

    pub(crate) fn radius(&self) -> R {
        self.radius
    }

    pub(crate) fn merge_floor_value(&self) -> R {
        self.merge_floor
    }
}
