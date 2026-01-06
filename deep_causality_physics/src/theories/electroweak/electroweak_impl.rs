/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    EM, ElectroweakField, ElectroweakOps, ElectroweakParams, PhysicsError, SIN2_THETA_W, W_MASS,
    Z_MASS,
};
use deep_causality_metric::{LorentzianMetric, WestCoastMetric};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, GaugeField, U1};

impl ElectroweakOps for ElectroweakField {
    fn new_field(
        base: deep_causality_topology::Manifold<f64>,
        connection: CausalTensor<f64>,
    ) -> Result<Self, PhysicsError> {
        let metric = WestCoastMetric::minkowski_4d().into_metric();

        // Initial field strength
        let num_points = base.len();
        let dim = 4;
        let lie_dim = 4; // SU(2)xU(1) = 3 + 1
        let field_strength = CausalTensor::zeros(&[num_points, dim, dim, lie_dim]);

        GaugeField::new(base, metric, connection, field_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn standard_model_params() -> ElectroweakParams {
        ElectroweakParams::standard_model()
    }

    fn extract_photon(&self) -> Result<EM, PhysicsError> {
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
