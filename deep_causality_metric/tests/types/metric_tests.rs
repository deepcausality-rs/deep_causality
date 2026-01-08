/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use std::collections::HashSet;

// =============================================================================
// dimension() tests
// =============================================================================

#[test]
fn test_euclidean_dimension() {
    assert_eq!(Metric::Euclidean(3).dimension(), 3);
    assert_eq!(Metric::Euclidean(10).dimension(), 10);
}

#[test]
fn test_non_euclidean_dimension() {
    assert_eq!(Metric::NonEuclidean(4).dimension(), 4);
}

#[test]
fn test_minkowski_dimension() {
    assert_eq!(Metric::Minkowski(4).dimension(), 4);
    assert_eq!(Metric::Minkowski(11).dimension(), 11);
}

#[test]
fn test_pga_dimension() {
    assert_eq!(Metric::PGA(4).dimension(), 4);
}

#[test]
fn test_generic_dimension() {
    let m = Metric::Generic { p: 3, q: 1, r: 0 };
    assert_eq!(m.dimension(), 4);
}

#[test]
fn test_custom_dimension() {
    let m = Metric::Custom {
        dim: 5,
        neg_mask: 0b11,
        zero_mask: 0,
    };
    assert_eq!(m.dimension(), 5);
}

// =============================================================================
// sign_of_sq() tests
// =============================================================================

#[test]
fn test_euclidean_signs() {
    let m = Metric::Euclidean(4);
    for i in 0..4 {
        assert_eq!(m.sign_of_sq(i), 1);
    }
}

#[test]
fn test_non_euclidean_signs() {
    let m = Metric::NonEuclidean(4);
    for i in 0..4 {
        assert_eq!(m.sign_of_sq(i), -1);
    }
}

#[test]
fn test_minkowski_signs() {
    // West Coast: (+---)
    let m = Metric::Minkowski(4);
    assert_eq!(m.sign_of_sq(0), 1); // time
    assert_eq!(m.sign_of_sq(1), -1); // space
    assert_eq!(m.sign_of_sq(2), -1);
    assert_eq!(m.sign_of_sq(3), -1);
}

#[test]
fn test_pga_signs() {
    // e0 = 0, others = +1
    let m = Metric::PGA(4);
    assert_eq!(m.sign_of_sq(0), 0);
    assert_eq!(m.sign_of_sq(1), 1);
    assert_eq!(m.sign_of_sq(2), 1);
    assert_eq!(m.sign_of_sq(3), 1);
}

#[test]
fn test_generic_signs() {
    // Cl(2, 1, 1): first 2 are +1, next 1 is -1, last 1 is 0
    let m = Metric::Generic { p: 2, q: 1, r: 1 };
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(1), 1);
    assert_eq!(m.sign_of_sq(2), -1);
    assert_eq!(m.sign_of_sq(3), 0);
}

#[test]
fn test_custom_signs() {
    // Custom: dim=4, neg_mask=0b0001 (-+++), zero_mask=0
    let m = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    assert_eq!(m.sign_of_sq(0), -1);
    assert_eq!(m.sign_of_sq(1), 1);
    assert_eq!(m.sign_of_sq(2), 1);
    assert_eq!(m.sign_of_sq(3), 1);
}

#[test]
fn test_custom_with_zero() {
    // Custom with degenerate dimension
    let m = Metric::Custom {
        dim: 4,
        neg_mask: 0b0010,
        zero_mask: 0b0001,
    };
    assert_eq!(m.sign_of_sq(0), 0); // zero_mask bit 0
    assert_eq!(m.sign_of_sq(1), -1); // neg_mask bit 1
    assert_eq!(m.sign_of_sq(2), 1);
    assert_eq!(m.sign_of_sq(3), 1);
}

// =============================================================================
// signature() tests
// =============================================================================

#[test]
fn test_euclidean_signature() {
    assert_eq!(Metric::Euclidean(4).signature(), (4, 0, 0));
}

#[test]
fn test_non_euclidean_signature() {
    assert_eq!(Metric::NonEuclidean(3).signature(), (0, 3, 0));
}

#[test]
fn test_minkowski_signature() {
    // Minkowski(4) is (+---) = (1, 3, 0)
    assert_eq!(Metric::Minkowski(4).signature(), (1, 3, 0));
}

#[test]
fn test_pga_signature() {
    // PGA(4) = (3, 0, 1)
    assert_eq!(Metric::PGA(4).signature(), (3, 0, 1));
}

#[test]
fn test_generic_signature() {
    let m = Metric::Generic { p: 2, q: 3, r: 1 };
    assert_eq!(m.signature(), (2, 3, 1));
}

#[test]
fn test_custom_signature() {
    // Custom: (-+++) = (3, 1, 0)
    let m = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    assert_eq!(m.signature(), (3, 1, 0));
}

// =============================================================================
// flip_time_space() tests
// =============================================================================

#[test]
fn test_flip_minkowski() {
    // Minkowski (+---) -> (-+++)
    let m = Metric::Minkowski(4);
    let flipped = m.flip_time_space();

    assert_eq!(flipped.sign_of_sq(0), -1);
    assert_eq!(flipped.sign_of_sq(1), 1);
    assert_eq!(flipped.sign_of_sq(2), 1);
    assert_eq!(flipped.sign_of_sq(3), 1);
}

#[test]
fn test_flip_roundtrip() {
    // flip(flip(m)) should have same signature
    let m = Metric::Minkowski(4);
    let double_flipped = m.flip_time_space().flip_time_space();

    assert_eq!(m.signature(), double_flipped.signature());
}

#[test]
fn test_flip_euclidean() {
    // Euclidean flipped becomes all negative
    let m = Metric::Euclidean(3);
    let flipped = m.flip_time_space();

    for i in 0..3 {
        assert_eq!(flipped.sign_of_sq(i), -1);
    }
}

// =============================================================================
// tensor_product() tests
// =============================================================================

#[test]
fn test_tensor_product_euclidean() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Euclidean(3);
    let product = m1.tensor_product(&m2);

    assert_eq!(product, Metric::Euclidean(5));
}

#[test]
fn test_tensor_product_non_euclidean() {
    let m1 = Metric::NonEuclidean(2);
    let m2 = Metric::NonEuclidean(2);
    let product = m1.tensor_product(&m2);

    assert_eq!(product, Metric::NonEuclidean(4));
}

#[test]
fn test_tensor_product_mixed() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Minkowski(2);
    let product = m1.tensor_product(&m2);

    // Euclidean(2) = (2, 0, 0) + Minkowski(2) = (1, 1, 0) = (3, 1, 0)
    assert_eq!(product.signature(), (3, 1, 0));
}

// =============================================================================
// is_compatible() tests
// =============================================================================

#[test]
fn test_compatible_same() {
    let m1 = Metric::Minkowski(4);
    let m2 = Metric::Minkowski(4);
    assert!(m1.is_compatible(&m2));
}

#[test]
fn test_compatible_different_repr() {
    // Same signature, different representation
    let m1 = Metric::Minkowski(4);
    let m2 = Metric::Generic { p: 1, q: 3, r: 0 };
    assert!(m1.is_compatible(&m2));
}

#[test]
fn test_not_compatible_different_dim() {
    let m1 = Metric::Euclidean(3);
    let m2 = Metric::Euclidean(4);
    assert!(!m1.is_compatible(&m2));
}

#[test]
fn test_not_compatible_different_signature() {
    let m1 = Metric::Euclidean(4);
    let m2 = Metric::Minkowski(4);
    assert!(!m1.is_compatible(&m2));
}

// =============================================================================
// to_generic() tests
// =============================================================================

#[test]
fn test_to_generic_euclidean() {
    let m = Metric::Euclidean(3).to_generic();
    assert_eq!(m, Metric::Generic { p: 3, q: 0, r: 0 });
}

#[test]
fn test_to_generic_minkowski() {
    let m = Metric::Minkowski(4).to_generic();
    assert_eq!(m, Metric::Generic { p: 1, q: 3, r: 0 });
}

// =============================================================================
// from_signature() tests
// =============================================================================

#[test]
fn test_from_signature_euclidean() {
    let m = Metric::from_signature(4, 0, 0);
    assert_eq!(m, Metric::Euclidean(4));
}

#[test]
fn test_from_signature_non_euclidean() {
    let m = Metric::from_signature(0, 3, 0);
    assert_eq!(m, Metric::NonEuclidean(3));
}

#[test]
fn test_from_signature_minkowski() {
    let m = Metric::from_signature(1, 3, 0);
    assert_eq!(m, Metric::Minkowski(4));
}

#[test]
fn test_from_signature_pga() {
    let m = Metric::from_signature(3, 0, 1);
    assert_eq!(m, Metric::PGA(4));
}

#[test]
fn test_from_signature_generic() {
    let m = Metric::from_signature(2, 2, 1);
    assert_eq!(m, Metric::Generic { p: 2, q: 2, r: 1 });
}

// =============================================================================
// from_signs() and to_signs() tests
// =============================================================================

#[test]
fn test_from_signs_basic() {
    let signs = [1, -1, -1, -1];
    let m = Metric::from_signs(&signs).unwrap();

    assert_eq!(m.dimension(), 4);
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(1), -1);
}

#[test]
fn test_from_signs_with_zero() {
    let signs = [0, 1, 1, 1];
    let m = Metric::from_signs(&signs).unwrap();

    assert_eq!(m.sign_of_sq(0), 0);
    assert_eq!(m.sign_of_sq(1), 1);
}

#[test]
fn test_from_signs_empty() {
    let signs: [i32; 0] = [];
    let result = Metric::from_signs(&signs);
    assert!(result.is_err());
}

#[test]
fn test_from_signs_invalid() {
    let signs = [1, 2, -1]; // 2 is invalid
    let result = Metric::from_signs(&signs);
    assert!(result.is_err());
}

#[test]
fn test_to_signs_roundtrip() {
    let original = Metric::Minkowski(4);
    let signs = original.to_signs();
    let reconstructed = Metric::from_signs(&signs).unwrap();

    assert_eq!(original.signature(), reconstructed.signature());
}

// =============================================================================
// Display tests
// =============================================================================

#[test]
fn test_display_euclidean() {
    let m = Metric::Euclidean(3);
    assert_eq!(format!("{}", m), "Euclidean(3)");
}

#[test]
fn test_display_minkowski() {
    let m = Metric::Minkowski(4);
    assert_eq!(format!("{}", m), "Minkowski(4)");
}

#[test]
fn test_display_generic() {
    let m = Metric::Generic { p: 2, q: 1, r: 1 };
    assert_eq!(format!("{}", m), "Cl(2, 1, 1)");
}

// =============================================================================
// Hash and Eq tests
// =============================================================================

#[test]
fn test_metric_hash() {
    let mut set = HashSet::new();
    set.insert(Metric::Euclidean(3));
    set.insert(Metric::Euclidean(3)); // duplicate
    set.insert(Metric::Minkowski(4));

    assert_eq!(set.len(), 2);
}

#[test]
fn test_metric_eq() {
    assert_eq!(Metric::Euclidean(3), Metric::Euclidean(3));
    assert_ne!(Metric::Euclidean(3), Metric::Euclidean(4));
    assert_ne!(Metric::Euclidean(4), Metric::Minkowski(4));
}
