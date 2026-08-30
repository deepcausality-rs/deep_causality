/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! SE(3) structure constants for index triples that are not permutations of (0, 1, 2).
//!
//! The se(3) brackets are
//! [J_i, J_j] = ε_ijk J_k, [J_i, P_j] = ε_ijk P_k and [P_i, P_j] = 0.
//! The Levi-Civita symbol vanishes whenever two of its three indices coincide,
//! so every triple below has an expected value of zero or a sign fixed by
//! antisymmetry of the bracket.

use deep_causality_topology::{GaugeGroup, SE3};

#[test]
fn test_se3_structure_constant_rot_rot_repeated_index_is_zero() {
    // [J_i, J_i] = 0 because ε_iik = 0.
    assert_eq!(SE3::structure_constant(0, 0, 1), 0.0);
    assert_eq!(SE3::structure_constant(1, 1, 2), 0.0);
    assert_eq!(SE3::structure_constant(2, 2, 0), 0.0);
}

#[test]
fn test_se3_structure_constant_rot_rot_repeated_target_is_zero() {
    // [J_0, J_1] = J_2, so the components along J_0 and J_1 both vanish.
    assert_eq!(SE3::structure_constant(0, 1, 0), 0.0);
    assert_eq!(SE3::structure_constant(0, 1, 1), 0.0);
}

#[test]
fn test_se3_structure_constant_rot_trans_repeated_index_is_zero() {
    // [J_0, P_0] = ε_00k P_k = 0. Generator 3 is P_0.
    assert_eq!(SE3::structure_constant(0, 3, 3), 0.0);
    // [J_1, P_1] = 0. Generator 4 is P_1.
    assert_eq!(SE3::structure_constant(1, 4, 4), 0.0);
    // [J_0, P_1] = P_2, so the component along P_1 vanishes.
    assert_eq!(SE3::structure_constant(0, 4, 4), 0.0);
}

#[test]
fn test_se3_structure_constant_trans_rot_antisymmetry() {
    // [P_0, J_1] = -[J_1, P_0] = -ε_102 P_2 = +P_2, so f^{3,1,5} = 1.
    assert_eq!(SE3::structure_constant(3, 1, 5), 1.0);
    // [P_0, J_0] = -[J_0, P_0] = 0.
    assert_eq!(SE3::structure_constant(3, 0, 5), 0.0);
    // [P_1, J_2] = -ε_210 P_0 = +P_0, so f^{4,2,3} = 1.
    assert_eq!(SE3::structure_constant(4, 2, 3), 1.0);
}
