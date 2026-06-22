/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the floating-point clamp branches in the closed-form
//! symmetric-3×3 eigenvalue solver used by `lambda2_kernel`
//! (coherent_structures.rs:254-255 and 257-258).
//!
//! The solver computes `r_val = det(B)/2` and clamps it into `[-1, 1]` before
//! `acos`, because rounding can push a value that is mathematically exactly
//! ±1 slightly outside the domain. The symmetric velocity gradients below were
//! found by an offline scan to drive `M = S² + Ω²` into a near-degenerate
//! configuration where `r_val` overshoots ±1 in IEEE-754 f64 arithmetic, so
//! `lambda2_kernel` exercises both clamp arms.

use deep_causality_physics::{VelocityGradient, lambda2_kernel};

#[test]
fn test_lambda2_clamps_rval_above_one() {
    // This non-symmetric velocity gradient drives `M = S² + Ω²` into a
    // near-degenerate configuration where `r_val = det(B)/2 ≈
    // 1.0000000000000004 (> 1)` in IEEE-754 f64 arithmetic, exercising the
    // upper clamp `r_val = R::one()` (coherent_structures.rs:257-258).
    // Found by an offline brute-force scan over half-integer gradients.
    let g = VelocityGradient::<f64>::new([[-2.5, 1.0, 2.5], [-1.0, -2.5, 1.5], [-2.5, -1.5, 2.5]])
        .unwrap();

    let result = lambda2_kernel(&g);
    assert!(result.is_ok(), "lambda2 should succeed, got {result:?}");
    // The middle eigenvalue must be finite (clamp prevents a NaN from acos).
    assert!(result.unwrap().is_finite());
}

#[test]
fn test_lambda2_clamps_rval_below_minus_one() {
    // This symmetric gradient yields r_val ≈ -1.0000000000000007 (< -1),
    // exercising the lower clamp (coherent_structures.rs:254-255).
    let g =
        VelocityGradient::<f64>::new([[-1.0, -3.0, 2.0], [-3.0, -1.0, -3.0], [2.0, -3.0, -1.0]])
            .unwrap();

    let result = lambda2_kernel(&g);
    assert!(result.is_ok(), "lambda2 should succeed, got {result:?}");
    assert!(result.unwrap().is_finite());
}
