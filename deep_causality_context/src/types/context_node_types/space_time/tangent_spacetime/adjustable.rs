/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::grid_slots::grid_slots;
use crate::{Adjustable, AdjustmentError, TangentSpacetime, UpdateError};
use deep_causality_algebra::RealField;
use deep_causality_data_structures::ArrayGrid;

/// Updates the event of the TangentSpacetime node. Grid positions 0 to 3 hold `t, x, y, z`, the
/// coordinate order. Position `k` is `PointIndex::new1d(k)` of a 1D grid,
/// `PointIndex::new2d(k, 0)` of a 2D grid, `PointIndex::new3d(0, 0, k)` of a 3D grid and
/// `PointIndex::new4d(0, 0, k, 0)` of a 4D grid. The metric tensor is updated through
/// [`MetricTensor4D::update_metric_tensor`](crate::MetricTensor4D::update_metric_tensor).
// `Default` is not implied by `RealField`; `ArrayGrid<T, ..>` requires it to initialise
// its backing array, so it is bounded here rather than on the struct.
impl<R: RealField + Default> Adjustable<R> for TangentSpacetime<R> {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        let [new_t, new_x, new_y, new_z] = grid_slots(array_grid);

        if !new_x.is_finite() {
            return Err(UpdateError(
                "Update failed, new X is not a finite value".into(),
            ));
        }

        if !new_y.is_finite() {
            return Err(UpdateError(
                "Update failed, new Y is not a finite value".into(),
            ));
        }

        if !new_z.is_finite() {
            return Err(UpdateError(
                "Update failed, new Z is not a finite value".into(),
            ));
        }

        if !new_t.is_finite() {
            return Err(UpdateError(
                "Update failed, new T is not a finite value".into(),
            ));
        }

        // Replace the internal data with the new data
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
        self.t = new_t;

        Ok(())
    }

    /// Adds grid positions 0 to 3 to `t, x, y, z`. The metric tensor is not adjusted.
    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        let [new_t, new_x, new_y, new_z] = grid_slots(array_grid);

        // Calculate the adjusted data by adding the new data to the current data
        let adjusted_x = self.x + new_x;
        let adjusted_y = self.y + new_y;
        let adjusted_z = self.z + new_z;
        let adjusted_t = self.t + new_t;

        // Reject non-finite adjusted coordinates (NaN, ±inf)
        if !adjusted_x.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, adjusted x value is not finite".into(),
            ));
        }

        if !adjusted_y.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, adjusted Y value is not finite".into(),
            ));
        }

        if !adjusted_z.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, adjusted z value is not finite".into(),
            ));
        }

        if !adjusted_t.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, adjusted t value is not finite".into(),
            ));
        }

        // Update the internal data with the adjusted data
        self.x = adjusted_x;
        self.y = adjusted_y;
        self.z = adjusted_z;
        self.t = adjusted_t;

        Ok(())
    }
}
