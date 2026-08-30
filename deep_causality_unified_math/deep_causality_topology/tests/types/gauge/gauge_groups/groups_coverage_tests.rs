/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Additional coverage for gauge group structure constants and matrix dimensions.

use deep_causality_topology::{GaugeGroup, SE3, SO3_1, SU3_SU2_U1};

// ============================================================================
// SE(3): exercise the negative Levi-Civita arm and the catch-all 0.0 arm.
// ============================================================================

#[test]
fn test_se3_structure_constant_negative_levi_civita() {
    // (true, true, true) dispatches to epsilon(a, b, c).
    // The odd permutations (0,2,1), (2,1,0), (1,0,2) return -1.0.
    assert_eq!(SE3::structure_constant(0, 2, 1), -1.0);
    assert_eq!(SE3::structure_constant(2, 1, 0), -1.0);
    assert_eq!(SE3::structure_constant(1, 0, 2), -1.0);
}

#[test]
fn test_se3_structure_constant_catch_all_zero() {
    // (rotation, rotation, translation) i.e. (true, true, false) matches none of
    // the explicit arms and falls through to the final `_ => 0.0`.
    assert_eq!(SE3::structure_constant(0, 1, 3), 0.0);
    // (translation, translation, rotation) -> (false, false, true) hits the
    // (false, false, _) commuting arm, still 0.0.
    assert_eq!(SE3::structure_constant(3, 4, 0), 0.0);
}

// ============================================================================
// SO(3,1): exercise the boost-rotation antisymmetry arm.
// ============================================================================

#[test]
fn test_so3_1_structure_constant_boost_rotation_antisymmetry() {
    // (false, true, false) with c >= 3: a is a boost (>=3), b is a rotation (<3),
    // c is a boost (>=3). Result = -epsilon(a-3, b, c-3).
    // [K0, J1] = -epsilon(0, 1, 2) K2 = -K2  -> structure_constant(3, 1, 5) = -1.0
    assert_eq!(SO3_1::structure_constant(3, 1, 5), -1.0);
    // [K1, J2] = -epsilon(1, 2, 0) K0 = -K0 -> structure_constant(4, 2, 3) = -1.0
    assert_eq!(SO3_1::structure_constant(4, 2, 3), -1.0);
}

// ============================================================================
// SU(3)xSU(2)xU(1): exercise matrix_dim().
// ============================================================================

#[test]
fn test_standard_model_matrix_dim_is_six() {
    // SU(3) (3x3) + SU(2) (2x2) + U(1) (1x1) block-diagonal -> 6x6.
    assert_eq!(SU3_SU2_U1::matrix_dim(), 6);
}
