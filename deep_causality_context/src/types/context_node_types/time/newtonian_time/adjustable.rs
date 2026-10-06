/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{ArrayGrid, PointIndex};

use crate::errors::{AdjustmentError, UpdateError};
use crate::{Adjustable, NewtonianTime};
use deep_causality_algebra::RealField;

// `Default` is not implied by `RealField`; `ArrayGrid<T, ..>` requires it to initialise
// its backing array, so it is bounded here rather than on the struct.
impl<R: RealField + Default> Adjustable<R> for NewtonianTime<R> {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let update_time = array_grid.get(p);

        if !update_time.is_finite() {
            return Err(UpdateError(
                "Update failed, new time is not a finite value".into(),
            ));
        }

        // Update the internal time to the new time
        self.time_unit = update_time;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let time_adjustment = array_grid.get(p);

        if time_adjustment.is_nan() {
            return Err(AdjustmentError("Adjustment failed, time is NaN".into()));
        }

        // Calculate the data adjustment
        let adjusted_time = self.time_unit + time_adjustment;

        // Reject non-finite results (NaN, ±inf)
        if !adjusted_time.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, result is not finite (NaN or Inf)".into(),
            ));
        }

        // replace the internal time with the adjusted time
        self.time_unit = adjusted_time;

        Ok(())
    }
}
