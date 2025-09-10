/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::IndexError;
use crate::{Coordinate, QuaternionSpace};
use deep_causality_macros::coord_match;

impl Coordinate<f64> for QuaternionSpace {
    /// Returns the number of dimensions in the coordinate system (always 4).
    fn dimension(&self) -> usize {
        4
    }

    /// Returns a reference to the coordinate value at the specified index.
    ///
    /// # Index Mapping
    /// - `0 => w`
    /// - `1 => x`
    /// - `2 => y`
    /// - `3 => z`
    ///
    /// # Errors
    /// Returns `IndexError` if the index is out of bounds.
    ///
    fn coordinate(&self, index: usize) -> Result<&f64, IndexError> {
        coord_match!(index,
            0 => &self.w,
            1 => &self.x,
            2 => &self.y,
            3 => &self.z,
        )
    }
}
