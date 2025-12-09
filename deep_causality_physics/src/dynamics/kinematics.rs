/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Frequency, Mass, MomentOfInertia, PhysicsError};
use deep_causality_multivector::{CausalMultiVector, MultiVector};

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalVector(pub CausalMultiVector<f64>);

impl Default for PhysicalVector {
    fn default() -> Self {
        // Return a scalar 0 multivector with Euclidean metric
        Self(
            CausalMultiVector::new(vec![0.0], deep_causality_multivector::Metric::Euclidean(0))
                .unwrap(),
        )
    }
}

impl PhysicalVector {
    pub fn new(val: CausalMultiVector<f64>) -> Self {
        Self(val)
    }
    pub fn inner(&self) -> &CausalMultiVector<f64> {
        &self.0
    }
    pub fn into_inner(self) -> CausalMultiVector<f64> {
        self.0
    }
}

/// Calculates translational kinetic energy: $K = \frac{1}{2} m v^2$.
///
/// # Arguments
/// * `mass` - Mass of the object.
/// * `velocity` - Velocity vector (multivector).
///
/// # Returns
/// * `Ok(f64)` - Kinetic energy in Joules.
/// * `Err(PhysicsError)` - If an error occurs (unlikely with valid inputs).
pub fn kinetic_energy_kernel(
    mass: Mass,
    velocity: &CausalMultiVector<f64>,
) -> Result<f64, PhysicsError> {
    // KE = 0.5 * m * v^2
    // v^2 in GA is v * v = |v|^2 (scalar) for vectors
    let v_sq = velocity.squared_magnitude();
    let e = 0.5 * mass.value() * v_sq;
    Ok(e)
}

/// Calculates rotational kinetic energy: $K_{rot} = \frac{1}{2} I \omega^2$.
///
/// # Arguments
/// * `inertia` - Moment of inertia.
/// * `omega` - Angular frequency (magnitude of angular velocity).
///
/// # Returns
/// * `Ok(f64)` - Rotational kinetic energy in Joules.
pub fn rotational_kinetic_energy_kernel(
    inertia: MomentOfInertia,
    omega: Frequency,
) -> Result<f64, PhysicsError> {
    // KE_rot = 0.5 * I * w^2
    let w = omega.value();
    let e = 0.5 * inertia.value() * w * w;
    Ok(e)
}

/// Calculates torque as the outer product of radius and force: $\tau = r \wedge F$.
///
/// In Geometric Algebra, torque is a bivector representing the plane of rotation.
///
/// # Arguments
/// * `radius` - Position vector from pivot.
/// * `force` - Force vector applied.
///
/// # Returns
/// * `Result<PhysicalVector, PhysicsError>` - The torque bivector wrapped in a physical vector.
pub fn torque_kernel(
    radius: &CausalMultiVector<f64>,
    force: &CausalMultiVector<f64>,
) -> Result<PhysicalVector, PhysicsError> {
    // Torque = r x F (Outer product in GA gives Bivector torque)
    // T = r ^ F
    if radius.metric() != force.metric() {
        return Err(PhysicsError::new(crate::PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch: {:?} != {:?}",
                radius.metric(),
                force.metric()
            ),
        )));
    }
    let t = radius.outer_product(force);
    Ok(PhysicalVector(t))
}

/// Calculates angular momentum as the outer product of radius and linear momentum: $L = r \wedge p$.
///
/// # Arguments
/// * `radius` - Position vector.
/// * `momentum` - Linear momentum vector ($p = mv$).
///
/// # Returns
/// * `Result<PhysicalVector, PhysicsError>` - The angular momentum bivector.
pub fn angular_momentum_kernel(
    radius: &CausalMultiVector<f64>,
    momentum: &CausalMultiVector<f64>,
) -> Result<PhysicalVector, PhysicsError> {
    // L = r x p = r ^ p
    if radius.metric() != momentum.metric() {
        return Err(PhysicsError::new(crate::PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch: {:?} != {:?}",
                radius.metric(),
                momentum.metric()
            ),
        )));
    }
    let l = radius.outer_product(momentum);
    Ok(PhysicalVector(l))
}
