/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Manifold;
use deep_causality_num::{Field, FromPrimitive, Zero};
use deep_causality_tensor::CausalTensor;
use std::fmt::Debug;
use std::ops::Mul;

impl<T> Manifold<T>
where
    T: Field
        + Copy
        + FromPrimitive
        + Mul<f64, Output = T>
        + core::ops::Neg<Output = T>
        + Default
        + PartialEq
        + Zero
        + Debug,
{
    /// Computes the codifferential `δ` (delta) of a k-form.
    ///
    /// The codifferential is the adjoint of `d` with respect to the inner product defined by the Mass Matrices (Hodge Stars).
    ///
    /// Formula: δ_k = M_{k-1}^{-1} * B_k * M_k
    ///
    /// * `M_k`: Mass Matrix for k-forms (Diagonal, stored in hodge_star_operators).
    /// * `B_k`: Boundary Operator mapping k -> k-1.
    /// * `M_{k-1}^{-1}`: Inverse Mass Matrix for (k-1)-forms.
    ///
    /// # Arguments
    /// * `k` - The degree of the form (must be > 0).
    ///
    /// # Returns
    /// A `CausalTensor` representing the (k-1)-form.
    pub fn codifferential(&self, k: usize) -> CausalTensor<T> {
        if k == 0 {
            // delta of a 0-form is zero
            return CausalTensor::new(vec![], vec![0]).unwrap();
        }

        // 1. Get Data: omega_k
        let k_form_data = self.get_k_form_data(k);

        // 2. Get Operators
        // M_k (Mass Matrix for k-simplices)
        let mass_k = &self.complex.hodge_star_operators[k];

        // B_k (Boundary Operator: k -> k-1)
        // Convention: boundary_operators[k-1] maps k-simplices (cols) to k-1 simplices (rows)
        // This follows the documented convention where boundary_operators[j] = ∂_{j+1}
        let boundary_k = &self.complex.boundary_operators[k - 1];

        // M_{k-1} (Mass Matrix for k-1 simplices)
        // We need the inverse of this. Since we store it as a diagonal CsrMatrix,
        // we can compute the inverse values element-wise.
        let mass_k_minus_1 = &self.complex.hodge_star_operators[k - 1];

        // 3. Compute: y = M_k * omega_k
        // Element-wise multiplication since M_k is diagonal
        let weighted_form = super::utils::apply_f64_operator(mass_k, &k_form_data);

        // 4. Compute: z = B_k * y
        // Sparse matrix multiplication
        let integrated_form = super::utils::apply_operator(boundary_k, &weighted_form);

        // 5. Compute: res = M_{k-1}^{-1} * z
        // Apply inverse weights.
        let prev_dim_size = self.complex.skeletons()[k - 1].simplices().len();
        let mut result_data = Vec::with_capacity(prev_dim_size);

        // We assume mass_k_minus_1 is strictly diagonal and aligned with the skeleton.
        // In CsrMatrix, diagonal means row_indices[i]..row_indices[i+1] contains col=i.
        // We iterate manually to be safe and efficient.

        // Fallback map if matrix is sparse/missing entries
        // In a full implementation, we'd use a dedicated DiagonalMatrix type.
        // Here we parse the CSR structure.
        for i in 0..prev_dim_size {
            let numerator = integrated_form.get(i).copied().unwrap_or(T::zero());

            // Find diagonal value M_{ii}
            let mut mass_val = 0.0;
            let start = mass_k_minus_1.row_indices()[i];
            let end = mass_k_minus_1.row_indices()[i + 1];

            for idx in start..end {
                if mass_k_minus_1.col_indices()[idx] == i {
                    mass_val = mass_k_minus_1.values()[idx];
                    break;
                }
            }

            // Apply Inverse Mass: 1 / M_{ii}
            // If Mass is 0 (degenerate), we effectively zero out the result to avoid NaN.
            if mass_val.abs() > 1e-12 {
                result_data.push(numerator * (1.0 / mass_val));
            } else {
                result_data.push(T::zero());
            }
        }

        CausalTensor::new(result_data, vec![prev_dim_size]).unwrap()
    }
}
