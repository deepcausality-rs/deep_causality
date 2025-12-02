/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::manifold::Manifold;
use deep_causality_num::Field;
use deep_causality_tensor::CausalTensor;

impl<T> Manifold<T>
where
    T: Field + Copy + std::ops::Neg<Output = T>,
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
    pub fn exterior_derivative(&self, k: usize) -> CausalTensor<T> {
        // For a discrete simplicial complex, the exterior derivative is
        // represented by the coboundary operator (transpose of boundary operator)

        // Check if we have a coboundary operator for this dimension
        if k >= self.complex.coboundary_operators.len() {
            // If k is at max dimension, derivative is zero
            return CausalTensor::new(vec![], vec![0]).expect("Failed to create empty tensor");
        }

        let coboundary = &self.complex.coboundary_operators[k];

        // Extract the k-forms from data
        // For simplicity, assume data is ordered by skeleton (0-simplices, 1-simplices, ...)
        let mut offset = 0;
        for i in 0..k {
            if i < self.complex.skeletons.len() {
                offset += self.complex.skeletons[i].simplices.len();
            }
        }

        let k_form_size = if k < self.complex.skeletons.len() {
            self.complex.skeletons[k].simplices.len()
        } else {
            0
        };

        // Extract k-form coefficients
        let k_form_data: Vec<T> = if offset + k_form_size <= self.data.len() {
            self.data.as_slice()[offset..offset + k_form_size].to_vec()
        } else {
            vec![]
        };

        // Apply coboundary operator
        // coboundary * k_form_data gives (k+1)-form
        let result = super::utils::apply_operator(coboundary, &k_form_data);
        let result_len = result.len();

        CausalTensor::new(result, vec![result_len]).expect("Failed to create result tensor")
    }
}
