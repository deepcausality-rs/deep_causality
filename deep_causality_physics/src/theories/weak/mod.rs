/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Weak Force (SU(2)) Theory Module
//!
//! Provides the SU(2) gauge theory describing the weak nuclear force
//! extending the `GaugeField<SU2>` topology.

use crate::error::PhysicsError;
use crate::theories::WeakField;
use std::f64::consts::PI;

// =============================================================================
// Physical Constants (PDG 2024 values)
// =============================================================================

/// Fermi coupling constant G_F in GeV⁻²
pub const FERMI_CONSTANT: f64 = 1.1663787e-5;

/// W boson mass in GeV
pub const W_MASS: f64 = 80.377;

/// Z boson mass in GeV
pub const Z_MASS: f64 = 91.1876;

/// Weak mixing angle (sin²θ_W)
pub const SIN2_THETA_W: f64 = 0.23121;

/// Higgs vacuum expectation value v = (√2 G_F)^(-1/2) ≈ 246 GeV
pub const HIGGS_VEV: f64 = 246.22;

// =============================================================================
// Pauli Matrices (SU(2) Generators)
// =============================================================================

/// Returns the three Pauli matrices σ₁, σ₂, σ₃ as 2×2 complex matrices.
pub fn pauli_matrices() -> [[(f64, f64); 4]; 3] {
    let sigma1 = [(0.0, 0.0), (1.0, 0.0), (1.0, 0.0), (0.0, 0.0)];
    let sigma2 = [(0.0, 0.0), (0.0, -1.0), (0.0, 1.0), (0.0, 0.0)];
    let sigma3 = [(1.0, 0.0), (0.0, 0.0), (0.0, 0.0), (-1.0, 0.0)];
    [sigma1, sigma2, sigma3]
}

/// Returns the SU(2) generators T_a = σ_a / 2.
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

pub trait WeakOps {
    /// Returns the Fermi coupling constant G_F.
    fn fermi_constant(&self) -> f64;

    /// Returns the W boson mass.
    fn w_mass(&self) -> f64;

    /// Returns the Z boson mass.
    fn z_mass(&self) -> f64;

    /// Returns the Weak mixing angle sin²θ_W.
    fn sin2_theta_w(&self) -> f64;

    /// Computes the charged current amplitude for W boson exchange.
    fn charged_current_propagator(momentum_transfer_sq: f64) -> Result<f64, PhysicsError>;

    /// Computes the neutral current amplitude for Z boson exchange.
    fn neutral_current_propagator(
        momentum_transfer_sq: f64,
        fermion: &WeakIsospin,
    ) -> Result<f64, PhysicsError>;

    /// Computes the weak decay rate for a fermion at rest.
    fn weak_decay_width(mass: f64) -> Result<f64, PhysicsError>;

    /// Computes the muon lifetime.
    fn muon_lifetime() -> f64;

    /// Computes the W boson width.
    fn w_boson_width() -> f64;

    /// Computes the Z boson width.
    fn z_boson_width() -> f64;
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
