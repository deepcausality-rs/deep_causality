/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::electromagnetism::quantities::PhysicalField;
use crate::error::PhysicsError;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::{CausalMultiVector, MultiVector};

// Kernels

pub fn lorentz_force_kernel(
    j: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> Result<CausalMultiVector<f64>, PhysicsError> {
    // F = J x B
    // Calculates the magnetic force density component of the Lorentz Force.
    // Full Lorentz force density: f = \rho E + J \times B
    // This kernel specifically computes the J \times B term.
    //
    // References:
    // - Jackson, J. D. Classical Electrodynamics. Section 12.1.
    // - Geometric Algebra: J \times B is the dual of the outer product part of J B?
    //   Actually in 3D, a x b = -I (a ^ b).
    //   CausalMultiVector::outer_product computes a ^ b (Bivector).
    //   To get the vector force, one typically multiplies by the pseudoscalar -I inverse.
    //   However, for this implementation we return the Bivector representation (Area/Torque-like)
    //   or assume the user handles the dual.
    //   Standard convention in this library: Return the Wedge Product (Bivector) and let context apply dual if needed.

    let f = j.outer_product(b);
    Ok(f)
}

// Wrappers

pub fn lorentz_force(
    j: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhysicalField> {
    match lorentz_force_kernel(j, b) {
        Ok(f) => PropagatingEffect::pure(PhysicalField(f)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
