/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;

/// Amount of Substance (Moles).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct AmountOfSubstance(f64);

impl AmountOfSubstance {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative AmountOfSubstance".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<AmountOfSubstance> for f64 {
    fn from(val: AmountOfSubstance) -> Self {
        val.0
    }
}

/// Half-Life (Seconds).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct HalfLife(f64);

impl HalfLife {
    /// Creates a new `HalfLife` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError` if `val <= 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "HalfLife must be finite: {}",
                val
            )));
        }
        if val <= 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "HalfLife must be positive (zero implies infinite decay rate)".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<HalfLife> for f64 {
    fn from(val: HalfLife) -> Self {
        val.0
    }
}

/// Radioactivity (Becquerels).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Activity(f64);

impl Activity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Activity".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Activity> for f64 {
    fn from(val: Activity) -> Self {
        val.0
    }
}

/// Energy Density (Joules per cubic meter).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct EnergyDensity(f64);

impl EnergyDensity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative EnergyDensity".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<EnergyDensity> for f64 {
    fn from(val: EnergyDensity) -> Self {
        val.0
    }
}

// =============================================================================
// Lund String Fragmentation Types
// =============================================================================

use core::ops::{Add, Sub};

/// Lorentz 4-momentum (E, px, py, pz) in natural units (c = 1).
///
/// Components are stored in (+---) signature convention (particle physics).
/// Energy E is component 0, spatial momentum (px, py, pz) are components 1-3.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FourMomentum {
    /// Energy component E (GeV)
    e: f64,
    /// x-component of 3-momentum px (GeV/c)
    px: f64,
    /// y-component of 3-momentum py (GeV/c)
    py: f64,
    /// z-component of 3-momentum pz (GeV/c)
    pz: f64,
}

impl FourMomentum {
    /// Creates a new 4-momentum.
    pub const fn new(e: f64, px: f64, py: f64, pz: f64) -> Self {
        Self { e, px, py, pz }
    }

    /// Creates a 4-momentum from mass and 3-momentum.
    ///
    /// Computes E = √(m² + |p|²)
    pub fn from_mass_and_momentum(mass: f64, px: f64, py: f64, pz: f64) -> Self {
        let p_sq = px * px + py * py + pz * pz;
        let e = (mass * mass + p_sq).sqrt();
        Self { e, px, py, pz }
    }

    /// Creates a 4-momentum for a particle at rest.
    pub const fn at_rest(mass: f64) -> Self {
        Self {
            e: mass,
            px: 0.0,
            py: 0.0,
            pz: 0.0,
        }
    }

    /// Energy component E.
    pub const fn e(&self) -> f64 {
        self.e
    }

    /// x-component of 3-momentum.
    pub const fn px(&self) -> f64 {
        self.px
    }

    /// y-component of 3-momentum.
    pub const fn py(&self) -> f64 {
        self.py
    }

    /// z-component of 3-momentum.
    pub const fn pz(&self) -> f64 {
        self.pz
    }

    /// Lorentz-invariant mass squared: m² = E² - |p|²
    pub fn invariant_mass_squared(&self) -> f64 {
        self.e * self.e - self.px * self.px - self.py * self.py - self.pz * self.pz
    }

    /// Lorentz-invariant mass: m = √(E² - |p|²)
    ///
    /// Returns 0.0 for massless or spacelike 4-vectors.
    pub fn invariant_mass(&self) -> f64 {
        let m_sq = self.invariant_mass_squared();
        if m_sq > 0.0 { m_sq.sqrt() } else { 0.0 }
    }

    /// Magnitude of 3-momentum: |p| = √(px² + py² + pz²)
    pub fn momentum_magnitude(&self) -> f64 {
        (self.px * self.px + self.py * self.py + self.pz * self.pz).sqrt()
    }

    /// Transverse momentum: pT = √(px² + py²)
    pub fn transverse_momentum(&self) -> f64 {
        (self.px * self.px + self.py * self.py).sqrt()
    }

    /// Transverse mass: mT = √(m² + pT²)
    pub fn transverse_mass(&self) -> f64 {
        let m_sq = self.invariant_mass_squared();
        let pt_sq = self.px * self.px + self.py * self.py;
        (m_sq + pt_sq).sqrt()
    }

    /// Rapidity: y = 0.5 * ln((E + pz) / (E - pz))
    ///
    /// Returns 0.0 if denominator is near zero.
    pub fn rapidity(&self) -> f64 {
        let denom = self.e - self.pz;
        if denom.abs() < 1e-10 {
            return 0.0;
        }
        0.5 * ((self.e + self.pz) / denom).ln()
    }

    /// Pseudorapidity: η = -ln(tan(θ/2))
    ///
    /// where θ is the polar angle from the beam axis (z).
    pub fn pseudorapidity(&self) -> f64 {
        let p = self.momentum_magnitude();
        if p.abs() < 1e-10 {
            return 0.0;
        }
        0.5 * ((p + self.pz) / (p - self.pz)).ln()
    }

    /// Azimuthal angle φ in the transverse plane.
    pub fn phi(&self) -> f64 {
        self.py.atan2(self.px)
    }

    /// Lightcone plus component: p+ = E + pz
    pub fn lightcone_plus(&self) -> f64 {
        self.e + self.pz
    }

    /// Lightcone minus component: p- = E - pz
    pub fn lightcone_minus(&self) -> f64 {
        self.e - self.pz
    }

    /// Lorentz boost along z-axis.
    pub fn boost_z(&self, beta: f64) -> Self {
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        let e_new = gamma * (self.e - beta * self.pz);
        let pz_new = gamma * (self.pz - beta * self.e);
        Self {
            e: e_new,
            px: self.px,
            py: self.py,
            pz: pz_new,
        }
    }
}

impl Add for FourMomentum {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            e: self.e + rhs.e,
            px: self.px + rhs.px,
            py: self.py + rhs.py,
            pz: self.pz + rhs.pz,
        }
    }
}

impl Sub for FourMomentum {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            e: self.e - rhs.e,
            px: self.px - rhs.px,
            py: self.py - rhs.py,
            pz: self.pz - rhs.pz,
        }
    }
}

/// A produced hadron from string fragmentation.
#[derive(Debug, Clone, PartialEq)]
pub struct Hadron {
    /// PDG Monte Carlo particle ID
    pdg_id: i32,
    /// 4-momentum in the lab frame
    momentum: FourMomentum,
}

impl Hadron {
    /// Creates a new hadron.
    pub const fn new(pdg_id: i32, momentum: FourMomentum) -> Self {
        Self { pdg_id, momentum }
    }

    /// PDG Monte Carlo particle ID.
    pub const fn pdg_id(&self) -> i32 {
        self.pdg_id
    }

    /// 4-momentum in the lab frame.
    pub const fn momentum(&self) -> FourMomentum {
        self.momentum
    }

    /// Invariant mass of the hadron.
    pub fn mass(&self) -> f64 {
        self.momentum.invariant_mass()
    }

    /// Energy of the hadron.
    pub fn energy(&self) -> f64 {
        self.momentum.e()
    }

    /// Transverse momentum.
    pub fn pt(&self) -> f64 {
        self.momentum.transverse_momentum()
    }

    /// Rapidity.
    pub fn rapidity(&self) -> f64 {
        self.momentum.rapidity()
    }
}

/// Configuration parameters for Lund String Fragmentation.
///
/// The Lund symmetric fragmentation function is:
/// $$ f(z) = \frac{1}{z} (1-z)^a \exp\left(-\frac{b \cdot m_T^2}{z}\right) $$
///
/// where z is the lightcone momentum fraction and mT is the transverse mass.
#[derive(Debug, Clone, PartialEq)]
pub struct LundParameters {
    /// String tension κ (GeV/fm) - energy per unit length of the color flux tube
    kappa: f64,
    /// Lund 'a' parameter - controls the shape of the fragmentation function
    lund_a: f64,
    /// Lund 'b' parameter (GeV⁻²) - controls mass dependence
    lund_b: f64,
    /// Transverse momentum width σ_pT (GeV) - Gaussian width for pT generation
    sigma_pt: f64,
    /// Strange quark suppression factor s/u (typically 0.2-0.4)
    strange_suppression: f64,
    /// Diquark suppression factor qq/q (typically 0.05-0.15)
    diquark_suppression: f64,
    /// Spin-1 meson suppression relative to spin-0 (vector/pseudoscalar ratio)
    vector_meson_fraction: f64,
    /// Minimum invariant mass to continue fragmentation (GeV)
    min_invariant_mass: f64,
}

impl Default for LundParameters {
    /// Default parameters tuned to LEP e+e- data.
    ///
    /// Based on PYTHIA 8 Monash tune.
    fn default() -> Self {
        Self {
            kappa: 1.0,                  // GeV/fm
            lund_a: 0.68,                // dimensionless
            lund_b: 0.98,                // GeV⁻²
            sigma_pt: 0.36,              // GeV
            strange_suppression: 0.30,   // s/u ratio
            diquark_suppression: 0.10,   // qq/q ratio
            vector_meson_fraction: 0.50, // V/(V+PS)
            min_invariant_mass: 0.5,     // GeV
        }
    }
}

impl LundParameters {
    /// Creates new Lund parameters.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        kappa: f64,
        lund_a: f64,
        lund_b: f64,
        sigma_pt: f64,
        strange_suppression: f64,
        diquark_suppression: f64,
        vector_meson_fraction: f64,
        min_invariant_mass: f64,
    ) -> Self {
        Self {
            kappa,
            lund_a,
            lund_b,
            sigma_pt,
            strange_suppression,
            diquark_suppression,
            vector_meson_fraction,
            min_invariant_mass,
        }
    }

    /// String tension κ (GeV/fm).
    pub const fn kappa(&self) -> f64 {
        self.kappa
    }

    /// Lund 'a' parameter.
    pub const fn lund_a(&self) -> f64 {
        self.lund_a
    }

    /// Lund 'b' parameter (GeV⁻²).
    pub const fn lund_b(&self) -> f64 {
        self.lund_b
    }

    /// Transverse momentum width σ_pT (GeV).
    pub const fn sigma_pt(&self) -> f64 {
        self.sigma_pt
    }

    /// Strange quark suppression factor s/u.
    pub const fn strange_suppression(&self) -> f64 {
        self.strange_suppression
    }

    /// Diquark suppression factor qq/q.
    pub const fn diquark_suppression(&self) -> f64 {
        self.diquark_suppression
    }

    /// Vector meson fraction V/(V+PS).
    pub const fn vector_meson_fraction(&self) -> f64 {
        self.vector_meson_fraction
    }

    /// Minimum invariant mass to continue fragmentation (GeV).
    pub const fn min_invariant_mass(&self) -> f64 {
        self.min_invariant_mass
    }
}
