// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{AdjustableEuclideanSpace, Coordinate};

impl Coordinate<f64> for AdjustableEuclideanSpace {
    fn dimension(&self) -> usize {
        3
    }

    fn coordinate(&self, index: usize) -> &f64 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("AdjustableEuclideanSpace: index out of bounds"),
        }
    }
}
