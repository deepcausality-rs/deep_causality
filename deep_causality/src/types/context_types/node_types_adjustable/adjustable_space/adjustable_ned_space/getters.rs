// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::AdjustableNedSpace;

impl AdjustableNedSpace {
    pub fn north(&self) -> f64 {
        self.north
    }

    pub fn east(&self) -> f64 {
        self.east
    }

    pub fn down(&self) -> f64 {
        self.down
    }
}
