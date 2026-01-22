/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for gauge group implementations.
//!
//! Covers U1, SU2, SU3, Electroweak, StandardModel, and Lorentz groups.

use deep_causality_metric::Metric;
use deep_causality_topology::{GaugeGroup, SE3, SO3_1, SU2, SU2_U1, SU3, SU3_SU2_U1, U1};

// ============================================================================
// U1 Tests
// ============================================================================

#[test]
fn test_u1_lie_algebra_dim() {
    assert_eq!(
        U1::LIE_ALGEBRA_DIM,
        1,
        "U(1) should have 1 generator (photon)"
    );
}

#[test]
fn test_u1_is_abelian() {
    const { assert!(U1::IS_ABELIAN) }; // U(1) should be abelian
}

#[test]
fn test_u1_spacetime_dim() {
    assert_eq!(
        U1::SPACETIME_DIM,
        4,
        "Default spacetime dimension should be 4"
    );
}

#[test]
fn test_u1_name() {
    assert_eq!(U1::name(), "U(1)");
}

#[test]
fn test_u1_matrix_dim() {
    assert_eq!(U1::matrix_dim(), 1, "U(1) is represented as 1x1 phase");
}

#[test]
fn test_u1_default_metric() {
    let metric = U1::default_metric();
    assert!(
        matches!(metric, Metric::Minkowski(4)),
        "U(1) uses West Coast Minkowski"
    );
}

#[test]
fn test_u1_structure_constant() {
    // U(1) is abelian, all structure constants are zero
    assert_eq!(U1::structure_constant(0, 0, 0), 0.0);
}

#[test]
fn test_u1_clone_debug() {
    let u1 = U1;
    let _cloned = u1.clone();
    let _debug = format!("{:?}", u1);
}

// ============================================================================
// SU2 Tests
// ============================================================================

#[test]
fn test_su2_lie_algebra_dim() {
    assert_eq!(SU2::LIE_ALGEBRA_DIM, 3, "SU(2) should have 3 generators");
}

#[test]
fn test_su2_is_abelian() {
    const { assert!(!SU2::IS_ABELIAN) }; // SU(2) should be non-abelian
}

#[test]
fn test_su2_name() {
    assert_eq!(SU2::name(), "SU(2)");
}

#[test]
fn test_su2_matrix_dim() {
    // SU(2): LIE_ALGEBRA_DIM = 3 = N² - 1, so N = 2
    assert_eq!(SU2::matrix_dim(), 2);
}

#[test]
fn test_su2_structure_constant_cyclic() {
    // ε_{012} = 1 (cyclic permutation)
    assert_eq!(SU2::structure_constant(0, 1, 2), 1.0);
    assert_eq!(SU2::structure_constant(1, 2, 0), 1.0);
    assert_eq!(SU2::structure_constant(2, 0, 1), 1.0);
}

#[test]
fn test_su2_structure_constant_anticyclic() {
    // ε_{021} = -1 (anti-cyclic permutation)
    assert_eq!(SU2::structure_constant(0, 2, 1), -1.0);
    assert_eq!(SU2::structure_constant(2, 1, 0), -1.0);
    assert_eq!(SU2::structure_constant(1, 0, 2), -1.0);
}

#[test]
fn test_su2_structure_constant_zero_cases() {
    // Cases with repeated indices are zero
    assert_eq!(SU2::structure_constant(0, 0, 0), 0.0);
    assert_eq!(SU2::structure_constant(1, 1, 0), 0.0);
    assert_eq!(SU2::structure_constant(0, 0, 2), 0.0);
}

#[test]
fn test_su2_clone_debug() {
    let su2 = SU2;
    let _cloned = su2.clone();
    let _debug = format!("{:?}", su2);
}

// ============================================================================
// SU3 Tests
// ============================================================================

#[test]
fn test_su3_lie_algebra_dim() {
    assert_eq!(
        SU3::LIE_ALGEBRA_DIM,
        8,
        "SU(3) should have 8 generators (gluons)"
    );
}

#[test]
fn test_su3_is_abelian() {
    const { assert!(!SU3::IS_ABELIAN) }; // SU(3) should be non-abelian
}

#[test]
fn test_su3_name() {
    assert_eq!(SU3::name(), "SU(3)");
}

#[test]
fn test_su3_matrix_dim() {
    // SU(3): LIE_ALGEBRA_DIM = 8 = N² - 1, so N = 3
    assert_eq!(SU3::matrix_dim(), 3);
}

#[test]
fn test_su3_structure_constant_default() {
    // SU3 uses default structure constant (0.0)
    // Full Gell-Mann structure constants not implemented
    assert_eq!(SU3::structure_constant(0, 1, 2), 0.0);
}

#[test]
fn test_su3_clone_debug() {
    let su3 = SU3;
    let _cloned = su3.clone();
    let _debug = format!("{:?}", su3);
}

// ============================================================================
// Electroweak Tests
// ============================================================================

#[test]
fn test_electroweak_lie_algebra_dim() {
    assert_eq!(SU2_U1::LIE_ALGEBRA_DIM, 4, "SU(2)×U(1) has 4 generators");
}

#[test]
fn test_electroweak_is_abelian() {
    const { assert!(!SU2_U1::IS_ABELIAN) }; // Electroweak is non-abelian (SU(2) factor)
}

#[test]
fn test_electroweak_name() {
    assert_eq!(SU2_U1::name(), "SU(2)×U(1)");
}

#[test]
fn test_electroweak_default_metric() {
    let metric = SU2_U1::default_metric();
    assert!(matches!(metric, Metric::Minkowski(4)));
}

#[test]
fn test_electroweak_clone_debug() {
    let ew = SU2_U1;
    let _cloned = ew.clone();
    let _debug = format!("{:?}", ew);
}

// ============================================================================
// StandardModel Tests
// ============================================================================

#[test]
fn test_standard_model_lie_algebra_dim() {
    assert_eq!(
        SU3_SU2_U1::LIE_ALGEBRA_DIM,
        12,
        "SU(3)×SU(2)×U(1) has 12 generators"
    );
}

#[test]
fn test_standard_model_is_abelian() {
    const { assert!(!SU3_SU2_U1::IS_ABELIAN) }; // Standard Model is non-abelian
}

#[test]
fn test_standard_model_name() {
    assert_eq!(SU3_SU2_U1::name(), "SU(3)×SU(2)×U(1)");
}

#[test]
fn test_standard_model_default_metric() {
    let metric = SU3_SU2_U1::default_metric();
    assert!(matches!(metric, Metric::Minkowski(4)));
}

#[test]
fn test_standard_model_clone_debug() {
    let sm = SU3_SU2_U1;
    let _cloned = sm.clone();
    let _debug = format!("{:?}", sm);
}

// ============================================================================
// Lorentz Tests
// ============================================================================

#[test]
fn test_lorentz_lie_algebra_dim() {
    assert_eq!(
        SO3_1::LIE_ALGEBRA_DIM,
        6,
        "SO(3,1) has 6 generators (3 rot + 3 boost)"
    );
}

#[test]
fn test_lorentz_is_abelian() {
    const { assert!(!SO3_1::IS_ABELIAN) }; // Lorentz group is non-abelian
}

#[test]
fn test_lorentz_name() {
    assert_eq!(SO3_1::name(), "SO(3,1)");
}

#[test]
fn test_lorentz_matrix_dim() {
    assert_eq!(SO3_1::matrix_dim(), 4, "SO(3,1) uses 4x4 matrices");
}

#[test]
fn test_lorentz_default_metric_east_coast() {
    let metric = SO3_1::default_metric();
    // East Coast: p=3 (positive spatial), q=1 (negative time)
    assert!(matches!(metric, Metric::Generic { p: 3, q: 1, r: 0 }));
}

#[test]
fn test_lorentz_structure_constant_rotation_rotation() {
    // [Jᵢ, Jⱼ] = εᵢⱼₖ Jₖ (rotation-rotation → rotation)
    // J0, J1, J2 are indices 0, 1, 2
    assert_eq!(SO3_1::structure_constant(0, 1, 2), 1.0); // [J0, J1] = J2
    assert_eq!(SO3_1::structure_constant(1, 2, 0), 1.0); // [J1, J2] = J0
    assert_eq!(SO3_1::structure_constant(2, 0, 1), 1.0); // [J2, J0] = J1
    assert_eq!(SO3_1::structure_constant(0, 2, 1), -1.0); // antisymmetry
}

#[test]
fn test_lorentz_structure_constant_rotation_boost() {
    // [Jᵢ, Kⱼ] = εᵢⱼₖ Kₖ (rotation-boost → boost)
    // K0, K1, K2 are indices 3, 4, 5
    assert_eq!(SO3_1::structure_constant(0, 3, 5), 0.0); // Need correct indices
    // More complex due to index mapping
}

#[test]
fn test_lorentz_structure_constant_boost_boost() {
    // [Kᵢ, Kⱼ] = -εᵢⱼₖ Jₖ (boost-boost → rotation with minus)
    // K0=3, K1=4, K2=5, result in J0=0, J1=1, J2=2
    assert_eq!(SO3_1::structure_constant(3, 4, 2), -1.0); // [K0, K1] = -J2
}

#[test]
fn test_lorentz_structure_constant_zero_cases() {
    // Mixed cases that don't match patterns
    assert_eq!(SO3_1::structure_constant(0, 0, 0), 0.0);
    assert_eq!(SO3_1::structure_constant(3, 3, 3), 0.0);
    assert_eq!(SO3_1::structure_constant(0, 3, 0), 0.0);
}

#[test]
fn test_lorentz_clone_debug() {
    let lorentz = SO3_1;
    let _cloned = lorentz.clone();
    let _debug = format!("{:?}", lorentz);
}

// ============================================================================
// SE(3) Tests
// ============================================================================

#[test]
fn test_se3_lie_algebra_dim() {
    assert_eq!(
        SE3::LIE_ALGEBRA_DIM,
        6,
        "SE(3) has 6 generators (3 rot + 3 trans)"
    );
}

#[test]
fn test_se3_is_abelian() {
    const { assert!(!SE3::IS_ABELIAN) }; // SE(3) is non-abelian
}

#[test]
fn test_se3_name() {
    assert_eq!(SE3::name(), "SE(3)");
}

#[test]
fn test_se3_matrix_dim() {
    assert_eq!(SE3::matrix_dim(), 4, "SE(3) uses 4x4 homogeneous matrices");
}

#[test]
fn test_se3_structure_constant_rot_rot() {
    // [Jᵢ, Jⱼ] = εᵢⱼₖ Jₖ (Standard SO(3) algebra)
    assert_eq!(SE3::structure_constant(0, 1, 2), 1.0);
    assert_eq!(SE3::structure_constant(1, 2, 0), 1.0);
    assert_eq!(SE3::structure_constant(0, 2, 1), -1.0);
}

#[test]
fn test_se3_structure_constant_rot_trans() {
    // [Jᵢ, Pⱼ] = εᵢⱼₖ Pₖ (Rotations rotate translations)
    // J: 0,1,2 | P: 3,4,5
    // [J0, P1] = P2 -> [0, 4] = 5 -> structure(0, 4, 5) = 1.0
    assert_eq!(SE3::structure_constant(0, 4, 5), 1.0);

    // [J1, P2] = P0 -> [1, 5] = 3 -> structure(1, 5, 3) = 1.0
    assert_eq!(SE3::structure_constant(1, 5, 3), 1.0);

    // [P1, J0] = -P2 -> [4, 0] = 5 -> structure(4, 0, 5) = -1.0
    assert_eq!(SE3::structure_constant(4, 0, 5), -1.0);
}

#[test]
fn test_se3_structure_constant_trans_trans() {
    // [Pᵢ, Pⱼ] = 0 (Translations commute)
    assert_eq!(SE3::structure_constant(3, 4, 5), 0.0);
    assert_eq!(SE3::structure_constant(4, 5, 3), 0.0);
}

#[test]
fn test_se3_clone_debug() {
    let se3 = SE3;
    let _cloned = se3.clone();
    let _debug = format!("{:?}", se3);
}

// ============================================================================
// GaugeGroup Trait Default Implementation Tests

// ============================================================================

#[test]
fn test_default_spacetime_dim_is_4() {
    // All groups should have SPACETIME_DIM = 4 by default
    assert_eq!(U1::SPACETIME_DIM, 4);
    assert_eq!(SU2::SPACETIME_DIM, 4);
    assert_eq!(SU3::SPACETIME_DIM, 4);
    assert_eq!(SU2_U1::SPACETIME_DIM, 4);
    assert_eq!(SU3_SU2_U1::SPACETIME_DIM, 4);
    assert_eq!(SO3_1::SPACETIME_DIM, 4);
}

#[test]
fn test_default_matrix_dim_formula() {
    // For SU(N): LIE_ALGEBRA_DIM = N² - 1
    // Default formula: N = sqrt(LIE_ALGEBRA_DIM + 1)
    // SU2: sqrt(3+1) = 2 ✓
    // SU3: sqrt(8+1) = 3 ✓
    assert_eq!(SU2::matrix_dim(), 2);
    assert_eq!(SU3::matrix_dim(), 3);
}

// ============================================================================
// PartialEq, Eq, Hash Tests
// ============================================================================

#[test]
fn test_gauge_groups_equality() {
    let u1_a = U1;
    let u1_b = U1;
    assert_eq!(u1_a, u1_b);

    let su2_a = SU2;
    let su2_b = SU2;
    assert_eq!(su2_a, su2_b);
}

#[test]
fn test_gauge_groups_default() {
    let _u1: U1 = Default::default();
    let _su2: SU2 = Default::default();
    let _su3: SU3 = Default::default();
    let _ew: SU2_U1 = Default::default();
    let _sm: SU3_SU2_U1 = Default::default();
    let _lorentz: SO3_1 = Default::default();
}

#[test]
fn test_gauge_groups_hash() {
    use std::collections::HashSet;

    // Test U1 set
    let mut u1_set = HashSet::new();
    u1_set.insert(U1);
    assert!(u1_set.contains(&U1));

    // Test SU2 set
    let mut su2_set = HashSet::new();
    su2_set.insert(SU2);
    assert!(su2_set.contains(&SU2));
}
