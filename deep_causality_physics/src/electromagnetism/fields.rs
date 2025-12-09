/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, MultiVector};

// Kernels

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;
use crate::PhysicsError;

/// Calculates the Maxwell gradient (Electromagnetic Field Tensor).
///
/// Computes the exterior derivative $F = dA$ of the electromagnetic potential 1-form $A$.
/// Requires the potential to be defined on a Manifold structure to provide spatial context.
///
/// # Arguments
/// * `potential_manifold` - Manifold containing the potential 1-form $A$ on its 1-simplices.
///
/// # Returns
/// * `Result<CausalTensor<f64>, PhysicsError>` - Field tensor $F$ (2-form) on the 2-simplices.
pub fn maxwell_gradient_kernel(
    potential_manifold: &Manifold<f64>,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // F = dA (Exterior Derivative)
    // potential_manifold contains the 1-form A.
    // exterior_derivative(1) computes the 2-form F on the faces.

    let f_tensor = potential_manifold.exterior_derivative(1);
    Ok(f_tensor)
}

/// Calculates the Lorenz gauge condition: $\nabla \cdot A = 0$.
///
/// Computes the codifferential $\delta A$ (divergence) of the potential 1-form.
///
/// # Arguments
/// * `potential_manifold` - Manifold containing the potential 1-form $A$.
///
/// # Returns
/// * `Result<CausalTensor<f64>, PhysicsError>` - Divergence scalar field (0-form) on vertices.
pub fn lorenz_gauge_kernel(
    potential_manifold: &Manifold<f64>,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // Lorenz Gauge: Div A = 0
    // Div A is represented by codifferential delta(A) for a 1-form.
    // delta: k-form -> (k-1)-form.
    // Result is a 0-form (scalar field on vertices).

    let divergence = potential_manifold.codifferential(1);
    Ok(divergence)
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
) -> Result<CausalMultiVector<f64>, PhysicsError> {
    // S = E x B / mu0
    // Returns the Poynting Vector (flux of energy).
    // DeepCausality uses Geometric Algebra.
    // The outer product E ^ B represents the specific plane of energy flux (bivector).
    // This is the dual of the classical vector cross product.
    // We return Energy Density Flux in this bivector form.
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
