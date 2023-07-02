// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Debug, Display, Formatter};

use crate::prelude::{Grid, PointIndex};

// Fixed sized static ArrayGrid
pub type ArrayGrid1DType<T, const H: usize> = Grid<[T; H], T>;
pub type ArrayGrid2DType<T, const W: usize, const H: usize> = Grid<[[T; W]; H], T>;
pub type ArrayGrid3DType<T, const W: usize, const H: usize, const D: usize> = Grid<[[[T; W]; H]; D], T>;
pub type ArrayGrid4DType<T, const W: usize, const H: usize, const D: usize, const C: usize> = Grid<[[[[T; W]; H]; D]; C], T>;

pub enum ArrayType {
    Array1D,
    Array2D,
    Array3D,
    Array4D,
}


// T Type
// W Width
// H Height
// D Depth
// C Chronos since T was already taken for Type T
pub enum ArrayGrid<T, const W: usize, const H: usize, const D: usize, const C: usize>
    where
        T: Copy,
{
    ArrayGrid1D(ArrayGrid1DType<T, H>),
    ArrayGrid2D(ArrayGrid2DType<T, W, H>),
    ArrayGrid3D(ArrayGrid3DType<T, W, H, D>),
    ArrayGrid4D(ArrayGrid4DType<T, W, H, D, C>),
}

impl<T, const W: usize, const H: usize, const D: usize, const C: usize> ArrayGrid<T, W, H, D, C>
    where
        T: Copy + Default,
{
    pub fn new(array_type: ArrayType) -> ArrayGrid<T, W, H, D, C> {
        match array_type {
            ArrayType::Array1D => ArrayGrid::ArrayGrid1D(Grid::new([T::default(); H])),
            ArrayType::Array2D => ArrayGrid::ArrayGrid2D(Grid::new([[T::default(); W]; H])),
            ArrayType::Array3D => ArrayGrid::ArrayGrid3D(Grid::new([[[T::default(); W]; H]; D])),
            ArrayType::Array4D => ArrayGrid::ArrayGrid4D(Grid::new([[[[T::default(); W]; H]; D]; C])),
        }
    }
}

impl<T, const W: usize, const H: usize, const D: usize, const C: usize> ArrayGrid<T, W, H, D, C>
    where
        T: Copy + Default,
{
    pub fn get(&self, p: PointIndex) -> T {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid2D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid3D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid4D(grid) => { grid.get(p) }
        }
    }

    pub fn set(&self, p: PointIndex, value: T) {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid2D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid3D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid4D(grid) => { grid.set(p, value) }
        }
    }
}

impl<T, const W: usize, const H: usize, const D: usize, const C: usize> ArrayGrid<T, W, H, D, C>
    where
        T: Copy + Default,
{
    pub fn array_grid_1d(&self) -> Option<&ArrayGrid1DType<T, H>>
    {
        if let ArrayGrid::ArrayGrid1D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }

    pub fn array_grid_2d(&self) -> Option<&ArrayGrid2DType<T, W, H>>
    {
        if let ArrayGrid::ArrayGrid2D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }

    pub fn array_grid_3d(&self) -> Option<&ArrayGrid3DType<T, W, H, D>> {
        if let ArrayGrid::ArrayGrid3D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }

    pub fn array_grid_4d(&self) -> Option<&ArrayGrid4DType<T, W, H, D, C>> {
        if let ArrayGrid::ArrayGrid4D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }
}

impl<T, const W: usize, const H: usize, const D: usize, const C: usize> Display for ArrayGrid<T, W, H, D, C>
    where
        T: Copy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayGrid::ArrayGrid1D(array_grid) => write!(f, "ArrayGrid1D: {:?}", array_grid),
            ArrayGrid::ArrayGrid2D(array_grid) => write!(f, "ArrayGrid2D: {:?}", array_grid),
            ArrayGrid::ArrayGrid3D(array_grid) => write!(f, "ArrayGrid3D: {:?}", array_grid),
            ArrayGrid::ArrayGrid4D(array_grid) => write!(f, "ArrayGrid4D: {:?}", array_grid),
        }
    }
}