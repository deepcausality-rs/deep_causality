/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::traits::has_hodge_star::HasHodgeStar;
use crate::types::manifold::differential::utils_differential;
use crate::{Manifold, SimplicialComplex};
use core::fmt::Debug;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

impl<R> Manifold<SimplicialComplex<R>, R>
where
    R: RealField + FromPrimitive + Default + PartialEq + Debug,
{
    /// Computes the Hodge star operator on a k-form.
    ///
    /// The Hodge star `⋆` maps k-forms to (n-k)-forms, where n is the dimension of the manifold.
    /// This implementation uses a pre-computed sparse matrix operator that represents a
    /// diagonal Hodge star.
    ///
    /// The matrix is obtained through `HasHodgeStar<R>` when a metric is attached to the
    /// manifold, and otherwise falls back to the simplicial complex's cached operator
    /// table for source compatibility with the pre-R4 API. The trait-routed path and
    /// the cache path return the same matrix, since the simplicial impl
    /// (`ReggeGeometry<R>: HasHodgeStar<R>`) borrows the cache directly. R4.5 removes
    /// the legacy fallback as part of the generic Manifold widening.
    pub fn hodge_star(&self, k: usize) -> CausalTensor<R> {
        let n = self.complex.max_simplex_dimension();

        if k > n {
            return CausalTensor::new(vec![], vec![0]).expect("Failed to create empty tensor");
        }

        let star = match self.metric.as_ref() {
            Some(metric) => metric.hodge_star_matrix(&self.complex, k),
            None => std::borrow::Cow::Borrowed(&self.complex.hodge_star_operators[k]),
        };

        let mut offset = 0;
        for i in 0..k {
            offset += self.complex.skeletons()[i].simplices().len();
        }

        let k_skeleton = &self.complex.skeletons()[k];
        let k_count = k_skeleton.simplices().len();
        let k_form_data = &self.data().as_slice()[offset..offset + k_count];

        let result_data = utils_differential::apply_metric_operator(star.as_ref(), k_form_data);
        let result_len = result_data.len();

        CausalTensor::new(result_data, vec![result_len]).expect("Failed to create dual form")
    }
}
