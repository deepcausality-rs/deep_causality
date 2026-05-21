/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::traits::chain_complex::ChainComplex;
use crate::types::manifold::differential::utils_differential;
use crate::{Manifold, SimplicialComplex};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

impl<R> Manifold<SimplicialComplex<R>, R>
where
    R: RealField + FromPrimitive + Default + PartialEq,
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
    pub fn codifferential(&self, k: usize) -> CausalTensor<R> {
        if k == 0 {
            return CausalTensor::new(vec![], vec![0]).unwrap();
        }

        let k_form_data = self.get_k_form_data(k);
        let mass_k = &self.complex.hodge_star_operators[k];

        let boundary_k_cow = self.complex.boundary_matrix(k);
        let boundary_k: &deep_causality_sparse::CsrMatrix<i8> = &boundary_k_cow;

        let mass_k_minus_1 = &self.complex.hodge_star_operators[k - 1];

        let weighted_form = utils_differential::apply_metric_operator(mass_k, &k_form_data);
        let integrated_form = utils_differential::apply_operator(boundary_k, &weighted_form);

        let prev_dim_size = self.complex.skeletons()[k - 1].simplices().len();
        let mut result_data = Vec::with_capacity(prev_dim_size);

        let zero_tol = <R as FromPrimitive>::from_f64(1e-12)
            .expect("1e-12 is representable in every RealField");

        for i in 0..prev_dim_size {
            let numerator = integrated_form.get(i).copied().unwrap_or(R::zero());

            let mut mass_val = R::zero();
            let start = mass_k_minus_1.row_indices()[i];
            let end = mass_k_minus_1.row_indices()[i + 1];

            for idx in start..end {
                if mass_k_minus_1.col_indices()[idx] == i {
                    mass_val = mass_k_minus_1.values()[idx];
                    break;
                }
            }

            if mass_val.abs() > zero_tol {
                result_data.push(numerator * (R::one() / mass_val));
            } else {
                result_data.push(R::zero());
            }
        }

        CausalTensor::new(result_data, vec![prev_dim_size]).unwrap()
    }
}
