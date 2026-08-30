/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::chain::Chain;
use crate::types::simplicial_complex::SimplicialComplex;
use deep_causality_algebra::AbelianGroup;
use deep_causality_linear::CsrMatrix;
use std::sync::Arc;

impl<R, G> Chain<R, G>
where
    G: AbelianGroup + Copy + PartialEq + Default + core::ops::Neg<Output = G>,
{
    /// Creates a zero chain for a given complex and grade.
    ///
    /// # Arguments
    /// * `complex` - The simplicial complex the chain belongs to.
    /// * `grade` - The dimension of the chain (k-chain).
    ///
    /// # Returns
    /// A chain with all weights set to zero.
    pub fn zero(complex: Arc<SimplicialComplex<R>>, grade: usize) -> Self {
        let size = complex.skeletons[grade].simplices.len();
        // Chain is represented as a 1 x N sparse matrix (row vector)
        let weights = CsrMatrix::zero(1, size);
        Self {
            complex,
            grade,
            weights,
        }
    }

    /// Adds two chains.
    ///
    /// # Panics
    /// Panics if the chains belong to different complexes or have different grades.
    pub fn add(&self, rhs: &Self) -> Self {
        self.check_compatibility(rhs);
        let weights = self.weights.add(&rhs.weights);
        Self {
            complex: Arc::clone(&self.complex),
            grade: self.grade,
            weights,
        }
    }

    /// Subtracts two chains.
    ///
    /// # Panics
    /// Panics if the chains belong to different complexes or have different grades.
    pub fn sub(&self, rhs: &Self) -> Self {
        self.check_compatibility(rhs);
        let weights = self.weights.sub(&rhs.weights);
        Self {
            complex: Arc::clone(&self.complex),
            grade: self.grade,
            weights,
        }
    }

    /// Negates the chain.
    pub fn neg(&self) -> Self {
        let weights = self.weights.neg();
        Self {
            complex: Arc::clone(&self.complex),
            grade: self.grade,
            weights,
        }
    }
}

impl<R, G> Chain<R, G> {
    fn check_compatibility(&self, rhs: &Self) {
        assert_eq!(self.grade, rhs.grade, "Chain grade mismatch");
        assert!(
            Arc::ptr_eq(&self.complex, &rhs.complex),
            "Chain complex mismatch"
        );
    }
}

impl<R, G> Chain<R, G>
where
    G: Copy + PartialEq + core::ops::Add<Output = G>,
{
    /// Adds two chains with an explicit zero value for contextual sparsity.
    ///
    /// # Arguments
    /// * `rhs` - The chain to add.
    /// * `zero` - The value to treat as zero.
    pub fn add_with_zero(&self, rhs: &Self, zero: G) -> Self {
        self.check_compatibility(rhs);
        let weights = self
            .weights
            .add_with_zero(&rhs.weights, zero)
            .expect("Matrix shape mismatch");
        Self {
            complex: Arc::clone(&self.complex),
            grade: self.grade,
            weights,
        }
    }
}
