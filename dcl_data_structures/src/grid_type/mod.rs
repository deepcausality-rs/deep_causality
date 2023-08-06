// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

#![forbid(unsafe_code)]

use std::fmt::{Debug, Display, Formatter};
use crate::prelude::{Grid, PointIndex};

pub mod grid;
pub mod point;
pub mod storage;
pub mod storage_array_1d;
pub mod storage_array_2d;
pub mod storage_array_3d;
pub mod storage_array_4d;

/// Type alias for fixed sized static ArrayGrid
pub type ArrayGrid1DType<T, const H: usize> = Grid<[T; H], T>;
pub type ArrayGrid2DType<T, const W: usize, const H: usize> = Grid<[[T; W]; H], T>;
pub type ArrayGrid3DType<T, const W: usize, const H: usize, const D: usize> = Grid<[[[T; W]; H]; D], T>;
pub type ArrayGrid4DType<T, const W: usize, const H: usize, const D: usize, const C: usize> = Grid<[[[[T; W]; H]; D]; C], T>;

/// ArrayType to determine what kind of ArrayGrid to build.
pub enum ArrayType {
    Array1D,
    Array2D,
    Array3D,
    Array4D,
}

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
    /// Returns a new ArrayGrid with the ArrayType given as argument
    ///
    /// # Const Generic Arguments
    /// * T - Type to store i.e. u64 or MyStruct
    /// * W: usize - Width (First ArrayGrid dimension)
    /// * H: usize - Height (Second ArrayGrid dimension)
    /// * D: usize - Depth (Third ArrayGrid dimension)
    /// * C: usize - Chronos (Time) since T was already taken for Type T (Fourth ArrayGrid dimension)
    ///
    /// Note, if you only use up to, say, 2D just set Depth and Time to 1.
    ///
    /// # Arguments
    ///
    /// * `array_type: ArrayType` - Enum that defines which type of ArrayGrid to build.
    ///
    /// # Example
    ///
    /// In most cases, the sample code below should suffice. You do need type annotation
    /// because of the const generic arguments.
    /// ```
    /// use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid};
    ///
    /// // Constants required for const generic parameters
    /// // Use these to check whether your PointIndex stays within the Array boundaries.
    /// const WIDTH: usize = 5;
    /// const HEIGHT: usize = 5;
    /// const DEPTH: usize = 5;
    /// const TIME: usize = 1; // set to 1 if unused.
    ///
    ///     // Make a simple 1D Array of type usize
    ///     let array_type = ArrayType::Array1D;
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    ///
    ///     // Make a 3D Array of type usize
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(ArrayType::Array3D);
    ///
    ///
    /// ```
    pub fn new(array_type: ArrayType) -> ArrayGrid<T, W, H, D, C> {
        match array_type {
            ArrayType::Array1D => ArrayGrid::ArrayGrid1D(Grid::new([T::default(); H])),
            ArrayType::Array2D => ArrayGrid::ArrayGrid2D(Grid::new([[T::default(); W]; H])),
            ArrayType::Array3D => ArrayGrid::ArrayGrid3D(Grid::new([[[T::default(); W]; H]; D])),
            ArrayType::Array4D => ArrayGrid::ArrayGrid4D(Grid::new([[[[T::default(); W]; H]; D]; C])),
        }
    }

    /// Returns the item for the given PointIndex
    ///
    /// # Arguments
    ///
    /// * `p: PointIndex` - PointIndex that must match the dimensionality of the ArrayGrid
    ///
    /// # Example
    ///
    /// ```
    /// use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};
    ///
    /// const WIDTH: usize = 5;
    /// const HEIGHT: usize = 5;
    /// const DEPTH: usize = 1;  // set to 1 if unused.
    /// const TIME: usize = 1;   // set to 1 if unused.
    ///
    ///     let array_type = ArrayType::Array1D;
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    ///
    ///     // Create a 1D PointIndex
    ///     let p = PointIndex::new1d(1);
    ///
    ///     // Get the value for the point index
    ///     let res = ag.get(p);
    ///
    /// ```
    pub fn get(&self, p: PointIndex) -> T {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid2D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid3D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid4D(grid) => { grid.get(p) }
        }
    }


    /// Sets the item at the given PointIndex
    ///
    /// # Arguments
    ///
    /// * `p: PointIndex` - PointIndex that must match the dimensionality of the ArrayGrid
    /// * `value: T` -  Value to store at the index position.
    ///
    /// # Example
    ///
    /// ```
    /// use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};
    ///
    /// const WIDTH: usize = 5;
    /// const HEIGHT: usize = 5;
    /// const DEPTH: usize = 1;  // set to 1 if unused.
    /// const TIME: usize = 1;   // set to 1 if unused.
    ///
    ///     let array_type = ArrayType::Array1D;
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    ///
    ///      // Create a 1D PointIndex
    ///      let p = PointIndex::new1d(1);
    ///
    ///      // Store a usize with the point index
    ///      ag.set(p, 42);
    ///
    /// ```
    pub fn set(&self, p: PointIndex, value: T) {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid2D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid3D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid4D(grid) => { grid.set(p, value) }
        }
    }

    /// Returns a 1D GridArray if the instance is 1D, otherwise returns None.
    /// This give access to the array dimensions available in the underlying Grid Type.
    ///
    /// # Returns
    ///  * Option<&ArrayGrid1DType>
    ///
    /// Note, ArrayGrid1DType is a type alias to Grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};
    ///
    /// const WIDTH: usize = 5;
    /// const HEIGHT: usize = 5;
    /// const DEPTH: usize = 1;  // set to 1 if unused.
    /// const TIME: usize = 1;   // set to 1 if unused.
    ///
    ///     let array_type = ArrayType::Array1D;
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    ///
    ///     let g = ag.array_grid_1d()
    ///         .expect("failed to get 1D array grid");
    ///
    ///     // Access dimensions i.e. for index bound checks before insert.
    ///     let height = g.height().unwrap();
    ///     assert_eq!(height, HEIGHT);
    ///
    /// ```
    pub fn array_grid_1d(&self) -> Option<&ArrayGrid1DType<T, H>>
    {
        if let ArrayGrid::ArrayGrid1D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }

    /// Returns a 2D GridArray if the instance is 3D, otherwise returns None.
    /// This give access to the array dimensions available in the underlying Grid Type.
    ///
    /// # Returns
    ///  * Option<&ArrayGrid2DType>
    ///
    /// Note, ArrayGrid2DType is a type alias to Grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};
    ///
    /// const WIDTH: usize = 5;
    /// const HEIGHT: usize = 5;
    /// const DEPTH: usize = 1;  // set to 1 if unused.
    /// const TIME: usize = 1;   // set to 1 if unused.
    ///
    ///     let array_type = ArrayType::Array2D;
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    ///
    ///     let g = ag.array_grid_2d()
    ///         .expect("failed to get 2D array grid");
    ///
    ///     // Access dimensions i.e. for index bound checks before insert.
    ///     let height = g.height().unwrap();
    ///     assert_eq!(height, HEIGHT);
    ///
    ///     let width = g.width().unwrap();
    ///     assert_eq!(width, WIDTH);
    ///
    /// ```
    pub fn array_grid_2d(&self) -> Option<&ArrayGrid2DType<T, W, H>>
    {
        if let ArrayGrid::ArrayGrid2D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }

    /// Returns a 3D GridArray if the instance is 3D, otherwise returns None.
    /// This give access to the array dimensions available in the underlying Grid Type.
    ///
    /// # Returns
    ///  * Option<&ArrayGrid3DType>
    ///
    /// Note, ArrayGrid3DType is a type alias to Grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};
    ///
    /// const WIDTH: usize = 5;
    /// const HEIGHT: usize = 5;
    /// const DEPTH: usize = 5;
    /// const TIME: usize = 1;   // set to 1 if unused.
    ///
    ///     let array_type = ArrayType::Array3D;
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    ///
    ///     let g = ag.array_grid_3d()
    ///         .expect("failed to get 3D array grid");
    ///
    ///     // Access dimensions i.e. for index bound checks before insert.
    ///     let height = g.height().unwrap();
    ///     assert_eq!(height, HEIGHT);
    ///
    ///     let width = g.width().unwrap();
    ///     assert_eq!(width, WIDTH);
    ///
    ///     let depth = g.depth().unwrap();
    ///     assert_eq!(depth, DEPTH);
    ///
    /// ```
    pub fn array_grid_3d(&self) -> Option<&ArrayGrid3DType<T, W, H, D>> {
        if let ArrayGrid::ArrayGrid3D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }

    /// Returns a 4D GridArray if the instance is 4D, otherwise returns None.
    /// This give access to the array dimensions available in the underlying Grid Type.
    ///
    /// # Returns
    ///  * Option<&ArrayGrid4DType>
    ///
    /// Note, ArrayGrid4DType is a type alias to Grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};
    ///
    /// const WIDTH: usize = 5;
    /// const HEIGHT: usize = 5;
    /// const DEPTH: usize = 5;
    /// const TIME: usize = 5;
    ///
    ///     let array_type = ArrayType::Array4D;
    ///     let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    ///
    ///     let g = ag.array_grid_4d()
    ///         .expect("failed to get 4D array grid");
    ///
    ///     // Access dimensions i.e. for index bound checks before insert.
    ///     let height = g.height().unwrap();
    ///     assert_eq!(height, HEIGHT);
    ///
    ///     let width = g.width().unwrap();
    ///     assert_eq!(width, WIDTH);
    ///
    ///     let depth = g.depth().unwrap();
    ///     assert_eq!(depth, DEPTH);
    ///
    ///    let time = g.time().unwrap();
    ///     assert_eq!(time, TIME);
    ///
    /// ```
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
