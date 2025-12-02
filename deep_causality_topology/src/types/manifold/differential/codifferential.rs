/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Manifold, SimplicialTopology};
use core::ops::Mul;
use deep_causality_num::{Field, FromPrimitive, Zero};
use deep_causality_tensor::CausalTensor;

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
        + std::fmt::Debug,
{
    /// Computes the codifferential `δ` of a k-form.
    ///
    /// The codifferential is the adjoint of the exterior derivative `d` with respect
    /// to the L2 inner product, and is defined as:
    ///
    /// δ = (-1)^(n(k-1)+1) * ⋆d⋆
    ///
    /// where `n` is the dimension of the manifold and `k` is the degree of the form.
    /// It maps k-forms to (k-1)-forms.
    pub fn codifferential(&self, k: usize) -> CausalTensor<T> {
        if k == 0 {
            // Codifferential of a 0-form is always the zero (-1)-form (empty tensor).
            return CausalTensor::new(vec![], vec![0]).unwrap();
        }

        let n = self.complex.max_simplex_dimension();

        // If k > n, the space of k-forms is trivial, return empty (k-1)-form
        if k > n {
            return CausalTensor::new(vec![], vec![0]).unwrap();
        }

        // 1. Apply first Hodge star: k-form -> (n-k)-form
        let star_form = self.hodge_star(k);

        // 2. Apply exterior derivative: (n-k)-form -> (n-k+1)-form
        // We need a temporary manifold to do this.
        let mut temp_data = vec![T::zero(); self.data().len()];

        // Calculate offset for (n-k)-skeleton
        let dual_k = n.saturating_sub(k);
        let mut offset = 0;
        for i in 0..dual_k {
            if i >= self.complex.skeletons().len() {
                break;
            }
            offset += self.complex.skeletons()[i].simplices().len();
        }

        // Check that we have the skeleton we need
        if dual_k >= self.complex.skeletons().len() {
            // No such skeleton, return empty
            let result_size = if k > 0 && k.saturating_sub(1) < self.complex.skeletons().len() {
                self.complex.skeletons()[k.saturating_sub(1)]
                    .simplices()
                    .len()
            } else {
                0
            };
            return CausalTensor::new(vec![T::zero(); result_size], vec![result_size]).unwrap();
        }

        let dual_form_size = self.complex.skeletons()[dual_k].simplices().len();
        if offset + dual_form_size <= temp_data.len() {
            temp_data[offset..offset + dual_form_size].copy_from_slice(star_form.as_slice());
        }

        let temp_manifold = Manifold::new(
            self.complex.clone(),
            CausalTensor::new(temp_data, vec![self.data().len()]).unwrap(),
            0,
        )
        .unwrap();
        let d_star_form = temp_manifold.exterior_derivative(dual_k);

        // 3. Apply second Hodge star: (n-k+1)-form -> (k-1)-form
        let mut temp_data2 = vec![T::zero(); self.data().len()];

        let dual_k_plus_1 = dual_k.saturating_add(1);
        let mut offset2 = 0;
        for i in 0..dual_k_plus_1 {
            if i >= self.complex.skeletons().len() {
                break;
            }
            offset2 += self.complex.skeletons()[i].simplices().len();
        }

        // Check that we have the skeleton we need
        if dual_k_plus_1 >= self.complex.skeletons().len() {
            // No such skeleton, return empty
            let result_size = if k > 0 && k.saturating_sub(1) < self.complex.skeletons().len() {
                self.complex.skeletons()[k.saturating_sub(1)]
                    .simplices()
                    .len()
            } else {
                0
            };
            return CausalTensor::new(vec![T::zero(); result_size], vec![result_size]).unwrap();
        }

        let d_dual_form_size = self.complex.skeletons()[dual_k_plus_1].simplices().len();
        if offset2 + d_dual_form_size <= temp_data2.len() {
            temp_data2[offset2..offset2 + d_dual_form_size].copy_from_slice(d_star_form.as_slice());
        }

        let temp_manifold2 = Manifold::new(
            self.complex.clone(),
            CausalTensor::new(temp_data2, vec![self.data().len()]).unwrap(),
            0,
        )
        .unwrap();
        let star_d_star_form = temp_manifold2.hodge_star(dual_k_plus_1);

        // 4. Apply sign correction
        let sign = if n.saturating_sub(k).saturating_add(1).is_multiple_of(2) {
            // Sign is for (-1)^(n*k - k + n + 1) which is (n*k - k + n + 1)%2
            1.0
        } else {
            -1.0
        };

        let result_data: Vec<T> = star_d_star_form
            .as_slice()
            .iter()
            .map(|&val| val * sign)
            .collect();

        CausalTensor::new(result_data, vec![star_d_star_form.len()]).unwrap()
    }
}
