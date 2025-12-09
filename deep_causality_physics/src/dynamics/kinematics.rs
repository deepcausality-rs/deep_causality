/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::dynamics::quantities::{Frequency, Mass, MomentOfInertia};
use crate::error::PhysicsError;
use crate::quantum::quantities::Energy;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::Metric;
use deep_causality_multivector::{CausalMultiVector, MultiVector};

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalVector(pub CausalMultiVector<f64>);

impl Default for PhysicalVector {
    fn default() -> Self {
        // Return a scalar 0 multivector with Euclidean metric
        Self(CausalMultiVector::new(vec![0.0], Metric::Euclidean(0)).unwrap())
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

// Kernels

pub fn kinetic_energy_kernel(
    mass: Mass,
    velocity: &CausalMultiVector<f64>,
) -> Result<f64, PhysicsError> {
    // KE = 0.5 * m * v^2
    // v^2 in GA is v * v = |v|^2 (scalar)
    let v_sq = velocity.squared_magnitude();
    let e = 0.5 * mass.value() * v_sq;
    Ok(e)
}

pub fn rotational_kinetic_energy_kernel(
    inertia: MomentOfInertia,
    omega: Frequency,
) -> Result<f64, PhysicsError> {
    // KE_rot = 0.5 * I * w^2
    let w = omega.value();
    let e = 0.5 * inertia.value() * w * w;
    Ok(e)
}

// Wrappers

pub fn kinetic_energy(mass: Mass, velocity: &CausalMultiVector<f64>) -> PropagatingEffect<Energy> {
    match kinetic_energy_kernel(mass, velocity) {
        Ok(val) => match Energy::new(val) {
            Ok(e) => PropagatingEffect::pure(e),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn rotational_kinetic_energy(
    inertia: MomentOfInertia,
    omega: Frequency,
) -> PropagatingEffect<Energy> {
    match rotational_kinetic_energy_kernel(inertia, omega) {
        Ok(val) => match Energy::new(val) {
            Ok(e) => PropagatingEffect::pure(e),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn torque(
    radius: &CausalMultiVector<f64>,
    force: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhysicalVector> {
    // Torque = r x F (Outer product in GA gives Bivector torque)
    // T = r ^ F
    let t = radius.outer_product(force);
    PropagatingEffect::pure(PhysicalVector(t))
}

pub fn angular_momentum(
    radius: &CausalMultiVector<f64>,
    momentum: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhysicalVector> {
    // L = r x p = r ^ p
    let l = radius.outer_product(momentum);
    PropagatingEffect::pure(PhysicalVector(l))
}
