/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Flavor selection for Lund string fragmentation.
//!
//! Handles quark-antiquark pair creation during string breaking.

use crate::nuclear::pdg::{pdg_mass, quark_masses};
use deep_causality_rand::{Distribution, Normal, Rng};

/// Flavor ID for quarks (1=d, 2=u, 3=s, 4=c, 5=b, 6=t).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum QuarkFlavor {
    Down = 1,
    Up = 2,
    Strange = 3,
    Charm = 4,
    Bottom = 5,
}

impl QuarkFlavor {
    /// Constituent mass in GeV.
    pub fn mass(self) -> f64 {
        match self {
            QuarkFlavor::Down => quark_masses::M_D,
            QuarkFlavor::Up => quark_masses::M_U,
            QuarkFlavor::Strange => quark_masses::M_S,
            QuarkFlavor::Charm => quark_masses::M_C,
            QuarkFlavor::Bottom => quark_masses::M_B,
        }
    }

    /// Electric charge in units of e.
    pub fn charge(self) -> f64 {
        match self {
            QuarkFlavor::Down | QuarkFlavor::Strange | QuarkFlavor::Bottom => -1.0 / 3.0,
            QuarkFlavor::Up | QuarkFlavor::Charm => 2.0 / 3.0,
        }
    }
}

/// Select a new quark flavor for string breaking.
///
/// Probabilities: u : d : s = 1 : 1 : strange_suppression
pub fn select_quark_flavor<R: Rng>(rng: &mut R, strange_suppression: f64) -> QuarkFlavor {
    let total = 2.0 + strange_suppression;
    let r: f64 = rng.random::<f64>() * total;

    if r < 1.0 {
        QuarkFlavor::Up
    } else if r < 2.0 {
        QuarkFlavor::Down
    } else {
        QuarkFlavor::Strange
    }
}

/// Meson formed from quark-antiquark pair.
#[derive(Debug, Clone, Copy)]
pub struct MesonState {
    /// First quark flavor
    pub q1: QuarkFlavor,
    /// Second quark flavor (antiquark implied)
    pub q2: QuarkFlavor,
    /// Is vector meson (J=1) vs pseudoscalar (J=0)
    pub is_vector: bool,
}

impl MesonState {
    /// Get PDG ID for this meson.
    pub fn pdg_id(&self) -> i32 {
        // Simplified PDG encoding for light mesons
        let id1 = self.q1 as i32;
        let id2 = self.q2 as i32;

        // Sort so larger flavor is first
        let (hi, lo) = if id1 > id2 { (id1, id2) } else { (id2, id1) };

        let base = hi * 100 + lo * 10 + if self.is_vector { 3 } else { 1 };

        // Determine sign from charge
        let charge = self.q1.charge() - self.q2.charge();
        if charge < 0.0 { -base } else { base }
    }

    /// Get mass of this meson from PDG database.
    pub fn mass(&self) -> f64 {
        let pdg = self.pdg_id();
        let m = pdg_mass(pdg.abs());
        if m > 0.0 {
            m
        } else {
            // Fallback to constituent quark model
            self.q1.mass() + self.q2.mass()
        }
    }
}

/// Select meson spin (vector vs pseudoscalar).
pub fn select_meson_spin<R: Rng>(rng: &mut R, vector_fraction: f64) -> bool {
    rng.random::<f64>() < vector_fraction
}

/// Generate transverse momentum according to Gaussian distribution.
///
/// Returns (px, py) with Gaussian distribution of width sigma.
pub fn generate_transverse_momentum<R: Rng>(rng: &mut R, sigma: f64) -> (f64, f64) {
    let normal = Normal::new(0.0, sigma).unwrap();
    (normal.sample(rng), normal.sample(rng))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quark_flavor_mass() {
        assert!((QuarkFlavor::Up.mass() - 0.33).abs() < 0.01);
        assert!((QuarkFlavor::Strange.mass() - 0.50).abs() < 0.01);
    }

    #[test]
    fn test_quark_flavor_charge() {
        assert!((QuarkFlavor::Up.charge() - 2.0 / 3.0).abs() < 0.01);
        assert!((QuarkFlavor::Down.charge() + 1.0 / 3.0).abs() < 0.01);
    }
}
