/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::electromagnetism::quantities::{MagneticFlux, PhysicalField};
use crate::electromagnetism::{fields, forces};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;

// Wrappers

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
pub fn maxwell_gradient(potential: &CausalMultiVector<f64>) -> PropagatingEffect<PhysicalField> {
    match fields::maxwell_gradient_kernel(potential) {
        Ok(f) => PropagatingEffect::pure(PhysicalField(f)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::lorenz_gauge_kernel`].
pub fn lorenz_gauge(potential: &CausalMultiVector<f64>) -> PropagatingEffect<f64> {
    match fields::lorenz_gauge_kernel(potential) {
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
