/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_multivector::{CausalMultiVector, MultiVector};

/// A standard solver for Maxwell's Equations in Geometric Algebra.
///
/// This solver provides methods to derive electromagnetic properties from potentials
/// and fields in a coordinate-free manner, suitable for relativistic electrodynamics.
///
/// # Theoretical Basis
///
/// In Geometric Algebra (Space-Time Algebra), Maxwell's equations are unified into a single equation:
/// $\nabla F = J$
/// where:
/// * $F = \nabla A$ is the electromagnetic field bivector.
/// * $A$ is the vector potential.
/// * $J$ is the source current density.
/// * $\nabla$ is the vector derivative (gradient).
pub struct MaxwellSolver;

impl MaxwellSolver {
    /// Calculates the Electromagnetic Field Tensor $F$ from the Potential $A$.
    ///
    /// $F = \nabla \wedge A$
    ///
    /// This represents the "Faraday bivector" containing both Electric (E) and Magnetic (B) fields.
    ///
    /// # Arguments
    /// * `gradient` - The vector derivative operator $\nabla$ (or directional derivatives).
    /// * `potential` - The vector potential $A$.
    ///
    /// # Returns
    /// * `Ok(CausalMultiVector)` - The field bivector $F$ (Grade 2).
    /// * `Err(PhysicsError)` - If dimensions or metrics mismatch.
    pub fn calculate_field_tensor(
        gradient: &CausalMultiVector<f64>,
        potential: &CausalMultiVector<f64>,
    ) -> Result<CausalMultiVector<f64>, PhysicsError> {
        Self::validate_compatibility(gradient, potential)?;

        // F = Grade-2Projection( d * A )
        // In GA, da = d.a (scalar) + d^a (bivector)
        // We want the bivector part.
        let da = gradient.geometric_product(potential);
        let f = da.grade_projection(2);

        Self::validate_finiteness(&f, "EM Field Tensor F")?;
        Ok(f)
    }

    /// Calculates the Lorenz Gauge scalar.
    ///
    /// $L = \nabla \cdot A$
    ///
    /// In the Lorenz gauge, this value should be zero (or near zero).
    ///
    /// # Returns
    /// * `Ok(f64)` - The scalar divergence.
    /// * `Err(PhysicsError)` - If inputs are not pure grade-1 vectors.
    pub fn calculate_potential_divergence(
        gradient: &CausalMultiVector<f64>,
        potential: &CausalMultiVector<f64>,
    ) -> Result<f64, PhysicsError> {
        Self::validate_compatibility(gradient, potential)?;
        Self::validate_pure_grade(gradient, 1, "gradient")?;
        Self::validate_pure_grade(potential, 1, "potential")?;

        // L = d . A (Scalar part of geometric product)
        let da = gradient.inner_product(potential);
        let scalar = *da.get(0).unwrap_or(&0.0);

        if !scalar.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite potential divergence".into(),
            ));
        }

        Ok(scalar)
    }

    /// Calculates the Source Current Density $J$ from the Field Tensor $F$.
    ///
    /// $J = \nabla \cdot F$ (Vector part of $\nabla F$)
    ///
    /// Note: The full source equation is $\nabla F = J$. Since $F$ is a bivector,
    /// $\nabla F$ has a vector part ($\nabla \cdot F$) and a trivector part ($\nabla \wedge F$).
    /// The trivector part is zero $\nabla \wedge F = 0$ (Bianchi identity) if F comes from a potential.
    /// Thus $J$ corresponds to the vector part.
    pub fn calculate_current_density(
        gradient: &CausalMultiVector<f64>,
        field: &CausalMultiVector<f64>,
    ) -> Result<CausalMultiVector<f64>, PhysicsError> {
        Self::validate_compatibility(gradient, field)?;

        // J = d . F
        let df = gradient.inner_product(field);

        // J is strictly a vector (Grade 1) in standard Maxwell theory
        let j = df.grade_projection(1);

        Self::validate_finiteness(&j, "Current Density J")?;
        Ok(j)
    }

    /// Calculates the Poynting Flux Vector $S$.
    ///
    /// $S = E \times B$
    ///
    /// Computes the energy flux density from separated Electric and Magnetic field vectors.
    ///
    /// # Arguments
    /// * `e_field` - Electric field vector E.
    /// * `b_field` - Magnetic field vector B.
    pub fn calculate_poynting_flux(
        e_field: &CausalMultiVector<f64>,
        b_field: &CausalMultiVector<f64>,
    ) -> Result<CausalMultiVector<f64>, PhysicsError> {
        Self::validate_compatibility(e_field, b_field)?;

        // In standard vector calculus: S = E x B
        // In 3D GA: a x b = -I(a ^ b)
        // However, the physics crate's existing kernel uses simple outer product E ^ B
        // which represents the flux plane bivector.
        // For production physics, if the user passes E and B vectors, they likely want the
        // FLUX vector.
        // But since this is a general ND library, returning the bivector (plane of flow)
        // is often more fundamental.
        // Let's stick to the bivector representation E ^ B consistent with the crate's kernel.

        let s = e_field.outer_product(b_field);

        Self::validate_finiteness(&s, "Poynting Flux S")?;
        Ok(s)
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn validate_compatibility(
        a: &CausalMultiVector<f64>,
        b: &CausalMultiVector<f64>,
    ) -> Result<(), PhysicsError> {
        if a.metric() != b.metric() {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Metric mismatch in Maxwell Solver: {:?} vs {:?}",
                a.metric(),
                b.metric()
            )));
        }
        Ok(())
    }

    fn validate_finiteness(mv: &CausalMultiVector<f64>, context: &str) -> Result<(), PhysicsError> {
        if mv.data().iter().any(|v| !v.is_finite()) {
            return Err(PhysicsError::NumericalInstability(format!(
                "Non-finite value detected in {}",
                context
            )));
        }
        Ok(())
    }

    fn validate_pure_grade(
        mv: &CausalMultiVector<f64>,
        expected_grade: u32,
        context: &str,
    ) -> Result<(), PhysicsError> {
        for (i, &val) in mv.data().iter().enumerate() {
            if val.abs() > 1e-10 {
                let grade = i.count_ones();
                if grade != expected_grade {
                    return Err(PhysicsError::PhysicalInvariantBroken(format!(
                        "{} must be pure grade {} multivector, but contains grade {} at index {}",
                        context, expected_grade, grade, i
                    )));
                }
            }
        }
        Ok(())
    }
}
