// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{PointIndex, Storage};

// T Type
// W Width
// H Height
// D Depth
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
    where
        T: Copy,
        [[[T; W]; H]; D]: Sized,
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]
    }

    fn set(&mut self, p: PointIndex, elem: T) {
        self[p.y][p.x][p.z] = elem
    }

    fn height(&self) -> Option<&usize> {
        Some(&H)
    }

    fn depth(&self) -> Option<&usize> {
        Some(&D)
    }

    fn width(&self) -> Option<&usize> {
        Some(&W)
    }
}