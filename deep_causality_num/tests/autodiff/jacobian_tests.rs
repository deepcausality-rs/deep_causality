/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::jacobian;

#[test]
fn test_jacobian_square_map() {
    // f(x, y) = [x·y, x + y] → J = [[y, x], [1, 1]] at (2, 3)
    // Row k is the gradient of output k.
    let j = jacobian::<f64, _, 2, 2>(|p| [p[0] * p[1], p[0] + p[1]], &[2.0, 3.0]);
    assert_eq!(j, [[3.0, 2.0], [1.0, 1.0]]);
}

#[test]
fn test_jacobian_tall_map_one_input_three_outputs() {
    // f(x) = [x, x², x³] : R¹ → R³ → J = [[1], [2x], [3x²]] at x = 2 → [[1], [4], [12]]
    let j = jacobian::<f64, _, 1, 3>(|p| [p[0], p[0] * p[0], p[0] * p[0] * p[0]], &[2.0]);
    assert_eq!(j, [[1.0], [4.0], [12.0]]);
}

#[test]
fn test_jacobian_wide_map_three_inputs_one_output() {
    // f(x, y, z) = x·y·z : R³ → R¹ → J = [[yz, xz, xy]] at (2, 3, 4)
    let j = jacobian::<f64, _, 3, 1>(|p| [p[0] * p[1] * p[2]], &[2.0, 3.0, 4.0]);
    assert_eq!(j, [[12.0, 8.0, 6.0]]);
}

#[test]
fn test_jacobian_is_precision_generic_f32() {
    let j = jacobian::<f32, _, 2, 2>(|p| [p[0] * p[1], p[0] + p[1]], &[2.0, 3.0]);
    assert_eq!(j, [[3.0_f32, 2.0], [1.0, 1.0]]);
}
