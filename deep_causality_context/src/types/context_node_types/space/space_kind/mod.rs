/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::errors::IndexError;
use crate::{Coordinate, EcefSpace, EuclideanSpace, GeoSpace, NedSpace, Spatial};
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;

/// An enumeration over supported spatial context types.
///
/// This abstraction allows unified handling of multiple space types:
/// - Geodetic (`GeoSpace`)
/// - Euclidean (`EuclideanSpace`)
/// - Cartesian Earth-fixed (`EcefSpace`)
/// - Local tangent frame (`NedSpace`)
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceKind<R>
where
    R: RealField,
{
    Geo(GeoSpace<R>),
    Ecef(EcefSpace<R>),
    Euclidean(EuclideanSpace<R>),
    Ned(NedSpace<R>),
}

impl<R: RealField> Coordinate for SpaceKind<R> {
    type Coord = R;
    fn dimension(&self) -> usize {
        match self {
            SpaceKind::Geo(s) => s.dimension(),
            SpaceKind::Ecef(s) => s.dimension(),
            SpaceKind::Euclidean(s) => s.dimension(),
            SpaceKind::Ned(s) => s.dimension(),
        }
    }

    fn coordinate(&self, index: usize) -> Result<&R, IndexError> {
        match self {
            SpaceKind::Geo(s) => s.coordinate(index),
            SpaceKind::Ecef(s) => s.coordinate(index),
            SpaceKind::Euclidean(s) => s.coordinate(index),
            SpaceKind::Ned(s) => s.coordinate(index),
        }
    }
}

impl<R: RealField> Identifiable for SpaceKind<R> {
    fn id(&self) -> ContextoidId {
        match self {
            SpaceKind::Geo(s) => s.id(),
            SpaceKind::Ecef(s) => s.id(),
            SpaceKind::Euclidean(s) => s.id(),
            SpaceKind::Ned(s) => s.id(),
        }
    }
}

impl<R: RealField> Spatial for SpaceKind<R> {}

impl<R: RealField + std::fmt::Display> std::fmt::Display for SpaceKind<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpaceKind::Geo(s) => write!(f, "{s}"),
            SpaceKind::Ecef(s) => write!(f, "{s}"),
            SpaceKind::Euclidean(s) => write!(f, "{s}"),
            SpaceKind::Ned(s) => write!(f, "{s}"),
        }
    }
}
