use dcl_data_structures::grid_type::ArrayGrid;

use crate::errors::{AdjustmentError, UpdateError};
// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Adjustable;
use crate::types::context_types::adjustable::adjustable_space::AdjustableSpace;

impl<T> Adjustable<T> for AdjustableSpace<T>
    where T: Copy + Default
{
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(&mut self, _array_grid: &ArrayGrid<T, W, H, D, C>) -> Result<(), UpdateError> {
        todo!()
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(&mut self, _array_grid: &ArrayGrid<T, W, H, D, C>) -> Result<(), AdjustmentError> {
        todo!()
    }
}
