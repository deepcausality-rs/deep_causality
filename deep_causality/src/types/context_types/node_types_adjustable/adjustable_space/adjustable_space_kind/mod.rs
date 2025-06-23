// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::errors::{AdjustmentError, UpdateError};
use crate::prelude::{
    Adjustable, AdjustableEcefSpace, AdjustableEuclideanSpace, AdjustableGeoSpace,
    AdjustableNedSpace, AdjustableQuaternionSpace, Coordinate, Identifiable, Spatial,
};
use dcl_data_structures::grid_type::ArrayGrid;

/// An enumeration over supported spatial context types.
///
/// This abstraction allows unified handling of multiple adjustable space types:
/// - Cartesian Earth-fixed (`AdjustableEcefSpace`)
/// - Euclidean (`AdjustableEuclideanSpace`)
/// - Geodetic (`AdjustableGeoSpace`)
/// - Local tangent frame (`AdjustableNedSpace`)
/// - 3D orientation (`AdjustableQuaternionSpace`)
#[derive(Debug, Clone, PartialEq)]
pub enum AdjustableSpaceKind {
    Ecef(AdjustableEcefSpace),
    Euclidean(AdjustableEuclideanSpace),
    Geo(AdjustableGeoSpace),
    Ned(AdjustableNedSpace),
    Quaternion(AdjustableQuaternionSpace),
}

impl Adjustable<f64> for AdjustableSpaceKind {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        match self {
            AdjustableSpaceKind::Ecef(ecef) => ecef.update(array_grid),
            AdjustableSpaceKind::Euclidean(euc) => euc.update(array_grid),
            AdjustableSpaceKind::Geo(geo) => geo.update(array_grid),
            AdjustableSpaceKind::Ned(ned) => ned.update(array_grid),
            AdjustableSpaceKind::Quaternion(quat) => quat.update(array_grid),
        }
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        match self {
            AdjustableSpaceKind::Ecef(ecef) => ecef.adjust(array_grid),
            AdjustableSpaceKind::Euclidean(euc) => euc.adjust(array_grid),
            AdjustableSpaceKind::Geo(geo) => geo.adjust(array_grid),
            AdjustableSpaceKind::Ned(ned) => ned.adjust(array_grid),
            AdjustableSpaceKind::Quaternion(quat) => quat.adjust(array_grid),
        }
    }
}

impl Coordinate<f64> for AdjustableSpaceKind {
    fn dimension(&self) -> usize {
        match self {
            AdjustableSpaceKind::Geo(s) => s.dimension(),
            AdjustableSpaceKind::Ecef(s) => s.dimension(),
            AdjustableSpaceKind::Euclidean(s) => s.dimension(),
            AdjustableSpaceKind::Ned(s) => s.dimension(),
            AdjustableSpaceKind::Quaternion(s) => s.dimension(),
        }
    }

    fn coordinate(&self, index: usize) -> &f64 {
        match self {
            AdjustableSpaceKind::Geo(s) => s.coordinate(index),
            AdjustableSpaceKind::Ecef(s) => s.coordinate(index),
            AdjustableSpaceKind::Euclidean(s) => s.coordinate(index),
            AdjustableSpaceKind::Ned(s) => s.coordinate(index),
            AdjustableSpaceKind::Quaternion(s) => s.coordinate(index),
        }
    }
}

impl Identifiable for AdjustableSpaceKind {
    fn id(&self) -> u64 {
        match self {
            AdjustableSpaceKind::Geo(s) => s.id(),
            AdjustableSpaceKind::Ecef(s) => s.id(),
            AdjustableSpaceKind::Euclidean(s) => s.id(),
            AdjustableSpaceKind::Ned(s) => s.id(),
            AdjustableSpaceKind::Quaternion(s) => s.id(),
        }
    }
}

impl Spatial<f64> for AdjustableSpaceKind {}

impl std::fmt::Display for AdjustableSpaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdjustableSpaceKind::Geo(s) => write!(f, "{s}"),
            AdjustableSpaceKind::Ecef(s) => write!(f, "{s}"),
            AdjustableSpaceKind::Euclidean(s) => write!(f, "{s}"),
            AdjustableSpaceKind::Ned(s) => write!(f, "{s}"),
            AdjustableSpaceKind::Quaternion(s) => write!(f, "{s}"),
        }
    }
}
