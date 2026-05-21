/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Manifold, SimplicialComplex};

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

impl<R> Manifold<SimplicialComplex<R>, R>
where
    R: RealField + FromPrimitive + Default + PartialEq + std::fmt::Debug,
{
    /// Computes the Hodge-Laplacian operator `Δ` on a k-form.
    ///
    /// The Laplacian is defined as: Δ = dδ + δd
    /// where `d` is the exterior derivative and `δ` is the codifferential.
    /// It maps k-forms to k-forms.
    pub fn laplacian(&self, k: usize) -> CausalTensor<R> {
        let n = self.complex.max_simplex_dimension();
        let current_dim_size = self.complex.skeletons()[k].simplices().len();

        let term_a = if k > 0 {
            let delta = self.codifferential(k);
            let temp_manifold = self.create_temp_manifold(k - 1, delta);
            temp_manifold.exterior_derivative(k - 1)
        } else {
            CausalTensor::new(vec![R::zero(); current_dim_size], vec![current_dim_size]).unwrap()
        };

        let term_b = if k < n {
            let d = self.exterior_derivative(k);
            let temp_manifold = self.create_temp_manifold(k + 1, d);
            temp_manifold.codifferential(k + 1)
        } else {
            CausalTensor::new(vec![R::zero(); current_dim_size], vec![current_dim_size]).unwrap()
        };

        let mut result_data = Vec::with_capacity(current_dim_size);
        let slice_a = term_a.as_slice();
        let slice_b = term_b.as_slice();

        let len = slice_a.len().max(slice_b.len());

        for i in 0..len {
            let a = slice_a.get(i).copied().unwrap_or(R::zero());
            let b = slice_b.get(i).copied().unwrap_or(R::zero());
            result_data.push(a + b);
        }

        CausalTensor::new(result_data, vec![current_dim_size]).unwrap()
    }
}
