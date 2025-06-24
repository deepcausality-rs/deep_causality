// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{AdjustableGeoSpace, Coordinate};

impl Coordinate<f64> for AdjustableGeoSpace {
    fn dimension(&self) -> usize {
        3
    }

    fn coordinate(&self, index: usize) -> &f64 {
        match index {
            0 => &self.lat,
            1 => &self.lon,
            2 => &self.alt,
            _ => panic!("AdjustableGeoSpace: index out of bounds"),
        }
    }
}
