/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{PointIndex, Storage};

// T Type
// W Width
// H Height
// D Depth
// C Chronos (Time) since T was already taken for Type T
impl<T, const W: usize, const H: usize, const D: usize, const C: usize> Storage<T>
    for [[[[T; W]; H]; D]; C]
where
    T: Copy,
    [[[[T; W]; H]; D]; C]: Sized,
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z][p.t]
    }

    fn set(&mut self, p: PointIndex, elem: T) {
        self[p.y][p.x][p.z][p.t] = elem
    }

    fn height(&self) -> Option<&usize> {
        Some(&H)
    }

    fn depth(&self) -> Option<&usize> {
        Some(&D)
    }

    fn time(&self) -> Option<&usize> {
        Some(&C)
    }

    fn width(&self) -> Option<&usize> {
        Some(&W)
    }
}
