/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::dynamics::estimation;
use crate::dynamics::kinematics;
use crate::units::energy::Energy;
use crate::{Frequency, Mass, MomentOfInertia};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_tensor::CausalTensor;

/// Causal wrapper for [`estimation::kalman_filter_linear_kernel`].
pub fn kalman_filter_linear(
    x_pred: &CausalTensor<f64>,
    p_pred: &CausalTensor<f64>,
    measurement: &CausalTensor<f64>,
    measurement_matrix: &CausalTensor<f64>,
    measurement_noise: &CausalTensor<f64>,
    process_noise: &CausalTensor<f64>,
) -> PropagatingEffect<(CausalTensor<f64>, CausalTensor<f64>)> {
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
pub fn kinetic_energy(mass: &Mass, velocity: &CausalMultiVector<f64>) -> PropagatingEffect<Energy> {
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
) -> PropagatingEffect<Energy> {
    match kinematics::rotational_kinetic_energy_kernel(*moment_of_inertia, *angular_velocity) {
        Ok(v) => match Energy::new(v) {
            Ok(e) => PropagatingEffect::pure(e),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::torque_kernel`].
pub fn torque(
    radius: &CausalMultiVector<f64>,
    force: &CausalMultiVector<f64>,
) -> PropagatingEffect<crate::dynamics::kinematics::PhysicalVector> {
    match kinematics::torque_kernel(radius, force) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::angular_momentum_kernel`].
pub fn angular_momentum(
    radius: &CausalMultiVector<f64>,
    momentum: &CausalMultiVector<f64>,
) -> PropagatingEffect<kinematics::PhysicalVector> {
    match kinematics::angular_momentum_kernel(radius, momentum) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
