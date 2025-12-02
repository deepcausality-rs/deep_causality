/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Manifold;
use crate::SimplicialTopology;
use core::ops::Mul;
use deep_causality_num::{Field, FromPrimitive};
use deep_causality_tensor::CausalTensor;

impl<T> Manifold<T>
where
    T: Field
        + Copy
        + FromPrimitive
        + Mul<f64, Output = T>
        + std::ops::Neg<Output = T>
        + std::fmt::Debug,
{
    /// Computes the Hodge star operator on a k-form.
    ///
    /// The Hodge star `⋆` maps k-forms to (n-k)-forms, where n is the dimension of the manifold.
    /// This implementation uses a pre-computed sparse matrix operator that represents a
    /// diagonal Hodge star.
    ///
    /// # Arguments
    /// * `k` - The degree of the form.
    ///
    /// # Returns
    /// A new `CausalTensor` representing the Hodge dual (n-k)-form.
    pub fn hodge_star(&self, k: usize) -> CausalTensor<T> {
        // Get dimension of manifold (highest skeleton dimension)
        let n = self.complex.max_simplex_dimension();

        if k > n {
            // Cannot compute Hodge star if k is out of bounds. The space of forms is trivial.
            return CausalTensor::new(vec![], vec![0]).expect("Failed to create empty tensor");
        }

        // Get the pre-computed Hodge star operator
        let hodge_operator = &self.complex.hodge_star_operators[k];

        // Extract the k-form coefficients from the manifold's flat data tensor.
        let mut offset = 0;
        for i in 0..k {
            offset += self.complex.skeletons()[i].simplices().len();
        }

        let k_skeleton = &self.complex.skeletons()[k];
        let k_count = k_skeleton.simplices().len();
        let k_form_data = &self.data().as_slice()[offset..offset + k_count];

        // Apply the Hodge star operator
        let result_data = super::utils::apply_f64_operator(hodge_operator, k_form_data);
        let result_len = result_data.len();

        CausalTensor::new(result_data, vec![result_len]).expect("Failed to create dual form")
    }
}
