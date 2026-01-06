/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    EM, ElectroweakField, ElectroweakOps, ElectroweakParams, PhysicsError, SIN2_THETA_W, W_MASS,
    Z_MASS,
};
use deep_causality_metric::{LorentzianMetric, WestCoastMetric};
use deep_causality_num::{Field, Float};
use deep_causality_tensor::{CausalTensor, TensorData};
use deep_causality_topology::{BaseTopology, GaugeField, Manifold, U1};

impl<S> ElectroweakOps<S> for ElectroweakField<S>
where
    S: Field + Float + Clone + From<f64> + Into<f64> + TensorData,
{
    fn new_field(base: Manifold<S, S>, connection: CausalTensor<S>) -> Result<Self, PhysicsError> {
        let metric = WestCoastMetric::minkowski_4d().into_metric();

        // Initial field strength
        let num_points = base.len();
        let dim = 4;
        let lie_dim = 4; // SU(2)xU(1) = 3 + 1
        // zero() from Field trait
        let field_strength = CausalTensor::zeros(&[num_points, dim, dim, lie_dim]);

        GaugeField::new(base, metric, connection, field_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn standard_model_params() -> ElectroweakParams {
        ElectroweakParams::standard_model()
    }

    fn extract_photon(&self) -> Result<EM<S>, PhysicsError> {
        let params = Self::standard_model_params();
        let cos_theta = <S as From<f64>>::from(params.cos_theta_w());
        let sin_theta = <S as From<f64>>::from(params.sin_theta_w());

        let connection = self.connection();
        let field_strength = self.field_strength();

        // Mix Connection
        let data = connection.data();
        let mixed_conn_data = data
            .chunks(4)
            .map(|chunk: &[S]| {
                let w3 = chunk.get(2).cloned().unwrap_or(S::zero());
                let b = chunk.get(3).cloned().unwrap_or(S::zero());
                b * cos_theta + w3 * sin_theta
            })
            .collect::<Vec<S>>();

        // Mix Field Strength
        let data = field_strength.data();
        let mixed_strength_data = data
            .chunks(4)
            .map(|chunk: &[S]| {
                let w3 = chunk.get(2).cloned().unwrap_or(S::zero());
                let b = chunk.get(3).cloned().unwrap_or(S::zero());
                b * cos_theta + w3 * sin_theta
            })
            .collect::<Vec<S>>();

        let num_points = self.base().len();
        let dim = 4;

        let new_conn = CausalTensor::new(mixed_conn_data, vec![num_points, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        let new_strength = CausalTensor::new(mixed_strength_data, vec![num_points, dim, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        GaugeField::new(self.base().clone(), self.metric(), new_conn, new_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn extract_z(&self) -> Result<GaugeField<U1, S, S, S>, PhysicsError> {
        let params = Self::standard_model_params();
        let cos_theta = <S as From<f64>>::from(params.cos_theta_w());
        let sin_theta = <S as From<f64>>::from(params.sin_theta_w());

        let connection = self.connection();
        let field_strength = self.field_strength();

        // Mix Connection
        let data = connection.data();
        let mixed_conn_data = data
            .chunks(4)
            .map(|chunk: &[S]| {
                let w3 = chunk.get(2).cloned().unwrap_or(S::zero());
                let b = chunk.get(3).cloned().unwrap_or(S::zero());
                // -b * sin_theta + w3 * cos_theta
                (S::zero() - b) * sin_theta + w3 * cos_theta
            })
            .collect::<Vec<S>>();

        let data = field_strength.data();
        let mixed_strength_data = data
            .chunks(4)
            .map(|chunk: &[S]| {
                let w3 = chunk.get(2).cloned().unwrap_or(S::zero());
                let b = chunk.get(3).cloned().unwrap_or(S::zero());
                (S::zero() - b) * sin_theta + w3 * cos_theta
            })
            .collect::<Vec<S>>();

        let num_points = self.base().len();
        let dim = 4;

        let new_conn = CausalTensor::new(mixed_conn_data, vec![num_points, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        let new_strength = CausalTensor::new(mixed_strength_data, vec![num_points, dim, dim, 1])
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))?;

        GaugeField::new(self.base().clone(), self.metric(), new_conn, new_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn sin2_theta_w(&self) -> S {
        <S as From<f64>>::from(SIN2_THETA_W)
    }
    fn w_mass(&self) -> S {
        <S as From<f64>>::from(W_MASS)
    }
    fn z_mass(&self) -> S {
        <S as From<f64>>::from(Z_MASS)
    }
}
