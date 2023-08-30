// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

// Grids in Rust, part 2: const generics
// https://blog.adamchalmers.com/grids-2/#benchmarks-1d-vs-2d-vec-vs-array

// Storage API:
// Single entry type Grid which is generic over its storage and implements for all traits.
// - Only expose a minimal subset to interact with the grid over the storage API
// - Implementing new storage types is a lot easier.
// - The disparity between different representations and what they implement is removed.
// https://github.com/petgraph/petgraph/issues/563

use crate::prelude::PointIndex;

pub trait Storage<T>
where
    T: Copy,
{
    fn get(&self, p: PointIndex) -> &T;
    fn set(&mut self, p: PointIndex, elem: T);
    fn height(&self) -> Option<&usize>;
    fn depth(&self) -> Option<&usize> {
        None
    }
    fn time(&self) -> Option<&usize> {
        None
    }
    fn width(&self) -> Option<&usize> {
        None
    }
}
