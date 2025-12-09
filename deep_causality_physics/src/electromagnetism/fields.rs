/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::electromagnetism::quantities::{MagneticFlux, PhysicalField};
use deep_causality_core::PropagatingEffect;
use deep_causality_multivector::{CausalMultiVector, MultiVector};

// Kernels

// Wrappers

pub fn maxwell_gradient(potential: &CausalMultiVector<f64>) -> PropagatingEffect<PhysicalField> {
    // Field Tensor F = dA (Exterior Derivative of Potential A)
    // F is a bivector (grade 2).
    // Currently, CausalMultiVector represents a value at a point, not a field over a manifold.
    // Therefore, spatial differentiation (gradient/curl) requests external context (Grid/Manifold).
    //
    // STUB: Returns the potential as-is to satisfy type signature, or would ideally return Zero/Error.
    // Returning Error might break pipelines relying on "pass-through" for now.
    // We wrap it in PhysicalField just to match signature.
    // TODO: Implement differentiation via `deep_causality_topology` Manifolds.
    PropagatingEffect::pure(PhysicalField(potential.clone()))
}

pub fn lorenz_gauge(_potential: &CausalMultiVector<f64>) -> PropagatingEffect<f64> {
    // Div A + d(phi)/dt = 0
    // Scalar part of derivative.
    // Stub:
    PropagatingEffect::pure(0.0)
}

pub fn poynting_vector(
    e: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhysicalField> {
    // S = E x B / mu0
    // Returns the Poynting Vector (flux of energy).
    // In GA, represented as vector, or bivector depending on duals.
    // We return Energy Density Flux (Vector implies Outer Product dual).
    // Here: Outer Product (Bivector representation of flux plane).
    let s = e.outer_product(b);
    PropagatingEffect::pure(PhysicalField(s))
}

pub fn magnetic_helicity_density(
    potential: &CausalMultiVector<f64>,
    field: &CausalMultiVector<f64>,
) -> PropagatingEffect<MagneticFlux> {
    // Helicity Density h = A . B
    // Total Helicity H is the integral of h over volume.
    // This function computes the local density.

    let h_scalar_mv = potential.inner_product(field);
    // Extract Grade 0 (Scalar)
    let h_val = h_scalar_mv.data()[0];

    match MagneticFlux::new(h_val) {
        Ok(h) => PropagatingEffect::pure(h),
        Err(e) => PropagatingEffect::from_error(e),
    }
}
