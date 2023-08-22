// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use dcl_data_structures::grid_type::ArrayGrid;

use crate::errors::{AdjustmentError, UpdateError};
use crate::prelude::Adjustable;

use super::*;

impl<T> Adjustable<T> for AdjustableSpaceTime<T>
    where T: Copy + Default
{
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(&mut self, _array_grid: &ArrayGrid<T, W, H, D, C>) -> Result<(), UpdateError> {
        todo!()
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(&mut self, _array_grid: &ArrayGrid<T, W, H, D, C>) -> Result<(), AdjustmentError> {
        todo!()
    }
}
