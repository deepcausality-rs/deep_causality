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

/// A position in the WGS 84 Earth-centred, Earth-fixed (ECEF) Cartesian coordinate system, in
/// metres.
///
/// The WGS 84 coordinate system is geocentric, with its origin at the centre of mass of the whole
/// Earth including oceans and atmosphere, and is a right-handed, Earth-fixed orthogonal system
/// (NGA 2014, §2.1). Its `Z` axis runs along the Earth's rotation axis, positive north; `X` passes
/// through the intersection of the prime meridian and the equator, and `Y` through the equator at
/// longitude 90° E (IOGP 2012, §2.2.1). [`Distance`](crate::Distance) is the Euclidean norm of
/// the coordinate difference, the straight-line distance in metres.
///
/// # Coordinate index mapping
/// - `0 => x`
/// - `1 => y`
/// - `2 => z`
///
/// # References
/// - National Geospatial-Intelligence Agency (2014). *World Geodetic System 1984: Its Definition
///   and Relationships with Local Geodetic Systems*, NGA.STND.0036_1.0.0_WGS84, version 1.0.0.
///   §2.1. Copy: `papers/NGA.STND.0036_1.0.0_WGS84.pdf`.
/// - IOGP (2012). *Geomatics Guidance Note 7, part 2: Coordinate Conversions and Transformations
///   including Formulas*, OGP Publication 373-7-2. §2.2.1, p. 89. Copy:
///   `papers/iogp_2012_373-7-2_coordinate_conversions_and_transformations.pdf`.
#[derive(Debug, Clone, PartialEq)]
pub struct EcefSpace<R>
where
    R: RealField,
{
    id: ContextoidId,
    x: R,
    y: R,
    z: R,
}

impl<R: RealField> EcefSpace<R> {
    pub fn new(id: ContextoidId, x: R, y: R, z: R) -> Self {
        Self { id, x, y, z }
    }
}
