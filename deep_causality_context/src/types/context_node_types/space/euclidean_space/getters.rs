/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EuclideanSpace;
use deep_causality_algebra::RealField;

impl<R: RealField> EuclideanSpace<R> {
    pub fn x(&self) -> R {
        self.x
    }

    pub fn y(&self) -> R {
        self.y
    }

    pub fn z(&self) -> R {
        self.z
    }
}
