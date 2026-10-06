/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::{AdjustmentError, UpdateError};
use crate::utils::grid_slots::grid_slots;
use crate::{Adjustable, NedSpace};
use deep_causality_algebra::RealField;
use deep_causality_data_structures::ArrayGrid;

// `Default` is not implied by `RealField`; `ArrayGrid<T, ..>` requires it to initialise
// its backing array, so it is bounded here rather than on the struct.
/// Grid positions 0 to 2 hold north, east and down. Position `k` is `PointIndex::new1d(k)` of a
/// 1D grid, `PointIndex::new2d(k, 0)` of a 2D grid, `PointIndex::new3d(0, 0, k)` of a 3D grid and
/// `PointIndex::new4d(0, 0, k, 0)` of a 4D grid.
impl<R: RealField + Default> Adjustable<R> for NedSpace<R> {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Get the data at the index position from the array grid
        let [new_north, new_east, new_down] = grid_slots(array_grid);

        // Reject non-finite replacement coordinates (NaN, ±inf)
        if !new_north.is_finite() {
            return Err(UpdateError(
                "Update failed, new north value is not finite".into(),
            ));
        }

        if !new_east.is_finite() {
            return Err(UpdateError(
                "Update failed, new east value is not finite".into(),
            ));
        }

        if !new_down.is_finite() {
            return Err(UpdateError(
                "Update failed, new down value is not finite".into(),
            ));
        }

        // Replace the internal data with the new data
        self.north = new_north;
        self.east = new_east;
        self.down = new_down;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Get the data at the index position from the array grid
        let [new_north, new_east, new_down] = grid_slots(array_grid);

        // Calculate the adjusted data by adding the new data to the current data
        let adjusted_north = self.north + new_north;
        let adjusted_east = self.east + new_east;
        let adjusted_down = self.down + new_down;

        // Reject non-finite adjusted coordinates (NaN, ±inf)
        if !adjusted_north.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new north value is not finite".into(),
            ));
        }

        if !adjusted_east.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new east value is not finite".into(),
            ));
        }

        if !adjusted_down.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new down value is not finite".into(),
            ));
        }

        // Update the internal data with the adjusted data
        self.north = adjusted_north;
        self.east = adjusted_east;
        self.down = adjusted_down;

        Ok(())
    }
}
