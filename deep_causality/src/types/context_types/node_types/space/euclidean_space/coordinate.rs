// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::coord_match;
use crate::errors::IndexError;
use crate::prelude::{Coordinate, EuclideanSpace};

impl Coordinate<f64> for EuclideanSpace {
    /// Returns the number of dimensions in the coordinate system (always 3).
    fn dimension(&self) -> usize {
        3
    }

    /// Returns a reference to the coordinate value at the specified index.
    ///
    /// # Index Mapping
    /// - `0 => x`
    /// - `1 => y`
    /// - `2 => z`
    ///
    /// # Errors
    /// Returns `IndexError` if the index is out of bounds.
    ///
    fn coordinate(&self, index: usize) -> Result<&f64, IndexError> {
        coord_match!(index,
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
        )
    }
}
