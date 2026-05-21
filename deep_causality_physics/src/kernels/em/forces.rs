/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;
use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_num::RealField;

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
/// * `Ok(CausalMultiVector<R>)` - Force bivector ($J \wedge B$).
/// * `Err(PhysicsError)` - If dimension mismatch occurs.
pub fn lorentz_force_kernel<R>(
    j: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
) -> Result<CausalMultiVector<R>, PhysicsError>
where
    R: RealField,
{
    if j.metric() != b.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Lorentz Force: {:?} vs {:?}",
            j.metric(),
            b.metric()
        )));
    }
    let f = j.outer_product(b);
    if f.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite value in Lorentz force (J ∧ B)".into(),
        ));
    }
    Ok(f)
}
