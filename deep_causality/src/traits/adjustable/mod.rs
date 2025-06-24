// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::ArrayGrid;

use crate::errors::{AdjustmentError, UpdateError};

pub trait Adjustable<T>
where
    T: Copy + Default,
{
    /// The default implementation does nothing to keep update optional.
    /// Override this method to implement a node update when needed.
    /// For a sample implementation, see src/types/context_types/node_types_adjustable
    fn update<const WIDTH: usize, const HEIGHT: usize, const DEPTH: usize, const TIME: usize>(
        &mut self,
        _array_grid: &ArrayGrid<T, WIDTH, HEIGHT, DEPTH, TIME>,
    ) -> Result<(), UpdateError> {
        Ok(())
    }

    /// The default implementation does nothing to keep adjustment optional.
    /// Override this method to implement a node adjustment when needed.
    /// Depending on the type of node adjustment, select a 1, 2,3, or 4 dimensional array grid
    /// that contains the transformation data to apply to the node.
    /// For a sample implementation, see src/types/context_types/node_types_adjustable
    fn adjust<const WIDTH: usize, const HEIGHT: usize, const DEPTH: usize, const TIME: usize>(
        &mut self,
        _array_grid: &ArrayGrid<T, WIDTH, HEIGHT, DEPTH, TIME>,
    ) -> Result<(), AdjustmentError> {
        Ok(())
    }
}
