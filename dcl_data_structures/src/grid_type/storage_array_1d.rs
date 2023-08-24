// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{PointIndex, Storage};

// T Type
// H Height
impl<T, const H: usize> Storage<T> for [T; H]
where
    T: Copy,
    [T; H]: Sized,
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.x]
    }

    fn set(&mut self, p: PointIndex, elem: T) {
        self[p.x] = elem
    }

    fn height(&self) -> Option<&usize> {
        Some(&H)
    }
}
