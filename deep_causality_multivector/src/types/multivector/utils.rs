/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalMultiVector, Metric};
use deep_causality_num::Num;
use std::ops::{AddAssign, SubAssign};

impl<T> CausalMultiVector<T> {
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
    pub(super) fn basis_product(a_map: usize, b_map: usize, metric: &Metric) -> (i32, usize)
    where
        T: Num + Copy + Clone + AddAssign + SubAssign,
    {
        let mut sign = 1;

        // 1. Calculate Sign from Swaps (Canonical Reordering)
        let mut swaps = 0;
        let dim = metric.dimension();

        for i in 0..dim {
            if (b_map >> i) & 1 == 1 {
                let higher_bits_in_a = (a_map >> (i + 1)).count_ones();
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
