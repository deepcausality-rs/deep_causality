/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{EM, ElectroweakParams, PhysicsError};
use deep_causality_num::RealField;
use deep_causality_tensor::{CausalTensor};
use deep_causality_topology::{GaugeField, Manifold, U1};

pub trait ElectroweakOps<S>
where
    S: RealField + Clone + From<f64> + Into<f64>
{
    /// Creates a new Electroweak Field (SU(2) x U(1)) with West Coast metric.
    fn new_field(base: Manifold<S, S>, connection: CausalTensor<S>) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Returns the Standard Model parameters.
    fn standard_model_params() -> ElectroweakParams<S>;
    fn extract_photon(&self) -> Result<EM<S>, PhysicsError>;
    fn extract_z(&self) -> Result<GaugeField<U1, S, S>, PhysicsError>;
    fn sin2_theta_w(&self) -> S;
    fn w_mass(&self) -> S;
    fn z_mass(&self) -> S;
}
