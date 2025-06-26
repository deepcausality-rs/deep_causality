/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::coord_match;
use crate::errors::IndexError;
use crate::prelude::{Coordinate, NedSpace};

impl Coordinate<f64> for NedSpace {
    /// Returns the number of dimensions in the coordinate system (always 3).
    fn dimension(&self) -> usize {
        3
    }

    /// Returns a reference to the coordinate value at the specified index.
    ///
    /// # Index Mapping
    /// - `0 => north`
    /// - `1 => east`
    /// - `2 => down`
    ///
    /// # Errors
    /// Returns `IndexError` if the index is out of bounds.
    ///
    fn coordinate(&self, index: usize) -> Result<&f64, IndexError> {
        coord_match!(index,
            0 => &self.north,
            1 => &self.east,
            2 => &self.down,
        )
    }
}
