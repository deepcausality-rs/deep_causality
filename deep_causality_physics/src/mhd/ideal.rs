/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::mhd::quantities::{AlfvenSpeed, MagneticPressure};
use crate::{Density, PhysicalField, PhysicsError, PhysicsErrorEnum};
use deep_causality_multivector::MultiVector;
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, SimplicialComplex};
use std::collections::HashMap;

/// Calculates the characteristic speed of Alfven waves.
/// $$ v_A = \frac{B}{\sqrt{\mu_0 \rho}} $$
///
/// # Arguments
/// *   `b_field` - Magnetic field $B$ (Uses magnitude $|B|$).
/// *   `density` - Plasma density $\rho$.
/// *   `permeability` - Magnetic permeability $\mu_0$.
///
/// # Returns
/// *   `Result<AlfvenSpeed, PhysicsError>` - Alfven speed $v_A$.
pub fn alfven_speed_kernel(
    b_field: &PhysicalField,
    density: &Density,
    permeability: f64,
) -> Result<AlfvenSpeed, PhysicsError> {
    let b_mag = b_field.inner().squared_magnitude().sqrt();
    let rho = density.value();

    if permeability <= 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("Permeability must be positive".into()),
        ));
    }

    if rho < 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("Density cannot be negative".into()),
        ));
    }

    if rho == 0.0 {
        return Err(PhysicsError::new(PhysicsErrorEnum::Singularity(
            "Zero density in Alfven speed".into(),
        )));
    }

    let denom = (permeability * rho).sqrt();
    let va = b_mag / denom;

    AlfvenSpeed::new(va)
}

/// Calculates magnetic pressure.
/// $$ P_B = \frac{B^2}{2\mu_0} $$
///
/// # Arguments
/// *   `b_field` - Magnetic field $B$.
/// *   `permeability` - Magnetic permeability $\mu_0$.
///
/// # Returns
/// *   `Result<MagneticPressure, PhysicsError>` - Magnetic pressure $P_B$.
pub fn magnetic_pressure_kernel(
    b_field: &PhysicalField,
    permeability: f64,
) -> Result<MagneticPressure, PhysicsError> {
    let b_sq = b_field.inner().squared_magnitude();

    if permeability <= 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("Permeability must be positive".into()),
        ));
    }

    let pb = b_sq / (2.0 * permeability);
    MagneticPressure::new(pb)
}

/// Calculates the time evolution of the magnetic field (Frozen-in flux).
/// $$ \frac{\partial \mathbf{B}}{\partial t} = \nabla \times (\mathbf{v} \times \mathbf{B}) $$
///
/// **Geometric Algebra Implementation**:
/// In the language of differential forms/GA on a Manifold:
/// $$ \partial_t B = -d(i_v B) $$
/// where $B$ is a 2-form (flux), $v$ is a vector field (represented as a 1-form),
/// $i_v$ is interior product (contraction), and $d$ is exterior derivative.
///
/// This implementation relies on the identity:
/// $$ i_v B = \star (v \wedge \star B) $$
/// (valid for 3D manifolds where $v$ and $\star B$ are 1-forms).
///
/// # Arguments
/// *   `v_manifold` - Manifold containing the velocity field $v$ (1-form).
/// *   `b_manifold` - Manifold containing the magnetic flux 2-form $B$.
///
/// # Returns
/// *   `Result<CausalTensor<f64>, PhysicsError>` - Rate of change of B (2-form), i.e., $-\partial_t B$.
///     Wait, the equation is $\partial_t B = \dots$. The function returns $\partial_t B$.
pub fn ideal_induction_kernel(
    v_manifold: &Manifold<f64>,
    b_manifold: &Manifold<f64>,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // 1. Validation
    let complex = v_manifold.complex();
    let skeletons = complex.skeletons();

    // Need at least 0, 1, 2 skeletons
    if skeletons.len() < 3 {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "Manifold must be at least 2D (preferably 3D) for induction".into(),
        )));
    }

    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();

    // Verify data lengths (Manifold enforces this on creation, but checks are cheap)
    if v_manifold.data().len() < n0 + n1 + n2 {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "v_manifold data too small".into(),
        )));
    }

    // 2. Extract Data Slices
    // v is 1-form: offset = n0, len = n1
    let v_offset = n0;
    let v_slice = &v_manifold.data().as_slice()[v_offset..v_offset + n1];

    // B is 2-form: offset = n0 + n1, len = n2
    let b_offset = n0 + n1;
    let b_slice = &b_manifold.data().as_slice()[b_offset..b_offset + n2];

    // 3. Compute Hodge Star of B (star_b)
    // star_b: 2-form -> 1-form (in 3D)
    // Using hodge_star_operators[2]
    if complex.hodge_star_operators().len() <= 2 {
        return Err(PhysicsError::new(PhysicsErrorEnum::CalculationError(
            "Hodge star operator for 2-forms not available".into(),
        )));
    }
    let h_star_2 = &complex.hodge_star_operators()[2];
    let star_b_data = apply_csr_f64(h_star_2, b_slice);

    // 4. Compute Wedge Product: v ^ star_b
    // v: 1-form, star_b: 1-form -> Result: 2-form
    let wedge_data = wedge_product_1form_1form(v_slice, &star_b_data, complex)?;

    // 5. Compute Interior Product proxy: iv_b = star(v ^ star_b)
    // wedge_data is 2-form. star maps to 1-form.
    let iv_b_data = apply_csr_f64(h_star_2, &wedge_data);

    // 6. Compute Exterior Derivative: d(iv_b)
    // iv_b is 1-form. d maps to 2-form.
    // Use coboundary_operators[1].
    if complex.coboundary_operators().len() <= 1 {
        return Err(PhysicsError::new(PhysicsErrorEnum::CalculationError(
            "Coboundary operator for 1-forms not available".into(),
        )));
    }
    let d_1 = &complex.coboundary_operators()[1];
    let dt_b_neg_data = apply_csr_i8_f64(d_1, &iv_b_data);

    // 7. Result
    // Returns the 2-form part of the change.
    let result_len = dt_b_neg_data.len();
    CausalTensor::new(dt_b_neg_data, vec![result_len]).map_err(PhysicsError::from)
}

// --- Helper Functions ---

/// Multiplies a CsrMatrix<f64> by a dense vector &[f64].
fn apply_csr_f64(matrix: &CsrMatrix<f64>, vector: &[f64]) -> Vec<f64> {
    let (rows, _cols) = matrix.shape();
    let mut result = vec![0.0; rows];

    // Using public getters
    let row_indices = matrix.row_indices();
    let col_indices = matrix.col_indices();
    let values = matrix.values();

    for i in 0..rows {
        let start = row_indices[i];
        let end = row_indices[i + 1];
        let mut sum = 0.0;
        for idx in start..end {
            let col = col_indices[idx];
            let val = values[idx];
            if col < vector.len() {
                sum += val * vector[col];
            }
        }
        result[i] = sum;
    }
    result
}

/// Multiplies a CsrMatrix<i8> by a dense vector &[f64].
fn apply_csr_i8_f64(matrix: &CsrMatrix<i8>, vector: &[f64]) -> Vec<f64> {
    let (rows, _cols) = matrix.shape();
    let mut result = vec![0.0; rows];

    let row_indices = matrix.row_indices();
    let col_indices = matrix.col_indices();
    let values = matrix.values();

    for i in 0..rows {
        let start = row_indices[i];
        let end = row_indices[i + 1];
        let mut sum = 0.0;
        for idx in start..end {
            let col = col_indices[idx];
            let val = values[idx]; // i8
            if col < vector.len() {
                sum += (val as f64) * vector[col];
            }
        }
        result[i] = sum;
    }
    result
}

/// Computes the Wedge Product of two 1-forms on a Simplicial Complex.
/// Result is a 2-form.
///
/// Formula used (Cup Product):
/// $(\alpha \cup \beta)([0,1,2]) = \alpha([0,1]) \cdot \beta([1,2])$
/// Wedge Product $\alpha \wedge \beta = \alpha \cup \beta - \beta \cup \alpha$.
fn wedge_product_1form_1form(
    alpha: &[f64],
    beta: &[f64],
    complex: &SimplicialComplex,
) -> Result<Vec<f64>, PhysicsError> {
    let skeletons = complex.skeletons();
    if skeletons.len() < 3 {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "Complex must have 2-simplices".into(),
        )));
    }
    let edges = skeletons[1].simplices();
    let faces = skeletons[2].simplices();

    // Build Edge Lookup Map: (min(u,v), max(u,v)) -> edge_index
    // This is O(E)
    let mut edge_map = HashMap::with_capacity(edges.len());
    for (idx, edge_simplex) in edges.iter().enumerate() {
        let verts = edge_simplex.vertices();
        if verts.len() >= 2 {
            edge_map.insert((verts[0], verts[1]), idx);
        }
    }

    let mut result = Vec::with_capacity(faces.len());

    // Iterate over faces (2-simplices)
    for face in faces {
        let verts = face.vertices(); // Sorted [v0, v1, v2]
        if verts.len() != 3 {
            // Non-triangular face? Skip or zero.
            result.push(0.0);
            continue;
        }
        let v0 = verts[0];
        let v1 = verts[1];
        let v2 = verts[2];

        // Edges for Cup Product
        // [v0, v1] and [v1, v2]
        let e01_idx = edge_map.get(&(v0, v1));
        let e12_idx = edge_map.get(&(v1, v2));

        // Check if edges exist
        if let (Some(&idx01), Some(&idx12)) = (e01_idx, e12_idx) {
            let val_alpha_01 = alpha.get(idx01).unwrap_or(&0.0);
            let val_beta_12 = beta.get(idx12).unwrap_or(&0.0);
            let val_beta_01 = beta.get(idx01).unwrap_or(&0.0);
            let val_alpha_12 = alpha.get(idx12).unwrap_or(&0.0);

            // \alpha \wedge \beta = \alpha \cup \beta - \beta \cup \alpha
            let term1 = val_alpha_01 * val_beta_12;
            let term2 = val_beta_01 * val_alpha_12;

            result.push(term1 - term2);
        } else {
            result.push(0.0);
        }
    }

    Ok(result)
}
