/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_algebra::RealField;
use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_num::FromPrimitive;

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
    pub fn calculate_field_tensor<R>(
        gradient: &CausalMultiVector<R>,
        potential: &CausalMultiVector<R>,
    ) -> Result<CausalMultiVector<R>, PhysicsError>
    where
        R: RealField,
    {
        Self::validate_compatibility(gradient, potential)?;

        let da = gradient.geometric_product(potential);
        let f = da.grade_projection(2);

        Self::validate_finiteness(&f, "EM Field Tensor F")?;
        Ok(f)
    }

    /// Calculates the Lorenz Gauge scalar.
    ///
    /// $L = \nabla \cdot A$
    pub fn calculate_potential_divergence<R>(
        gradient: &CausalMultiVector<R>,
        potential: &CausalMultiVector<R>,
    ) -> Result<R, PhysicsError>
    where
        R: RealField + FromPrimitive,
    {
        Self::validate_compatibility(gradient, potential)?;
        Self::validate_pure_grade(gradient, 1, "gradient")?;
        Self::validate_pure_grade(potential, 1, "potential")?;

        let da = gradient.inner_product(potential);
        let scalar = *da.get(0).unwrap_or(&R::zero());

        if !scalar.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite potential divergence".into(),
            ));
        }

        Ok(scalar)
    }

    /// Calculates the Source Current Density $J$ from the Field Tensor $F$.
    pub fn calculate_current_density<R>(
        gradient: &CausalMultiVector<R>,
        field: &CausalMultiVector<R>,
    ) -> Result<CausalMultiVector<R>, PhysicsError>
    where
        R: RealField,
    {
        Self::validate_compatibility(gradient, field)?;

        let df = gradient.inner_product(field);
        let j = df.grade_projection(1);

        Self::validate_finiteness(&j, "Current Density J")?;
        Ok(j)
    }

    /// Calculates the Poynting Flux Vector $S$.
    ///
    /// $S = E \times B$
    pub fn calculate_poynting_flux<R>(
        e_field: &CausalMultiVector<R>,
        b_field: &CausalMultiVector<R>,
    ) -> Result<CausalMultiVector<R>, PhysicsError>
    where
        R: RealField,
    {
        Self::validate_compatibility(e_field, b_field)?;

        let s = e_field.outer_product(b_field);

        Self::validate_finiteness(&s, "Poynting Flux S")?;
        Ok(s)
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn validate_compatibility<R>(
        a: &CausalMultiVector<R>,
        b: &CausalMultiVector<R>,
    ) -> Result<(), PhysicsError>
    where
        R: RealField,
    {
        if a.metric() != b.metric() {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Metric mismatch in Maxwell Solver: {:?} vs {:?}",
                a.metric(),
                b.metric()
            )));
        }
        Ok(())
    }

    fn validate_finiteness<R>(mv: &CausalMultiVector<R>, context: &str) -> Result<(), PhysicsError>
    where
        R: RealField,
    {
        if mv.data().iter().any(|v| !v.is_finite()) {
            return Err(PhysicsError::NumericalInstability(format!(
                "Non-finite value detected in {}",
                context
            )));
        }
        Ok(())
    }

    fn validate_pure_grade<R>(
        mv: &CausalMultiVector<R>,
        expected_grade: u32,
        context: &str,
    ) -> Result<(), PhysicsError>
    where
        R: RealField + FromPrimitive,
    {
        let eps = R::from_f64(1e-10)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1e-10)".into()))?;
        for (i, &val) in mv.data().iter().enumerate() {
            if val.abs() > eps {
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
