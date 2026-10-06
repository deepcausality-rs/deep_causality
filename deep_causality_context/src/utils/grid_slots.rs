/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{ArrayGrid, PointIndex};

/// The values at positions 0 to `N - 1` of `grid`.
///
/// Position `k` is `PointIndex::new1d(k)` of a 1D grid, `PointIndex::new2d(k, 0)` of a 2D grid,
/// `PointIndex::new3d(0, 0, k)` of a 3D grid and `PointIndex::new4d(0, 0, k, 0)` of a 4D grid:
/// along x for 1D and 2D, along z for 3D and 4D.
///
/// Panics, as [`ArrayGrid::get`] does, when the axis read holds fewer than `N` entries.
pub(crate) fn grid_slots<
    const N: usize,
    T,
    const W: usize,
    const H: usize,
    const D: usize,
    const C: usize,
>(
    grid: &ArrayGrid<T, W, H, D, C>,
) -> [T; N]
where
    T: Copy + Default,
{
    let position = |k| match grid {
        ArrayGrid::ArrayGrid1D(_) => PointIndex::new1d(k),
        ArrayGrid::ArrayGrid2D(_) => PointIndex::new2d(k, 0),
        ArrayGrid::ArrayGrid3D(_) => PointIndex::new3d(0, 0, k),
        ArrayGrid::ArrayGrid4D(_) => PointIndex::new4d(0, 0, k, 0),
    };
    core::array::from_fn(|k| grid.get(position(k)))
}
