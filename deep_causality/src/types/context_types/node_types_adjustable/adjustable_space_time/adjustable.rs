use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Add;

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;

use crate::prelude::{Adjustable, AdjustmentError, UpdateError};

use super::*;

impl<T> Adjustable<T> for AdjustableSpaceTime<T>
where
    T: Debug
        + Default
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + PartialOrd,
{
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 4D PointIndex for each of the updated x,y,z coordinates plus time t
        let p1 = PointIndex::new4d(0, 0, 0, 0);
        let p2 = PointIndex::new4d(0, 0, 0, 1);
        let p3 = PointIndex::new4d(0, 0, 0, 2);
        let pt = PointIndex::new4d(0, 0, 0, 3);

        // get the data at the index position
        let new_x = array_grid.get(p1);
        let new_y = array_grid.get(p2);
        let new_z = array_grid.get(p3);
        let new_t = array_grid.get(pt);

        // Check if the new data are okay to update
        if new_x == T::default() {
            return Err(UpdateError("Update failed, new X data is ZERO".into()));
        }

        if new_y == T::default() {
            return Err(UpdateError("Update failed, new Y data is ZERO".into()));
        }

        if new_z == T::default() {
            return Err(UpdateError("Update failed, new Z data is ZERO".into()));
        }

        if new_t < T::default() {
            return Err(UpdateError(
                "Update failed, new T (Time) is Negative".into(),
            ));
        }

        // Update the internal data
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
        self.time_unit = new_t;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<T, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Create a 4D PointIndex for each of the updated x,y,z coordinates plus time t
        let p1 = PointIndex::new4d(0, 0, 0, 0);
        let p2 = PointIndex::new4d(0, 0, 0, 1);
        let p3 = PointIndex::new4d(0, 0, 0, 2);
        let pt = PointIndex::new4d(0, 0, 0, 3);

        // get the data at the index position
        let new_x = array_grid.get(p1);
        let new_y = array_grid.get(p2);
        let new_z = array_grid.get(p3);
        let new_t = array_grid.get(pt);

        // Calculate the adjusted data
        let adjusted_x = self.x + new_x;
        let adjusted_y = self.y + new_y;
        let adjusted_z = self.z + new_z;
        let adjusted_t = self.time_unit + new_t;

        // Check if the adjusted data are okay to update
        if adjusted_x < T::default() {
            return Err(AdjustmentError(
                "Adjustment failed, new X data is NEGATIVE".into(),
            ));
        }

        if adjusted_y < T::default() {
            return Err(AdjustmentError(
                "Adjustment failed, new Y data is NEGATIVE".into(),
            ));
        }

        if adjusted_z < T::default() {
            return Err(AdjustmentError(
                "Adjustment failed, new Z data is NEGATIVE".into(),
            ));
        }

        if adjusted_t < T::default() {
            return Err(AdjustmentError(
                "Adjustment failed, new T (Time) is Negative".into(),
            ));
        }

        // Replace the internal data with the adjusted data
        self.x = adjusted_x;
        self.y = adjusted_y;
        self.z = adjusted_z;
        self.time_unit = adjusted_t;

        Ok(())
    }
}
