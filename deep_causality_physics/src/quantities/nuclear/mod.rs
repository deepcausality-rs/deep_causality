/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;

/// Amount of Substance (Moles).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AmountOfSubstance<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for AmountOfSubstance<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> AmountOfSubstance<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative AmountOfSubstance".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<AmountOfSubstance<R>> for f64 {
    fn from(val: AmountOfSubstance<R>) -> Self {
        val.0.into()
    }
}

/// Half-Life (Seconds).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HalfLife<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for HalfLife<R> {
    fn default() -> Self {
        Self(R::epsilon())
    }
}

impl<R: deep_causality_num::RealField> HalfLife<R> {
    /// Creates a new `HalfLife` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError` if `val <= 0.0`.
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "HalfLife must be finite".into(),
            ));
        }
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "HalfLife must be positive (zero implies infinite decay rate)".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<HalfLife<R>> for f64 {
    fn from(val: HalfLife<R>) -> Self {
        val.0.into()
    }
}

/// Radioactivity (Becquerels).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Activity<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Activity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Activity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Activity".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<Activity<R>> for f64 {
    fn from(val: Activity<R>) -> Self {
        val.0.into()
    }
}

/// Energy Density (Joules per cubic meter).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct EnergyDensity<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for EnergyDensity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> EnergyDensity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative EnergyDensity".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<EnergyDensity<R>> for f64 {
    fn from(val: EnergyDensity<R>) -> Self {
        val.0.into()
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FourMomentum<R: deep_causality_num::RealField> {
    /// Energy component E (GeV)
    e: R,
    /// x-component of 3-momentum px (GeV/c)
    px: R,
    /// y-component of 3-momentum py (GeV/c)
    py: R,
    /// z-component of 3-momentum pz (GeV/c)
    pz: R,
}

impl<R: deep_causality_num::RealField> Default for FourMomentum<R> {
    fn default() -> Self {
        Self {
            e: R::zero(),
            px: R::zero(),
            py: R::zero(),
            pz: R::zero(),
        }
    }
}

impl<R: deep_causality_num::RealField> FourMomentum<R> {
    /// Creates a new 4-momentum.
    pub const fn new(e: R, px: R, py: R, pz: R) -> Self {
        Self { e, px, py, pz }
    }

    /// Creates a 4-momentum from mass and 3-momentum.
    ///
    /// Computes E = √(m² + |p|²)
    pub fn from_mass_and_momentum(mass: R, px: R, py: R, pz: R) -> Self {
        let p_sq = px * px + py * py + pz * pz;
        let e = (mass * mass + p_sq).sqrt();
        Self { e, px, py, pz }
    }

    /// Creates a 4-momentum for a particle at rest.
    pub fn at_rest(mass: R) -> Self {
        Self {
            e: mass,
            px: R::zero(),
            py: R::zero(),
            pz: R::zero(),
        }
    }

    /// Energy component E.
    pub fn e(&self) -> R {
        self.e
    }

    /// x-component of 3-momentum.
    pub fn px(&self) -> R {
        self.px
    }

    /// y-component of 3-momentum.
    pub fn py(&self) -> R {
        self.py
    }

    /// z-component of 3-momentum.
    pub fn pz(&self) -> R {
        self.pz
    }

    /// Lorentz-invariant mass squared: m² = E² - |p|²
    pub fn invariant_mass_squared(&self) -> R {
        self.e * self.e - self.px * self.px - self.py * self.py - self.pz * self.pz
    }

    /// Lorentz-invariant mass: m = √(E² - |p|²)
    ///
    /// Returns 0 for massless or spacelike 4-vectors.
    pub fn invariant_mass(&self) -> R {
        let m_sq = self.invariant_mass_squared();
        if m_sq > R::zero() {
            m_sq.sqrt()
        } else {
            R::zero()
        }
    }

    /// Magnitude of 3-momentum: |p| = √(px² + py² + pz²)
    pub fn momentum_magnitude(&self) -> R {
        (self.px * self.px + self.py * self.py + self.pz * self.pz).sqrt()
    }

    /// Transverse momentum: pT = √(px² + py²)
    pub fn transverse_momentum(&self) -> R {
        (self.px * self.px + self.py * self.py).sqrt()
    }

    /// Transverse mass: mT = √(m² + pT²)
    pub fn transverse_mass(&self) -> R {
        let m_sq = self.invariant_mass_squared();
        let pt_sq = self.px * self.px + self.py * self.py;
        (m_sq + pt_sq).sqrt()
    }

    /// Lightcone plus component: p+ = E + pz
    pub fn lightcone_plus(&self) -> R {
        self.e + self.pz
    }

    /// Lightcone minus component: p- = E - pz
    pub fn lightcone_minus(&self) -> R {
        self.e - self.pz
    }

    /// Azimuthal angle φ in the transverse plane.
    pub fn phi(&self) -> R {
        self.py.atan2(self.px)
    }
}

impl<R: deep_causality_num::RealField + deep_causality_num::FromPrimitive> FourMomentum<R> {
    /// Rapidity: y = 0.5 * ln((E + pz) / (E - pz))
    ///
    /// Returns 0.0 if denominator is near zero.
    pub fn rapidity(&self) -> R {
        let denom = self.e - self.pz;
        let eps = R::from_f64(1e-10).expect("R::from_f64(1e-10) failed");
        if denom.abs() < eps {
            return R::zero();
        }
        let half = R::from_f64(0.5).expect("R::from_f64(0.5) failed");
        half * ((self.e + self.pz) / denom).ln()
    }

    /// Pseudorapidity: η = -ln(tan(θ/2))
    ///
    /// where θ is the polar angle from the beam axis (z).
    pub fn pseudorapidity(&self) -> R {
        let p = self.momentum_magnitude();
        let eps = R::from_f64(1e-10).expect("R::from_f64(1e-10) failed");
        if p.abs() < eps {
            return R::zero();
        }
        let half = R::from_f64(0.5).expect("R::from_f64(0.5) failed");
        half * ((p + self.pz) / (p - self.pz)).ln()
    }

    /// Lorentz boost along z-axis.
    pub fn boost_z(&self, beta: R) -> Self {
        let one = R::one();
        let gamma = one / (one - beta * beta).sqrt();
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

impl<R: deep_causality_num::RealField> Add for FourMomentum<R> {
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

impl<R: deep_causality_num::RealField> Sub for FourMomentum<R> {
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
pub struct Hadron<R: deep_causality_num::RealField> {
    /// PDG Monte Carlo particle ID
    pdg_id: i32,
    /// 4-momentum in the lab frame
    momentum: FourMomentum<R>,
}

impl<R: deep_causality_num::RealField> Hadron<R> {
    /// Creates a new hadron.
    pub const fn new(pdg_id: i32, momentum: FourMomentum<R>) -> Self {
        Self { pdg_id, momentum }
    }

    /// PDG Monte Carlo particle ID.
    pub const fn pdg_id(&self) -> i32 {
        self.pdg_id
    }

    /// 4-momentum in the lab frame.
    pub fn momentum(&self) -> FourMomentum<R> {
        self.momentum
    }

    /// Invariant mass of the hadron.
    pub fn mass(&self) -> R {
        self.momentum.invariant_mass()
    }

    /// Energy of the hadron.
    pub fn energy(&self) -> R {
        self.momentum.e()
    }

    /// Transverse momentum.
    pub fn pt(&self) -> R {
        self.momentum.transverse_momentum()
    }
}

impl<R: deep_causality_num::RealField + deep_causality_num::FromPrimitive> Hadron<R> {
    /// Rapidity.
    pub fn rapidity(&self) -> R {
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
pub struct LundParameters<R: deep_causality_num::RealField> {
    /// String tension κ (GeV/fm) - energy per unit length of the color flux tube
    kappa: R,
    /// Lund 'a' parameter - controls the shape of the fragmentation function
    lund_a: R,
    /// Lund 'b' parameter (GeV⁻²) - controls mass dependence
    lund_b: R,
    /// Transverse momentum width σ_pT (GeV) - Gaussian width for pT generation
    sigma_pt: R,
    /// Strange quark suppression factor s/u (typically 0.2-0.4)
    strange_suppression: R,
    /// Diquark suppression factor qq/q (typically 0.05-0.15)
    diquark_suppression: R,
    /// Spin-1 meson suppression relative to spin-0 (vector/pseudoscalar ratio)
    vector_meson_fraction: R,
    /// Minimum invariant mass to continue fragmentation (GeV)
    min_invariant_mass: R,
}

impl<R: deep_causality_num::RealField + deep_causality_num::FromPrimitive> Default
    for LundParameters<R>
{
    /// Default parameters tuned to LEP e+e- data.
    ///
    /// Based on PYTHIA 8 Monash tune. The `f64`-defined tune values are cast into
    /// the target real field `R` via [`crate::real_from_f64`].
    fn default() -> Self {
        use crate::real_from_f64;
        Self {
            kappa: real_from_f64(1.0),                  // GeV/fm
            lund_a: real_from_f64(0.68),                // dimensionless
            lund_b: real_from_f64(0.98),                // GeV⁻²
            sigma_pt: real_from_f64(0.36),              // GeV
            strange_suppression: real_from_f64(0.30),   // s/u ratio
            diquark_suppression: real_from_f64(0.10),   // qq/q ratio
            vector_meson_fraction: real_from_f64(0.50), // V/(V+PS)
            min_invariant_mass: real_from_f64(0.5),     // GeV
        }
    }
}

impl<R: deep_causality_num::RealField> LundParameters<R> {
    /// Creates new Lund parameters.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        kappa: R,
        lund_a: R,
        lund_b: R,
        sigma_pt: R,
        strange_suppression: R,
        diquark_suppression: R,
        vector_meson_fraction: R,
        min_invariant_mass: R,
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
    pub fn kappa(&self) -> R {
        self.kappa
    }

    /// Lund 'a' parameter.
    pub fn lund_a(&self) -> R {
        self.lund_a
    }

    /// Lund 'b' parameter (GeV⁻²).
    pub fn lund_b(&self) -> R {
        self.lund_b
    }

    /// Transverse momentum width σ_pT (GeV).
    pub fn sigma_pt(&self) -> R {
        self.sigma_pt
    }

    /// Strange quark suppression factor s/u.
    pub fn strange_suppression(&self) -> R {
        self.strange_suppression
    }

    /// Diquark suppression factor qq/q.
    pub fn diquark_suppression(&self) -> R {
        self.diquark_suppression
    }

    /// Vector meson fraction V/(V+PS).
    pub fn vector_meson_fraction(&self) -> R {
        self.vector_meson_fraction
    }

    /// Minimum invariant mass to continue fragmentation (GeV).
    pub fn min_invariant_mass(&self) -> R {
        self.min_invariant_mass
    }
}
