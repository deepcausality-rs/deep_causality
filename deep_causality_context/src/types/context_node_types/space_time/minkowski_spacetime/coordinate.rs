/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::IndexError;
use crate::{Coordinate, MinkowskiSpacetime};
use alloc::format;
use deep_causality_algebra::RealField;

impl<R: RealField> Coordinate for MinkowskiSpacetime<R> {
    type Coord = R;
    /// Returns the number of dimensions in the coordinate system (always 4).
    fn dimension(&self) -> usize {
        4
    }

    /// Returns a reference to the coordinate value at the specified index.
    ///
    /// # Index Mapping
    /// - `0 => t`
    /// - `1 => x`
    /// - `2 => y`
    /// - `3 => z`
    ///
    /// # Errors
    /// Returns `IndexError` if the index is out of bounds.
    ///
    fn coordinate(&self, index: usize) -> Result<&R, IndexError> {
        match index {
            0 => Ok(&self.t),
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
