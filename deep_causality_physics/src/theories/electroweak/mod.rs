/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Electroweak Theory (SU(2) × U(1)) Module
//!
//! Provides the unified electroweak theory combining electromagnetic and weak forces
//! extending the `GaugeField<Electroweak>` topology.

use super::weak::{FERMI_CONSTANT, HIGGS_VEV, SIN2_THETA_W, W_MASS, Z_MASS};
use crate::error::PhysicsError;
use crate::theories::{ElectroweakField, QED};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, GaugeField, U1};
use std::f64::consts::PI;

// =============================================================================
// Electroweak Parameters
// =============================================================================

/// Fine structure constant α = e²/(4π) ≈ 1/137
pub const ALPHA_EM: f64 = 1.0 / 137.035999084;

/// Electromagnetic coupling e = √(4πα)
pub const EM_COUPLING: f64 = 0.3028221;

/// Higgs boson mass in GeV
pub const HIGGS_MASS: f64 = 125.25;

/// Top quark mass in GeV (heaviest fermion)
pub const TOP_MASS: f64 = 172.69;

// =============================================================================
// Electroweak Operations Trait
// =============================================================================

pub trait ElectroweakOps {
    /// Returns the Standard Model parameters.
    fn standard_model_params() -> ElectroweakParams;

    /// Computes the photon field A from the electroweak connection.
    /// A_μ = B_μ cos θ_W + W³_μ sin θ_W
    fn extract_photon(&self) -> Result<QED, PhysicsError>;

    /// Computes the Z boson field Z from the electroweak connection.
    /// Z_μ = -B_μ sin θ_W + W³_μ cos θ_W
    fn extract_z(&self) -> Result<GaugeField<U1, f64, f64>, PhysicsError>;

    // Forwarding parameter methods for convenience
    fn sin2_theta_w(&self) -> f64;
    fn w_mass(&self) -> f64;
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

        Ok(
            GaugeField::new(self.base().clone(), self.metric(), new_conn, new_strength)
                .map_err(|e| PhysicsError::TopologyError(e.to_string()))?,
        )
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

        Ok(
            GaugeField::new(self.base().clone(), self.metric(), new_conn, new_strength)
                .map_err(|e| PhysicsError::TopologyError(e.to_string()))?,
        )
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

/// Electroweak theory configuration.
#[derive(Debug, Clone, Copy)]
pub struct ElectroweakParams {
    pub sin2_theta_w: f64,
    pub higgs_vev: f64,
    pub g: f64,
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
}
