/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::IndexError;
use crate::{Coordinate, QuaternionSpace};

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
        match index {
            0 => Ok(&self.w),
            1 => Ok(&self.x),
            2 => Ok(&self.y),
            3 => Ok(&self.z),
            _ => Err(IndexError(format!(
                "Coordinate index out of bounds: {}",
                index
            ))),
        }
    }
}
