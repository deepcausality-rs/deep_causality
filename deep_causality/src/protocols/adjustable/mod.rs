// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::ArrayGrid;

use crate::errors::{AdjustmentError, UpdateError};

pub trait Adjustable<T> where T: Copy + Default,
{
    /// The default implementation does nothing to keep update optional.
    /// Override this method to implement a node update when needed.
    /// For a sample implementation, see src/types/context_types/adjustable
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self, _array_grid: &ArrayGrid<T, W, H, D, C>, ) -> Result<(), UpdateError> {
        Ok(())
    }

    /// The default implementation does nothing to keep adjustment optional.
    /// Override this method to implement a node adjustment when needed.
    /// Depending on the type of node adjustment, select a 1, 2,3, or 4 dimensional array grid
    /// that contains the transformation data to apply to the node.
    /// For a sample implementation, see src/types/context_types/adjustable
    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self, _array_grid: &ArrayGrid<T, W, H, D, C>, ) -> Result<(), AdjustmentError> {
        Ok(())
    }
}
