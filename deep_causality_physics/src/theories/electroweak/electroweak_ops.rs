/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{EM, ElectroweakParams, PhysicsError};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GaugeField, U1};

pub trait ElectroweakOps {
    /// Creates a new Electroweak Field (SU(2) x U(1)) with West Coast metric.
    fn new_field(
        base: deep_causality_topology::Manifold<f64>,
        connection: CausalTensor<f64>,
    ) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Returns the Standard Model parameters.
    fn standard_model_params() -> ElectroweakParams;

    fn extract_photon(&self) -> Result<EM, PhysicsError>;
    fn extract_z(&self) -> Result<GaugeField<U1, f64, f64, f64>, PhysicsError>;
    fn sin2_theta_w(&self) -> f64;
    fn w_mass(&self) -> f64;
    fn z_mass(&self) -> f64;
}
