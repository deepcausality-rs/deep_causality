/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::manifold::Manifold;
use crate::types::manifold::differential::utils_differential;
use core::fmt::Debug;
use deep_causality_num::Field;
use deep_causality_tensor::CausalTensor;

impl<C, D> Manifold<C, D>
where
    C: Default + Copy + PartialEq + deep_causality_num::Zero,
    D: Field + Copy + Default + PartialEq + core::ops::Neg<Output = D> + Debug,
{
    /// Computes the exterior derivative of a k-form.
    ///
    /// The exterior derivative `d` maps k-forms to (k+1)-forms.
    /// For a discrete simplicial complex, this is represented by the boundary operator.
    ///
    /// # Arguments
    /// * `k` - The degree of the form (0 for scalar fields, 1 for 1-forms, etc.)
    ///
    /// # Returns
    /// A new tensor representing the (k+1)-form `df`.
    ///
    /// # Mathematical Background
    /// The exterior derivative satisfies:
    /// - Linearity: `d(αf + βg) = α df + β dg`
    /// - Nilpotency: `d² = 0` (applying d twice gives zero)
    /// - Leibniz rule: `d(f ∧ g) = df ∧ g + (-1)^k f ∧ dg`
    ///
    /// For discrete differential forms on simplicial complexes,
    /// the exterior derivative is represented by the coboundary operator.
    pub fn exterior_derivative(&self, k: usize) -> CausalTensor<D> {
        // The exterior derivative d_k is represented by the coboundary operator C_k = B_{k+1}^T.
        if k >= self.complex.coboundary_operators.len() {
            // d of the highest dimension is zero
            return CausalTensor::new(vec![], vec![0]).expect("Tensor alloc failed");
        }

        let coboundary = &self.complex.coboundary_operators[k];
        let k_form_data = self.get_k_form_data(k);

        // Operation: C_k * omega_k
        let result = utils_differential::apply_operator(coboundary, &k_form_data);

        // Result size matches the number of (k+1)-simplices
        let next_dim_size = self.complex.skeletons()[k + 1].simplices().len();

        // Safety check
        if result.len() != next_dim_size {
            // This handles the case where the sparse matrix might have implicit dimensions
            // essentially padding/truncating to the correct skeleton size.
            let mut corrected = result;
            corrected.resize(next_dim_size, D::zero());
            return CausalTensor::new(corrected, vec![next_dim_size]).unwrap();
        }

        CausalTensor::new(result, vec![next_dim_size]).expect("Tensor alloc failed")
    }
}
