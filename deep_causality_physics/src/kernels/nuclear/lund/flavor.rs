/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Flavor selection for Lund string fragmentation.
//!
//! Handles quark-antiquark pair creation during string breaking.
//!
//! # Precision boundary
//!
//! The internal arithmetic on physical quantities (masses, transverse momenta) is generic over
//! `R: RealField`. The Monte Carlo draws are taken at `f64` and lifted into `R` via `R::from_f64`
//! at the boundary; for `R = f64` that is a no-op.
//!
//! Sampling wider is available and is not used. `deep_causality_rand` implements
//! `Distribution<Float106>` for `StandardUniform`, `Open01`, `OpenClosed01` and `StandardNormal`.
//! The three draws here do not benefit from it:
//!
//! - **Flavour selection** scales a uniform by the weight total and walks the cumulative weights.
//!   The output is a discrete index, so bits below `2^-53` change it only when the scaled draw
//!   lands within about `1e-16` of a weight boundary.
//! - **The vector/pseudoscalar choice** compares a uniform against a fixed fraction and returns a
//!   bool, with the same measure argument.
//! - **Transverse momentum** is a Gaussian, and here the wider draw is the narrower one. The `f64`
//!   `StandardNormal` is a ziggurat with the Marsaglia tail algorithm, whose reach is unbounded.
//!   The `Float106` implementation is Box–Muller, `radius = sqrt(-2·ln u1)` over an `Open01` whose
//!   floor is `2^-53`, so it cannot exceed `|z| ≈ 8.57`. Both caps sit past `P ≈ 1e-17`, beyond
//!   reach at any feasible sample count, so neither is a reason to prefer the other.
//!
//! A `Float106` stream is also not a refinement of the `f64` one: a `Float106` uniform consumes
//! two `f64` draws and its normal four, so switching would replace every seeded sequence rather
//! than extend it.

use crate::kernels::nuclear::pdg::{pdg_mass, quark_masses};
use crate::real_from_f64;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
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
    /// Constituent mass in GeV, lifted into the target field `R`.
    pub fn mass<R: RealField + FromPrimitive>(self) -> R {
        let m = match self {
            QuarkFlavor::Down => quark_masses::M_D,
            QuarkFlavor::Up => quark_masses::M_U,
            QuarkFlavor::Strange => quark_masses::M_S,
            QuarkFlavor::Charm => quark_masses::M_C,
            QuarkFlavor::Bottom => quark_masses::M_B,
        };
        real_from_f64(m)
    }

    /// Electric charge in units of e, lifted into `R`.
    pub fn charge<R: RealField + FromPrimitive>(self) -> R {
        let q = match self {
            QuarkFlavor::Down | QuarkFlavor::Strange | QuarkFlavor::Bottom => -1.0 / 3.0,
            QuarkFlavor::Up | QuarkFlavor::Charm => 2.0 / 3.0,
        };
        real_from_f64(q)
    }
}

/// Select a new quark flavor for string breaking.
///
/// Probabilities: u : d : s = 1 : 1 : strange_suppression.
///
/// The raw uniform draw is sampled at f64 (RNG boundary) and lifted into `R`;
/// the threshold comparison is then done entirely in the target field.
pub fn select_quark_flavor<R, RNG>(rng: &mut RNG, strange_suppression: R) -> QuarkFlavor
where
    R: RealField + FromPrimitive,
    RNG: Rng,
{
    let two = real_from_f64::<R>(2.0);
    let total = two + strange_suppression;
    let r = real_from_f64::<R>(rng.random::<f64>()) * total;

    if r < R::one() {
        QuarkFlavor::Up
    } else if r < two {
        QuarkFlavor::Down
    } else {
        QuarkFlavor::Strange
    }
}

/// Meson formed from quark-antiquark pair.
#[derive(Debug, Clone, Copy)]
pub struct MesonState {
    pub q1: QuarkFlavor,
    pub q2: QuarkFlavor,
    pub is_vector: bool,
}

impl MesonState {
    /// Get PDG ID for this meson.
    pub fn pdg_id(&self) -> i32 {
        let id1 = self.q1 as i32;
        let id2 = self.q2 as i32;

        let (hi, lo) = if id1 > id2 { (id1, id2) } else { (id2, id1) };

        let base = hi * 100 + lo * 10 + if self.is_vector { 3 } else { 1 };

        // Sign from charge — compute at f64 since pure integer flavor lookup.
        let charge_f64 = self.q1.charge::<f64>() - self.q2.charge::<f64>();
        if charge_f64 < 0.0 { -base } else { base }
    }

    /// Get mass of this meson from PDG database, lifted into `R`.
    pub fn mass<R: RealField + FromPrimitive>(&self) -> R {
        let pdg = self.pdg_id();
        let m_f64 = pdg_mass(pdg.abs());
        if m_f64 > 0.0 {
            real_from_f64(m_f64)
        } else {
            // Fallback to constituent quark model
            self.q1.mass::<R>() + self.q2.mass::<R>()
        }
    }
}

/// Select meson spin (vector vs pseudoscalar).
///
/// The uniform draw is sampled at f64 (RNG boundary) and lifted into `R` for the
/// comparison against the (now field-typed) vector fraction.
pub fn select_meson_spin<R, RNG>(rng: &mut RNG, vector_fraction: R) -> bool
where
    R: RealField + FromPrimitive,
    RNG: Rng,
{
    real_from_f64::<R>(rng.random::<f64>()) < vector_fraction
}

/// Generate transverse momentum according to Gaussian distribution.
///
/// Returns (px, py) in the target field `R`. The Gaussian is sampled as a **unit** normal at f64
/// via `deep_causality_rand::Normal`, lifted into `R`, then scaled by the field-typed width
/// `sigma` — so the physical width never leaves `R`. See the module docs for why the draw stays
/// at f64 when a `Float106` normal exists.
pub fn generate_transverse_momentum<R, RNG>(rng: &mut RNG, sigma: R) -> (R, R)
where
    R: RealField + FromPrimitive,
    RNG: Rng,
{
    let normal = Normal::new(0.0, 1.0).unwrap();
    let px = sigma * real_from_f64::<R>(normal.sample(rng));
    let py = sigma * real_from_f64::<R>(normal.sample(rng));
    (px, py)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quark_flavor_mass() {
        assert!((QuarkFlavor::Up.mass::<f64>() - 0.33).abs() < 0.01);
        assert!((QuarkFlavor::Strange.mass::<f64>() - 0.50).abs() < 0.01);
    }

    #[test]
    fn test_quark_flavor_charge() {
        assert!((QuarkFlavor::Up.charge::<f64>() - 2.0 / 3.0).abs() < 0.01);
        assert!((QuarkFlavor::Down.charge::<f64>() + 1.0 / 3.0).abs() < 0.01);
    }
}
