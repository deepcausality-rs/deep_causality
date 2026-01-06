/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, MultiVector};

// Kernels

use crate::PhysicsError;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

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
    // F = dA (Exterior Derivative) on 1-forms -> 2-forms
    let f_tensor = potential_manifold.exterior_derivative(1);
    // Validate that a 2-form was actually produced (non-empty, expected rank)
    if f_tensor.is_empty() || f_tensor.shape().is_empty() {
        return Err(PhysicsError::DimensionMismatch(
            "Maxwell gradient produced empty or invalid 2-form".into(),
        ));
    }
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
/// Computes the classical 3D cross product of E and B fields.
/// The result is returned as a 3D vector in the same multivector format.
///
/// # Arguments
/// * `e` - Electric field vector $E$ (spatial components at indices 2, 3, 4).
/// * `b` - Magnetic field vector $B$ (spatial components at indices 2, 3, 4).
///
/// # Returns
/// * `Result<CausalMultiVector<f64>, PhysicsError>` - Poynting vector $S = E \times B$.
pub fn poynting_vector_kernel(
    e: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> Result<CausalMultiVector<f64>, PhysicsError> {
    if e.metric() != b.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Poynting Vector: {:?} vs {:?}",
            e.metric(),
            b.metric()
        )));
    }
    if e.data().iter().any(|v| !v.is_finite()) || b.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite input in Poynting Vector".into(),
        ));
    }

    // Classical cross product: S = E × B
    let s = e.euclidean_cross_product_3d(b);

    if s.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite result in Poynting Vector".into(),
        ));
    }

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
    if potential.metric() != field.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Magnetic Helicity: {:?} vs {:?}",
            potential.metric(),
            field.metric()
        )));
    }

    let h_scalar_mv = potential.inner_product(field);
    // Extract Grade 0 (Scalar)
    let h_val = h_scalar_mv.data()[0];
    Ok(h_val)
}

/// Calculates the Proca Equation (Massive Electromagnetism): $\delta F + m^2 A = J$.
///
/// Computes the source current density 1-form $J$ given the field $F$, potential $A$, and mass $m$.
///
/// # Arguments
/// * `field_manifold` - Manifold containing the field 2-form $F$ (maxwell gradient).
/// * `potential_manifold` - Manifold containing the potential 1-form $A$.
/// * `mass` - Mass of the photon $m$ (typically $\approx 0$, but $>0$ in Proca theory).
///
/// # Returns
/// * `Result<CausalTensor<f64>, PhysicsError>` - Current density 1-form $J$.
pub fn proca_equation_kernel(
    field_manifold: &Manifold<f64>,
    potential_manifold: &Manifold<f64>,
    mass: f64,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // Proca: delta F + m^2 A = J
    // F is 2-form. delta F is 1-form.
    // A is 1-form. m^2 A is 1-form.
    // Result J is 1-form.

    // 1. Compute delta F (codifferential of 2-form)
    let delta_f = field_manifold.codifferential(2);

    if !mass.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite mass in Proca".into(),
        ));
    }
    if delta_f.as_slice().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "delta(F) has non-finite entries".into(),
        ));
    }

    // 2. Compute m^2 A (ensure it's a 1-form tensor compatible with delta F)
    let m2 = mass * mass;
    if !m2.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "m^2 overflowed in Proca".into(),
        ));
    }
    let a_full = potential_manifold.data(); // underlying data tensor

    // Build an A tensor on the same shape as delta_f (1-form domain)
    let a_shape = delta_f.shape().to_vec();
    let needed_len = delta_f.len();
    if a_full.len() < needed_len {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Proca: potential data shorter than 1-form domain: {} < {}",
            a_full.len(),
            needed_len
        )));
    }
    if a_full.as_slice()[..needed_len]
        .iter()
        .any(|v| !v.is_finite())
    {
        return Err(PhysicsError::NumericalInstability(
            "A(1-form) has non-finite entries".into(),
        ));
    }
    let a_1form = CausalTensor::new(a_full.as_slice()[..needed_len].to_vec(), a_shape)?;

    let m2_a = a_1form * m2;
    if m2_a.as_slice().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "m^2 A has non-finite entries".into(),
        ));
    }

    // 3. Sum: J = delta F + m^2 A
    // Note: CausalTensor implements Add
    // Check shapes before addition (J = delta_f + m2_a)
    if delta_f.shape() != m2_a.shape() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Shape mismatch in Proca Equation: delta F {:?} vs m^2 A {:?}",
            delta_f.shape(),
            m2_a.shape()
        )));
    }
    let j = delta_f + m2_a;

    if j.as_slice().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Proca current J has non-finite entries".into(),
        ));
    }

    Ok(j)
}

/// Calculates the Electromagnetic Energy Density: $u = \frac{1}{2}(E^2 + B^2)$.
///
/// This is the $T^{00}$ component of the stress-energy tensor in natural units.
///
/// # Arguments
/// * `e` - Electric field vector $E$.
/// * `b` - Magnetic field vector $B$.
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Energy density scalar $u$.
pub fn energy_density_kernel(
    e: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> Result<f64, PhysicsError> {
    // u = (E² + B²) / 2  (in natural units where ε₀ = μ₀ = 1)
    // E² = |E|² = E · E (squared magnitude)
    // B² = |B|² = B · B (squared magnitude)
    if e.metric() != b.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Energy Density: {:?} vs {:?}",
            e.metric(),
            b.metric()
        )));
    }

    // Check for non-finite inputs
    if e.data().iter().any(|v| !v.is_finite()) || b.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite input in Energy Density".into(),
        ));
    }

    // Compute Euclidean squared magnitudes of 3D spatial vectors.
    // We use Euclidean norm (not Lorentzian) because energy density is positive-definite.
    let e_squared = e.euclidean_squared_magnitude_3d();
    let b_squared = b.euclidean_squared_magnitude_3d();

    if !e_squared.is_finite() || !b_squared.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite squared magnitude in Energy Density".into(),
        ));
    }

    // Energy density in natural units: u = ½(|E|² + |B|²)
    let u = 0.5 * (e_squared + b_squared);

    if !u.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite result in Energy Density".into(),
        ));
    }

    Ok(u)
}

/// Calculates the Electromagnetic Lagrangian Density: $\mathcal{L} = -\frac{1}{4} F_{\mu\nu} F^{\mu\nu}$.
///
/// In terms of E and B fields: $\mathcal{L} = \frac{1}{2}(E^2 - B^2)$ (West Coast convention).
///
/// # Arguments
/// * `e` - Electric field vector $E$.
/// * `b` - Magnetic field vector $B$.
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Lagrangian density scalar $\mathcal{L}$.
pub fn lagrangian_density_kernel(
    e: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> Result<f64, PhysicsError> {
    // L = -¼ F_μν F^μν = ½(E² - B²)  (West Coast convention)
    // In particle physics convention (+---), F_{0i} = E_i and F_{ij} = ε_{ijk}B_k
    // The Lagrangian density is L = (E² - B²)/2
    if e.metric() != b.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Lagrangian Density: {:?} vs {:?}",
            e.metric(),
            b.metric()
        )));
    }

    // Check for non-finite inputs
    if e.data().iter().any(|v| !v.is_finite()) || b.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite input in Lagrangian Density".into(),
        ));
    }

    // Compute Euclidean squared magnitudes of 3D spatial vectors.
    let e_squared = e.euclidean_squared_magnitude_3d();
    let b_squared = b.euclidean_squared_magnitude_3d();

    if !e_squared.is_finite() || !b_squared.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite squared magnitude in Lagrangian Density".into(),
        ));
    }

    // Lagrangian density (West Coast: L = (E² - B²)/2)
    let lagrangian = 0.5 * (e_squared - b_squared);

    if !lagrangian.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite result in Lagrangian Density".into(),
        ));
    }

    Ok(lagrangian)
}
