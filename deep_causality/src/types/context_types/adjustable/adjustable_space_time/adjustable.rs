use std::ops::Add;

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use dcl_data_structures::grid_type::ArrayGrid;

use crate::prelude::{Adjustable, AdjustmentError, UpdateError};

use super::*;

impl<T> Adjustable<T> for AdjustableSpaceTime<T>
where
    T: Copy + Default + Add<Output = T> + PartialOrd<i64>,
{
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        _array_grid: &ArrayGrid<T, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        _array_grid: &ArrayGrid<T, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        Ok(())
    }
}
