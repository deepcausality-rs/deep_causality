/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Weak Force — SU(2)_L Gauge Theory Module
//!
//! # Mathematical Foundation
//!
//! The weak force is described by an SU(2)_L gauge theory with spontaneous
//! symmetry breaking via the Higgs mechanism.
//!
//! ## Gauge Field (Non-Abelian Connection)
//! ```text
//! W_μ = W_μ^a T_a = W_μ^a (σ_a/2)   for a = 1,2,3
//! ```
//! where σ_a are the Pauli matrices and T_a = σ_a/2 are the SU(2) generators.
//!
//! ## Field Strength Tensor (Non-Abelian Curvature)
//! ```text
//! W_μν^a = ∂_μ W_ν^a - ∂_ν W_μ^a + g ε^{abc} W_μ^b W_ν^c
//! ```
//! The extra term gε^{abc}W^b W^c arises from the non-commutativity of SU(2).
//!
//! ## Physical Bosons (After Symmetry Breaking)
//! ```text
//! W^± = (W^1 ∓ iW^2) / √2     (charged current)
//! Z^0 = W^3 cos θ_W - B sin θ_W   (neutral current)
//! A   = W^3 sin θ_W + B cos θ_W   (photon)
//! ```
//!
//! ## Fermi Theory (Low Energy Limit)
//! ```text
//! G_F / √2 = g² / (8 M_W²)
//! ```

use crate::error::PhysicsError;
use crate::theories::WeakField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::GaugeFieldWitness;
use std::f64::consts::PI;

// =============================================================================
// Physical Constants (PDG 2024 values)
// =============================================================================

/// Fermi coupling constant G_F in GeV⁻²
///
/// ```text
/// G_F / √2 = g² / (8 M_W²) ≈ 1.166 × 10⁻⁵ GeV⁻²
/// ```
pub const FERMI_CONSTANT: f64 = 1.1663787e-5;

/// W boson mass in GeV
///
/// ```text
/// M_W = g v / 2 ≈ 80.4 GeV
/// ```
pub const W_MASS: f64 = 80.377;

/// Z boson mass in GeV
///
/// ```text
/// M_Z = M_W / cos θ_W ≈ 91.2 GeV
/// ```
pub const Z_MASS: f64 = 91.1876;

/// Weak mixing angle (sin²θ_W)
///
/// ```text
/// sin² θ_W = 1 - (M_W / M_Z)² ≈ 0.231
/// ```
pub const SIN2_THETA_W: f64 = 0.23121;

/// Higgs vacuum expectation value v = (√2 G_F)^(-1/2) ≈ 246 GeV
///
/// ```text
/// v = (√2 G_F)^{-1/2} = 2 M_W / g ≈ 246 GeV
/// ```
pub const HIGGS_VEV: f64 = 246.22;

// =============================================================================
// Pauli Matrices (SU(2) Generators)
// =============================================================================

/// Returns the three Pauli matrices σ₁, σ₂, σ₃ as 2×2 complex matrices.
///
/// # Mathematical Definition
/// ```text
/// σ₁ = ⎛0 1⎞   σ₂ = ⎛0 -i⎞   σ₃ = ⎛1  0⎞
///      ⎝1 0⎠        ⎝i  0⎠        ⎝0 -1⎠
/// ```
/// Satisfies [σ_a, σ_b] = 2i ε_{abc} σ_c
pub fn pauli_matrices() -> [[(f64, f64); 4]; 3] {
    let sigma1 = [(0.0, 0.0), (1.0, 0.0), (1.0, 0.0), (0.0, 0.0)];
    let sigma2 = [(0.0, 0.0), (0.0, -1.0), (0.0, 1.0), (0.0, 0.0)];
    let sigma3 = [(1.0, 0.0), (0.0, 0.0), (0.0, 0.0), (-1.0, 0.0)];
    [sigma1, sigma2, sigma3]
}

/// Returns the SU(2) generators T_a = σ_a / 2.
///
/// # Mathematical Definition
/// ```text
/// T_a = σ_a / 2
/// ```
/// Satisfies [T_a, T_b] = i ε_{abc} T_c
pub fn su2_generators() -> [[(f64, f64); 4]; 3] {
    let pauli = pauli_matrices();
    let mut generators = [[(0.0, 0.0); 4]; 3];
    for (i, matrix) in pauli.iter().enumerate() {
        for (j, (re, im)) in matrix.iter().enumerate() {
            generators[i][j] = (re / 2.0, im / 2.0);
        }
    }
    generators
}

// =============================================================================
// Weak Field Operations Trait
// =============================================================================

/// Operations for the Weak Force — SU(2)_L gauge theory.
///
/// # Mathematical Foundation
///
/// The weak force is a non-abelian SU(2)_L gauge theory. Key features:
///
/// ## Field Strength (Non-Abelian)
/// ```text
/// W_μν^a = ∂_μ W_ν^a - ∂_ν W_μ^a + g ε^{abc} W_μ^b W_ν^c
/// ```
///
/// ## Propagators
/// ```text
/// D_W(q²) = 1 / (q² - M_W²)           (W boson)
/// D_Z(q²) = (g_V² + g_A²) / (q² - M_Z²)  (Z boson)
/// ```
///
/// ## Decay Width (Fermi Theory)
/// ```text
/// Γ = G_F² m⁵ / (192 π³)
/// ```
pub trait WeakOps {
    /// Returns the Fermi coupling constant G_F.
    ///
    /// ```text
    /// G_F ≈ 1.166 × 10⁻⁵ GeV⁻²
    /// ```
    fn fermi_constant(&self) -> f64;

    /// Returns the W boson mass M_W.
    ///
    /// ```text
    /// M_W = g v / 2 ≈ 80.4 GeV
    /// ```
    fn w_mass(&self) -> f64;

    /// Returns the Z boson mass M_Z.
    ///
    /// ```text
    /// M_Z = M_W / cos θ_W ≈ 91.2 GeV
    /// ```
    fn z_mass(&self) -> f64;

    /// Returns the weak mixing angle sin²θ_W.
    ///
    /// ```text
    /// sin² θ_W = 1 - (M_W / M_Z)² ≈ 0.231
    /// ```
    fn sin2_theta_w(&self) -> f64;

    /// Computes the W boson propagator (charged current).
    ///
    /// # Mathematical Definition
    /// ```text
    /// D_W(q²) = 1 / (q² - M_W²)
    /// ```
    fn charged_current_propagator(momentum_transfer_sq: f64) -> Result<f64, PhysicsError>;

    /// Computes the Z boson propagator (neutral current).
    ///
    /// # Mathematical Definition
    /// ```text
    /// D_Z(q²) = (g_V² + g_A²) / (q² - M_Z²)
    /// ```
    /// where g_V and g_A are vector and axial couplings.
    fn neutral_current_propagator(
        momentum_transfer_sq: f64,
        fermion: &WeakIsospin,
    ) -> Result<f64, PhysicsError>;

    /// Computes the weak decay width using Fermi theory.
    ///
    /// # Mathematical Definition
    /// ```text
    /// Γ = G_F² m⁵ / (192 π³)
    /// ```
    fn weak_decay_width(mass: f64) -> Result<f64, PhysicsError>;

    /// Computes the muon lifetime τ_μ.
    ///
    /// # Mathematical Definition
    /// ```text
    /// τ_μ = ℏ · 192 π³ / (G_F² m_μ⁵) ≈ 2.2 μs
    /// ```
    fn muon_lifetime() -> f64;

    /// Computes the W boson total width Γ_W.
    ///
    /// # Mathematical Definition
    /// ```text
    /// Γ_W = G_F M_W³ (3 + 2×3) / (6√2 π) ≈ 2.1 GeV
    /// ```
    /// Sum over 3 lepton families + 2 quark generations.
    fn w_boson_width() -> f64;

    /// Computes the Z boson total width Γ_Z.
    ///
    /// # Mathematical Definition
    /// ```text
    /// Γ_Z = G_F M_Z³ Σ_f N_c (g_V² + g_A²) / (6√2 π) ≈ 2.5 GeV
    /// ```
    fn z_boson_width() -> f64;

    /// Computes the non-abelian field strength W_μν.
    ///
    /// # Mathematical Definition
    /// ```text
    /// W_μν^a = ∂_μ W_ν^a - ∂_ν W_μ^a + g ε^{abc} W_μ^b W_ν^c
    /// ```
    /// Uses `GaugeFieldWitness` for HKT-compliant computation.
    fn weak_field_strength(&self) -> CausalTensor<f64>;
}

impl WeakOps for WeakField {
    fn fermi_constant(&self) -> f64 {
        FERMI_CONSTANT
    }
    fn w_mass(&self) -> f64 {
        W_MASS
    }
    fn z_mass(&self) -> f64 {
        Z_MASS
    }
    fn sin2_theta_w(&self) -> f64 {
        SIN2_THETA_W
    }

    fn charged_current_propagator(momentum_transfer_sq: f64) -> Result<f64, PhysicsError> {
        if !momentum_transfer_sq.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite q² in propagator".into(),
            ));
        }
        let denominator = momentum_transfer_sq - W_MASS * W_MASS;
        if denominator.abs() < 1e-10 {
            return Err(PhysicsError::NumericalInstability(
                "q² ≈ M_W² (on-shell W)".into(),
            ));
        }
        Ok(1.0 / denominator)
    }

    fn neutral_current_propagator(
        momentum_transfer_sq: f64,
        fermion: &WeakIsospin,
    ) -> Result<f64, PhysicsError> {
        if !momentum_transfer_sq.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite q² in propagator".into(),
            ));
        }
        let denominator = momentum_transfer_sq - Z_MASS * Z_MASS;
        if denominator.abs() < 1e-10 {
            return Err(PhysicsError::NumericalInstability(
                "q² ≈ M_Z² (on-shell Z)".into(),
            ));
        }
        let g_v = fermion.vector_coupling();
        let g_a = fermion.axial_coupling();
        let coupling = g_v * g_v + g_a * g_a;
        Ok(coupling / denominator)
    }

    fn weak_decay_width(mass: f64) -> Result<f64, PhysicsError> {
        if mass <= 0.0 || !mass.is_finite() {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Mass must be positive: {}",
                mass
            )));
        }
        Ok(FERMI_CONSTANT * FERMI_CONSTANT * mass.powi(5) / (192.0 * PI.powi(3)))
    }

    fn muon_lifetime() -> f64 {
        let m_muon: f64 = 0.1056583755;
        let rate = FERMI_CONSTANT * FERMI_CONSTANT * m_muon.powi(5) / (192.0 * PI.powi(3));
        let hbar = 6.582119569e-25;
        hbar / rate
    }

    fn w_boson_width() -> f64 {
        let factor = FERMI_CONSTANT * W_MASS.powi(3) / (6.0 * 2.0_f64.sqrt() * PI);
        let channels = 3.0 + 2.0 * 3.0;
        factor * channels
    }

    fn z_boson_width() -> f64 {
        let factor = FERMI_CONSTANT * Z_MASS.powi(3) / (6.0 * 2.0_f64.sqrt() * PI);
        let mut width = 0.0;
        let nu = WeakIsospin::neutrino();
        width += 3.0 * (nu.vector_coupling().powi(2) + nu.axial_coupling().powi(2));
        let lepton = WeakIsospin::lepton_doublet();
        width += 3.0 * (lepton.vector_coupling().powi(2) + lepton.axial_coupling().powi(2));
        let up = WeakIsospin::up_quark();
        width += 2.0 * 3.0 * (up.vector_coupling().powi(2) + up.axial_coupling().powi(2));
        let down = WeakIsospin::down_quark();
        width += 3.0 * 3.0 * (down.vector_coupling().powi(2) + down.axial_coupling().powi(2));
        factor * width
    }

    fn weak_field_strength(&self) -> CausalTensor<f64> {
        // g = 2 M_W / v
        let g = 2.0 * self.w_mass() / HIGGS_VEV;
        GaugeFieldWitness::compute_field_strength_non_abelian(self, g)
    }
}

// =============================================================================
// Weak Isospin Structure (Matter Field Helper)
// =============================================================================

/// Weak isospin representation for a fermion doublet.
#[derive(Debug, Clone, Copy)]
pub struct WeakIsospin {
    pub isospin: f64,
    pub i3: f64,
    pub hypercharge: f64,
}

impl WeakIsospin {
    pub fn new(isospin: f64, i3: f64, charge: f64) -> Result<Self, PhysicsError> {
        if i3.abs() > isospin + 1e-10 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "I₃ = {} must satisfy |I₃| ≤ I = {}",
                i3, isospin
            )));
        }
        let hypercharge = 2.0 * (charge - i3);
        Ok(Self {
            isospin,
            i3,
            hypercharge,
        })
    }

    pub fn lepton_doublet() -> Self {
        Self {
            isospin: 0.5,
            i3: -0.5,
            hypercharge: -1.0,
        }
    }

    pub fn neutrino() -> Self {
        Self {
            isospin: 0.5,
            i3: 0.5,
            hypercharge: -1.0,
        }
    }

    pub fn up_quark() -> Self {
        Self {
            isospin: 0.5,
            i3: 0.5,
            hypercharge: 1.0 / 3.0,
        }
    }

    pub fn down_quark() -> Self {
        Self {
            isospin: 0.5,
            i3: -0.5,
            hypercharge: 1.0 / 3.0,
        }
    }

    pub fn right_handed(charge: f64) -> Self {
        Self {
            isospin: 0.0,
            i3: 0.0,
            hypercharge: 2.0 * charge,
        }
    }

    pub fn electric_charge(&self) -> f64 {
        self.i3 + self.hypercharge / 2.0
    }

    pub fn vector_coupling(&self) -> f64 {
        let q = self.electric_charge();
        self.i3 - 2.0 * q * SIN2_THETA_W
    }

    pub fn axial_coupling(&self) -> f64 {
        self.i3
    }
    pub fn left_coupling(&self) -> f64 {
        self.vector_coupling() + self.axial_coupling()
    }
    pub fn right_coupling(&self) -> f64 {
        self.vector_coupling() - self.axial_coupling()
    }
}

impl Default for WeakIsospin {
    fn default() -> Self {
        Self::lepton_doublet()
    }
}
