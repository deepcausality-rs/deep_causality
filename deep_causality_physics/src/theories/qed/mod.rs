/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quantum Electrodynamics (QED) Theory Module
//!
//! Provides a unified interface for electromagnetic field operations in the U(1) gauge theory
//! taking advantage of the topological `GaugeField` structure.

use crate::error::PhysicsError;
use crate::theories::QED;
use crate::{
    energy_density_kernel, lagrangian_density_kernel, lorentz_force_kernel, poynting_vector_kernel,
};
use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, GaugeField, Manifold, Simplex, SimplicialComplexBuilder,
};

/// Operations specific to Quantum Electrodynamics (QED).
pub trait QedOps {
    /// Creates a new QED field from electric and magnetic field vectors.
    fn from_fields(
        base: Manifold<f64>,
        electric_field: CausalMultiVector<f64>,
        magnetic_field: CausalMultiVector<f64>,
    ) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Creates a QED field from field components in 3D Euclidean space.
    /// Uses a default empty 4D manifold.
    fn from_components(
        ex: f64,
        ey: f64,
        ez: f64,
        bx: f64,
        by: f64,
        bz: f64,
    ) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Creates a QED field for a plane wave with orthogonal E and B.
    /// Uses a default empty 4D manifold.
    fn plane_wave(amplitude: f64, polarization: usize) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Returns the electric field vector E extracted from the field tensor F_μν.
    fn electric_field(&self) -> Result<CausalMultiVector<f64>, PhysicsError>;

    /// Returns the magnetic field vector B extracted from the field tensor F_μν.
    fn magnetic_field(&self) -> Result<CausalMultiVector<f64>, PhysicsError>;

    /// Computes the electromagnetic energy density.
    fn energy_density(&self) -> Result<f64, PhysicsError>;

    /// Computes the Lagrangian density.
    fn lagrangian_density(&self) -> Result<f64, PhysicsError>;

    /// Computes the Poynting vector.
    fn poynting_vector(&self) -> Result<CausalMultiVector<f64>, PhysicsError>;

    /// Computes the Lorentz force density.
    fn lorentz_force(
        &self,
        current_density: &CausalMultiVector<f64>,
    ) -> Result<CausalMultiVector<f64>, PhysicsError>;

    /// Computes the field invariant: 2(B² - E²).
    fn field_invariant(&self) -> Result<f64, PhysicsError>;

    /// Computes the dual invariant: -4E·B.
    fn dual_invariant(&self) -> Result<f64, PhysicsError>;

    /// Checks if fields are orthogonal (E ⟂ B).
    fn is_radiation_field(&self) -> Result<bool, PhysicsError>;

    /// Checks if fields have equal magnitude (|E| = |B|).
    fn is_null_field(&self) -> Result<bool, PhysicsError>;

    /// Computes the momentum density.
    fn momentum_density(&self) -> Result<CausalMultiVector<f64>, PhysicsError>;

    /// Computes the intensity.
    fn intensity(&self) -> Result<f64, PhysicsError>;
}

impl QedOps for QED {
    fn from_fields(
        base: Manifold<f64>,
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

        // QED typically uses 4D spacetime
        let metric = electric_field.metric();
        let dim = 4;
        let num_points = base.len();

        let connection = CausalTensor::zeros(&[num_points, dim, 1]); // U(1) has dim 1

        // Populate Field Strength Tensor F_mn
        // Shape: [num_points, 4, 4, 1] -> Flat size 16 * num_points
        let mut f_data = vec![0.0; num_points * 16];
        let e_data = electric_field.data();
        let b_data = magnetic_field.data();

        for i in 0..num_points {
            let offset = i * 16;

            // Get components (assuming 16-stride for 4D MV).
            // Indices 2, 3, 4 correspond to spatial X, Y, Z in +--- signature (0=s, 1=t, 2=x, 3=y, 4=z).
            let ex = e_data.get(offset + 2).copied().unwrap_or(0.0);
            let ey = e_data.get(offset + 3).copied().unwrap_or(0.0);
            let ez = e_data.get(offset + 4).copied().unwrap_or(0.0);

            let bx = b_data.get(offset + 2).copied().unwrap_or(0.0);
            let by = b_data.get(offset + 3).copied().unwrap_or(0.0);
            let bz = b_data.get(offset + 4).copied().unwrap_or(0.0);

            // F_01 = E_x (Index 1)
            f_data[offset + 1] = ex;
            f_data[offset + 4] = -ex; // F_10

            // F_02 = E_y (Index 2)
            f_data[offset + 2] = ey;
            f_data[offset + 8] = -ey; // F_20

            // F_03 = E_z (Index 3)
            f_data[offset + 3] = ez;
            f_data[offset + 12] = -ez; // F_30

            // F_ij = -epsilon_ijk B_k
            // B_x = F_23 (Index 11)
            f_data[offset + 11] = bx;
            f_data[offset + 14] = -bx; // F_32

            // B_y = F_31 (Index 13)
            f_data[offset + 13] = by;
            f_data[offset + 7] = -by; // F_13

            // B_z = F_12 (Index 6)
            f_data[offset + 6] = bz;
            f_data[offset + 9] = -bz; // F_21
        }

        let field_strength =
            CausalTensor::new(f_data, vec![num_points, dim, dim, 1]).map_err(|e| {
                PhysicsError::DimensionMismatch(format!("Failed to create F tensor: {:?}", e))
            })?;

        Ok(GaugeField::new(base, metric, connection, field_strength))
    }

    fn from_components(
        ex: f64,
        ey: f64,
        ez: f64,
        bx: f64,
        by: f64,
        bz: f64,
    ) -> Result<Self, PhysicsError> {
        let metric = Metric::Minkowski(4);

        // Indices 2,3,4 for spatial vectors
        let mut e_data = vec![0.0; 16];
        e_data[2] = ex;
        e_data[3] = ey;
        e_data[4] = ez;

        let mut b_data = vec![0.0; 16];
        b_data[2] = bx;
        b_data[3] = by;
        b_data[4] = bz;

        let e = CausalMultiVector::new(e_data, metric)
            .map_err(|e| PhysicsError::DimensionMismatch(format!("E field error: {:?}", e)))?;
        let b = CausalMultiVector::new(b_data, metric)
            .map_err(|e| PhysicsError::DimensionMismatch(format!("B field error: {:?}", e)))?;

        // Create minimal manifold (1 point) to satisfy initialization invariants
        let mut builder = SimplicialComplexBuilder::new(0);
        let _ = builder.add_simplex(Simplex::new(vec![0]));
        let complex = builder.build().map_err(|e| {
            PhysicsError::DimensionMismatch(format!("Failed to build complex: {:?}", e))
        })?;

        // Data len must match complex size (1 simplex)
        let data = CausalTensor::new(vec![0.0], vec![1]).map_err(|e| {
            PhysicsError::DimensionMismatch(format!("Failed to create tensor: {:?}", e))
        })?;

        let base: Manifold<f64> = Manifold::new(complex, data, 0).map_err(|e| {
            PhysicsError::DimensionMismatch(format!("Failed to create default manifold: {:?}", e))
        })?;

        Self::from_fields(base, e, b)
    }

    fn plane_wave(amplitude: f64, polarization: usize) -> Result<Self, PhysicsError> {
        if !amplitude.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Amplitude must be finite".into(),
            ));
        }

        match polarization {
            0 => Self::from_components(amplitude, 0.0, 0.0, 0.0, amplitude, 0.0),
            1 => Self::from_components(0.0, amplitude, 0.0, 0.0, 0.0, amplitude),
            _ => Err(PhysicsError::DimensionMismatch(
                "Polarization must be 0 or 1".into(),
            )),
        }
    }

    fn electric_field(&self) -> Result<CausalMultiVector<f64>, PhysicsError> {
        // E_i = F_0i (in +--- signature, index 1,2,3)
        let f_tensor = self.field_strength();

        let data = f_tensor.data();

        let e_vec = if data.len() >= 16 {
            // Extract F_01, F_02, F_03
            let ex = data[1];
            let ey = data[2];
            let ez = data[3];

            let mut v = vec![0.0; 16];
            // Put into indices 2,3,4
            v[2] = ex;
            v[3] = ey;
            v[4] = ez;
            v
        } else {
            vec![0.0; 16]
        };

        CausalMultiVector::new(e_vec, self.metric())
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))
    }

    fn magnetic_field(&self) -> Result<CausalMultiVector<f64>, PhysicsError> {
        // B_x = F_23 = index 11
        // B_y = F_31 = -F_13 = -index 7
        // B_z = F_12 = index 6

        let f_tensor = self.field_strength();
        let data = f_tensor.data();

        let b_vec = if data.len() >= 16 {
            let bx = data[11];
            let by = -data[7];
            let bz = data[6];

            let mut v = vec![0.0; 16];
            // Put into indices 2,3,4
            v[2] = bx;
            v[3] = by;
            v[4] = bz;
            v
        } else {
            vec![0.0; 16]
        };

        CausalMultiVector::new(b_vec, self.metric())
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))
    }

    fn energy_density(&self) -> Result<f64, PhysicsError> {
        energy_density_kernel(&self.electric_field()?, &self.magnetic_field()?)
    }

    fn lagrangian_density(&self) -> Result<f64, PhysicsError> {
        lagrangian_density_kernel(&self.electric_field()?, &self.magnetic_field()?)
    }

    fn poynting_vector(&self) -> Result<CausalMultiVector<f64>, PhysicsError> {
        poynting_vector_kernel(&self.electric_field()?, &self.magnetic_field()?)
    }

    fn lorentz_force(
        &self,
        current_density: &CausalMultiVector<f64>,
    ) -> Result<CausalMultiVector<f64>, PhysicsError> {
        lorentz_force_kernel(current_density, &self.magnetic_field()?)
    }

    fn field_invariant(&self) -> Result<f64, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        let e_sq = e.squared_magnitude();
        let b_sq = b.squared_magnitude();

        if !e_sq.is_finite() || !b_sq.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite invariant".into(),
            ));
        }

        // F_uv F^uv = 2(B^2 - E^2)
        Ok(2.0 * (b_sq - e_sq))
    }

    fn dual_invariant(&self) -> Result<f64, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;
        let inner = e.inner_product(&b);
        let e_dot_b = inner.data().first().copied().unwrap_or(0.0);
        Ok(-4.0 * e_dot_b)
    }

    fn is_radiation_field(&self) -> Result<bool, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;
        let inner = e.inner_product(&b);
        Ok(inner.data().first().copied().unwrap_or(0.0).abs() < 1e-10)
    }

    fn is_null_field(&self) -> Result<bool, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;
        let e_sq = e.squared_magnitude();
        let b_sq = b.squared_magnitude();
        Ok((e_sq - b_sq).abs() < 1e-10 * (e_sq + b_sq).max(1.0))
    }

    fn momentum_density(&self) -> Result<CausalMultiVector<f64>, PhysicsError> {
        self.poynting_vector()
    }

    fn intensity(&self) -> Result<f64, PhysicsError> {
        let s = self.poynting_vector()?;
        Ok(s.squared_magnitude().abs().sqrt())
    }
}
