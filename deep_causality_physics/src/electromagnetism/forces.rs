/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;
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
    // Check for metric compatibility
    if j.metric() != b.metric() {
        return Err(PhysicsError::new(
            crate::PhysicsErrorEnum::DimensionMismatch(format!(
                "Metric mismatch in Lorentz Force: {:?} vs {:?}",
                j.metric(),
                b.metric()
            )),
        ));
    }
    let f = j.outer_product(b);
    if f.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::new(
            crate::PhysicsErrorEnum::NumericalInstability(
                "Non-finite value in Lorentz force (J ∧ B)".into(),
            ),
        ));
    }
    Ok(f)
}
