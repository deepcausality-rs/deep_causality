/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::dynamics::estimation;
use crate::dynamics::kinematics;
use crate::dynamics::kinematics::PhysicalVector;
use crate::units::types::energy::Energy;
use crate::{Frequency, Mass, MomentOfInertia, Probability};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;

/// Causal wrapper for [`estimation::kalman_filter_linear_kernel`].
pub fn kalman_filter_linear<R>(
    x_pred: &CausalTensor<R>,
    p_pred: &CausalTensor<R>,
    measurement: &CausalTensor<R>,
    measurement_matrix: &CausalTensor<R>,
    measurement_noise: &CausalTensor<R>,
    process_noise: &CausalTensor<R>,
) -> PropagatingEffect<(CausalTensor<R>, CausalTensor<R>)>
where
    R: RealField + Default + Debug + core::iter::Sum,
{
    match estimation::kalman_filter_linear_kernel(
        x_pred,
        p_pred,
        measurement,
        measurement_matrix,
        measurement_noise,
        process_noise,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::kinetic_energy_kernel`].
pub fn kinetic_energy(
    mass: &Mass,
    velocity: &CausalMultiVector<f64>,
) -> PropagatingEffect<Energy<f64>> {
    match kinematics::kinetic_energy_kernel(*mass, velocity) {
        Ok(v) => match Energy::new(v) {
            Ok(e) => PropagatingEffect::pure(e),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::rotational_kinetic_energy_kernel`].
pub fn rotational_kinetic_energy(
    moment_of_inertia: &MomentOfInertia,
    angular_velocity: &Frequency,
) -> PropagatingEffect<Energy<f64>> {
    match kinematics::rotational_kinetic_energy_kernel(*moment_of_inertia, *angular_velocity) {
        Ok(v) => match Energy::new(v) {
            Ok(e) => PropagatingEffect::pure(e),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::torque_kernel`].
pub fn torque<R>(
    radius: &CausalMultiVector<R>,
    force: &CausalMultiVector<R>,
) -> PropagatingEffect<PhysicalVector<R>>
where
    R: RealField + Debug,
{
    match kinematics::torque_kernel(radius, force) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::angular_momentum_kernel`].
pub fn angular_momentum<R>(
    radius: &CausalMultiVector<R>,
    momentum: &CausalMultiVector<R>,
) -> PropagatingEffect<PhysicalVector<R>>
where
    R: RealField + Debug,
{
    match kinematics::angular_momentum_kernel(radius, momentum) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`estimation::generalized_master_equation_kernel`].
pub fn generalized_master_equation<R>(
    state: &[Probability<R>],
    history: &[Vec<Probability<R>>],
    markov_operator: Option<&CausalTensor<R>>,
    memory_kernel: &[CausalTensor<R>],
) -> PropagatingEffect<Vec<Probability<R>>>
where
    R: RealField + Default + Debug,
{
    match estimation::generalized_master_equation_kernel(
        state,
        history,
        markov_operator,
        memory_kernel,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
