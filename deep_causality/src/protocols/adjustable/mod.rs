// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::errors::{AdjustmentError, PropagateError, UpdateError};
use dcl_data_structures::prelude::ArrayGrid;

pub trait Adjustable
{
    /// The default implementation does nothing by default to keep update optional.
    fn update<T: Copy + Default>(
        data: T
    )
        -> Result<(), UpdateError>
    {
        let _ = data.to_owned();
        Ok(())
    }

    /// The default implementation does nothing by default to keep propagate optional.
    fn propagate() -> Result<(), PropagateError>
    {
        Ok(())
    }

    /// The default implementation does nothing by default to keep adjustment optional.
    /// Override this method to implement a node adjustment when needed.
    fn adjust<T: Copy + Default, const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>,
    )
        -> Result<(), AdjustmentError>
    {
        // Depending on the type of node adjustment,
        // select a 2,3, or 4 dimensional array grid that
        // contains the transformation data to apply to the node.
        let _ = array_grid.array_grid_2d();

        // Check for errors i.e. div by zero / overflow and return either an error or OK().
        Ok(())
    }
}
