/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::CausalMultiVector;
use deep_causality_num::Num;
use std::ops::{AddAssign, Neg, SubAssign};

impl<T> CausalMultiVector<T> {
    /// Computes the outer product (wedge product) $A \wedge B$.
    ///
    /// The outer product of two multivectors of grades $r$ and $s$ is the grade $r+s$ part of their geometric product.
    /// $$ A \wedge B = \langle AB \rangle_{r+s} $$
    ///
    /// For basis blades $e_I$ and $e_J$, $e_I \wedge e_J$ is non-zero only if $I \cap J = \emptyset$.
    pub(super) fn outer_product_impl(&self, rhs: &Self) -> Self
    where
        T: Num + Copy + Clone + AddAssign + SubAssign,
    {
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
                    // Calculate sign from swaps only, not the full geometric product.
                    let mut swaps = 0;
                    for k in 0..dim {
                        if (j >> k) & 1 == 1 {
                            swaps += (i >> (k + 1)).count_ones();
                        }
                    }
                    let sign = if swaps % 2 == 0 { 1 } else { -1 };

                    let result_idx = i | j; // For disjoint sets, XOR is equivalent to OR
                    let val = self.data[i] * rhs.data[j];

                    if sign > 0 {
                        result_data[result_idx] += val;
                    } else {
                        result_data[result_idx] -= val;
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
    pub(super) fn inner_product_impl(&self, rhs: &Self) -> Self
    where
        T: Num + Copy + Clone + AddAssign + SubAssign,
    {
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
                    let (sign, result_idx) = Self::basis_product(i, j, &self.metric);

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

    pub(super) fn commutator_lie_impl(&self, rhs: &Self) -> Self
    where
        T: Num + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
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

                // 1. Calculate Directions
                let (sign_ab, k) = Self::basis_product(i, j, &self.metric);
                if sign_ab == 0 {
                    continue;
                } // Handle degenerate metric

                let (sign_ba, _) = Self::basis_product(j, i, &self.metric);

                // 2. [A, B] = AB - BA
                // If signs differ, we have non-zero commutator
                if sign_ab != sign_ba {
                    let val = self.data[i] * rhs.data[j];

                    // Result is 2 * sign_ab * val
                    // We use (val + val) to double it efficiently
                    let double_val = val + val;

                    if sign_ab > 0 {
                        result_data[k] += double_val;
                    } else {
                        result_data[k] -= double_val;
                    }
                }
            }
        }

        Self {
            data: result_data,
            metric: self.metric,
        }
    }

    // The scaled logic 0.5 * (AB - BA)
    pub(super) fn commutator_geometric_impl(&self, rhs: &Self) -> Self
    where
        T: Num + Copy + Clone + AddAssign + SubAssign + Neg<Output = T> + std::ops::Div<Output = T>,
    {
        // 1. Calculate the raw Lie bracket (AB - BA)
        let lie_bracket = self.commutator_lie_impl(rhs);

        // 2. Define "2" using the generic One trait
        // This works for f64 (1.0+1.0=2.0), Complex (1+i0 + 1+i0 = 2+i0), etc.
        let two = T::one() + T::one();

        // 3. Check for division by zero (e.g. in Boolean algebra 1+1=0, or GF(2))
        if two.is_zero() {
            // In a field where 1+1=0 (Characteristic 2), division by 2 is undefined.
            // For Physics (R/C), this branch is never taken.
            // You might panic or return the Lie bracket depending on philosophy.
            panic!(
                "Cannot compute Geometric Commutator (scale by 1/2) in a field with characteristic 2"
            );
        }

        // 4. Scale the result
        let scaled_data = lie_bracket.data.into_iter().map(|val| val / two).collect();

        Self {
            data: scaled_data,
            metric: lie_bracket.metric,
        }
    }
}
