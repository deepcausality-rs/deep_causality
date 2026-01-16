/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    EM, ElectroweakField, ElectroweakOps, ElectroweakParams, PhysicsError, SIN2_THETA_W, W_MASS,
    Z_MASS,
};
use deep_causality_metric::{LorentzianMetric, WestCoastMetric};
use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, GaugeField, GaugeFieldWitness, Manifold, U1};

impl<S> ElectroweakOps<S> for ElectroweakField<S>
where
    S: RealField + Clone + From<f64> + Into<f64> + Default,
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

    fn standard_model_params() -> ElectroweakParams<S> {
        ElectroweakParams::standard_model()
    }

    fn extract_photon(&self) -> Result<EM<S>, PhysicsError> {
        let params = Self::standard_model_params();
        let cos_theta = params.cos_theta_w();
        let sin_theta = params.sin_theta_w();

        // Use topology gauge_rotation method from the GaugeField
        // Photon: A_μ = W³_μ sin(θ_W) + B_μ cos(θ_W)
        // index_a = 2 (W³), index_b = 3 (B)
        let (new_conn, new_strength) = GaugeFieldWitness::<S>::gauge_rotation(
            self.connection(),
            self.field_strength(),
            2, // W³ index
            3, // B index
            cos_theta,
            sin_theta,
        );

        GaugeField::new(self.base().clone(), self.metric(), new_conn, new_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn extract_z(&self) -> Result<GaugeField<U1, S, S>, PhysicsError> {
        let params = Self::standard_model_params();
        let cos_theta = params.cos_theta_w();
        let sin_theta = params.sin_theta_w();

        // Use the topology gauge_rotation method from the GaugeField.
        // Z_μ = W³_μ cos(θ_W) - B_μ sin(θ_W)
        // This is equivalent to: A^a cos(θ) + A^b (-sin(θ))
        // So we pass -sin_theta for the second component
        let neg_sin_theta = S::zero() - sin_theta;
        let (new_conn, new_strength) = GaugeFieldWitness::<S>::gauge_rotation(
            self.connection(),
            self.field_strength(),
            2, // W³ index
            3, // B index
            neg_sin_theta,
            cos_theta,
        );

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
