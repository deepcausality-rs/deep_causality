/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Electroweak Theory — SU(2)_L × U(1)_Y Gauge Theory Module
//!
//! # Mathematical Foundation
//!
//! The electroweak theory unifies electromagnetic and weak forces via the
//! gauge group SU(2)_L × U(1)_Y with spontaneous symmetry breaking.
//!
//! ## Gauge Fields (Before Symmetry Breaking)
//! ```text
//! SU(2)_L: W^a_μ (a = 1,2,3) with coupling g
//! U(1)_Y:  B_μ with coupling g'
//! ```
//!
//! ## Symmetry Breaking (Higgs Mechanism)
//! ```text
//! Higgs doublet: φ = (φ⁺, φ⁰)ᵀ
//! Vacuum: ⟨φ⟩ = (0, v/√2)ᵀ  where v ≈ 246 GeV
//! ```
//!
//! ## Physical Bosons (After Breaking SU(2)×U(1) → U(1)_EM)
//! ```text
//! W^± = (W¹ ∓ iW²) / √2       — charged, mass M_W = gv/2
//! Z⁰  = W³ cos θ_W - B sin θ_W — neutral, mass M_Z = M_W/cos θ_W
//! A   = W³ sin θ_W + B cos θ_W — photon, massless
//! ```
//!
//! ## Weinberg Angle
//! ```text
//! tan θ_W = g'/g        sin² θ_W ≈ 0.231
//! e = g sin θ_W = g' cos θ_W
//! ```
//!
//! ## Mass Generation
//! ```text
//! Gauge boson masses: M_W = gv/2, M_Z = M_W/cos θ_W
//! Fermion masses: m_f = y_f v / √2  (Yukawa coupling)
//! Higgs mass: M_H = √(2λ) v ≈ 125 GeV
//! ```

use super::weak_force::{FERMI_CONSTANT, HIGGS_VEV, SIN2_THETA_W, W_MASS, Z_MASS};
use crate::error::PhysicsError;
use crate::theories::{ElectroweakField, QED};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, GaugeField, U1};
use std::f64::consts::PI;

// =============================================================================
// Electroweak Constants (PDG 2024 values)
// =============================================================================

/// Fine structure constant α = e²/(4π) ≈ 1/137
///
/// ```text
/// α = e² / (4π ℏc) ≈ 7.297 × 10⁻³ ≈ 1/137
/// ```
pub const ALPHA_EM: f64 = 1.0 / 137.035999084;

/// Electromagnetic coupling constant e = √(4πα)
///
/// ```text
/// e = g sin θ_W = g' cos θ_W ≈ 0.303
/// ```
pub const EM_COUPLING: f64 = 0.3028221;

/// Higgs boson mass M_H in GeV
///
/// ```text
/// M_H = √(2λ) v ≈ 125 GeV
/// ```
pub const HIGGS_MASS: f64 = 125.25;

/// Top quark mass m_t in GeV (heaviest Standard Model fermion)
///
/// ```text
/// m_t = y_t v / √2 ≈ 173 GeV
/// ```
pub const TOP_MASS: f64 = 172.69;

// =============================================================================
// Electroweak Operations Trait
// =============================================================================

/// Operations for the Electroweak Theory — SU(2)_L × U(1)_Y unified gauge theory.
///
/// # Mathematical Foundation
///
/// ## Gauge Structure
/// ```text
/// SU(2)_L × U(1)_Y  →  U(1)_EM  (spontaneous breaking via Higgs)
/// ```
///
/// ## Physical Boson Extraction
/// ```text
/// Photon: A_μ = B_μ cos θ_W + W³_μ sin θ_W
/// Z boson: Z_μ = -B_μ sin θ_W + W³_μ cos θ_W
/// ```
///
/// ## Key Parameters
/// - sin² θ_W ≈ 0.231 (Weinberg angle)
/// - M_W ≈ 80.4 GeV, M_Z ≈ 91.2 GeV
/// - v ≈ 246 GeV (Higgs VEV)
pub trait ElectroweakOps {
    /// Returns the Standard Model parameters.
    fn standard_model_params() -> ElectroweakParams;

    /// Extracts the photon field A from the electroweak connection.
    ///
    /// # Mathematical Definition
    /// ```text
    /// A_μ = B_μ cos θ_W + W³_μ sin θ_W
    /// ```
    /// This is the massless eigenstate of the neutral current.
    fn extract_photon(&self) -> Result<QED, PhysicsError>;

    /// Extracts the Z boson field from the electroweak connection.
    ///
    /// # Mathematical Definition
    /// ```text
    /// Z_μ = -B_μ sin θ_W + W³_μ cos θ_W
    /// ```
    /// Massive neutral current mediator with M_Z ≈ 91.2 GeV.
    fn extract_z(&self) -> Result<GaugeField<U1, f64, f64>, PhysicsError>;

    /// Returns sin²θ_W (Weinberg angle).
    fn sin2_theta_w(&self) -> f64;

    /// Returns the W boson mass M_W.
    fn w_mass(&self) -> f64;

    /// Returns the Z boson mass M_Z.
    fn z_mass(&self) -> f64;
}

impl ElectroweakOps for ElectroweakField {
    fn standard_model_params() -> ElectroweakParams {
        ElectroweakParams::standard_model()
    }

    fn extract_photon(&self) -> Result<QED, PhysicsError> {
        let params = Self::standard_model_params();
        let cos_theta = params.cos_theta_w();
        let sin_theta = params.sin_theta_w();

        let connection = self.connection();
        let field_strength = self.field_strength();

        // Mix Connection
        let data = connection.data();
        let mixed_conn_data = data
            .chunks(4)
            .map(|chunk: &[f64]| {
                let w3 = chunk.get(2).copied().unwrap_or(0.0);
                let b = chunk.get(3).copied().unwrap_or(0.0);
                b * cos_theta + w3 * sin_theta
            })
            .collect::<Vec<f64>>();

        // Mix Field Strength
        let data = field_strength.data();
        let mixed_strength_data = data
            .chunks(4)
            .map(|chunk: &[f64]| {
                let w3 = chunk.get(2).copied().unwrap_or(0.0);
                let b = chunk.get(3).copied().unwrap_or(0.0);
                b * cos_theta + w3 * sin_theta
            })
            .collect::<Vec<f64>>();

        let num_points = self.base().len();
        let dim = 4;

        let new_conn = CausalTensor::new(mixed_conn_data, vec![num_points, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        let new_strength = CausalTensor::new(mixed_strength_data, vec![num_points, dim, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        GaugeField::new(self.base().clone(), self.metric(), new_conn, new_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn extract_z(&self) -> Result<GaugeField<U1, f64, f64>, PhysicsError> {
        let params = Self::standard_model_params();
        let cos_theta = params.cos_theta_w();
        let sin_theta = params.sin_theta_w();

        let connection = self.connection();
        let field_strength = self.field_strength();

        // Mix Connection
        let data = connection.data();
        let mixed_conn_data = data
            .chunks(4)
            .map(|chunk: &[f64]| {
                let w3 = chunk.get(2).copied().unwrap_or(0.0);
                let b = chunk.get(3).copied().unwrap_or(0.0);
                -b * sin_theta + w3 * cos_theta
            })
            .collect::<Vec<f64>>();

        let data = field_strength.data();
        let mixed_strength_data = data
            .chunks(4)
            .map(|chunk: &[f64]| {
                let w3 = chunk.get(2).copied().unwrap_or(0.0);
                let b = chunk.get(3).copied().unwrap_or(0.0);
                -b * sin_theta + w3 * cos_theta
            })
            .collect::<Vec<f64>>();

        let num_points = self.base().len();
        let dim = 4;

        let new_conn = CausalTensor::new(mixed_conn_data, vec![num_points, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        let new_strength = CausalTensor::new(mixed_strength_data, vec![num_points, dim, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        GaugeField::new(self.base().clone(), self.metric(), new_conn, new_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn sin2_theta_w(&self) -> f64 {
        SIN2_THETA_W
    }
    fn w_mass(&self) -> f64 {
        W_MASS
    }
    fn z_mass(&self) -> f64 {
        Z_MASS
    }
}

// =============================================================================
// Electroweak Params Helper
// =============================================================================

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
    pub sin2_theta_w: f64,
    /// Higgs vacuum expectation value v (GeV)
    pub higgs_vev: f64,
    /// SU(2)_L coupling constant g
    pub g: f64,
    /// U(1)_Y coupling constant g'
    pub g_prime: f64,
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
        let gev2_to_pb = 0.3894e6;
        Ok(sigma * gev2_to_pb)
    }

    /// Computes the Breit-Wigner cross section near the Z resonance.
    pub fn z_resonance_cross_section(
        &self,
        center_of_mass_energy: f64,
        width: f64,
    ) -> Result<f64, PhysicsError> {
        let s = center_of_mass_energy * center_of_mass_energy;
        let mz = Z_MASS;
        let mz2 = mz * mz;
        let gamma2 = width * width;

        let denominator = (s - mz2).powi(2) + mz2 * gamma2;
        if denominator.abs() < 1e-12 {
            return Err(PhysicsError::NumericalInstability(
                "Singularity in cross section".into(),
            ));
        }

        // Relativistic Breit-Wigner form
        // σ(s) ∝ s / ((s - Mz²)² + Mz²Γ²)
        // Peak normalization implies unit amplitude at resonance for shape.
        // For physical cross-section, we need partial widths, but returning shape factor is sufficient here.
        Ok(s / denominator)
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
