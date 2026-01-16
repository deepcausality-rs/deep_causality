/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::GaugeEmOps;
use crate::error::PhysicsError;
use deep_causality_metric::{LorentzianMetric, WestCoastMetric};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, GaugeField, GaugeFieldWitness, Manifold, Simplex, SimplicialComplexBuilder, U1,
};

/// Blanket implementation of GaugeEmOps for GaugeField<U1, S, S> where S: Field + Float + TensorData
impl<S> GaugeEmOps<S> for GaugeField<U1, S, S>
where
    S: RealField + From<f64> + Into<f64> + std::default::Default,
{
    fn from_fields(
        base: Manifold<S, S>,
        electric_field: CausalMultiVector<S>,
        magnetic_field: CausalMultiVector<S>,
    ) -> Result<Self, PhysicsError> {
        if electric_field.metric() != magnetic_field.metric() {
            return Err(PhysicsError::DimensionMismatch(format!(
                "E and B field metrics must match: {:?} vs {:?}",
                electric_field.metric(),
                magnetic_field.metric()
            )));
        }

        let metric = electric_field.metric();
        let dim = 4;
        let num_points = base.len();

        let connection = CausalTensor::zeros(&[num_points, dim, 1]);

        // Populate Field Strength Tensor F_mn
        let mut f_data: Vec<S> = vec![S::zero(); num_points * 16];
        let e_data = electric_field.data();
        let b_data = magnetic_field.data();

        for i in 0..num_points {
            let offset = i * 16;

            let ex = e_data.get(offset + 2).cloned().unwrap_or_else(S::zero);
            let ey = e_data.get(offset + 3).cloned().unwrap_or_else(S::zero);
            let ez = e_data.get(offset + 4).cloned().unwrap_or_else(S::zero);

            let bx = b_data.get(offset + 2).cloned().unwrap_or_else(S::zero);
            let by = b_data.get(offset + 3).cloned().unwrap_or_else(S::zero);
            let bz = b_data.get(offset + 4).cloned().unwrap_or_else(S::zero);

            // F_01 = E_x
            f_data[offset + 1] = ex;
            f_data[offset + 4] = -ex; // F_10

            // F_02 = E_y
            f_data[offset + 2] = ey;
            f_data[offset + 8] = -ey; // F_20

            // F_03 = E_z
            f_data[offset + 3] = ez;
            f_data[offset + 12] = -ez; // F_30

            // F_ij = -epsilon_ijk B_k
            f_data[offset + 11] = bx;
            f_data[offset + 14] = -bx; // F_32

            f_data[offset + 13] = by;
            f_data[offset + 7] = -by; // F_13

            f_data[offset + 6] = bz;
            f_data[offset + 9] = -bz; // F_21
        }

        let field_strength =
            CausalTensor::new(f_data, vec![num_points, dim, dim, 1]).map_err(|e| {
                PhysicsError::DimensionMismatch(format!("Failed to create F tensor: {:?}", e))
            })?;

        GaugeField::new(base, metric, connection, field_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn from_components(ex: S, ey: S, ez: S, bx: S, by: S, bz: S) -> Result<Self, PhysicsError> {
        let metric = WestCoastMetric::minkowski_4d().into_metric();

        // Create minimal manifold (1 point)
        let mut builder = SimplicialComplexBuilder::new(0);
        let _ = builder.add_simplex(Simplex::new(vec![0]));
        let complex = builder.build().map_err(|e| {
            PhysicsError::DimensionMismatch(format!("Failed to build complex: {:?}", e))
        })?;

        let data = CausalTensor::new(vec![S::zero()], vec![1]).map_err(|e| {
            PhysicsError::DimensionMismatch(format!("Failed to create tensor: {:?}", e))
        })?;

        let base: Manifold<S, S> = Manifold::new(complex, data, 0).map_err(|e| {
            PhysicsError::DimensionMismatch(format!("Failed to create default manifold: {:?}", e))
        })?;

        let num_points = 1;
        let dim = 4;
        let connection = CausalTensor::zeros(&[num_points, dim, 1]);

        let e_vec = [ex, ey, ez];
        let b_vec = [bx, by, bz];
        // Use the topology method from the GaugeField to construct field strength from E/B vectors
        let field_strength =
            GaugeFieldWitness::<S>::field_strength_from_eb_vectors(&e_vec, &b_vec, num_points);

        GaugeField::new(base, metric, connection, field_strength)
            .map_err(|e| PhysicsError::TopologyError(e.to_string()))
    }

    fn plane_wave(amplitude: S, polarization: usize) -> Result<Self, PhysicsError> {
        let amp_f64: f64 = amplitude.into();
        if !amp_f64.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Amplitude must be finite".into(),
            ));
        }

        let zero = S::zero();
        match polarization {
            0 => Self::from_components(amplitude, zero, zero, zero, amplitude, zero),
            1 => Self::from_components(zero, amplitude, zero, zero, zero, amplitude),
            _ => Err(PhysicsError::DimensionMismatch(
                "Polarization must be 0 or 1".into(),
            )),
        }
    }

    fn electric_field(&self) -> Result<CausalMultiVector<S>, PhysicsError> {
        let f_tensor = self.field_strength();
        let data = f_tensor.data();

        let e_vec: Vec<S> = if data.len() >= 16 {
            let ex = data[1];
            let ey = data[2];
            let ez = data[3];

            let mut v: Vec<S> = vec![S::zero(); 16];
            v[2] = ex;
            v[3] = ey;
            v[4] = ez;
            v
        } else {
            vec![S::zero(); 16]
        };

        CausalMultiVector::new(e_vec, self.metric())
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))
    }

    fn magnetic_field(&self) -> Result<CausalMultiVector<S>, PhysicsError> {
        let f_tensor = self.field_strength();
        let data = f_tensor.data();

        let b_vec: Vec<S> = if data.len() >= 16 {
            let bx = data[11];
            let by = -data[7];
            let bz = data[6];

            let mut v: Vec<S> = vec![S::zero(); 16];
            v[2] = bx;
            v[3] = by;
            v[4] = bz;
            v
        } else {
            vec![S::zero(); 16]
        };

        CausalMultiVector::new(b_vec, self.metric())
            .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))
    }

    fn energy_density(&self) -> Result<S, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        // u = (E² + B²) / 2
        let e_sq = squared_magnitude_3d(&e);
        let b_sq = squared_magnitude_3d(&b);

        let half: S = <S as From<f64>>::from(0.5);
        Ok(half * (e_sq + b_sq))
    }

    fn lagrangian_density(&self) -> Result<S, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        // L = (E² - B²) / 2
        let e_sq = squared_magnitude_3d(&e);
        let b_sq = squared_magnitude_3d(&b);

        let half: S = <S as From<f64>>::from(0.5);
        Ok(half * (e_sq - b_sq))
    }

    fn poynting_vector(&self) -> Result<CausalMultiVector<S>, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        // S = E × B (cross product)
        cross_product_3d(&e, &b)
    }

    fn lorentz_force(
        &self,
        current_density: &CausalMultiVector<S>,
    ) -> Result<CausalMultiVector<S>, PhysicsError> {
        let b = self.magnetic_field()?;
        // f = J × B (simplified, ignoring charge density term)
        cross_product_3d(current_density, &b)
    }

    fn field_invariant(&self) -> Result<S, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        let e_sq = squared_magnitude_3d(&e);
        let b_sq = squared_magnitude_3d(&b);

        // F_uv F^uv = 2(B² - E²)
        let two: S = <S as From<f64>>::from(2.0);
        Ok(two * (b_sq - e_sq))
    }

    fn dual_invariant(&self) -> Result<S, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        let e_dot_b = dot_product_3d(&e, &b);
        let four: S = <S as From<f64>>::from(4.0);
        Ok(-four * e_dot_b)
    }

    fn is_radiation_field(&self) -> Result<bool, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        let e_dot_b = dot_product_3d(&e, &b);
        let abs_val: f64 = e_dot_b.abs().into();
        Ok(abs_val < 1e-10)
    }

    fn is_null_field(&self) -> Result<bool, PhysicsError> {
        let e = self.electric_field()?;
        let b = self.magnetic_field()?;

        let e_sq = squared_magnitude_3d(&e);
        let b_sq = squared_magnitude_3d(&b);

        let diff: f64 = (e_sq - b_sq).abs().into();
        let sum: f64 = (e_sq + b_sq).into();
        let threshold = 1e-10 * sum.max(1.0);
        Ok(diff < threshold)
    }

    fn momentum_density(&self) -> Result<CausalMultiVector<S>, PhysicsError> {
        self.poynting_vector()
    }

    fn intensity(&self) -> Result<S, PhysicsError> {
        let s = self.poynting_vector()?;
        Ok(magnitude_3d(&s))
    }

    fn computed_field_strength(&self) -> Result<CausalTensor<S>, PhysicsError> {
        // Return the stored field strength tensor
        Ok(self.field_strength().clone())
    }
}

// =============================================================================
// Helper functions for 3D vector operations
// =============================================================================

/// Computes the squared magnitude of a 3D vector (indices 2, 3, 4)
fn squared_magnitude_3d<S>(mv: &CausalMultiVector<S>) -> S
where
    S: RealField + Clone + From<f64> + Default,
{
    let data = mv.data();
    let x = data.get(2).cloned().unwrap_or_else(S::zero);
    let y = data.get(3).cloned().unwrap_or_else(S::zero);
    let z = data.get(4).cloned().unwrap_or_else(S::zero);

    x * x + y * y + z * z
}

/// Computes the magnitude of a 3D vector
fn magnitude_3d<S>(mv: &CausalMultiVector<S>) -> S
where
    S: RealField + Clone + From<f64> + Default,
{
    squared_magnitude_3d(mv).sqrt()
}

/// Computes the dot product of two 3D vectors
fn dot_product_3d<S>(a: &CausalMultiVector<S>, b: &CausalMultiVector<S>) -> S
where
    S: RealField + Clone + From<f64> + Default,
{
    let a_data = a.data();
    let b_data = b.data();

    let ax = a_data.get(2).cloned().unwrap_or_else(S::zero);
    let ay = a_data.get(3).cloned().unwrap_or_else(S::zero);
    let az = a_data.get(4).cloned().unwrap_or_else(S::zero);

    let bx = b_data.get(2).cloned().unwrap_or_else(S::zero);
    let by = b_data.get(3).cloned().unwrap_or_else(S::zero);
    let bz = b_data.get(4).cloned().unwrap_or_else(S::zero);

    ax * bx + ay * by + az * bz
}

/// Computes the cross product of two 3D vectors
fn cross_product_3d<S>(
    a: &CausalMultiVector<S>,
    b: &CausalMultiVector<S>,
) -> Result<CausalMultiVector<S>, PhysicsError>
where
    S: RealField + Clone + From<f64> + Default,
{
    let a_data = a.data();
    let b_data = b.data();

    let ax = a_data.get(2).cloned().unwrap_or_else(S::zero);
    let ay = a_data.get(3).cloned().unwrap_or_else(S::zero);
    let az = a_data.get(4).cloned().unwrap_or_else(S::zero);

    let bx = b_data.get(2).cloned().unwrap_or_else(S::zero);
    let by = b_data.get(3).cloned().unwrap_or_else(S::zero);
    let bz = b_data.get(4).cloned().unwrap_or_else(S::zero);

    // c = a × b
    let cx = ay * bz - az * by;
    let cy = az * bx - ax * bz;
    let cz = ax * by - ay * bx;

    let mut result: Vec<S> = vec![S::zero(); 16];
    result[2] = cx;
    result[3] = cy;
    result[4] = cz;

    CausalMultiVector::new(result, a.metric())
        .map_err(|e| PhysicsError::DimensionMismatch(e.to_string()))
}
