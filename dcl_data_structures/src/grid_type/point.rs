// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PointIndexType {
    OneD = 0,
    TwoD = 1,
    ThreeD = 2,
    FourD = 3,
}

/// A point used to index a GridArray up to four dimensions.
#[derive(Debug, Clone, Copy)]
pub struct PointIndex {
    pub x: usize,
    // Height
    pub y: usize,
    // Width
    pub z: usize,
    // Depth
    pub t: usize, // Time
    point_type: PointIndexType,
}

impl PointIndex {
    pub fn new1d(x: usize) -> Self {
        Self {
            x,
            y: 0,
            z: 0,
            t: 0,
            point_type: PointIndexType::OneD,
        }
    }

    pub fn new2d(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            z: 0,
            t: 0,
            point_type: PointIndexType::TwoD,
        }
    }

    pub fn new3d(x: usize, y: usize, z: usize) -> Self {
        Self {
            x,
            y,
            z,
            t: 0,
            point_type: PointIndexType::ThreeD,
        }
    }

    pub fn new4d(x: usize, y: usize, z: usize, t: usize) -> Self {
        Self {
            x,
            y,
            z,
            t,
            point_type: PointIndexType::FourD,
        }
    }

    /// Returns the type of point index (1D, 2D, 3D, or 4D)
    pub fn point_type(&self) -> PointIndexType {
        self.point_type
    }
}

impl fmt::Display for PointIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.point_type {
            PointIndexType::OneD => write!(f, "(x:{})", self.x),
            PointIndexType::TwoD => write!(f, "(x:{}, y:{})", self.x, self.y),
            PointIndexType::ThreeD => write!(f, "(x:{}, y:{}, z:{})", self.x, self.y, self.z),
            PointIndexType::FourD => write!(
                f,
                "(x:{}, y:{}, z:{}, t:{})",
                self.x, self.y, self.z, self.t
            ),
        }
    }
}
