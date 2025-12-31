/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Particle Data Group (PDG) particle database for QCD calculations.
//!
//! Contains masses, charges, and quantum numbers for common hadrons.
//! All masses are in GeV/c², consistent with natural units (ℏ = c = 1).
//!
//! Reference: PDG Review of Particle Physics (2024)

/// Particle data from the Particle Data Group.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleData {
    /// PDG Monte Carlo particle ID
    pdg_id: i32,
    /// Rest mass in GeV/c²
    mass: f64,
    /// Electric charge in units of e
    charge: f64,
    /// Spin quantum number J
    spin: f64,
    /// Particle name
    name: &'static str,
}

impl ParticleData {
    /// Creates new particle data.
    pub const fn new(pdg_id: i32, mass: f64, charge: f64, spin: f64, name: &'static str) -> Self {
        Self {
            pdg_id,
            mass,
            charge,
            spin,
            name,
        }
    }

    /// PDG Monte Carlo particle ID.
    pub const fn pdg_id(&self) -> i32 {
        self.pdg_id
    }

    /// Rest mass in GeV/c².
    pub const fn mass(&self) -> f64 {
        self.mass
    }

    /// Electric charge in units of e.
    pub const fn charge(&self) -> f64 {
        self.charge
    }

    /// Spin quantum number J.
    pub const fn spin(&self) -> f64 {
        self.spin
    }

    /// Particle name.
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

// =============================================================================
// PDG Particle Database
// =============================================================================

/// Common hadron database following PDG conventions.
///
/// PDG ID scheme:
/// - Mesons: ±(n₁ n₂ n₃ nJ) where n₁ specifies radial excitation
/// - Baryons: (n₁ n₂ n₃ nJ) with n₁=0 for ground states
pub const PDG_PARTICLES: &[ParticleData] = &[
    // =========================================================================
    // Pseudoscalar Mesons (J^PC = 0^-+)
    // =========================================================================
    ParticleData::new(211, 0.13957039, 1.0, 0.0, "pi+"),
    ParticleData::new(-211, 0.13957039, -1.0, 0.0, "pi-"),
    ParticleData::new(111, 0.1349768, 0.0, 0.0, "pi0"),
    ParticleData::new(321, 0.493677, 1.0, 0.0, "K+"),
    ParticleData::new(-321, 0.493677, -1.0, 0.0, "K-"),
    ParticleData::new(311, 0.497611, 0.0, 0.0, "K0"),
    ParticleData::new(-311, 0.497611, 0.0, 0.0, "K0bar"),
    ParticleData::new(130, 0.497611, 0.0, 0.0, "K0_L"),
    ParticleData::new(310, 0.497611, 0.0, 0.0, "K0_S"),
    ParticleData::new(221, 0.547862, 0.0, 0.0, "eta"),
    ParticleData::new(331, 0.95778, 0.0, 0.0, "eta_prime"),
    ParticleData::new(411, 1.86965, 1.0, 0.0, "D+"),
    ParticleData::new(-411, 1.86965, -1.0, 0.0, "D-"),
    ParticleData::new(421, 1.86483, 0.0, 0.0, "D0"),
    ParticleData::new(-421, 1.86483, 0.0, 0.0, "D0bar"),
    ParticleData::new(431, 1.96834, 1.0, 0.0, "Ds+"),
    ParticleData::new(-431, 1.96834, -1.0, 0.0, "Ds-"),
    ParticleData::new(511, 5.27965, 0.0, 0.0, "B0"),
    ParticleData::new(-511, 5.27965, 0.0, 0.0, "B0bar"),
    ParticleData::new(521, 5.27934, 1.0, 0.0, "B+"),
    ParticleData::new(-521, 5.27934, -1.0, 0.0, "B-"),
    // =========================================================================
    // Vector Mesons (J^PC = 1^--)
    // =========================================================================
    ParticleData::new(113, 0.77526, 0.0, 1.0, "rho0"),
    ParticleData::new(213, 0.77511, 1.0, 1.0, "rho+"),
    ParticleData::new(-213, 0.77511, -1.0, 1.0, "rho-"),
    ParticleData::new(223, 0.78265, 0.0, 1.0, "omega"),
    ParticleData::new(333, 1.019461, 0.0, 1.0, "phi"),
    ParticleData::new(313, 0.89555, 0.0, 1.0, "K*0"),
    ParticleData::new(-313, 0.89555, 0.0, 1.0, "K*0bar"),
    ParticleData::new(323, 0.89166, 1.0, 1.0, "K*+"),
    ParticleData::new(-323, 0.89166, -1.0, 1.0, "K*-"),
    ParticleData::new(443, 3.09690, 0.0, 1.0, "J/psi"),
    ParticleData::new(553, 9.4603, 0.0, 1.0, "Upsilon"),
    // =========================================================================
    // Light Baryons (J = 1/2)
    // =========================================================================
    ParticleData::new(2212, 0.938272081, 1.0, 0.5, "p"),
    ParticleData::new(-2212, 0.938272081, -1.0, 0.5, "pbar"),
    ParticleData::new(2112, 0.939565413, 0.0, 0.5, "n"),
    ParticleData::new(-2112, 0.939565413, 0.0, 0.5, "nbar"),
    // =========================================================================
    // Strange Baryons
    // =========================================================================
    ParticleData::new(3122, 1.115683, 0.0, 0.5, "Lambda"),
    ParticleData::new(-3122, 1.115683, 0.0, 0.5, "Lambda_bar"),
    ParticleData::new(3222, 1.18937, 1.0, 0.5, "Sigma+"),
    ParticleData::new(-3222, 1.18937, -1.0, 0.5, "Sigma-bar"),
    ParticleData::new(3212, 1.192642, 0.0, 0.5, "Sigma0"),
    ParticleData::new(-3212, 1.192642, 0.0, 0.5, "Sigma0bar"),
    ParticleData::new(3112, 1.197449, -1.0, 0.5, "Sigma-"),
    ParticleData::new(-3112, 1.197449, 1.0, 0.5, "Sigma+bar"),
    ParticleData::new(3322, 1.31486, 0.0, 0.5, "Xi0"),
    ParticleData::new(-3322, 1.31486, 0.0, 0.5, "Xi0bar"),
    ParticleData::new(3312, 1.32171, -1.0, 0.5, "Xi-"),
    ParticleData::new(-3312, 1.32171, 1.0, 0.5, "Xi+bar"),
    // =========================================================================
    // Delta Baryons (J = 3/2)
    // =========================================================================
    ParticleData::new(2224, 1.232, 2.0, 1.5, "Delta++"),
    ParticleData::new(-2224, 1.232, -2.0, 1.5, "Delta--bar"),
    ParticleData::new(2214, 1.232, 1.0, 1.5, "Delta+"),
    ParticleData::new(-2214, 1.232, -1.0, 1.5, "Delta-bar"),
    ParticleData::new(2114, 1.232, 0.0, 1.5, "Delta0"),
    ParticleData::new(-2114, 1.232, 0.0, 1.5, "Delta0bar"),
    ParticleData::new(1114, 1.232, -1.0, 1.5, "Delta-"),
    ParticleData::new(-1114, 1.232, 1.0, 1.5, "Delta+bar"),
    // =========================================================================
    // Omega Baryon
    // =========================================================================
    ParticleData::new(3334, 1.67245, -1.0, 1.5, "Omega-"),
    ParticleData::new(-3334, 1.67245, 1.0, 1.5, "Omega+bar"),
    // =========================================================================
    // Charmed Baryons
    // =========================================================================
    ParticleData::new(4122, 2.28646, 1.0, 0.5, "Lambda_c+"),
    ParticleData::new(-4122, 2.28646, -1.0, 0.5, "Lambda_c-"),
];

/// Lookup particle data by PDG ID.
///
/// # Arguments
/// * `pdg_id` - The PDG Monte Carlo particle ID
///
/// # Returns
/// * `Some(&ParticleData)` if found, `None` otherwise
pub fn pdg_lookup(pdg_id: i32) -> Option<&'static ParticleData> {
    PDG_PARTICLES.iter().find(|p| p.pdg_id() == pdg_id)
}

/// Get particle mass by PDG ID (convenience function).
///
/// # Returns
/// * Mass in GeV/c² or 0.0 if particle not found
pub fn pdg_mass(pdg_id: i32) -> f64 {
    pdg_lookup(pdg_id).map(|p| p.mass()).unwrap_or(0.0)
}

/// Light quark constituent masses for string fragmentation.
pub mod quark_masses {
    /// Up quark constituent mass (GeV/c²)
    pub const M_U: f64 = 0.33;
    /// Down quark constituent mass (GeV/c²)
    pub const M_D: f64 = 0.33;
    /// Strange quark constituent mass (GeV/c²)
    pub const M_S: f64 = 0.50;
    /// Charm quark constituent mass (GeV/c²)
    pub const M_C: f64 = 1.5;
    /// Bottom quark constituent mass (GeV/c²)
    pub const M_B: f64 = 4.8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdg_lookup_proton() {
        let p = pdg_lookup(2212).expect("proton should exist");
        assert_eq!(p.name(), "p");
        assert!((p.mass() - 0.938272081).abs() < 1e-6);
        assert!((p.charge() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pdg_lookup_pion_plus() {
        let pi = pdg_lookup(211).expect("pi+ should exist");
        assert_eq!(pi.name(), "pi+");
        assert!((pi.mass() - 0.13957039).abs() < 1e-6);
    }

    #[test]
    fn test_pdg_lookup_not_found() {
        assert!(pdg_lookup(99999).is_none());
    }

    #[test]
    fn test_pdg_mass() {
        assert!((pdg_mass(2212) - 0.938272081).abs() < 1e-6);
        assert_eq!(pdg_mass(99999), 0.0);
    }
}
