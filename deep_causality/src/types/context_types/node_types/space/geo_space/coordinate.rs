// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{Coordinate, GeoSpace};

impl Coordinate<f64> for GeoSpace {
    fn dimension(&self) -> usize {
        3
    }

    fn coordinate(&self, index: usize) -> &f64 {
        match index {
            0 => &self.lat,
            1 => &self.lon,
            2 => &self.alt,
            _ => panic!("GeoSpace: index out of bounds"),
        }
    }
}
