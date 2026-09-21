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
    fn distance(&self, other: &Self) -> R {
        let radius: R = lift(6_371_000.0); // Earth's mean radius in meters
        let two: R = lift(2.0);

        let dlat = to_radians(other.lat - self.lat);
        let dlon = to_radians(other.lon - self.lon);

        let sin_dlat = (dlat / two).sin();
        let sin_dlon = (dlon / two).sin();
        let a = sin_dlat * sin_dlat
            + to_radians(self.lat).cos() * to_radians(other.lat).cos() * sin_dlon * sin_dlon;

        let c = two * a.sqrt().atan2((R::one() - a).sqrt());
        let surface_distance = radius * c;
        let alt_diff = other.alt - self.alt;

        (surface_distance * surface_distance + alt_diff * alt_diff).sqrt()
    }
}
