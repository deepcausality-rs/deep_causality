/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::{AdjustmentError, UpdateError};
use crate::utils::grid_slots::grid_slots;
use crate::{Adjustable, EuclideanSpace};
use deep_causality_algebra::RealField;
use deep_causality_data_structures::ArrayGrid;

// `Default` is not implied by `RealField`; `ArrayGrid<T, ..>` requires it to initialise its
// backing array, so it is bounded here rather than on the struct.
/// Grid positions 0 to 2 hold `x, y, z`. Position `k` is `PointIndex::new1d(k)` of a 1D grid,
/// `PointIndex::new2d(k, 0)` of a 2D grid, `PointIndex::new3d(0, 0, k)` of a 3D grid and
/// `PointIndex::new4d(0, 0, k, 0)` of a 4D grid.
impl<R: RealField + Default> Adjustable<R> for EuclideanSpace<R> {
    fn update<const WIDTH: usize, const HEIGHT: usize, const DEPTH: usize, const TIME: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, WIDTH, HEIGHT, DEPTH, TIME>,
    ) -> Result<(), UpdateError> {
        // get the data at the index position
        let [new_x, new_y, new_z] = grid_slots(array_grid);

        // Check if the adjusted data are safe to update i.e. finite in the working scalar
        if !new_x.is_finite() {
            return Err(UpdateError(
                "Update failed, new X value is not finite".into(),
            ));
        }

        if !new_y.is_finite() {
            return Err(UpdateError(
                "Update failed, new Y value is not finite".into(),
            ));
        }

        if !new_z.is_finite() {
            return Err(UpdateError(
                "Update failed, new Z value is not finite".into(),
            ));
        }

        // Update the internal data
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;

        Ok(())
    }

    fn adjust<const WIDTH: usize, const HEIGHT: usize, const DEPTH: usize, const TIME: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, WIDTH, HEIGHT, DEPTH, TIME>,
    ) -> Result<(), AdjustmentError> {
        // Get the data at the index position from the array grid
        let [new_x, new_y, new_z] = grid_slots(array_grid);

        // Calculate the adjusted data by adding the new data to the current data
        let adjusted_x = self.x + new_x;
        let adjusted_y = self.y + new_y;
        let adjusted_z = self.z + new_z;

        // Check if the adjusted data are safe to update i.e. finite in the working scalar
        if !adjusted_x.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new X value is not finite".into(),
            ));
        }

        if !adjusted_y.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new Y value is not finite".into(),
            ));
        }

        if !adjusted_z.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new Z value is not finite".into(),
            ));
        }

        // Update the internal data
        self.x = adjusted_x;
        self.y = adjusted_y;
        self.z = adjusted_z;

        Ok(())
    }
}
