/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, MultiVector};

// Kernels

/// Stub: Calculates the Maxwell gradient (Electromagnetic Field Tensor).
///
/// Currently returns the potential as-is wrapped in `PhysicalField` due to lack of spatial differentiation context.
///
/// # Arguments
/// * `potential` - Electromagnetic potential vector field $A$.
///
/// # Returns
/// * `Result<CausalMultiVector<f64>, PhysicsError>` - Field tensor $F$ (Stub).
pub fn maxwell_gradient_kernel(
    potential: &CausalMultiVector<f64>,
) -> Result<CausalMultiVector<f64>, crate::PhysicsError> {
    // Field Tensor F = dA (Exterior Derivative of Potential A)
    // F is a bivector (grade 2).
    // Currently, CausalMultiVector represents a value at a point, not a field over a manifold.
    // Therefore, spatial differentiation (gradient/curl) requests external context (Grid/Manifold).
    //
    // STUB: Returns the potential as-is to satisfy type signature.
    // TODO: Implement differentiation via `deep_causality_topology` Manifolds.
    Ok(potential.clone())
}

/// Stub: Calculates the Lorenz gauge condition.
///
/// Returns 0.0 as a placeholder for $\nabla \cdot A + \frac{1}{c^2} \frac{\partial \phi}{\partial t} = 0$.
pub fn lorenz_gauge_kernel(
    _potential: &CausalMultiVector<f64>,
) -> Result<f64, crate::PhysicsError> {
    // Div A + d(phi)/dt = 0
    // Scalar part of derivative.
    // Stub:
    Ok(0.0)
}

/// Calculates the Poynting Vector (Energy Flux): $S = E \times B$.
///
/// Uses the outer product $E \wedge B$ to represent flux as a bivector.
///
/// # Arguments
/// * `e` - Electric field vector $E$.
/// * `b` - Magnetic field vector $B$.
///
/// # Returns
/// * `Result<CausalMultiVector<f64>, PhysicsError>` - Poynting vector (as bivector).
pub fn poynting_vector_kernel(
    e: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> Result<CausalMultiVector<f64>, crate::PhysicsError> {
    // S = E x B / mu0
    // Returns the Poynting Vector (flux of energy).
    // In GA, represented as vector, or bivector depending on duals.
    // We return Energy Density Flux (Vector implies Outer Product dual).
    // Here: Outer Product (Bivector representation of flux plane).
    let s = e.outer_product(b);
    Ok(s)
}

/// Calculates Magnetic Helicity Density: $h = A \cdot B$.
///
/// # Arguments
/// * `potential` - Vector potential $A$.
/// * `field` - Magnetic field $B$.
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Helicity density scalar.
pub fn magnetic_helicity_density_kernel(
    potential: &CausalMultiVector<f64>,
    field: &CausalMultiVector<f64>,
) -> Result<f64, crate::PhysicsError> {
    // Helicity Density h = A . B
    // Total Helicity H is the integral of h over volume.
    // This function computes the local density.

    let h_scalar_mv = potential.inner_product(field);
    // Extract Grade 0 (Scalar)
    let h_val = h_scalar_mv.data()[0];
    Ok(h_val)
}
