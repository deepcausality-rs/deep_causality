/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::{FERMI_CONSTANT, HIGGS_VEV, SIN2_THETA_W, W_MASS, Z_MASS};
use crate::error::PhysicsError;
use crate::theories::WeakField;
use crate::{WeakFieldOps, WeakIsospin};
use deep_causality_metric::{LorentzianMetric, WestCoastMetric};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, GaugeField, GaugeFieldWitness, Manifold};
use std::f64::consts::PI;

impl WeakFieldOps for WeakField {
    fn new_field(base: Manifold<f64>, connection: CausalTensor<f64>) -> Result<Self, PhysicsError> {
        // Enforce West Coast metric (+---)
        let metric = WestCoastMetric::minkowski_4d().into_metric();

        // Initial field strength (can be updated later)
        let num_points = base.len();
        let dim = 4;
        let lie_dim = 3; // SU(2) has 3 generators
        let field_strength = CausalTensor::zeros(&[num_points, dim, dim, lie_dim]);

        GaugeField::new(base, metric, connection, field_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

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
