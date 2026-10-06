/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use deep_causality_algebra::RealField;
mod adjustable;
mod coordinate;
mod display;
mod getters;
mod identifiable;
mod metric;
mod recordable;
mod spatial;

/// A position in a local north-east-down (NED) Cartesian frame, in metres.
///
/// A topocentric coordinate system is a three-dimensional Cartesian system with an origin on or
/// near the surface of the Earth and mutually perpendicular axes `U` east, `V` north and `W` up,
/// right-handed. Its height axis is either the direction of gravity at the origin or the normal to
/// the ellipsoid through it (IOGP 2012, §2.2.2, EPSG method 9836). NED orders and orients the same
/// axes as `north = V`, `east = U`, `down = −W`, which is again right-handed: `down` is positive
/// toward the Earth.
///
/// [`Distance`](crate::Distance) returns the Euclidean norm of the coordinate difference. The
/// frame is Cartesian, so for two points of the same NED frame that norm is their straight-line
/// separation; for points of different frames it is not. This type stores neither the origin nor
/// which height axis was chosen, so the caller tracks which points share a frame and supplies both
/// to relate a `NedSpace` to a geodetic or Earth-centred position.
///
/// # Coordinate index mapping
/// - `0 => north`
/// - `1 => east`
/// - `2 => down`
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// let origin = NedSpace::new(1, 0.0, 0.0, 0.0);
/// let point = NedSpace::new(2, 100.0, 50.0, 10.0); // 100 m north, 50 m east, 10 m below
///
/// assert_eq!(point.dimension(), 3);
/// assert_eq!(origin.distance(&point), (100.0_f64.powi(2) + 50.0_f64.powi(2) + 10.0_f64.powi(2)).sqrt());
/// ```
///
/// # References
/// - IOGP (2012). *Geomatics Guidance Note 7, part 2: Coordinate Conversions and Transformations
///   including Formulas*, OGP Publication 373-7-2. §2.2.2, p. 91. Copy:
///   `papers/iogp_2012_373-7-2_coordinate_conversions_and_transformations.pdf`.
#[derive(Debug, Clone, PartialEq)]
pub struct NedSpace<R>
where
    R: RealField,
{
    /// Unique numeric ID for this local NED context
    id: ContextoidId,
    /// Distance north from the reference point (in meters)
    north: R,
    /// Distance east from the reference point (in meters)
    east: R,
    /// Vertical distance down from the reference point (in meters, positive = downward)
    down: R,
}

impl<R: RealField> NedSpace<R> {
    pub fn new(id: ContextoidId, north: R, east: R, down: R) -> Self {
        Self {
            id,
            north,
            east,
            down,
        }
    }
}
