// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use deep_causality_macros::Constructor;

mod coordinate;
mod display;
mod identifiable;
mod metric;
mod spatial;
mod getters;

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
#[derive(Constructor, Debug, Clone, PartialEq)]
pub struct EcefSpace {
    id: u64,
    x: f64,
    y: f64,
    z: f64,
}

