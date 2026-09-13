/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils_tests::hand_built_complex::HandBuiltComplex;
use alloc::vec;

/// The `[[4, 2, 2]]` code as a chain complex: two 0-cells, four 1-cells each running from the first
/// vertex to the second, and one 2-cell whose boundary is the four edges with alternating signs.
///
/// Over ℤ, `∂₁ = [[−1, −1, −1, −1], [1, 1, 1, 1]]` and `∂₂ = [1, −1, 1, −1]ᵀ`, so `∂₁ ∂₂ = 0`.
/// Over 𝔽₂ the one column of `∂₂` is the Z-check `ZZZZ`, the two rows of `∂₁` are both the X-check
/// `XXXX`, `rank ∂₁ = rank ∂₂ = 1`, and `k = 4 − 1 − 1 = 2`. Every non-trivial logical operator has
/// weight at least 2, so the distance is 2. The composite Choi of a channel from its four qubits to
/// its two logical qubits has `2^12` entries.
pub fn four_two_two() -> HandBuiltComplex {
    let d1: [(usize, usize, i8); 8] = [
        (0, 0, -1),
        (0, 1, -1),
        (0, 2, -1),
        (0, 3, -1),
        (1, 0, 1),
        (1, 1, 1),
        (1, 2, 1),
        (1, 3, 1),
    ];
    let d2: [(usize, usize, i8); 4] = [(0, 0, 1), (1, 0, -1), (2, 0, 1), (3, 0, -1)];
    HandBuiltComplex::new(vec![2, 4, 1], &[&d1, &d2])
}
