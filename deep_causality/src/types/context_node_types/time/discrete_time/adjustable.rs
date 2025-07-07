/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;

use crate::errors::{AdjustmentError, UpdateError};
use crate::{Adjustable, DiscreteTime};

impl Adjustable<u64> for DiscreteTime {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<u64, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let update_time = array_grid.get(p);

        // Update the internal time to the new time
        self.tick_unit = update_time;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<u64, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Create a 1D PointIndex
        let p = PointIndex::new1d(0);

        // get the data at the index position
        let time_adjustment = array_grid.get(p);

        // Calculate checked data adjustment
        let Some(adjusted_time) = self.tick_unit.checked_add(time_adjustment) else {
            return Err(AdjustmentError("Adjustment failed, u64 overflow".into()));
        };

        // Check if the new time is non-zero
        if adjusted_time == u64::default() {
            return Err(AdjustmentError(
                "Adjustment failed, new time is ZERO".into(),
            ));
        }

        // replace the internal time with the adjusted time
        self.tick_unit = adjusted_time;

        Ok(())
    }
}
