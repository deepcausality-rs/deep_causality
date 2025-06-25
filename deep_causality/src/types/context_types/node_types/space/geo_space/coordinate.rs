// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::coord_match;
use crate::errors::IndexError;
use crate::prelude::{Coordinate, GeoSpace};

impl Coordinate<f64> for GeoSpace {
    fn dimension(&self) -> usize {
        3
    }

    fn coordinate(&self, index: usize) -> Result<&f64, IndexError> {
        coord_match!(index,
            0 => &self.lat,
            1 => &self.lon,
            2 => &self.alt,
        )
    }
}
