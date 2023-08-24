// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::ops::Add;

use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;

use crate::prelude::{Adjustable, AdjustmentError, UpdateError};

use super::*;

impl<T> Adjustable<T> for AdjustableSpace<T>
where
    T: Copy + Default + Add<Output = T> + PartialOrd<i64>,
{
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 3D PointIndex for each of the updated x,y,z coordinates
        let p1 = PointIndex::new3d(0, 0, 0);
        let p2 = PointIndex::new3d(0, 0, 1);
        let p3 = PointIndex::new3d(0, 0, 2);

        // get the data at the index position
        let new_x = array_grid.get(p1);
        let new_y = array_grid.get(p2);
        let new_z = array_grid.get(p3);

        // Check if the new data are okay to update
        if new_x == 0 {
            return Err(UpdateError("Update failed, new X data is ZERO".into()));
        }

        if new_y == 0 {
            return Err(UpdateError("Update failed, new Y data is ZERO".into()));
        }

        if new_z == 0 {
            return Err(UpdateError("Update failed, new Z data is ZERO".into()));
        }

        // Update the internal data
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        _array_grid: &ArrayGrid<T, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // // Create a 3D PointIndex for each of the updated x,y,z coordinates
        // let p1 = PointIndex::new3d(0, 0, 0);
        // let p2 = PointIndex::new3d(0, 0, 1);
        // let p3 = PointIndex::new3d(0, 0, 2);
        //
        // // Get the data at the index position
        // let new_x = array_grid.get(p1);
        // let new_y = array_grid.get(p2);
        // let new_z = array_grid.get(p3);
        //
        // // Calculate the adjusted data
        // let adjusted_x = self.x + new_x;
        // let adjusted_y = self.y + new_y;
        // let adjusted_z = self.z + new_z;
        //
        // // Check if the adjusted data are okay to update
        // if adjusted_x < 0 {
        //     return Err(AdjustmentError("Adjustment failed, new X data is NEGATIVE".into()));
        // }
        //
        // if adjusted_y < 0 {
        //     return Err(AdjustmentError("Adjustment failed, new Y data is NEGATIVE".into()));
        // }
        //
        // if adjusted_z < 0 {
        //     return Err(AdjustmentError("Adjustment failed, new Z data is NEGATIVE".into()));
        // }
        //
        // // Replace the internal data with the adjusted data
        // self.x = adjusted_x;
        // self.y = adjusted_y;
        // self.z = adjusted_z;

        Ok(())
    }
}
