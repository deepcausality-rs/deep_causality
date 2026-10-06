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

    /// The position `[x, y, z]`, in metres.
    pub fn position(&self) -> [R; 3] {
        [self.x, self.y, self.z]
    }

    /// The tangent vector `[dt, dx, dy, dz]`, in the coordinate order.
    pub fn velocity(&self) -> [R; 4] {
        [self.dt, self.dx, self.dy, self.dz]
    }

    /// The tangent vector's time component `dt`.
    pub fn time_velocity(&self) -> R {
        self.dt
    }

    /// The tangent vector's spatial components `[dx, dy, dz]`.
    pub fn velocity_vector(&self) -> [R; 3] {
        [self.dx, self.dy, self.dz]
    }
}
