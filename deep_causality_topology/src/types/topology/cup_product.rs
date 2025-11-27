/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;
use core::ops::{AddAssign, Neg, SubAssign};
use deep_causality_multivector::MultiVector;
use deep_causality_num::Num;
use deep_causality_tensor::CausalTensor;

use crate::Topology;

impl<T> Topology<T>
where
    T: MultiVector<T> + Clone + Num + Copy + AddAssign + SubAssign + Neg<Output = T>,
{
    /// The Cup Product: (k-form) U (l-form) -> (k+l)-form
    /// Used for Generative Quantum Gates: S = exp(a U a)
    pub fn cup_product(&self, other: &Topology<T>) -> Topology<T> {
        let k = self.grade;
        let l = other.grade;
        let dim = k + l;

        // Ensure target dimension exists
        if dim >= self.complex.skeletons.len() {
            panic!("Cup product dimension exceeds complex dimension");
        }

        let target_skeleton = &self.complex.skeletons[dim];
        let mut result_data = Vec::with_capacity(target_skeleton.simplices.len());

        // Iterate over all (k+l)-simplices
        for simplex in &target_skeleton.simplices {
            // 1. Alexander-Whitney Approximation
            // Split vertices [0...n] into Front [0...k] and Back [k...n]
            // The vertices are sorted, so this corresponds to the standard AW map.

            // Front face: vertices 0 to k
            let front_face = simplex.subsimplex(0..=k);

            // Back face: vertices k to k+l (which is dim)
            let back_face = simplex.subsimplex(k..=dim);

            // 2. Look up values
            // We must find the global index of these faces to retrieve data from the topologies.
            let front_idx = self.complex.skeletons[k]
                .get_index(&front_face)
                .expect("Front face not found in skeleton");
            let back_idx = self.complex.skeletons[l]
                .get_index(&back_face)
                .expect("Back face not found in skeleton");

            // Use as_slice() to get data
            let v1 = self
                .data
                .as_slice()
                .get(front_idx)
                .expect("Data missing for front face");
            let v2 = other
                .data
                .as_slice()
                .get(back_idx)
                .expect("Data missing for back face");

            // 3. Geometric Product
            // The cup product on forms corresponds to the geometric product of multivectors
            // in the discrete setting (under certain metric assumptions).
            result_data.push(v1.geometric_product(v2));
        }

        // Return new topology
        Topology {
            complex: self.complex.clone(),
            grade: dim,
            data: CausalTensor::new(result_data, vec![target_skeleton.simplices.len()]).unwrap(),
            cursor: 0,
        }
    }
}
