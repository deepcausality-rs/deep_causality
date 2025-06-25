/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod grid;
#[cfg(not(feature = "unsafe"))]
pub mod grid_safe;
#[cfg(feature = "unsafe")]
pub mod grid_unsafe;
pub mod point;
pub mod storage;
pub mod storage_array_1d;
pub mod storage_array_2d;
pub mod storage_array_3d;
pub mod storage_array_4d;

use std::fmt::{Debug, Display, Formatter};

pub use point::PointIndexType;

use crate::prelude::{Grid, PointIndex};

#[derive(Debug, Copy, Clone)]
pub enum ArrayType {
    Array1D,
    Array2D,
    Array3D,
    Array4D,
}

type GridType<T, const W: usize, const H: usize, const D: usize, const C: usize> =
    Grid<[[[[T; W]; H]; D]; C], T>;

// ArrayGrid is a wrapper around Grid that provides a more convenient API
// for working with arrays of different dimensions.
#[derive(Debug)]
pub enum ArrayGrid<T, const W: usize, const H: usize, const D: usize, const C: usize>
where
    T: Copy + Default,
{
    ArrayGrid1D(Grid<[T; H], T>),
    ArrayGrid2D(Grid<[[T; W]; H], T>),
    ArrayGrid3D(Grid<[[[T; W]; H]; D], T>),
    ArrayGrid4D(GridType<T, W, H, D, C>),
}

impl<T, const W: usize, const H: usize, const D: usize, const C: usize> ArrayGrid<T, W, H, D, C>
where
    T: Copy + Default,
{
    #[inline(always)]
    /// Returns a new ArrayGrid with the ArrayType given as argument
    pub fn new(array_type: ArrayType) -> ArrayGrid<T, W, H, D, C> {
        match array_type {
            ArrayType::Array1D => ArrayGrid::ArrayGrid1D(Grid::new([T::default(); H])),
            ArrayType::Array2D => ArrayGrid::ArrayGrid2D(Grid::new([[T::default(); W]; H])),
            ArrayType::Array3D => ArrayGrid::ArrayGrid3D(Grid::new([[[T::default(); W]; H]; D])),
            ArrayType::Array4D => ArrayGrid::ArrayGrid4D(GridType::<T, W, H, D, C>::new(
                [[[[T::default(); W]; H]; D]; C],
            )),
        }
    }

    #[inline(always)]
    pub fn array_grid_1d(&self) -> Option<&Grid<[T; H], T>> {
        if let ArrayGrid::ArrayGrid1D(grid) = self {
            Some(grid)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn array_grid_2d(&self) -> Option<&Grid<[[T; W]; H], T>> {
        if let ArrayGrid::ArrayGrid2D(grid) = self {
            Some(grid)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn array_grid_3d(&self) -> Option<&Grid<[[[T; W]; H]; D], T>> {
        if let ArrayGrid::ArrayGrid3D(grid) = self {
            Some(grid)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn array_grid_4d(&self) -> Option<&GridType<T, W, H, D, C>> {
        if let ArrayGrid::ArrayGrid4D(grid) = self {
            Some(grid)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get(&self, p: PointIndex) -> T {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => grid.get(p),
            ArrayGrid::ArrayGrid2D(grid) => grid.get(p),
            ArrayGrid::ArrayGrid3D(grid) => grid.get(p),
            ArrayGrid::ArrayGrid4D(grid) => grid.get(p),
        }
    }

    #[inline(always)]
    pub fn set(&self, p: PointIndex, value: T) {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => grid.set(p, value),
            ArrayGrid::ArrayGrid2D(grid) => grid.set(p, value),
            ArrayGrid::ArrayGrid3D(grid) => grid.set(p, value),
            ArrayGrid::ArrayGrid4D(grid) => grid.set(p, value),
        }
    }
}

impl<T, const W: usize, const H: usize, const D: usize, const C: usize> Display
    for ArrayGrid<T, W, H, D, C>
where
    T: Copy + Default + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayGrid::ArrayGrid1D(_) => write!(f, "ArrayGrid1D"),
            ArrayGrid::ArrayGrid2D(_) => write!(f, "ArrayGrid2D"),
            ArrayGrid::ArrayGrid3D(_) => write!(f, "ArrayGrid3D"),
            ArrayGrid::ArrayGrid4D(_) => write!(f, "ArrayGrid4D"),
        }
    }
}
