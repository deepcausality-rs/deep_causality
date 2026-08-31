/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Chain, SimplicialComplex};
use core::fmt::Debug;
use deep_causality_linear::CsrMatrix;
use deep_causality_num::Num;

impl<R> SimplicialComplex<R> {
    /// Computes the boundary of a chain: ∂c
    /// Maps a k-chain to a (k-1)-chain.
    ///
    /// The boundary operator has entries in `{-1, 0, 1}`, and `G: From<i8>` is what lets a
    /// coefficient type receive those incidence signs. The `Num` bound is the wider numeric tower —
    /// addition, subtraction, multiplication, division and remainder, over a zero and a one — so
    /// the coefficient types this reaches are the numeric ones. A coefficient group carrying only
    /// addition is outside it, even though the incidence sum uses nothing past `+` and `*`.
    ///
    /// It reads no geometry, which is why the complex's precision `R` is unconstrained here.
    pub fn boundary<G>(&self, chain: &Chain<R, G>) -> Chain<R, G>
    where
        G: Copy + Num + Default + From<i8> + Debug,
    {
        if chain.grade == 0 {
            panic!("Cannot take boundary of 0-chain");
        }

        // This is N_{k-1} x N_k
        let boundary_op = &self.boundary_operators[chain.grade - 1];
        let k_minus_1_size = self.skeletons[chain.grade - 1].simplices.len();
        let mut new_triplets = Vec::new();

        // Manual mat-vec mul: v_out = M * v_in
        // Iterate over rows of M (simplices in k-1 skeleton)
        for r in 0..boundary_op.shape().0 {
            let mut val = G::zero();
            // Iterate over non-zero elements in this row of M
            let row_start = boundary_op.row_indices()[r];
            let row_end = boundary_op.row_indices()[r + 1];

            for i in row_start..row_end {
                let c = boundary_op.col_indices()[i]; // This is a simplex in k skeleton
                let m_val = G::from(boundary_op.values()[i]);

                // Find corresponding value in input chain vector (which is a single-row sparse matrix)
                let chain_row_start = chain.weights.row_indices()[0];
                let chain_row_end = chain.weights.row_indices()[1];
                for j in chain_row_start..chain_row_end {
                    if chain.weights.col_indices()[j] == c {
                        let v_val = &chain.weights.values()[j];
                        val = val + (m_val * *v_val);
                        break; // Found the column, move to next element in M's row
                    }
                }
            }

            if val != G::zero() {
                new_triplets.push((0, r, val));
            }
        }

        let new_weights = CsrMatrix::from_triplets(1, k_minus_1_size, &new_triplets).unwrap();

        Chain {
            complex: chain.complex.clone(),
            grade: chain.grade - 1,
            weights: new_weights,
        }
    }

    /// Computes the coboundary (exterior derivative) of a cochain: dω
    /// Maps a k-cochain to a (k+1)-cochain.
    pub fn coboundary<G>(&self, chain: &Chain<R, G>) -> Chain<R, G>
    where
        G: Copy + Num + Default + From<i8>,
    {
        if chain.grade >= self.skeletons.len() - 1 {
            panic!("Cannot take coboundary of max-dim chain");
        }

        let coboundary_op = &self.coboundary_operators[chain.grade]; // This is N_{k+1} x N_k
        let k_plus_1_size = self.skeletons[chain.grade + 1].simplices.len();

        let mut new_triplets = Vec::new();

        // Manual mat-vec mul
        for r in 0..coboundary_op.shape().0 {
            let mut val = G::zero();
            let row_start = coboundary_op.row_indices()[r];
            let row_end = coboundary_op.row_indices()[r + 1];

            for i in row_start..row_end {
                let c = coboundary_op.col_indices()[i];
                let m_val = G::from(coboundary_op.values()[i]);

                // Find corresponding value in input chain vector
                let chain_row_start = chain.weights.row_indices()[0];
                let chain_row_end = chain.weights.row_indices()[1];
                for j in chain_row_start..chain_row_end {
                    if chain.weights.col_indices()[j] == c {
                        let v_val = &chain.weights.values()[j];
                        val = val + (m_val * *v_val);
                        break;
                    }
                }
            }

            if val != G::zero() {
                new_triplets.push((0, r, val));
            }
        }

        let new_weights = CsrMatrix::from_triplets(1, k_plus_1_size, &new_triplets).unwrap();

        Chain {
            complex: chain.complex.clone(),
            grade: chain.grade + 1,
            weights: new_weights,
        }
    }
}
