/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::electromagnetism::quantities::PhysicalField;
use crate::error::PhysicsError;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::{CausalMultiVector, MultiVector};

// Kernels

/// Calculates the magnetic force component of the Lorentz Force: $F = J \times B$.
///
/// In Geometric Algebra, this is computed via the outer product $J \wedge B$, representing the
/// force as a bivector (torque-like/plane) in this implementation.
/// To recover the vector force in 3D, one would typically take the dual.
///
/// # Arguments
/// * `j` - Current density vector $J$.
/// * `b` - Magnetic field vector $B$.
///
/// # Returns
/// * `Ok(CausalMultiVector<f64>)` - Force bivector ($J \wedge B$).
/// * `Err(PhysicsError)` - If dimension mismatch occurs.
pub fn lorentz_force_kernel(
    j: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> Result<CausalMultiVector<f64>, PhysicsError> {
    // F = J x B
    // Calculates the magnetic force density component of the Lorentz Force.
    // Full Lorentz force density: f = \rho E + J \times B
    // This kernel specifically computes the J \times B term.
    let f = j.outer_product(b);
    Ok(f)
}

// Wrappers

/// Causal wrapper for [`lorentz_force_kernel`].
pub fn lorentz_force(
    j: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhysicalField> {
    match lorentz_force_kernel(j, b) {
        Ok(f) => PropagatingEffect::pure(PhysicalField(f)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
