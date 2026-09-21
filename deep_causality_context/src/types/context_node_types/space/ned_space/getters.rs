/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NedSpace;
use deep_causality_algebra::RealField;

impl<R: RealField> NedSpace<R> {
    pub fn north(&self) -> R {
        self.north
    }

    pub fn east(&self) -> R {
        self.east
    }

    pub fn down(&self) -> R {
        self.down
    }
}
