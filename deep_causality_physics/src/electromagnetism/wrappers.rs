/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MagneticFlux, PhysicalField};
use crate::{fields, forces};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

/// Causal wrapper for [`forces::lorentz_force_kernel`].
pub fn lorentz_force(
    j: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhysicalField> {
    match forces::lorentz_force_kernel(j, b) {
        Ok(f) => PropagatingEffect::pure(PhysicalField(f)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::maxwell_gradient_kernel`].
pub fn maxwell_gradient(
    potential_manifold: &Manifold<f64>,
) -> PropagatingEffect<CausalTensor<f64>> {
    match fields::maxwell_gradient_kernel(potential_manifold) {
        Ok(f) => PropagatingEffect::pure(f),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::lorenz_gauge_kernel`].
pub fn lorenz_gauge(potential_manifold: &Manifold<f64>) -> PropagatingEffect<CausalTensor<f64>> {
    match fields::lorenz_gauge_kernel(potential_manifold) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::poynting_vector_kernel`].
pub fn poynting_vector(
    e: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhysicalField> {
    match fields::poynting_vector_kernel(e, b) {
        Ok(val) => PropagatingEffect::pure(PhysicalField(val)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::magnetic_helicity_density_kernel`].
pub fn magnetic_helicity_density(
    potential: &CausalMultiVector<f64>,
    field: &CausalMultiVector<f64>,
) -> PropagatingEffect<MagneticFlux> {
    match fields::magnetic_helicity_density_kernel(potential, field) {
        Ok(val) => match MagneticFlux::new(val) {
            Ok(h) => PropagatingEffect::pure(h),
            Err(e) => PropagatingEffect::from_error(e),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::proca_equation_kernel`].
pub fn proca_equation(
    field_manifold: &Manifold<f64>,
    potential_manifold: &Manifold<f64>,
    mass: f64,
) -> PropagatingEffect<CausalTensor<f64>> {
    match fields::proca_equation_kernel(field_manifold, potential_manifold, mass) {
        Ok(j) => PropagatingEffect::pure(j),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
