/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, CoordinateError, VerticalDatum};
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift};
mod adjustable;
mod coordinate;
mod display;
mod getters;
mod identifiable;
mod metric;
mod recordable;
mod spatial;

/// A position given by WGS 84 geodetic latitude and longitude, in degrees, and an altitude in
/// metres measured against a [`VerticalDatum`].
///
/// Latitude `φ` and longitude `λ` are geodetic coordinates on the WGS 84 ellipsoid, whose
/// defining parameters are `a = 6 378 137.0 m` and `1/f = 298.257223563` (NGA 2014, Table 3.1).
/// Latitude lies in `[−90°, 90°]`, positive north; longitude is positive east, and two values that
/// differ by 360° name the same meridian. Equality compares coordinates, so it does not identify
/// two such longitudes, or two longitudes at a pole.
///
/// # Distance
/// [`Distance`](crate::Distance) is the straight-line distance between the two points in space.
/// Each point is converted to Earth-centred Cartesian coordinates by the geographic-to-geocentric
/// conversion (IOGP 2012, §2.2.1, EPSG method 9602):
///
/// ```text
/// X = (ν + h) cos φ cos λ
/// Y = (ν + h) cos φ sin λ
/// Z = ((1 − e²) ν + h) sin φ,    ν = a / √(1 − e² sin² φ),    e² = 2f − f²
/// ```
///
/// where `h` is the height above the ellipsoid, and the distance is the Euclidean norm of the
/// difference, the distance [`EcefSpace`](crate::EcefSpace) gives for the same two points. The
/// conversion needs `h`, which is the altitude only under [`VerticalDatum::WGS84`]. A
/// gravity-related height must first be converted to an ellipsoidal one with a geoid model
/// (IOGP 2012, §2.2.1), which this type does not carry; ISA and terrain altitudes are not heights
/// above the ellipsoid. When either point's datum is not `WGS84` the distance is NaN.
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// let berlin = GeoSpace::new(1, 52.520008, 13.404954, 34.0, VerticalDatum::WGS84).unwrap();
/// let paris = GeoSpace::new(2, 48.856613, 2.352222, 35.0, VerticalDatum::WGS84).unwrap();
///
/// let distance: f64 = berlin.distance(&paris);
/// assert!((distance - 879_004.3).abs() < 0.1);
///
/// // A latitude beyond the pole names no point.
/// assert!(GeoSpace::new(3, 91.0, 0.0, 0.0, VerticalDatum::WGS84).is_err());
/// ```
///
/// # References
/// - National Geospatial-Intelligence Agency (2014). *World Geodetic System 1984: Its Definition
///   and Relationships with Local Geodetic Systems*, NGA.STND.0036_1.0.0_WGS84, version 1.0.0.
///   §2.1, Tables 3.1 and 3.5. Copy: `papers/NGA.STND.0036_1.0.0_WGS84.pdf`.
/// - IOGP (2012). *Geomatics Guidance Note 7, part 2: Coordinate Conversions and Transformations
///   including Formulas*, OGP Publication 373-7-2. §2.2.1, pp. 89–90. Copy:
///   `papers/iogp_2012_373-7-2_coordinate_conversions_and_transformations.pdf`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoSpace<R>
where
    R: RealField,
{
    /// Unique numeric ID for the spatial context
    id: ContextoidId,
    /// Geodetic latitude in degrees, in `[−90, 90]`, positive north
    lat: R,
    /// Geodetic longitude in degrees, positive east
    lon: R,
    /// Altitude in meters, measured against `datum`
    alt: R,
    /// The reference `alt` is measured against
    datum: VerticalDatum,
}

impl<R: RealField + FromPrimitive> GeoSpace<R> {
    /// A position at latitude `lat` and longitude `lon`, in degrees, and altitude `alt`, in
    /// metres against `datum`.
    ///
    /// # Errors
    /// [`CoordinateError`] when a coordinate is not finite or `lat` lies outside `[−90, 90]`.
    pub fn new(
        id: ContextoidId,
        lat: R,
        lon: R,
        alt: R,
        datum: VerticalDatum,
    ) -> Result<Self, CoordinateError> {
        check(lat, lon, alt)?;
        Ok(Self {
            id,
            lat,
            lon,
            alt,
            datum,
        })
    }
}

/// `Ok` when all three coordinates are finite and the latitude lies in `[−90, 90]`.
pub(super) fn check<R: RealField + FromPrimitive>(
    lat: R,
    lon: R,
    alt: R,
) -> Result<(), CoordinateError> {
    if !(lat.is_finite() && lon.is_finite() && alt.is_finite()) {
        return Err(CoordinateError(
            "a GeoSpace coordinate is not finite".into(),
        ));
    }
    let ninety: R = lift(90.0);
    if lat < -ninety || lat > ninety {
        return Err(CoordinateError(
            "a GeoSpace latitude lies outside [-90, 90] degrees".into(),
        ));
    }
    Ok(())
}
