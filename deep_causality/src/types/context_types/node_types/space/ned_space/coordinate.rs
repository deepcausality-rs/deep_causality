// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{Coordinate, NedSpace};

impl Coordinate<f64> for NedSpace {
    fn dimension(&self) -> usize {
        3
    }

    fn coordinate(&self, index: usize) -> &f64 {
        match index {
            0 => &self.north,
            1 => &self.east,
            2 => &self.down,
            _ => panic!("NedSpace: coordinate index out of bounds"),
        }
    }
}
