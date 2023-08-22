// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::Add;

use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;

use crate::errors::{AdjustmentError, UpdateError};
use crate::prelude::Adjustable;

use super::*;

impl<T> Adjustable<T> for AdjustableData<T>
    where T: Copy + Default + Add<Output=T> + PartialOrd<i32>,
{
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>,
    )
        -> Result<(), UpdateError>
    {

        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let update_data = array_grid.get(p);

        // Check if the new data are okay to update
        if update_data == 0 {
            return Err(UpdateError("Update failed, new data is ZERO".into()));
        }

        // Update the internal data
        self.data = update_data;

        Ok(())
    }


    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>,
    )
        -> Result<(), AdjustmentError>
    {
        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let new_data = array_grid.get(p);

        // Calculate the data adjustment
        let adjusted_data = self.data + new_data;

        // Check for errors i.e. div by zero / overflow and return either an error or OK().
        if adjusted_data < 0 {
            return Err(AdjustmentError("Adjustment failed, result is a negative number".into()));
        }

        // replace the internal data with the adjusted data
        self.data = adjusted_data;

        Ok(())
    }
}
