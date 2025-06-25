/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;

use crate::prelude::{Adjustable, AdjustmentError, LorentzianTime, UpdateError};

impl Adjustable<f64> for LorentzianTime {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let update_time = array_grid.get(p);

        // Update the internal time to the new time
        self.time_unit = update_time;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let time_adjustment = array_grid.get(p);

        if time_adjustment.is_nan() {
            return Err(AdjustmentError("Adjustment failed, time is NaN".into()));
        }

        // Check if the new time is non-negative. Unless you want to go back in time...
        if time_adjustment < f64::default() {
            return Err(AdjustmentError(
                "Adjustment failed, new time is NEGATIVE".into(),
            ));
        }

        // Calculate the data adjustment
        let adjusted_time = self.time_unit + time_adjustment;

        if !adjusted_time.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, result is not finite".into(),
            ));
        }

        // Check for errors i.e. div by zero / overflow and return either an error or OK().
        if adjusted_time < f64::default() {
            return Err(AdjustmentError(
                "Adjustment failed, result is a negative number".into(),
            ));
        }

        // Check if the new time is non-zero
        if adjusted_time == f64::default() {
            return Err(AdjustmentError(
                "Adjustment failed, new time is ZERO".into(),
            ));
        }

        // replace the internal time with the adjusted time
        self.time_unit = adjusted_time;

        Ok(())
    }
}
