/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TangentSpacetime;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<R: RealField + FromPrimitive> TangentSpacetime<R> {
    pub fn x(&self) -> R {
        self.x
    }

    pub fn y(&self) -> R {
        self.y
    }

    pub fn z(&self) -> R {
        self.z
    }

    /// Returns position as [x, y, z]
    pub fn position(&self) -> [R; 3] {
        [self.x, self.y, self.z]
    }

    /// Returns velocity as [dt, dx, dy, dz]
    pub fn velocity(&self) -> [R; 4] {
        [self.dt, self.dx, self.dy, self.dz]
    }

    /// Returns the coordinate-time velocity (∂t/∂τ)
    pub fn time_velocity(&self) -> R {
        self.dt
    }

    /// Computes spatial velocity magnitude (ignoring dt)
    pub fn spatial_velocity(&self) -> R {
        (self.dx * self.dx + self.dy * self.dy + self.dz * self.dz).sqrt()
    }

    /// Returns 3D velocity vector
    pub fn velocity_vector(&self) -> [R; 3] {
        [self.dx, self.dy, self.dz]
    }

    /// Computes Euclidean spatial distance to another point
    pub fn euclidean_distance(&self, other: &Self) -> R {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
