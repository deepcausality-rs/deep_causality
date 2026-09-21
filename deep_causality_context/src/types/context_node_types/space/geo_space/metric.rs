/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distance, GeoSpace};

use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift};

/// Degrees to radians.
///
/// `RealField` carries no `to_radians`, so the conversion is written out. `R::pi()` is the
/// working type's own constant, which keeps the result as exact as that type allows.
fn to_radians<R: RealField + FromPrimitive>(degrees: R) -> R {
    degrees * R::pi() / lift(180.0)
}

// Distance (with simple haversine approximation)
impl<R: RealField + FromPrimitive> Distance for GeoSpace<R> {
    /// Haversine surface distance combined with the altitude difference.
    ///
    /// # Precondition
    ///
    /// Both operands must carry the same [`VerticalDatum`](crate::VerticalDatum). The altitude
    /// term is `other.alt - self.alt`, and that subtraction is only a length when both numbers
    /// are measured against the same reference: an ellipsoidal height and an orthometric height
    /// of equal magnitude name different points, so their difference is not a height difference.
    /// Converting between datums needs a geoid model, which this type does not carry.
    ///
    /// The precondition is checked by `debug_assert_eq!`, so a mismatch panics in debug builds
    /// and is computed as if the datums agreed in release builds.
    fn distance(&self, other: &Self) -> R {
        debug_assert_eq!(
            self.datum, other.datum,
            "GeoSpace::distance requires both operands to use the same VerticalDatum"
        );

        let radius: R = lift(6_371_000.0); // Earth's mean radius in meters
        let two: R = lift(2.0);

        let dlat = to_radians(other.lat - self.lat);
        let dlon = to_radians(other.lon - self.lon);

        let sin_dlat = (dlat / two).sin();
        let sin_dlon = (dlon / two).sin();
        // Mathematically `a` lies in [0, 1], but rounding carries it past 1 for near-antipodal
        // inputs, and `(1 - a).sqrt()` is then NaN. Clamping restores the domain of the `atan2`.
        let a = (sin_dlat * sin_dlat
            + to_radians(self.lat).cos() * to_radians(other.lat).cos() * sin_dlon * sin_dlon)
            .clamp(R::zero(), R::one());

        let c = two * a.sqrt().atan2((R::one() - a).sqrt());
        let surface_distance = radius * c;
        let alt_diff = other.alt - self.alt;

        (surface_distance * surface_distance + alt_diff * alt_diff).sqrt()
    }
}
