/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::manifold::differential::utils_differential;
use crate::{Manifold, SimplicialComplex};
use core::fmt::Debug;
use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;

impl<R> Manifold<SimplicialComplex<R>, R>
where
    R: RealField + Default + PartialEq + Debug,
{
    /// Computes the Hodge star operator on a k-form.
    ///
    /// The Hodge star `⋆` maps k-forms to (n-k)-forms, where n is the dimension of the manifold.
    /// This implementation uses a pre-computed sparse matrix operator that represents a
    /// diagonal Hodge star.
    pub fn hodge_star(&self, k: usize) -> CausalTensor<R> {
        let n = self.complex.max_simplex_dimension();

        if k > n {
            return CausalTensor::new(vec![], vec![0]).expect("Failed to create empty tensor");
        }

        let hodge_operator = &self.complex.hodge_star_operators[k];

        let mut offset = 0;
        for i in 0..k {
            offset += self.complex.skeletons()[i].simplices().len();
        }

        let k_skeleton = &self.complex.skeletons()[k];
        let k_count = k_skeleton.simplices().len();
        let k_form_data = &self.data().as_slice()[offset..offset + k_count];

        let result_data = utils_differential::apply_metric_operator(hodge_operator, k_form_data);
        let result_len = result_data.len();

        CausalTensor::new(result_data, vec![result_len]).expect("Failed to create dual form")
    }
}
