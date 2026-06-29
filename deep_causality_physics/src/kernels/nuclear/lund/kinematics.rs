/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lightcone kinematics for Lund string fragmentation.
//!
//! The Lund model uses lightcone coordinates where:
//! - p+ = E + pz (forward lightcone momentum)
//! - p- = E - pz (backward lightcone momentum)
//! - m²_T = m² + p²_T (transverse mass squared)
//!
//! The key relation is: p+ * p- = m²_T
//!
//! Internally these data carriers are generic over `R: RealField`. Random
//! sampling in `sample_z` still happens at f64 via `deep_causality_rand`
//! (its `Standard` / `StandardNormal` distributions only implement
//! `Distribution` for `f32` and `f64`), with sampled uniforms lifted into `R`
//! via `R::from_f64` at the RNG boundary.

use crate::FourMomentum;
use crate::real_from_f64;
use deep_causality_num::{FromPrimitive, RealField};

/// A string endpoint in lightcone coordinates.
#[derive(Debug, Clone, Copy)]
pub struct LightconeEndpoint<R: RealField> {
    pub p_plus: R,
    pub p_minus: R,
    pub pt_x: R,
    pub pt_y: R,
}

impl<R: RealField + FromPrimitive> LightconeEndpoint<R> {
    /// Create from 4-momentum.
    pub fn from_four_momentum(p: &FourMomentum<R>) -> Self {
        Self {
            p_plus: p.lightcone_plus(),
            p_minus: p.lightcone_minus(),
            pt_x: p.px(),
            pt_y: p.py(),
        }
    }

    /// Convert to 4-momentum.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_four_momentum(&self) -> FourMomentum<R> {
        let two = real_from_f64::<R>(2.0);
        let e = (self.p_plus + self.p_minus) / two;
        let pz = (self.p_plus - self.p_minus) / two;
        FourMomentum::<R>::new(e, self.pt_x, self.pt_y, pz)
    }

    /// Invariant mass squared.
    #[allow(dead_code)]
    pub fn invariant_mass_squared(&self) -> R {
        self.p_plus * self.p_minus - self.pt_x * self.pt_x - self.pt_y * self.pt_y
    }
}

/// A string segment in lightcone coordinates.
#[derive(Debug, Clone, Copy)]
pub struct StringSegment<R: RealField> {
    pub quark: LightconeEndpoint<R>,
    pub antiquark: LightconeEndpoint<R>,
}

impl<R: RealField + FromPrimitive> StringSegment<R> {
    /// Create from two 4-momenta (quark moving in +z, antiquark in -z).
    pub fn from_endpoints(quark: &FourMomentum<R>, antiquark: &FourMomentum<R>) -> Self {
        Self {
            quark: LightconeEndpoint::from_four_momentum(quark),
            antiquark: LightconeEndpoint::from_four_momentum(antiquark),
        }
    }

    /// Total invariant mass squared of the string.
    pub fn invariant_mass_squared(&self) -> R {
        let total = self.quark.to_four_momentum() + self.antiquark.to_four_momentum();
        total.invariant_mass_squared()
    }

    /// Total invariant mass.
    pub fn invariant_mass(&self) -> R {
        let m_sq = self.invariant_mass_squared();
        if m_sq > R::zero() {
            m_sq.sqrt()
        } else {
            R::zero()
        }
    }

    /// Total available lightcone momentum for fragmentation from quark end.
    pub fn w_plus(&self) -> R {
        self.quark.p_plus + self.antiquark.p_plus
    }

    /// Total available lightcone momentum for fragmentation from antiquark end.
    pub fn w_minus(&self) -> R {
        self.quark.p_minus + self.antiquark.p_minus
    }
}

/// Compute the momentum fraction z for hadron production.
///
/// Using the Lund symmetric fragmentation function:
/// $$ f(z) = \frac{1}{z} (1-z)^a \exp(-b m_T^2 / z) $$
///
/// We sample z using the rejection method. The uniforms used by rejection
/// sampling come from the RNG at f64 (per the precision-boundary note above)
/// and are lifted into `R`.
pub fn sample_z<R, RNG>(rng: &mut RNG, lund_a: R, lund_b: R, mt_squared: R) -> R
where
    R: RealField + FromPrimitive,
    RNG: deep_causality_rand::Rng,
{
    let one = R::one();
    let small = real_from_f64::<R>(0.01);

    // Maximum is approximately at z_max = 1 / (1 + b * m_T^2 / (a+1))
    let denom_factor = if lund_a + one > small {
        lund_a + one
    } else {
        small
    };
    let z_max = one / (one + lund_b * mt_squared / denom_factor);
    let f_max = lund_function(z_max, lund_a, lund_b, mt_squared);

    let z_min = real_from_f64::<R>(0.01);
    let z_max_cutoff = real_from_f64::<R>(0.99);

    loop {
        // Sample uniform z in [z_min, z_max_cutoff].
        let u: R = real_from_f64(rng.random::<f64>());
        let z = z_min + (z_max_cutoff - z_min) * u;

        let f_z = lund_function(z, lund_a, lund_b, mt_squared);

        // Accept/reject — second uniform also lifted from f64.
        let u2: R = real_from_f64(rng.random::<f64>());
        if u2 * f_max < f_z {
            return z;
        }
    }
}

/// Evaluate the Lund symmetric fragmentation function.
///
/// f(z) = (1/z) * (1-z)^a * exp(-b * m_T^2 / z)
fn lund_function<R: RealField>(z: R, a: R, b: R, mt_sq: R) -> R {
    if z <= R::zero() || z >= R::one() {
        return R::zero();
    }

    (R::one() / z) * (R::one() - z).powf(a) * (-b * mt_sq / z).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lightcone_round_trip() {
        let p = FourMomentum::<f64>::new(10.0, 1.0, 2.0, 8.0);
        let lc: LightconeEndpoint<f64> = LightconeEndpoint::from_four_momentum(&p);
        let p_back = lc.to_four_momentum();

        assert!((p.e() - p_back.e()).abs() < 1e-10);
        assert!((p.px() - p_back.px()).abs() < 1e-10);
        assert!((p.py() - p_back.py()).abs() < 1e-10);
        assert!((p.pz() - p_back.pz()).abs() < 1e-10);
    }

    #[test]
    fn test_lund_function_bounds() {
        // Should be 0 at boundaries
        assert_eq!(lund_function::<f64>(0.0, 0.68, 0.98, 0.5), 0.0);
        assert_eq!(lund_function::<f64>(1.0, 0.68, 0.98, 0.5), 0.0);

        // Should be positive inside
        assert!(lund_function::<f64>(0.5, 0.68, 0.98, 0.5) > 0.0);
    }
}
