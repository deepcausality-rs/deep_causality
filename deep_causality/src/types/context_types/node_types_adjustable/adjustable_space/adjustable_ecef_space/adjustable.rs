// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::errors::{AdjustmentError, UpdateError};
use crate::prelude::{Adjustable, AdjustableEcefSpace};
use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;
use std::f64;

impl Adjustable<f64> for AdjustableEcefSpace {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 3D PointIndex for each of the updated x,y,z coordinates
        let p1 = PointIndex::new3d(0, 0, 0);
        let p2 = PointIndex::new3d(0, 0, 1);
        let p3 = PointIndex::new3d(0, 0, 2);

        // Get the data at the index position from the array grid
        let new_x = array_grid.get(p1);
        let new_y = array_grid.get(p2);
        let new_z = array_grid.get(p3);

        // Replace the internal data with the new data
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Create a 3D PointIndex for each of the updated x,y,z coordinates
        let p1 = PointIndex::new3d(0, 0, 0);
        let p2 = PointIndex::new3d(0, 0, 1);
        let p3 = PointIndex::new3d(0, 0, 2);

        // Get the data at the index position from the array grid
        let new_x = array_grid.get(p1);
        let new_y = array_grid.get(p2);
        let new_z = array_grid.get(p3);

        // Calculate the adjusted data by adding the new data to the current data
        let adjusted_x = self.x + new_x;
        let adjusted_y = self.y + new_y;
        let adjusted_z = self.z + new_z;

        // Check if the adjusted data are safe to update i.e. not greater than max f64 value
        if adjusted_x > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new X data exceeds max f64 value ".into(),
            ));
        }

        if adjusted_y > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new Y data exceeds max f64 value ".into(),
            ));
        }

        if adjusted_z > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new Z data exceeds max f64 value ".into(),
            ));
        }

        // Update the internal data with the adjusted data
        self.x = adjusted_x;
        self.y = adjusted_y;
        self.z = adjusted_z;

        Ok(())
    }
}
