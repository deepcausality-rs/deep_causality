/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::coord_match;
use crate::errors::IndexError;
use crate::prelude::{Coordinate, LorentzianSpacetime};

impl Coordinate<f64> for LorentzianSpacetime {
    /// Returns the number of dimensions in the coordinate system (always 4).l
    fn dimension(&self) -> usize {
        4
    }

    /// Returns a reference to the coordinate value at the specified index.
    ///
    /// # Index Mapping
    /// - `0 => x`
    /// - `1 => y`
    /// - `2 => z`
    /// - `3 => t`
    ///
    /// # Errors
    /// Returns `IndexError` if the index is out of bounds.
    ///
    fn coordinate(&self, index: usize) -> Result<&f64, IndexError> {
        coord_match!(index,
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.t,
        )
    }
}
