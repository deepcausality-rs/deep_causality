/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::CausalMultiVector;
use crate::Metric;
use deep_causality_num::Num;
use std::ops::{AddAssign, SubAssign};

impl<T> CausalMultiVector<T>
where
    T: Num + Copy + Clone + AddAssign + SubAssign,
{
    /// Computes the outer product (wedge product) $A \wedge B$.
    ///
    /// The outer product of two multivectors of grades $r$ and $s$ is the grade $r+s$ part of their geometric product.
    /// $$ A \wedge B = \langle AB \rangle_{r+s} $$
    ///
    /// For basis blades $e_I$ and $e_J$, $e_I \wedge e_J$ is non-zero only if $I \cap J = \emptyset$.
    pub fn outer_product(&self, rhs: &Self) -> Self {
        if self.metric != rhs.metric {
            panic!("Metric mismatch");
        }

        let dim = self.metric.dimension();
        let count = 1 << dim;
        let mut result_data = vec![T::zero(); count];

        for i in 0..count {
            if self.data[i].is_zero() {
                continue;
            }
            for j in 0..count {
                if rhs.data[j].is_zero() {
                    continue;
                }

                // Outer product is non-zero only if blades are disjoint
                if (i & j) == 0 {
                    let (sign, result_idx) = Self::calculate_basis_product(i, j, &self.metric);

                    if sign != 0 {
                        let val = self.data[i] * rhs.data[j];
                        if sign > 0 {
                            result_data[result_idx] += val;
                        } else {
                            result_data[result_idx] -= val;
                        }
                    }
                }
            }
        }
        Self {
            data: result_data,
            metric: self.metric,
        }
    }

    /// Computes the inner product (left contraction) $A \cdot B$ (or $A \rfloor B$).
    ///
    /// The inner product of a grade $r$ multivector $A$ and a grade $s$ multivector $B$ is the grade $s-r$ part of their geometric product.
    /// $$ A \cdot B = \langle AB \rangle_{s-r} $$
    ///
    /// For basis blades $e_I$ and $e_J$, $e_I \cdot e_J$ is non-zero only if $I \subseteq J$.
    pub fn inner_product(&self, rhs: &Self) -> Self {
        if self.metric != rhs.metric {
            panic!("Metric mismatch");
        }

        let dim = self.metric.dimension();
        let count = 1 << dim;
        let mut result_data = vec![T::zero(); count];

        for i in 0..count {
            if self.data[i].is_zero() {
                continue;
            }
            for j in 0..count {
                if rhs.data[j].is_zero() {
                    continue;
                }

                // Left contraction requires I subset J
                if (i & j) == i {
                    let (sign, result_idx) = Self::calculate_basis_product(i, j, &self.metric);

                    if sign != 0 {
                        let val = self.data[i] * rhs.data[j];
                        if sign > 0 {
                            result_data[result_idx] += val;
                        } else {
                            result_data[result_idx] -= val;
                        }
                    }
                }
            }
        }
        Self {
            data: result_data,
            metric: self.metric,
        }
    }

    /// Helper function to calculate the sign and index of the geometric product of two basis blades.
    ///
    /// Given two basis blades $e_A$ and $e_B$ (represented by bitmaps `a_map` and `b_map`),
    /// this function computes the resulting basis blade $e_C$ (bitmap `result_map`) and the sign $s$ such that:
    /// $$ e_A e_B = s e_C $$
    ///
    /// The sign accounts for:
    /// 1. Canonical reordering (swaps).
    /// 2. Metric signature (squaring of basis vectors).
    ///
    /// If any basis vector in the intersection squares to 0 (degenerate metric), the result is 0.
    pub fn calculate_basis_product(a_map: usize, b_map: usize, metric: &Metric) -> (i32, usize) {
        let mut sign = 1;

        // 1. Calculate Sign from Swaps (Canonical Reordering)
        let a_temp = a_map;
        let mut swaps = 0;
        let dim = metric.dimension();

        for i in 0..dim {
            if (b_map >> i) & 1 == 1 {
                let higher_bits_in_a = (a_temp >> (i + 1)).count_ones();
                swaps += higher_bits_in_a;
            }
        }
        if swaps % 2 != 0 {
            sign *= -1;
        }

        // 2. Calculate Sign from Metric (Squaring generators)
        let intersection = a_map & b_map;
        for i in 0..dim {
            if (intersection >> i) & 1 == 1 {
                let sq_sign = metric.sign_of_sq(i);

                // If any generator in the intersection squares to 0,
                // the whole term is annihilated.
                if sq_sign == 0 {
                    return (0, 0);
                }

                sign *= sq_sign;
            }
        }

        // 3. Resulting Bitmap (XOR)
        let result_map = a_map ^ b_map;

        (sign, result_map)
    }
}
