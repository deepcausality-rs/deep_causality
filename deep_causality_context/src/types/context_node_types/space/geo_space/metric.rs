/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distance, GeoSpace, VerticalDatum};
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift};

/// Degrees to radians.
///
/// `RealField` carries no `to_radians`, so the conversion is written out. `R::pi()` is the
/// working type's own constant, which keeps the result as exact as that type allows.
fn to_radians<R: RealField + FromPrimitive>(degrees: R) -> R {
    degrees * R::pi() / lift(180.0)
}

/// Earth-centred Cartesian coordinates `[X, Y, Z]`, in metres, of a point with WGS 84 geodetic
/// latitude and longitude `lat`, `lon`, in degrees, and height `h` above the ellipsoid, in metres
/// (IOGP 2012, §2.2.1; `a` and `1/f` from NGA 2014, Table 3.1).
fn geocentric<R: RealField + FromPrimitive>(lat: R, lon: R, h: R) -> [R; 3] {
    let a: R = lift(6_378_137.0);
    let f: R = R::one() / lift(298.257_223_563);
    let e2 = f * (lift::<R>(2.0) - f);
    let (phi, lambda) = (to_radians(lat), to_radians(lon));
    let sin_phi = phi.sin();
    let nu = a / (R::one() - e2 * sin_phi * sin_phi).sqrt();
    [
        (nu + h) * phi.cos() * lambda.cos(),
        (nu + h) * phi.cos() * lambda.sin(),
        ((R::one() - e2) * nu + h) * sin_phi,
    ]
}

impl<R: RealField + FromPrimitive> Distance for GeoSpace<R> {
    /// The straight-line distance in metres between the two points, through their Earth-centred
    /// Cartesian coordinates; NaN unless both altitudes are measured against
    /// [`VerticalDatum::WGS84`], the only datum whose altitude is the height above the ellipsoid
    /// the conversion needs.
    fn distance(&self, other: &Self) -> R {
        if self.datum != VerticalDatum::WGS84 || other.datum != VerticalDatum::WGS84 {
            return R::nan();
        }
        let p = geocentric(self.lat, self.lon, self.alt);
        let q = geocentric(other.lat, other.lon, other.alt);
        let (dx, dy, dz) = (p[0] - q[0], p[1] - q[1], p[2] - q[2]);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
