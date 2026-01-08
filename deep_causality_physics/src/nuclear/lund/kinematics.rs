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

use crate::nuclear::quantities::FourMomentum;

/// A string endpoint in lightcone coordinates.
#[derive(Debug, Clone, Copy)]
pub struct LightconeEndpoint {
    /// Forward lightcone momentum p+
    pub p_plus: f64,
    /// Backward lightcone momentum p-
    pub p_minus: f64,
    /// Transverse momentum x-component
    pub pt_x: f64,
    /// Transverse momentum y-component
    pub pt_y: f64,
}

impl LightconeEndpoint {
    /// Create from 4-momentum.
    pub fn from_four_momentum(p: &FourMomentum) -> Self {
        Self {
            p_plus: p.lightcone_plus(),
            p_minus: p.lightcone_minus(),
            pt_x: p.px(),
            pt_y: p.py(),
        }
    }

    /// Convert to 4-momentum.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_four_momentum(&self) -> FourMomentum {
        let e = (self.p_plus + self.p_minus) / 2.0;
        let pz = (self.p_plus - self.p_minus) / 2.0;
        FourMomentum::new(e, self.pt_x, self.pt_y, pz)
    }

    /// Invariant mass squared.
    #[allow(dead_code)]
    pub fn invariant_mass_squared(&self) -> f64 {
        self.p_plus * self.p_minus - self.pt_x * self.pt_x - self.pt_y * self.pt_y
    }
}

/// A string segment in lightcone coordinates.
#[derive(Debug, Clone, Copy)]
pub struct StringSegment {
    /// Quark endpoint
    pub quark: LightconeEndpoint,
    /// Antiquark endpoint
    pub antiquark: LightconeEndpoint,
}

impl StringSegment {
    /// Create from two 4-momenta (quark moving in +z, antiquark in -z).
    pub fn from_endpoints(quark: &FourMomentum, antiquark: &FourMomentum) -> Self {
        Self {
            quark: LightconeEndpoint::from_four_momentum(quark),
            antiquark: LightconeEndpoint::from_four_momentum(antiquark),
        }
    }

    /// Total invariant mass squared of the string.
    pub fn invariant_mass_squared(&self) -> f64 {
        let total = self.quark.to_four_momentum() + self.antiquark.to_four_momentum();
        total.invariant_mass_squared()
    }

    /// Total invariant mass.
    pub fn invariant_mass(&self) -> f64 {
        let m_sq = self.invariant_mass_squared();
        if m_sq > 0.0 { m_sq.sqrt() } else { 0.0 }
    }

    /// Total available lightcone momentum for fragmentation from quark end.
    pub fn w_plus(&self) -> f64 {
        self.quark.p_plus + self.antiquark.p_plus
    }

    /// Total available lightcone momentum for fragmentation from antiquark end.
    pub fn w_minus(&self) -> f64 {
        self.quark.p_minus + self.antiquark.p_minus
    }
}

/// Compute the momentum fraction z for hadron production.
///
/// Using the Lund symmetric fragmentation function:
/// f(z) = (1/z) * (1-z)^a * exp(-b * m_T^2 / z)
///
/// We sample z using the rejection method.
pub fn sample_z<R: deep_causality_rand::Rng>(
    rng: &mut R,
    lund_a: f64,
    lund_b: f64,
    mt_squared: f64,
) -> f64 {
    // Use rejection sampling for the Lund function
    // Maximum is approximately at z_max = 1 / (1 + b * m_T^2 / a)
    let z_max = 1.0 / (1.0 + lund_b * mt_squared / (lund_a + 1.0).max(0.01));
    let f_max = lund_function(z_max, lund_a, lund_b, mt_squared);

    loop {
        // Sample z uniformly in [z_min, z_max_cutoff]
        let z_min = 0.01;
        let z_max_cutoff = 0.99;
        let z: f64 = z_min + (z_max_cutoff - z_min) * rng.random::<f64>();

        // Evaluate function
        let f_z = lund_function(z, lund_a, lund_b, mt_squared);

        // Accept/reject
        if rng.random::<f64>() * f_max < f_z {
            return z;
        }
    }
}

/// Evaluate the Lund symmetric fragmentation function.
///
/// f(z) = (1/z) * (1-z)^a * exp(-b * m_T^2 / z)
fn lund_function(z: f64, a: f64, b: f64, mt_sq: f64) -> f64 {
    if z <= 0.0 || z >= 1.0 {
        return 0.0;
    }

    (1.0 / z) * (1.0 - z).powf(a) * (-b * mt_sq / z).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lightcone_round_trip() {
        let p = FourMomentum::new(10.0, 1.0, 2.0, 8.0);
        let lc = LightconeEndpoint::from_four_momentum(&p);
        let p_back = lc.to_four_momentum();

        assert!((p.e() - p_back.e()).abs() < 1e-10);
        assert!((p.px() - p_back.px()).abs() < 1e-10);
        assert!((p.py() - p_back.py()).abs() < 1e-10);
        assert!((p.pz() - p_back.pz()).abs() < 1e-10);
    }

    #[test]
    fn test_lund_function_bounds() {
        // Should be 0 at boundaries
        assert_eq!(lund_function(0.0, 0.68, 0.98, 0.5), 0.0);
        assert_eq!(lund_function(1.0, 0.68, 0.98, 0.5), 0.0);

        // Should be positive inside
        assert!(lund_function(0.5, 0.68, 0.98, 0.5) > 0.0);
    }
}
