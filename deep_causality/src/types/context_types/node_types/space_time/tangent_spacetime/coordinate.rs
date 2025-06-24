// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{Coordinate, TangentSpacetime};

impl Coordinate<f64> for TangentSpacetime {
    fn dimension(&self) -> usize {
        4
    }

    fn coordinate(&self, index: usize) -> &f64 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.t,
            _ => panic!("TangentBundleSpacetime: index out of bounds"),
        }
    }
}
