// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{Coordinate, AdjustableEuclideanSpace};


impl Coordinate<f64> for AdjustableEuclideanSpace {
    fn dimension(&self) -> usize {
        self.coords.len()
    }

    fn coordinate(&self, index: usize) -> &f64 {
        &self.coords[index] // panics if index ≥ 3 — per contract
    }
}