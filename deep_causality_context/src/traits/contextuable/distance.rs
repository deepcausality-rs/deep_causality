/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Coordinate;

/// The distance between two points of the same type, as a length.
///
/// Each implementation in this crate with Cartesian positions returns the distance in the unit of
/// those positions: metres, and in [`EuclideanSpace`](crate::EuclideanSpace) the unit its
/// coordinates are given in. [`GeoSpace`](crate::GeoSpace) takes latitude and longitude in
/// degrees and returns metres.
///
/// For points with finite coordinates, every implementation in this crate satisfies, wherever the
/// geometry defines the distance, `d(a, a) = 0`, `d(a, b) = d(b, a) ≥ 0` and
/// `d(a, c) ≤ d(a, b) + d(b, c)`: the axioms of a pseudometric. Floating point keeps them up to
/// rounding, and only while no squared coordinate difference overflows the scalar: past that the
/// distance is infinite.
/// Distinct points may be at distance zero: two events of
/// [`NewtonianSpacetime`](crate::NewtonianSpacetime) at the same place and different times are.
/// Where the geometry defines no distance between two points the value is NaN:
/// [`GalileanSpacetime`](crate::GalileanSpacetime) between events at different times, and
/// [`GeoSpace`](crate::GeoSpace) when an altitude is not a height above the WGS 84 ellipsoid.
pub trait Distance: Coordinate {
    /// The distance between `self` and `other`, or NaN where the geometry defines none.
    fn distance(&self, other: &Self) -> Self::Coord;
}
