/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Frequency, Mass, MomentOfInertia, PhysicsError};
use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_num::RealField;

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalVector<R: RealField>(pub CausalMultiVector<R>);

impl<R: RealField> Default for PhysicalVector<R> {
    fn default() -> Self {
        Self(
            CausalMultiVector::new(
                vec![R::zero()],
                deep_causality_multivector::Metric::Euclidean(0),
            )
            .unwrap(),
        )
    }
}

impl<R: RealField> PhysicalVector<R> {
    pub fn new(val: CausalMultiVector<R>) -> Self {
        Self(val)
    }
    pub fn inner(&self) -> &CausalMultiVector<R> {
        &self.0
    }
    pub fn into_inner(self) -> CausalMultiVector<R> {
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
    mass: Mass<f64>,
    velocity: &CausalMultiVector<f64>,
) -> Result<f64, PhysicsError> {
    // Ensure physically meaningful squared speed
    let v_sq = velocity.squared_magnitude();
    if !v_sq.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Velocity squared magnitude is not finite".into(),
        ));
    }
    if v_sq < -1e-12 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Negative squared speed in kinetic energy calculation".into(),
        ));
    }
    let v_sq_clamped = if v_sq < 0.0 { 0.0 } else { v_sq };
    let e = 0.5 * mass.value() * v_sq_clamped;
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
    inertia: MomentOfInertia<f64>,
    omega: Frequency<f64>,
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
pub fn torque_kernel<R>(
    radius: &CausalMultiVector<R>,
    force: &CausalMultiVector<R>,
) -> Result<PhysicalVector<R>, PhysicsError>
where
    R: RealField,
{
    if radius.metric() != force.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch: {:?} != {:?}",
            radius.metric(),
            force.metric()
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
pub fn angular_momentum_kernel<R>(
    radius: &CausalMultiVector<R>,
    momentum: &CausalMultiVector<R>,
) -> Result<PhysicalVector<R>, PhysicsError>
where
    R: RealField,
{
    if radius.metric() != momentum.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch: {:?} != {:?}",
            radius.metric(),
            momentum.metric()
        )));
    }
    let l = radius.outer_product(momentum);
    Ok(PhysicalVector(l))
}
