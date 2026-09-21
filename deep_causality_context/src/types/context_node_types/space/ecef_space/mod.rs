/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
mod adjustable;
mod coordinate;
mod display;
mod getters;
mod identifiable;
mod metric;
mod spatial;

/// A spatial context in Earth-Centered, Earth-Fixed (ECEF) coordinates.
///
/// `EcefSpace` represents a position in 3D Cartesian space relative to the Earth's center of mass.
/// It is commonly used in GNSS (e.g., GPS), satellite systems, and global geospatial models.
/// The origin `(0, 0, 0)` is at the Earth's center, with axes aligned to the WGS84 ellipsoid:
/// - `x`: points toward the intersection of the equator and the prime meridian (0° lat/lon)
/// - `y`: points toward 90° east longitude
/// - `z`: points toward the north pole
///
/// # Fields
/// - `id`: Unique identifier for the location
/// - `x`: X-coordinate in meters
/// - `y`: Y-coordinate in meters
/// - `z`: Z-coordinate in meters
///
/// # Coordinate Index Mapping
/// When used with the `Coordinate` trait, the following index mapping applies:
/// - `0 => x`
/// - `1 => y`
/// - `2 => z`
///
/// # Trait Implementations
/// This type implements:
/// - `Identifiable`
/// - `Coordinate`
/// - `Distance` using the Haversine approximation
/// - `Spatial`
/// - `Display` for human-readable output
///
#[derive(Debug, Clone, PartialEq)]
pub struct EcefSpace {
    id: ContextoidId,
    x: f64,
    y: f64,
    z: f64,
}

impl EcefSpace {
    pub fn new(id: ContextoidId, x: f64, y: f64, z: f64) -> Self {
        Self { id, x, y, z }
    }
}
