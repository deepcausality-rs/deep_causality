/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! String representation for Lund model.
//!
//! A string connects quark endpoints and stores the current state
//! during iterated fragmentation.

use crate::nuclear::quantities::FourMomentum;

use super::kinematics::{LightconeEndpoint, StringSegment};

/// A QCD string connecting a quark and antiquark.
#[derive(Debug, Clone)]
pub struct LundString {
    /// Current string segment
    segment: StringSegment,
    /// Remaining momentum fraction from quark end
    z_used_quark: f64,
    /// Remaining momentum fraction from antiquark end
    z_used_antiquark: f64,
}

impl LundString {
    /// Create a new string from endpoint 4-momenta.
    pub fn new(quark: FourMomentum, antiquark: FourMomentum) -> Self {
        Self {
            segment: StringSegment::from_endpoints(&quark, &antiquark),
            z_used_quark: 0.0,
            z_used_antiquark: 0.0,
        }
    }

    /// Total invariant mass of the string.
    pub fn invariant_mass(&self) -> f64 {
        self.segment.invariant_mass()
    }

    /// Check if string has enough mass to continue fragmentation.
    pub fn can_fragment(&self, min_mass: f64) -> bool {
        self.invariant_mass() > min_mass
    }

    /// Available forward lightcone momentum (W+).
    pub fn w_plus(&self) -> f64 {
        self.segment.w_plus() * (1.0 - self.z_used_quark)
    }

    /// Available backward lightcone momentum (W-).
    pub fn w_minus(&self) -> f64 {
        self.segment.w_minus() * (1.0 - self.z_used_antiquark)
    }

    /// Take a momentum fraction z from the quark end.
    ///
    /// Returns the hadron 4-momentum.
    pub fn take_from_quark(&mut self, z: f64, pt_x: f64, pt_y: f64, mass: f64) -> FourMomentum {
        let w_plus = self.w_plus();
        let hadron_p_plus = z * w_plus;

        // Compute p- from mass constraint: p+ * p- = m_T^2
        let mt_sq = mass * mass + pt_x * pt_x + pt_y * pt_y;
        let hadron_p_minus = mt_sq / hadron_p_plus.max(1e-10);

        // Update used fraction
        self.z_used_quark += z * (1.0 - self.z_used_quark);

        // Also need to account for p- taken from antiquark end
        let w_minus = self.segment.w_minus();
        if w_minus > 1e-10 {
            self.z_used_antiquark += hadron_p_minus / w_minus;
        }

        // Convert to 4-momentum
        let e = (hadron_p_plus + hadron_p_minus) / 2.0;
        let pz = (hadron_p_plus - hadron_p_minus) / 2.0;

        FourMomentum::new(e, pt_x, pt_y, pz)
    }

    /// Take a momentum fraction z from the antiquark end.
    ///
    /// Returns the hadron 4-momentum.
    pub fn take_from_antiquark(&mut self, z: f64, pt_x: f64, pt_y: f64, mass: f64) -> FourMomentum {
        let w_minus = self.w_minus();
        let hadron_p_minus = z * w_minus;

        // Compute p+ from mass constraint: p+ * p- = m_T^2
        let mt_sq = mass * mass + pt_x * pt_x + pt_y * pt_y;
        let hadron_p_plus = mt_sq / hadron_p_minus.max(1e-10);

        // Update used fraction
        self.z_used_antiquark += z * (1.0 - self.z_used_antiquark);

        // Also need to account for p+ taken from quark end
        let w_plus = self.segment.w_plus();
        if w_plus > 1e-10 {
            self.z_used_quark += hadron_p_plus / w_plus;
        }

        // Convert to 4-momentum
        let e = (hadron_p_plus + hadron_p_minus) / 2.0;
        let pz = (hadron_p_plus - hadron_p_minus) / 2.0;

        FourMomentum::new(e, pt_x, pt_y, pz)
    }

    /// Get quark endpoint.
    #[allow(dead_code)]
    pub fn quark_endpoint(&self) -> &LightconeEndpoint {
        &self.segment.quark
    }

    /// Get antiquark endpoint.
    #[allow(dead_code)]
    pub fn antiquark_endpoint(&self) -> &LightconeEndpoint {
        &self.segment.antiquark
    }

    /// Create the final hadron from remaining string momentum.
    ///
    /// This is called when the string can't fragment further.
    pub fn final_hadron(&self, _mass: f64) -> FourMomentum {
        // Use remaining momentum
        let p_plus = self.w_plus();
        let p_minus = self.w_minus();

        let e = (p_plus + p_minus) / 2.0;
        let pz = (p_plus - p_minus) / 2.0;

        // Transverse momentum is approximately zero for final hadron
        FourMomentum::new(e, 0.0, 0.0, pz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lund_string_creation() {
        let q = FourMomentum::new(50.0, 0.0, 0.0, 50.0);
        let qbar = FourMomentum::new(50.0, 0.0, 0.0, -50.0);
        let string = LundString::new(q, qbar);

        assert!(string.invariant_mass() > 99.0);
        assert!(string.can_fragment(10.0));
    }

    #[test]
    fn test_take_from_quark() {
        let q = FourMomentum::new(50.0, 0.0, 0.0, 50.0);
        let qbar = FourMomentum::new(50.0, 0.0, 0.0, -50.0);
        let mut string = LundString::new(q, qbar);

        let hadron = string.take_from_quark(0.3, 0.0, 0.0, 0.14);

        // Hadron should have positive energy
        assert!(hadron.e() > 0.0);
        // Should be moving in +z direction (from quark end)
        assert!(hadron.pz() > 0.0);
    }
}
