/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::{FERMI_CONSTANT, HIGGS_VEV, SIN2_THETA_W, W_MASS, Z_MASS};
use crate::error::PhysicsError;
use crate::theories::WeakField;
use crate::{WeakFieldOps, WeakIsospin};
use deep_causality_metric::{LorentzianMetric, WestCoastMetric};
use deep_causality_num::{Field, Float};
use deep_causality_tensor::{CausalTensor, TensorData};
use deep_causality_topology::{BaseTopology, GaugeField, GaugeFieldWitness, Manifold};
use std::f64::consts::PI;

impl<S> WeakFieldOps<S> for WeakField<S>
where
    S: Field + Float + Clone + From<f64> + Into<f64> + TensorData,
{
    fn new_field(base: Manifold<S, S>, connection: CausalTensor<S>) -> Result<Self, PhysicsError> {
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

    fn fermi_constant(&self) -> S {
        <S as From<f64>>::from(FERMI_CONSTANT)
    }
    fn w_mass(&self) -> S {
        <S as From<f64>>::from(W_MASS)
    }
    fn z_mass(&self) -> S {
        <S as From<f64>>::from(Z_MASS)
    }
    fn sin2_theta_w(&self) -> S {
        <S as From<f64>>::from(SIN2_THETA_W)
    }

    fn charged_current_propagator(momentum_transfer_sq: S) -> Result<S, PhysicsError> {
        if !momentum_transfer_sq.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite q² in propagator".into(),
            ));
        }
        let w_mass = <S as From<f64>>::from(W_MASS);
        let denominator = momentum_transfer_sq - w_mass * w_mass;
        let eps = <S as From<f64>>::from(1e-10);

        if denominator.abs() < eps {
            return Err(PhysicsError::NumericalInstability(
                "q² ≈ M_W² (on-shell W)".into(),
            ));
        }
        Ok(S::one() / denominator)
    }

    fn neutral_current_propagator(
        momentum_transfer_sq: S,
        fermion: &WeakIsospin,
    ) -> Result<S, PhysicsError> {
        if !momentum_transfer_sq.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite q² in propagator".into(),
            ));
        }
        let z_mass = <S as From<f64>>::from(Z_MASS);
        let denominator = momentum_transfer_sq - z_mass * z_mass;
        let eps = <S as From<f64>>::from(1e-10);

        if denominator.abs() < eps {
            return Err(PhysicsError::NumericalInstability(
                "q² ≈ M_Z² (on-shell Z)".into(),
            ));
        }
        let g_v = <S as From<f64>>::from(fermion.vector_coupling());
        let g_a = <S as From<f64>>::from(fermion.axial_coupling());
        let coupling = g_v * g_v + g_a * g_a;
        Ok(coupling / denominator)
    }

    fn weak_decay_width(mass: S) -> Result<S, PhysicsError> {
        if mass <= S::zero() || !mass.is_finite() {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Mass must be positive: {:?}",
                mass.to_f64().unwrap_or(0.0)
            )));
        }
        let g_f = <S as From<f64>>::from(FERMI_CONSTANT);
        let pi = <S as From<f64>>::from(PI);
        let factor = <S as From<f64>>::from(192.0);
        let pi_3 = pi * pi * pi; // powi(3) often just multiply

        Ok(g_f * g_f * mass.powi(5) / (factor * pi_3))
    }

    fn muon_lifetime() -> S {
        let m_muon = <S as From<f64>>::from(0.1056583755);
        let g_f = <S as From<f64>>::from(FERMI_CONSTANT);
        let pi = <S as From<f64>>::from(PI);
        let factor = <S as From<f64>>::from(192.0);
        let pi_3 = pi * pi * pi;

        let rate = g_f * g_f * m_muon.powi(5) / (factor * pi_3);
        let hbar = <S as From<f64>>::from(6.582119569e-25);
        hbar / rate
    }

    fn w_boson_width() -> S {
        let g_f = <S as From<f64>>::from(FERMI_CONSTANT);
        let w_mass = <S as From<f64>>::from(W_MASS);
        let pi = <S as From<f64>>::from(PI);
        let factor_val = <S as From<f64>>::from(6.0) * <S as From<f64>>::from(2.0).sqrt() * pi;
        let factor = g_f * w_mass.powi(3) / factor_val;

        let channels = <S as From<f64>>::from(3.0 + 2.0 * 3.0);
        factor * channels
    }

    fn z_boson_width() -> S {
        let g_f = <S as From<f64>>::from(FERMI_CONSTANT);
        let z_mass = <S as From<f64>>::from(Z_MASS);
        let pi = <S as From<f64>>::from(PI);
        let factor_val = <S as From<f64>>::from(6.0) * <S as From<f64>>::from(2.0).sqrt() * pi;
        let factor = g_f * z_mass.powi(3) / factor_val;

        let mut width = S::zero();
        let nu = WeakIsospin::neutrino();
        let g_v_nu = <S as From<f64>>::from(nu.vector_coupling());
        let g_a_nu = <S as From<f64>>::from(nu.axial_coupling());
        width = width + <S as From<f64>>::from(3.0) * (g_v_nu * g_v_nu + g_a_nu * g_a_nu);

        let lepton = WeakIsospin::lepton_doublet();
        let g_v_l = <S as From<f64>>::from(lepton.vector_coupling());
        let g_a_l = <S as From<f64>>::from(lepton.axial_coupling());
        width = width + <S as From<f64>>::from(3.0) * (g_v_l * g_v_l + g_a_l * g_a_l);

        let up = WeakIsospin::up_quark();
        let g_v_u = <S as From<f64>>::from(up.vector_coupling());
        let g_a_u = <S as From<f64>>::from(up.axial_coupling());
        width = width + <S as From<f64>>::from(6.0) * (g_v_u * g_v_u + g_a_u * g_a_u); // 2*3

        let down = WeakIsospin::down_quark();
        let g_v_d = <S as From<f64>>::from(down.vector_coupling());
        let g_a_d = <S as From<f64>>::from(down.axial_coupling());
        width = width + <S as From<f64>>::from(9.0) * (g_v_d * g_v_d + g_a_d * g_a_d); // 3*3

        factor * width
    }

    fn weak_field_strength(&self) -> CausalTensor<S> {
        // g = 2 M_W / v
        let two = <S as From<f64>>::from(2.0);
        let vev = <S as From<f64>>::from(HIGGS_VEV);
        let g = two * self.w_mass() / vev;
        GaugeFieldWitness::compute_field_strength_non_abelian(self, g)
    }
}
