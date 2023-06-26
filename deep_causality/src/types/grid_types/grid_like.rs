// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use crate::prelude::PointIndex;

/// A container which stores elements at 2D points.
pub trait Gridlike<T>
{
    fn width(&self) -> usize;

    fn height(&self) -> usize;

    /// Get the element at the given point.
    fn get(&self, p: PointIndex) -> &T;

    /// Set the element at the given point.
    fn set(&self, p: PointIndex, element: T);
}