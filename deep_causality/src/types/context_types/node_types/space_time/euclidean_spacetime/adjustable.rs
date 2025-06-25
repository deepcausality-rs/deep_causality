/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::{AdjustmentError, UpdateError};
use crate::prelude::{Adjustable, EuclideanSpacetime};
use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;

impl Adjustable<f64> for EuclideanSpacetime {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 3D PointIndex for each of the updated x,y,z coordinates
        let p1 = PointIndex::new3d(0, 0, 0);
        let p2 = PointIndex::new3d(0, 0, 1);
        let p3 = PointIndex::new3d(0, 0, 2);
        let p4 = PointIndex::new3d(0, 0, 3);

        // get the data at the index position
        let new_x = array_grid.get(p1);
        let new_y = array_grid.get(p2);
        let new_z = array_grid.get(p3);
        let new_t = array_grid.get(p4);

        if !new_x.is_finite() {
            return Err(UpdateError(
                "Update failed, new X is not a finite value ".into(),
            ));
        }

        if !new_y.is_finite() {
            return Err(UpdateError(
                "Update failed, new Y is not a finite value ".into(),
            ));
        }

        if !new_z.is_finite() {
            return Err(UpdateError(
                "Update failed, new Z is not a finite value ".into(),
            ));
        }

        if !new_t.is_finite() {
            return Err(UpdateError(
                "Update failed, new T is not a finite value ".into(),
            ));
        }

        // Check if the new data are non zero and good to update
        if new_x == f64::default() {
            return Err(UpdateError("Update failed, new X data is ZERO".into()));
        }

        if new_y == f64::default() {
            return Err(UpdateError("Update failed, new Y data is ZERO".into()));
        }

        if new_z == f64::default() {
            return Err(UpdateError("Update failed, new Z data is ZERO".into()));
        }
        if new_t == f64::default() {
            return Err(UpdateError("Update failed, new T data is ZERO".into()));
        }

        // Update the internal data
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
        self.t = new_t;

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
        let p4 = PointIndex::new3d(0, 0, 3);

        // get the data at the index position
        let new_x = array_grid.get(p1);
        let new_y = array_grid.get(p2);
        let new_z = array_grid.get(p3);
        let new_t = array_grid.get(p4);

        // Calculate the adjusted data by adding the new data to the current data
        let adjusted_x = self.x + new_x;
        let adjusted_y = self.y + new_y;
        let adjusted_z = self.z + new_z;
        let adjusted_t = self.t + new_t;

        // Check if the adjusted data are safe to update i.e. not greater than max f64 value
        if !adjusted_x.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new X is not a finite value ".into(),
            ));
        }

        if !adjusted_y.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new Y is not a finite value ".into(),
            ));
        }

        if !adjusted_z.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new Z is not a finite value ".into(),
            ));
        }

        if !adjusted_t.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new T is not a finite value ".into(),
            ));
        }

        // Update the internal data
        self.x = adjusted_x;
        self.y = adjusted_y;
        self.z = adjusted_z;
        self.t = adjusted_t;

        Ok(())
    }
}
