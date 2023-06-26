// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

/// A point used to index a GridArray.
#[derive(Debug, Clone, Copy)]
pub struct PointIndex {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub t: usize,
}

impl PointIndex
{
    pub fn new1d(x: usize) -> Self {
        Self { x, y: 0, z: 0, t: 0 }
    }

    pub fn new2d(x: usize, y: usize) -> Self {
        Self { x, y, z: 0, t: 0 }
    }

    pub fn new3d(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z, t: 0 }
    }

    pub fn new4d(x: usize, y: usize, z: usize, t: usize) -> Self {
        Self { x, y, z, t }
    }
}