// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::ArrayGrid;

use crate::errors::{AdjustmentError, UpdateError};

pub trait Adjustable<T>
    where T: Copy + Default,
{
    /// The default implementation does nothing by default to keep update optional.
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>
    )
        -> Result<(), UpdateError>
    {
        let _ = array_grid.to_owned();

        // Example:
        //
        // You can access the data in the array_grid directly via index
        // Create a 1D PointIndex
        // let p = PointIndex::new1d(0);
        //
        // // get the data at the index position
        // let update_data = array_grid.get(p);
        //
        // // Check if the new data are okay to update
        // if update_data == 0 {
        //     return Err(UpdateError("Update failed, new data id ZERO".into()));
        // }
        //
        // // Update the internal data
        // self.data = update_data;

        Ok(())
    }

    /// The default implementation does nothing by default to keep adjustment optional.
    /// Override this method to implement a node adjustment when needed.
    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>,
    )
        -> Result<(), AdjustmentError>
    {
        // Depending on the type of node adjustment,
        // select a 1, 2,3, or 4 dimensional array grid that
        // contains the transformation data to apply to the node.
        let _ = array_grid.to_owned();

        // Example:
        //
        // // Create a 1D PointIndex
        // let p = PointIndex::new1d(0);
        //
        // // get the data at the index position
        // let new_data = array_grid.get(p);
        //
        // // Calculate the data adjustment
        // let adjusted_data = self.data + new_data;
        //
        // // Check for errors i.e. div by zero / overflow and return either an error or OK().
        // if adjusted_data < 0 {
        //     return Err(AdjustmentError("Adjustment failed, result is a negative number".into()));
        // }
        //
        // // replace the internal data with the adjusted data
        // self.data = adjusted_data;

        Ok(())
    }
}
