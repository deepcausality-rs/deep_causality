/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, VerticalDatum};
use deep_causality_algebra::RealField;
mod adjustable;
mod coordinate;
mod display;
mod getters;
mod identifiable;
mod metric;
mod spatial;

/// A non-Euclidean spatial context based on geodetic coordinates.
///
/// `GeoSpace` represents a geographic location on Earth by **latitude**, **longitude**, **altitude**
/// and the [`VerticalDatum`] that altitude is measured against. Horizontal position follows the
/// [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) standard. It is commonly used in
/// systems that model real-world positions, such as navigation, mapping, remote sensing, and
/// sensor fusion applications.
///
/// Unlike Euclidean coordinates, geodetic coordinates model the Earth's surface as a **curved ellipsoid** rather than a flat plane.
/// This makes `GeoSpace` a simple yet powerful non-Euclidean spatial representation that integrates naturally with GPS and global datasets.
///
/// # Fields
/// - `id`: A unique numeric identifier for the location (e.g., sensor ID, region ID)
/// - `lat`: Latitude in degrees (positive north, negative south)
/// - `lon`: Longitude in degrees (positive east, negative west)
/// - `alt`: Altitude in meters, measured against `datum`
/// - `datum`: The reference `alt` is measured against
///
/// # Trait Implementations
/// This type implements:
/// - `Identifiable`
/// - `Coordinate`
/// - `Distance` using the Haversine approximation
/// - `Spatial`
/// - `Display` for human-readable output
///
/// # Common Use Cases
/// - Geographic sensor modeling (e.g., magnetometers on aircraft, buoys, satellites)
/// - Location-aware causal contexts (e.g., MagNav, remote sensing calibration)
/// - Contextualizing non-Euclidean geospatial data in DeepCausality graphs
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// let g1 = GeoSpace::new(1, 52.520008, 13.404954, 34.0, VerticalDatum::WGS84); // Berlin, Germany
/// let g2 = GeoSpace::new(2, 48.856613, 2.352222, 35.0, VerticalDatum::WGS84);   // Paris, France
///
/// println!("{}", g1);
///
/// let distance = g1.distance(&g2);
/// println!("Distance (approx): {:.2} km", distance / 1000.0);
/// ```
///
/// # Output
/// ```text
/// GeoSpace(id=1, lat=52.5200, lon=13.4050, alt=34.0000, datum=WGS84)
/// Distance (approx): 878.84 km
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GeoSpace<R>
where
    R: RealField,
{
    /// Unique numeric ID for the spatial context
    id: ContextoidId,
    /// Latitude in decimal degrees (positive north, negative south)
    lat: R,
    /// Longitude in decimal degrees (positive east, negative west)
    lon: R,
    /// Altitude in meters, measured against `datum`
    alt: R,
    /// The reference `alt` is measured against
    datum: VerticalDatum,
}

impl<R: RealField> GeoSpace<R> {
    pub fn new(id: ContextoidId, lat: R, lon: R, alt: R, datum: VerticalDatum) -> Self {
        Self {
            id,
            lat,
            lon,
            alt,
            datum,
        }
    }
}
