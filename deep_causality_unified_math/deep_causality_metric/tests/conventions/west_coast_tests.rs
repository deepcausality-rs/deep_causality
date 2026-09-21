/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{LorentzianMetric, Metric, MetricError, WestCoastMetric};

// =============================================================================
// Construction tests
// =============================================================================

#[test]
fn test_west_coast_minkowski_4d_constant() {
    let m = WestCoastMetric::MINKOWSKI_4D;
    assert_eq!(m.dimension(), 4);
    assert_eq!(m.time_sign(), 1);
    assert_eq!(m.space_sign(), -1);
}

#[test]
fn test_west_coast_minkowski_3d_constant() {
    let m = WestCoastMetric::MINKOWSKI_3D;
    assert_eq!(m.dimension(), 3);
    assert!(m.is_west_coast());
}

#[test]
fn test_west_coast_new_valid() {
    let metric = Metric::Minkowski(4); // (+---)
    let west = WestCoastMetric::new(metric);
    assert!(west.is_ok());
}

#[test]
fn test_west_coast_new_invalid_time() {
    // East Coast metric should be rejected
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    let west = WestCoastMetric::new(metric);
    assert!(west.is_err());
}

#[test]
fn test_west_coast_new_invalid_dim() {
    let metric = Metric::Euclidean(1);
    let west = WestCoastMetric::new(metric);
    assert!(west.is_err());
}

#[test]
fn test_west_coast_from_east_coast() {
    let east = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    let west = WestCoastMetric::from_east_coast(east);
    assert!(west.is_ok());

    let west = west.unwrap();
    assert_eq!(west.time_sign(), 1);
    assert_eq!(west.space_sign(), -1);
}

#[test]
fn test_west_coast_new_nd() {
    let west = WestCoastMetric::new_nd(5).unwrap();
    assert_eq!(west.dimension(), 5);
    assert_eq!(west.as_metric().sign_of_sq(0), 1);
    for i in 1..5 {
        assert_eq!(west.as_metric().sign_of_sq(i), -1);
    }
}

#[test]
fn test_west_coast_new_nd_invalid() {
    assert!(WestCoastMetric::new_nd(1).is_err());
    assert!(WestCoastMetric::new_nd(65).is_err());
}

// =============================================================================
// LorentzianMetric trait tests
// =============================================================================

#[test]
fn test_west_coast_trait_minkowski_4d() {
    let m = WestCoastMetric::minkowski_4d();
    assert_eq!(m, WestCoastMetric::MINKOWSKI_4D);
}

#[test]
fn test_west_coast_trait_minkowski_3d() {
    let m = WestCoastMetric::minkowski_3d();
    assert_eq!(m, WestCoastMetric::MINKOWSKI_3D);
}

#[test]
fn test_west_coast_trait_is_west_coast() {
    let m = WestCoastMetric::minkowski_4d();
    assert!(m.is_west_coast());
    assert!(!m.is_east_coast());
}

#[test]
fn test_west_coast_trait_signs() {
    let m = WestCoastMetric::minkowski_4d();
    assert_eq!(m.time_sign(), 1);
    assert_eq!(m.space_sign(), -1);
}

#[test]
fn test_west_coast_trait_signature() {
    let m = WestCoastMetric::minkowski_4d();
    // West Coast 4D: (+---) = (1, 3, 0)
    assert_eq!(m.signature(), (1, 3, 0));
}

#[test]
fn test_west_coast_into_metric() {
    let west = WestCoastMetric::minkowski_4d();
    let metric = west.into_metric();
    assert_eq!(metric.dimension(), 4);
}

#[test]
fn test_west_coast_inner() {
    let west = WestCoastMetric::minkowski_4d();
    let inner = west.inner();
    assert_eq!(inner.dimension(), 4);
}

// =============================================================================
// Trait bound tests
// =============================================================================

#[test]
fn test_west_coast_clone() {
    let m1 = WestCoastMetric::minkowski_4d();
    let m2 = m1;
    assert_eq!(m1, m2);
}

#[test]
fn test_west_coast_copy() {
    let m1 = WestCoastMetric::minkowski_4d();
    let m2 = m1; // Copy
    assert_eq!(m1, m2);
}

#[test]
fn test_west_coast_debug() {
    let m = WestCoastMetric::minkowski_4d();
    let debug = format!("{:?}", m);
    assert!(debug.contains("WestCoastMetric"));
}

#[test]
fn test_west_coast_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(WestCoastMetric::MINKOWSKI_4D);
    set.insert(WestCoastMetric::MINKOWSKI_4D);
    set.insert(WestCoastMetric::MINKOWSKI_3D);
    assert_eq!(set.len(), 2);
}

// =============================================================================
// Space-dimension refusal
// =============================================================================

#[test]
fn test_west_coast_new_rejects_space_dimension_with_the_wrong_sign() {
    // Time is +1, so the construction clears the first check and is refused only by the loop over
    // the space dimensions. Euclidean(4) is (++++): e0 is right and e1..e3 are not.
    let err = WestCoastMetric::new(Metric::Euclidean(4))
        .expect_err("(++++) is not West Coast in its space dimensions");

    assert_eq!(
        err,
        MetricError::sign_convention_mismatch(
            "West Coast convention requires space dimensions to have sign -1"
        ),
        "the refusal must name the space dimensions, not the time dimension"
    );
}

#[test]
fn test_west_coast_new_rejects_a_single_wrong_space_dimension() {
    // Only the last generator is wrong, so a loop that stops early accepts this metric.
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b0110, // e0 = +1 (correct), e1, e2 = -1 (correct), e3 = +1 (wrong)
        zero_mask: 0,
    };
    assert_eq!(metric.sign_of_sq(0), 1);
    assert_eq!(metric.sign_of_sq(1), -1);
    assert_eq!(metric.sign_of_sq(2), -1);
    assert_eq!(metric.sign_of_sq(3), 1);

    let err = WestCoastMetric::new(metric).expect_err("e3 = +1 is not West Coast");
    assert_eq!(
        err,
        MetricError::sign_convention_mismatch(
            "West Coast convention requires space dimensions to have sign -1"
        )
    );
}

#[test]
fn test_west_coast_new_accepts_a_degenerate_space_dimension() {
    // The loop admits 0 beside -1, so a null space generator is not a convention violation.
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b0110, // e1, e2 = -1
        zero_mask: 0b1000, // e3 = 0
    };
    let west = WestCoastMetric::new(metric).expect("a null space generator is admitted");
    assert_eq!(west.dimension(), 4);
    assert_eq!(west.signature(), (1, 2, 1));
}
