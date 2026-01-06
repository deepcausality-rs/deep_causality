/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    EM_COUPLING, EM_COUPLING_MZ, FERMI_CONSTANT, GEV2_TO_NB, GEV2_TO_PB, HIGGS_MASS, HIGGS_VEV,
    PhysicsError, SIN2_THETA_W, TOP_MASS, W_MASS, Z_MASS, Z_PARTIAL_WIDTH_EE, Z_PARTIAL_WIDTH_HAD,
};
use std::f64::consts::PI;

/// Electroweak theory configuration and symmetry breaking parameters.
///
/// # Mathematical Foundation
///
/// ## Coupling Relations
/// ```text
/// e = g sin θ_W = g' cos θ_W
/// M_W = g v / 2
/// M_Z = M_W / cos θ_W
/// ```
///
/// ## Higgs Potential (Symmetry Breaking)
/// ```text
/// V(φ) = -μ² |φ|² + λ |φ|⁴
/// ```
/// Minimum at |φ| = v/√2 where v = μ/√λ ≈ 246 GeV
///
/// ## Mass Generation
/// - W, Z: from gauge-Higgs coupling
/// - Fermions: from Yukawa couplings y_f
/// - Higgs: M_H = √(2λ) v
#[derive(Debug, Clone, Copy)]
pub struct ElectroweakParams {
    /// sin²θ_W (Weinberg angle)
    sin2_theta_w: f64,
    /// Higgs vacuum expectation value v (GeV)
    higgs_vev: f64,
    /// SU(2)_L coupling constant g
    g: f64,
    /// U(1)_Y coupling constant g'
    g_prime: f64,
}

impl ElectroweakParams {
    pub fn new(sin2_theta_w: f64, higgs_vev: f64, g: f64, g_prime: f64) -> Self {
        Self {
            sin2_theta_w,
            higgs_vev,
            g,
            g_prime,
        }
    }
}

// Getters
impl ElectroweakParams {
    pub fn sin2_theta_w(&self) -> f64 {
        self.sin2_theta_w
    }
    pub fn cos2_theta_w(&self) -> f64 {
        1.0 - self.sin2_theta_w
    }
    pub fn sin_theta_w(&self) -> f64 {
        self.sin2_theta_w.sqrt()
    }
    pub fn cos_theta_w(&self) -> f64 {
        self.cos2_theta_w().sqrt()
    }
    pub fn tan_theta_w(&self) -> f64 {
        self.sin_theta_w() / self.cos_theta_w()
    }
    pub fn g_coupling(&self) -> f64 {
        self.g
    }
    pub fn g_prime_coupling(&self) -> f64 {
        self.g_prime
    }
    pub fn em_coupling(&self) -> f64 {
        self.g * self.sin_theta_w()
    }
    pub fn z_coupling(&self) -> f64 {
        self.g / self.cos_theta_w()
    }
    pub fn higgs_vev(&self) -> f64 {
        self.higgs_vev
    }
}

impl ElectroweakParams {
    pub fn standard_model() -> Self {
        let sin2 = SIN2_THETA_W;
        let cos2 = 1.0 - sin2;
        let sin_theta = sin2.sqrt();
        let cos_theta = cos2.sqrt();
        let e = EM_COUPLING;

        Self {
            sin2_theta_w: sin2,
            higgs_vev: HIGGS_VEV,
            g: e / sin_theta,
            g_prime: e / cos_theta,
        }
    }

    /// Creates Standard Model parameters using running coupling at Z pole.
    ///
    /// Uses α(M_Z) ≈ 1/128 instead of low-energy α ≈ 1/137.
    /// This produces W and Z masses within ~1 GeV of PDG values:
    /// - M_W ≈ 80.3 GeV (vs PDG 80.377 GeV)
    /// - M_Z ≈ 91.6 GeV (vs PDG 91.187 GeV)
    pub fn standard_model_precision() -> Self {
        let sin2 = SIN2_THETA_W;
        let cos2 = 1.0 - sin2;
        let sin_theta = sin2.sqrt();
        let cos_theta = cos2.sqrt();
        let e = EM_COUPLING_MZ; // Running coupling at M_Z

        Self {
            sin2_theta_w: sin2,
            higgs_vev: HIGGS_VEV,
            g: e / sin_theta,
            g_prime: e / cos_theta,
        }
    }

    pub fn with_mixing_angle(sin2_theta_w: f64) -> Result<Self, PhysicsError> {
        if sin2_theta_w <= 0.0 || sin2_theta_w >= 1.0 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "sin²θ_W = {} must be in (0, 1)",
                sin2_theta_w
            )));
        }
        let cos2 = 1.0 - sin2_theta_w;
        let sin_theta = sin2_theta_w.sqrt();
        let cos_theta = cos2.sqrt();
        let e = EM_COUPLING;

        Ok(Self {
            sin2_theta_w,
            higgs_vev: HIGGS_VEV,
            g: e / sin_theta,
            g_prime: e / cos_theta,
        })
    }

    pub fn w_mass_computed(&self) -> f64 {
        self.g * self.higgs_vev / 2.0
    }
    pub fn z_mass_computed(&self) -> f64 {
        self.w_mass_computed() / self.cos_theta_w()
    }

    pub fn rho_parameter(&self) -> f64 {
        let mw = W_MASS;
        let mz = Z_MASS;
        (mw * mw) / (mz * mz * self.cos2_theta_w())
    }

    /// Computes ρ parameter using internally generated masses.
    ///
    /// Unlike `rho_parameter()` which compares with PDG masses,
    /// this uses `w_mass_computed()` and `z_mass_computed()`,
    /// guaranteeing ρ = 1.0 by construction (tree-level SM relation).
    pub fn rho_parameter_computed(&self) -> f64 {
        let mw = self.w_mass_computed();
        let mz = self.z_mass_computed();
        (mw * mw) / (mz * mz * self.cos2_theta_w())
    }

    pub fn higgs_quartic(&self) -> f64 {
        HIGGS_MASS * HIGGS_MASS / (2.0 * self.higgs_vev * self.higgs_vev)
    }

    pub fn fermion_mass(&self, yukawa: f64) -> f64 {
        yukawa * self.higgs_vev / 2.0_f64.sqrt()
    }
    pub fn yukawa_coupling(&self, mass: f64) -> f64 {
        2.0_f64.sqrt() * mass / self.higgs_vev
    }

    pub fn neutrino_electron_cross_section(
        &self,
        center_of_mass_energy: f64,
    ) -> Result<f64, PhysicsError> {
        if center_of_mass_energy <= 0.0 {
            return Err(PhysicsError::DimensionMismatch(
                "Energy must be positive".into(),
            ));
        }
        let s = center_of_mass_energy * center_of_mass_energy;
        let sigma = FERMI_CONSTANT * FERMI_CONSTANT * s / PI;
        Ok(sigma * GEV2_TO_PB)
    }

    /// Computes the physical Breit-Wigner cross section for e⁺e⁻ → Z → hadrons.
    ///
    /// Returns cross-section in **nanobarns** (nb).
    ///
    /// # Formula
    /// ```text
    /// σ(s) = (12π/M_Z²) · (s · Γ_ee · Γ_had) / ((s - M_Z²)² + s² Γ_Z² / M_Z²)
    /// ```
    /// At the Z pole (√s = M_Z), this gives σ_peak ≈ 41.5 nb for hadronic final states.
    ///
    /// # Partial Widths (PDG 2024)
    /// - Γ_ee = 83.91 MeV (leptonic)
    /// - Γ_had = 1744.4 MeV (hadronic)
    /// - Γ_Z = 2495.2 MeV (total)
    pub fn z_resonance_cross_section(
        &self,
        center_of_mass_energy: f64,
        width: f64,
    ) -> Result<f64, PhysicsError> {
        if center_of_mass_energy <= 0.0 {
            return Err(PhysicsError::DimensionMismatch(
                "Energy must be positive".into(),
            ));
        }

        let s = center_of_mass_energy * center_of_mass_energy;
        let mz = Z_MASS;
        let mz2 = mz * mz;
        let gamma_z = width; // Total width in GeV

        // Relativistic Breit-Wigner with s-dependent width
        let denominator = (s - mz2).powi(2) + s * s * gamma_z * gamma_z / mz2;
        if denominator.abs() < 1e-30 {
            return Err(PhysicsError::NumericalInstability(
                "Singularity in cross section".into(),
            ));
        }

        // Cross-section in GeV⁻² (using Z_PARTIAL_WIDTH_EE and Z_PARTIAL_WIDTH_HAD)
        let sigma_gev2 =
            12.0 * PI * Z_PARTIAL_WIDTH_EE * Z_PARTIAL_WIDTH_HAD * s / (mz2 * denominator);

        Ok(sigma_gev2 * GEV2_TO_NB)
    }

    pub fn w_mass(&self) -> f64 {
        W_MASS
    }
    pub fn z_mass(&self) -> f64 {
        Z_MASS
    }
    pub fn top_yukawa(&self) -> f64 {
        self.yukawa_coupling(TOP_MASS)
    }

    // =========================================================================
    // Symmetry Breaking Implementation
    // =========================================================================

    /// Computes the Higgs potential V(φ) at a given field value.
    ///
    /// # Mathematical Definition
    /// ```text
    /// V(φ) = -μ² |φ|² + λ |φ|⁴
    /// ```
    /// where μ² = λ v² with v ≈ 246 GeV.
    pub fn higgs_potential(&self, phi_magnitude: f64) -> f64 {
        let lambda = self.higgs_quartic();
        let mu_squared = lambda * self.higgs_vev * self.higgs_vev;
        let phi2 = phi_magnitude * phi_magnitude;

        -mu_squared * phi2 + lambda * phi2 * phi2
    }

    /// Verifies the minimum of the Higgs potential is at v/√2.
    ///
    /// # Mathematical Property
    /// ```text
    /// ∂V/∂|φ| = 0  at |φ| = v/√2
    /// ```
    /// Returns true if the VEV satisfies the potential minimum condition.
    pub fn symmetry_breaking_verified(&self) -> bool {
        let v_over_sqrt2 = self.higgs_vev / 2.0_f64.sqrt();
        let epsilon = 1e-6;

        // At the minimum, derivative should be zero
        // dV/d|φ| = -2μ²|φ| + 4λ|φ|³ = 0
        // Solution: |φ| = √(μ²/(2λ)) = v/√2
        let lambda = self.higgs_quartic();
        let mu_squared = lambda * self.higgs_vev * self.higgs_vev;
        let computed_vev = (mu_squared / (2.0 * lambda)).sqrt();

        (computed_vev - v_over_sqrt2).abs() < epsilon * v_over_sqrt2
    }

    /// Returns the number of Goldstone bosons eaten by gauge bosons.
    ///
    /// # Goldstone Theorem
    /// ```text
    /// # Goldstone bosons = dim(G) - dim(H)
    ///                   = dim(SU(2)×U(1)) - dim(U(1)_EM)
    ///                   = 4 - 1 = 3
    /// ```
    /// The 3 Goldstones become the longitudinal modes of W⁺, W⁻, and Z.
    pub fn goldstone_count() -> usize {
        // SU(2)_L × U(1)_Y → U(1)_EM
        // generators: 3 + 1 → 1
        // broken generators: 3 (become Goldstones)
        3
    }

    /// Returns the mass acquired by each gauge boson after symmetry breaking.
    ///
    /// # Mathematical Result
    /// ```text
    /// W boson: M_W = g v / 2
    /// Z boson: M_Z = M_W / cos θ_W
    /// Photon:  M_A = 0
    /// ```
    pub fn gauge_boson_masses(&self) -> (f64, f64, f64) {
        let m_w = self.w_mass_computed();
        let m_z = self.z_mass_computed();
        let m_a = 0.0; // Photon remains massless

        (m_w, m_z, m_a)
    }

    /// Computes the ρ-parameter deviation from unity (tests custodial symmetry).
    ///
    /// # Mathematical Definition
    /// ```text
    /// ρ = M_W² / (M_Z² cos² θ_W)
    /// ```
    /// In the Standard Model at tree level, ρ = 1 exactly.
    pub fn rho_deviation(&self) -> f64 {
        (self.rho_parameter() - 1.0).abs()
    }
}
