/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PointIndex, Storage};

// T Type
// W Width
// H Height
// D Depth
/// Implements `Storage` for 3D arrays `[[[T; W]; H]; D]`
/// indexed along X (width), Y (height), and Z (depth) axes.
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
where
    T: Copy,
    [[[T; W]; H]; D]: Sized,
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.z][p.y][p.x]
    }

    fn set(&mut self, p: PointIndex, elem: T) {
        self[p.z][p.y][p.x] = elem
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
