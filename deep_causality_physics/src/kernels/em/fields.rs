/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, MultiVector};

// Kernels

use crate::PhysicsError;
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

/// Calculates the Maxwell gradient (Electromagnetic Field Tensor).
///
/// Computes the exterior derivative $F = dA$ of the electromagnetic potential 1-form $A$.
/// Requires the potential to be defined on a Manifold structure to provide spatial context.
///
/// # Arguments
/// * `potential_manifold` - Manifold containing the potential 1-form $A$ on its 1-simplices.
///
/// # Returns
/// * `Result<CausalTensor<R>, PhysicsError>` - Field tensor $F$ (2-form) on the 2-simplices.
pub fn maxwell_gradient_kernel<R>(
    potential_manifold: &SimplicialManifold<R, R>,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + Default + PartialEq + Debug,
{
    let f_tensor = potential_manifold.exterior_derivative(1);
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
/// * `Result<CausalTensor<R>, PhysicsError>` - Divergence scalar field (0-form) on vertices.
pub fn lorenz_gauge_kernel<R>(
    potential_manifold: &SimplicialManifold<R, R>,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq,
{
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
/// * `Result<CausalMultiVector<R>, PhysicsError>` - Poynting vector $S = E \times B$.
pub fn poynting_vector_kernel<R>(
    e: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
) -> Result<CausalMultiVector<R>, PhysicsError>
where
    R: RealField + MaybeParallel,
{
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
/// * `Result<R, PhysicsError>` - Helicity density scalar.
pub fn magnetic_helicity_density_kernel<R>(
    potential: &CausalMultiVector<R>,
    field: &CausalMultiVector<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel,
{
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
/// * `Result<CausalTensor<R>, PhysicsError>` - Current density 1-form $J$.
pub fn proca_equation_kernel<R>(
    field_manifold: &SimplicialManifold<R, R>,
    potential_manifold: &SimplicialManifold<R, R>,
    mass: R,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    // 1. Compute delta F (codifferential of 2-form)
    let delta_f = field_manifold.codifferential(2);

    if !mass.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite mass in Proca".into(),
        ));
    }
    if delta_f.as_slice().iter().any(|v: &R| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "delta(F) has non-finite entries".into(),
        ));
    }

    // 2. Compute m^2 A
    let m2 = mass * mass;
    if !m2.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "m^2 overflowed in Proca".into(),
        ));
    }
    let a_full = potential_manifold.data();

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
        .any(|v: &R| !v.is_finite())
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
    if delta_f.shape() != m2_a.shape() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Shape mismatch in Proca Equation: delta F {:?} vs m^2 A {:?}",
            delta_f.shape(),
            m2_a.shape()
        )));
    }
    let j = delta_f + m2_a;

    if j.as_slice().iter().any(|v: &R| !v.is_finite()) {
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
/// * `Result<R, PhysicsError>` - Energy density scalar $u$.
pub fn energy_density_kernel<R>(
    e: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    if e.metric() != b.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Energy Density: {:?} vs {:?}",
            e.metric(),
            b.metric()
        )));
    }

    if e.data().iter().any(|v| !v.is_finite()) || b.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite input in Energy Density".into(),
        ));
    }

    let e_squared = e.euclidean_squared_magnitude_3d();
    let b_squared = b.euclidean_squared_magnitude_3d();

    if !e_squared.is_finite() || !b_squared.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite squared magnitude in Energy Density".into(),
        ));
    }

    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let u = half * (e_squared + b_squared);

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
/// * `Result<R, PhysicsError>` - Lagrangian density scalar $\mathcal{L}$.
pub fn lagrangian_density_kernel<R>(
    e: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    if e.metric() != b.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Lagrangian Density: {:?} vs {:?}",
            e.metric(),
            b.metric()
        )));
    }

    if e.data().iter().any(|v| !v.is_finite()) || b.data().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite input in Lagrangian Density".into(),
        ));
    }

    let e_squared = e.euclidean_squared_magnitude_3d();
    let b_squared = b.euclidean_squared_magnitude_3d();

    if !e_squared.is_finite() || !b_squared.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite squared magnitude in Lagrangian Density".into(),
        ));
    }

    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let lagrangian = half * (e_squared - b_squared);

    if !lagrangian.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite result in Lagrangian Density".into(),
        ));
    }

    Ok(lagrangian)
}
