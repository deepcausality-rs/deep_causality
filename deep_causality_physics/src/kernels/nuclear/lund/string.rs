/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! String representation for Lund model.
//!
//! A string connects quark endpoints and stores the current state
//! during iterated fragmentation. The state is generic over `R: RealField`;
//! see the precision-boundary note in `flavor.rs` / `kinematics.rs` for how
//! the f64-bound RNG sampling is bridged into `R`.

use crate::FourMomentum;
use crate::real_from_f64;
use deep_causality_num::{FromPrimitive, RealField};

use super::kinematics::{LightconeEndpoint, StringSegment};

/// A QCD string connecting a quark and antiquark.
#[derive(Debug, Clone)]
pub struct LundString<R: RealField> {
    /// Current string segment
    segment: StringSegment<R>,
    /// Remaining momentum fraction from quark end
    z_used_quark: R,
    /// Remaining momentum fraction from antiquark end
    z_used_antiquark: R,
}

impl<R: RealField + FromPrimitive> LundString<R> {
    /// Create a new string from endpoint 4-momenta.
    pub fn new(quark: FourMomentum<R>, antiquark: FourMomentum<R>) -> Self {
        Self {
            segment: StringSegment::from_endpoints(&quark, &antiquark),
            z_used_quark: R::zero(),
            z_used_antiquark: R::zero(),
        }
    }

    /// Total invariant mass of the string.
    pub fn invariant_mass(&self) -> R {
        self.segment.invariant_mass()
    }

    /// Check if string has enough mass to continue fragmentation.
    pub fn can_fragment(&self, min_mass: R) -> bool {
        self.invariant_mass() > min_mass
    }

    /// Available forward lightcone momentum (W+).
    pub fn w_plus(&self) -> R {
        self.segment.w_plus() * (R::one() - self.z_used_quark)
    }

    /// Available backward lightcone momentum (W-).
    pub fn w_minus(&self) -> R {
        self.segment.w_minus() * (R::one() - self.z_used_antiquark)
    }

    /// Take a momentum fraction z from the quark end.
    pub fn take_from_quark(&mut self, z: R, pt_x: R, pt_y: R, mass: R) -> FourMomentum<R> {
        let two = real_from_f64::<R>(2.0);
        let eps = real_from_f64::<R>(1e-10);

        let w_plus = self.w_plus();
        let hadron_p_plus = z * w_plus;

        // Compute p- from mass constraint: p+ * p- = m_T^2
        let mt_sq = mass * mass + pt_x * pt_x + pt_y * pt_y;
        let denom = if hadron_p_plus > eps {
            hadron_p_plus
        } else {
            eps
        };
        let hadron_p_minus = mt_sq / denom;

        // Update used fraction
        self.z_used_quark += z * (R::one() - self.z_used_quark);

        // Also need to account for p- taken from antiquark end
        let w_minus = self.segment.w_minus();
        if w_minus > eps {
            self.z_used_antiquark += hadron_p_minus / w_minus;
        }

        // Convert to 4-momentum
        let e = (hadron_p_plus + hadron_p_minus) / two;
        let pz = (hadron_p_plus - hadron_p_minus) / two;

        FourMomentum::<R>::new(e, pt_x, pt_y, pz)
    }

    /// Take a momentum fraction z from the antiquark end.
    pub fn take_from_antiquark(&mut self, z: R, pt_x: R, pt_y: R, mass: R) -> FourMomentum<R> {
        let two = real_from_f64::<R>(2.0);
        let eps = real_from_f64::<R>(1e-10);

        let w_minus = self.w_minus();
        let hadron_p_minus = z * w_minus;

        // Compute p+ from mass constraint: p+ * p- = m_T^2
        let mt_sq = mass * mass + pt_x * pt_x + pt_y * pt_y;
        let denom = if hadron_p_minus > eps {
            hadron_p_minus
        } else {
            eps
        };
        let hadron_p_plus = mt_sq / denom;

        // Update used fraction
        self.z_used_antiquark += z * (R::one() - self.z_used_antiquark);

        // Also need to account for p+ taken from quark end
        let w_plus = self.segment.w_plus();
        if w_plus > eps {
            self.z_used_quark += hadron_p_plus / w_plus;
        }

        // Convert to 4-momentum
        let e = (hadron_p_plus + hadron_p_minus) / two;
        let pz = (hadron_p_plus - hadron_p_minus) / two;

        FourMomentum::<R>::new(e, pt_x, pt_y, pz)
    }

    /// Get quark endpoint.
    #[allow(dead_code)]
    pub fn quark_endpoint(&self) -> &LightconeEndpoint<R> {
        &self.segment.quark
    }

    /// Get antiquark endpoint.
    #[allow(dead_code)]
    pub fn antiquark_endpoint(&self) -> &LightconeEndpoint<R> {
        &self.segment.antiquark
    }

    /// Create the final hadron from remaining string momentum.
    ///
    /// This is called when the string can't fragment further.
    pub fn final_hadron(&self, _mass: R) -> FourMomentum<R> {
        let two = real_from_f64::<R>(2.0);

        let p_plus = self.w_plus();
        let p_minus = self.w_minus();

        let e = (p_plus + p_minus) / two;
        let pz = (p_plus - p_minus) / two;

        // Transverse momentum is approximately zero for final hadron
        FourMomentum::<R>::new(e, R::zero(), R::zero(), pz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lund_string_creation() {
        let q = FourMomentum::<f64>::new(50.0, 0.0, 0.0, 50.0);
        let qbar = FourMomentum::<f64>::new(50.0, 0.0, 0.0, -50.0);
        let string: LundString<f64> = LundString::new(q, qbar);

        assert!(string.invariant_mass() > 99.0);
        assert!(string.can_fragment(10.0));
    }

    #[test]
    fn test_take_from_quark() {
        let q = FourMomentum::<f64>::new(50.0, 0.0, 0.0, 50.0);
        let qbar = FourMomentum::<f64>::new(50.0, 0.0, 0.0, -50.0);
        let mut string: LundString<f64> = LundString::new(q, qbar);

        let hadron = string.take_from_quark(0.3, 0.0, 0.0, 0.14);

        // Hadron should have positive energy
        assert!(hadron.e() > 0.0);
        // Should be moving in +z direction (from quark end)
        assert!(hadron.pz() > 0.0);
    }
}
