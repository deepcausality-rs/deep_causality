/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quantum Electrodynamics (QED) Theory Module
//!
//! Provides a unified interface for electromagnetic field operations in the U(1) gauge theory.
//!
//! # Overview
//!
//! QED describes the interaction of light (photons) with charged matter (electrons).
//! The electromagnetic field is represented by the 4-potential $A_\mu$ and field tensor $F_{\mu\nu}$.
//!
//! # Example
//!
//! ```ignore
//! use deep_causality_physics::theories::QED;
//!
//! // Create QED field from E and B components
//! let qed = QED::from_fields(e_field, b_field)?;
//!
//! // Compute physical quantities
//! let energy = qed.energy_density()?;
//! let lagrangian = qed.lagrangian_density()?;
//! let poynting = qed.poynting_vector()?;
//! ```

use crate::error::PhysicsError;
use crate::{
    energy_density_kernel, lagrangian_density_kernel, lorentz_force_kernel, poynting_vector_kernel,
};
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};

/// QED (Quantum Electrodynamics) field representation.
///
/// Stores the electric and magnetic field components in geometric algebra form.
/// All operations use the West Coast metric convention (+---) standard in particle physics.
#[derive(Debug, Clone)]
pub struct QED {
    /// Electric field vector E
    electric_field: CausalMultiVector<f64>,
    /// Magnetic field vector B
    magnetic_field: CausalMultiVector<f64>,
    /// Metric signature (should be Minkowski for QED)
    metric: Metric,
}

impl QED {
    /// Creates a new QED field from electric and magnetic field vectors.
    ///
    /// # Arguments
    /// * `electric_field` - Electric field vector $\mathbf{E}$
    /// * `magnetic_field` - Magnetic field vector $\mathbf{B}$
    ///
    /// # Returns
    /// * `Ok(QED)` - Configured QED field
    /// * `Err(PhysicsError)` - If metrics don't match or inputs are invalid
    pub fn from_fields(
        electric_field: CausalMultiVector<f64>,
        magnetic_field: CausalMultiVector<f64>,
    ) -> Result<Self, PhysicsError> {
        if electric_field.metric() != magnetic_field.metric() {
            return Err(PhysicsError::DimensionMismatch(format!(
                "E and B field metrics must match: {:?} vs {:?}",
                electric_field.metric(),
                magnetic_field.metric()
            )));
        }

        let metric = electric_field.metric();

        Ok(Self {
            electric_field,
            magnetic_field,
            metric,
        })
    }

    /// Creates a QED field from field components in 3D Euclidean space.
    ///
    /// # Arguments
    /// * `ex`, `ey`, `ez` - Electric field components
    /// * `bx`, `by`, `bz` - Magnetic field components
    ///
    /// # Returns
    /// * `Ok(QED)` - Configured QED field in Euclidean 3D metric
    pub fn from_components(
        ex: f64,
        ey: f64,
        ez: f64,
        bx: f64,
        by: f64,
        bz: f64,
    ) -> Result<Self, PhysicsError> {
        let metric = Metric::Euclidean(3);

        // Create 8-component multivector for 3D (2^3 = 8)
        let e = CausalMultiVector::new(vec![0.0, ex, ey, ez, 0.0, 0.0, 0.0, 0.0], metric)
            .map_err(|e| PhysicsError::DimensionMismatch(format!("E field error: {:?}", e)))?;

        let b = CausalMultiVector::new(vec![0.0, bx, by, bz, 0.0, 0.0, 0.0, 0.0], metric)
            .map_err(|e| PhysicsError::DimensionMismatch(format!("B field error: {:?}", e)))?;

        Self::from_fields(e, b)
    }

    /// Creates a QED field for a plane wave with orthogonal E and B.
    ///
    /// # Arguments
    /// * `amplitude` - Field amplitude (|E| = |B| in natural units)
    /// * `polarization` - Polarization direction (0 = x, 1 = y)
    ///
    /// # Returns
    /// * `Ok(QED)` - Plane wave field configuration
    pub fn plane_wave(amplitude: f64, polarization: usize) -> Result<Self, PhysicsError> {
        if !amplitude.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Amplitude must be finite".into(),
            ));
        }

        match polarization {
            0 => Self::from_components(amplitude, 0.0, 0.0, 0.0, amplitude, 0.0), // E along x, B along y
            1 => Self::from_components(0.0, amplitude, 0.0, 0.0, 0.0, amplitude), // E along y, B along z
            _ => Err(PhysicsError::DimensionMismatch(format!(
                "Polarization must be 0 or 1, got {}",
                polarization
            ))),
        }
    }

    /// Returns a reference to the electric field vector.
    pub fn electric_field(&self) -> &CausalMultiVector<f64> {
        &self.electric_field
    }

    /// Returns a reference to the magnetic field vector.
    pub fn magnetic_field(&self) -> &CausalMultiVector<f64> {
        &self.magnetic_field
    }

    /// Returns the metric signature used by this field.
    pub fn metric(&self) -> Metric {
        self.metric
    }

    // ========================================================================
    // Physical Quantities
    // ========================================================================

    /// Computes the electromagnetic energy density: $u = \frac{1}{2}(E^2 + B^2)$.
    ///
    /// This is the $T^{00}$ component of the stress-energy tensor in natural units.
    ///
    /// # Returns
    /// * `Ok(f64)` - Energy density scalar
    pub fn energy_density(&self) -> Result<f64, PhysicsError> {
        energy_density_kernel(&self.electric_field, &self.magnetic_field)
    }

    /// Computes the Lagrangian density: $\mathcal{L} = -\frac{1}{4}F_{\mu\nu}F^{\mu\nu} = \frac{1}{2}(E^2 - B^2)$.
    ///
    /// Uses West Coast convention (+---).
    ///
    /// # Returns
    /// * `Ok(f64)` - Lagrangian density scalar
    pub fn lagrangian_density(&self) -> Result<f64, PhysicsError> {
        lagrangian_density_kernel(&self.electric_field, &self.magnetic_field)
    }

    /// Computes the Poynting vector: $\mathbf{S} = \mathbf{E} \times \mathbf{B}$.
    ///
    /// Represents the energy flux (power per unit area) of the electromagnetic field.
    /// Returned as a bivector in geometric algebra form.
    ///
    /// # Returns
    /// * `Ok(CausalMultiVector<f64>)` - Poynting vector (as bivector)
    pub fn poynting_vector(&self) -> Result<CausalMultiVector<f64>, PhysicsError> {
        poynting_vector_kernel(&self.electric_field, &self.magnetic_field)
    }

    /// Computes the Lorentz force density: $\mathbf{f} = \mathbf{J} \times \mathbf{B}$.
    ///
    /// For a current density J in a magnetic field B.
    ///
    /// # Arguments
    /// * `current_density` - Current density vector $\mathbf{J}$
    ///
    /// # Returns
    /// * `Ok(CausalMultiVector<f64>)` - Force density (as bivector)
    pub fn lorentz_force(
        &self,
        current_density: &CausalMultiVector<f64>,
    ) -> Result<CausalMultiVector<f64>, PhysicsError> {
        lorentz_force_kernel(current_density, &self.magnetic_field)
    }

    /// Computes the field invariant: $F_{\mu\nu}F^{\mu\nu} = 2(B^2 - E^2)$.
    ///
    /// This is a Lorentz scalar (same value in all reference frames).
    ///
    /// # Returns
    /// * `Ok(f64)` - Field invariant scalar
    pub fn field_invariant(&self) -> Result<f64, PhysicsError> {
        let e_sq = self.electric_field.squared_magnitude();
        let b_sq = self.magnetic_field.squared_magnitude();

        if !e_sq.is_finite() || !b_sq.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite field magnitude in invariant".into(),
            ));
        }

        // F_μν F^μν = 2(B² - E²)
        Ok(2.0 * (b_sq - e_sq))
    }

    /// Computes the dual invariant: $F_{\mu\nu}\tilde{F}^{\mu\nu} = -4\mathbf{E} \cdot \mathbf{B}$.
    ///
    /// Non-zero for CP-violating configurations.
    ///
    /// # Returns
    /// * `Ok(f64)` - Dual invariant scalar
    pub fn dual_invariant(&self) -> Result<f64, PhysicsError> {
        let inner = self.electric_field.inner_product(&self.magnetic_field);
        let e_dot_b = inner.data()[0]; // Scalar component

        if !e_dot_b.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite E·B in dual invariant".into(),
            ));
        }

        // F_μν F̃^μν = -4 E·B
        Ok(-4.0 * e_dot_b)
    }

    /// Checks if fields are orthogonal (E ⟂ B), as in a plane wave.
    ///
    /// # Returns
    /// * `true` if E·B ≈ 0 within tolerance
    pub fn is_radiation_field(&self) -> bool {
        let inner = self.electric_field.inner_product(&self.magnetic_field);
        inner.data()[0].abs() < 1e-10
    }

    /// Checks if fields have equal magnitude (|E| = |B|), as in a plane wave.
    ///
    /// # Returns
    /// * `true` if |E| ≈ |B| within tolerance
    pub fn is_null_field(&self) -> bool {
        let e_sq = self.electric_field.squared_magnitude();
        let b_sq = self.magnetic_field.squared_magnitude();
        (e_sq - b_sq).abs() < 1e-10 * (e_sq + b_sq).max(1.0)
    }

    /// Computes the momentum density: $\mathbf{g} = \mathbf{S}/c^2 = \epsilon_0(\mathbf{E} \times \mathbf{B})$.
    ///
    /// In natural units (c = 1), this equals the Poynting vector.
    ///
    /// # Returns
    /// * `Ok(CausalMultiVector<f64>)` - Momentum density vector
    pub fn momentum_density(&self) -> Result<CausalMultiVector<f64>, PhysicsError> {
        // In natural units, g = S
        self.poynting_vector()
    }

    /// Computes the intensity (time-averaged power per unit area).
    ///
    /// For sinusoidal fields: $I = \frac{1}{2}|S_{max}|$.
    ///
    /// # Returns
    /// * `Ok(f64)` - Intensity scalar
    pub fn intensity(&self) -> Result<f64, PhysicsError> {
        let s = self.poynting_vector()?;

        // Magnitude of Poynting vector
        let s_sq = s.squared_magnitude();
        if !s_sq.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite Poynting magnitude".into(),
            ));
        }

        Ok(s_sq.abs().sqrt())
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl Default for QED {
    /// Returns a zero-field configuration.
    fn default() -> Self {
        let metric = Metric::Euclidean(3);
        let zero = CausalMultiVector::new(vec![0.0; 8], metric).unwrap();
        Self {
            electric_field: zero.clone(),
            magnetic_field: zero,
            metric,
        }
    }
}
