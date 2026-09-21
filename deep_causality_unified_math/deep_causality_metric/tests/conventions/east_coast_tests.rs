/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{EastCoastMetric, LorentzianMetric, Metric, MetricError};

// =============================================================================
// Construction tests
// =============================================================================

#[test]
fn test_east_coast_minkowski_4d_constant() {
    let m = EastCoastMetric::MINKOWSKI_4D;
    assert_eq!(m.dimension(), 4);
    assert_eq!(m.time_sign(), -1);
    assert_eq!(m.space_sign(), 1);
}

#[test]
fn test_east_coast_minkowski_3d_constant() {
    let m = EastCoastMetric::MINKOWSKI_3D;
    assert_eq!(m.dimension(), 3);
    assert!(m.is_east_coast());
}

#[test]
fn test_east_coast_new_valid() {
    // Create a valid East Coast metric manually
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    let east = EastCoastMetric::new(metric);
    assert!(east.is_ok());
}

#[test]
fn test_east_coast_new_invalid_time() {
    // West Coast metric should be rejected
    let metric = Metric::Minkowski(4); // (+---)
    let east = EastCoastMetric::new(metric);
    assert!(east.is_err());
}

#[test]
fn test_east_coast_new_invalid_dim() {
    let metric = Metric::Euclidean(1); // Only 1 dimension
    let east = EastCoastMetric::new(metric);
    assert!(east.is_err());
}

#[test]
fn test_east_coast_from_west_coast() {
    let west = Metric::Minkowski(4);
    let east = EastCoastMetric::from_west_coast(west);
    assert!(east.is_ok());

    let east = east.unwrap();
    assert_eq!(east.time_sign(), -1);
    assert_eq!(east.space_sign(), 1);
}

#[test]
fn test_east_coast_new_nd() {
    let east = EastCoastMetric::new_nd(5).unwrap();
    assert_eq!(east.dimension(), 5);
    assert_eq!(east.as_metric().sign_of_sq(0), -1);
    for i in 1..5 {
        assert_eq!(east.as_metric().sign_of_sq(i), 1);
    }
}

#[test]
fn test_east_coast_new_nd_invalid() {
    assert!(EastCoastMetric::new_nd(1).is_err());
    assert!(EastCoastMetric::new_nd(65).is_err());
}

// =============================================================================
// LorentzianMetric trait tests
// =============================================================================

#[test]
fn test_east_coast_trait_minkowski_4d() {
    let m = EastCoastMetric::minkowski_4d();
    assert_eq!(m, EastCoastMetric::MINKOWSKI_4D);
}

#[test]
fn test_east_coast_trait_minkowski_3d() {
    let m = EastCoastMetric::minkowski_3d();
    assert_eq!(m, EastCoastMetric::MINKOWSKI_3D);
}

#[test]
fn test_east_coast_trait_is_east_coast() {
    let m = EastCoastMetric::minkowski_4d();
    assert!(m.is_east_coast());
    assert!(!m.is_west_coast());
}

#[test]
fn test_east_coast_trait_signs() {
    let m = EastCoastMetric::minkowski_4d();
    assert_eq!(m.time_sign(), -1);
    assert_eq!(m.space_sign(), 1);
}

#[test]
fn test_east_coast_trait_signature() {
    let m = EastCoastMetric::minkowski_4d();
    // East Coast 4D: (-+++) = (3, 1, 0)
    assert_eq!(m.signature(), (3, 1, 0));
}

#[test]
fn test_east_coast_into_metric() {
    let east = EastCoastMetric::minkowski_4d();
    let metric = east.into_metric();
    assert_eq!(metric.dimension(), 4);
}

#[test]
fn test_east_coast_inner() {
    let east = EastCoastMetric::minkowski_4d();
    let inner = east.inner();
    assert_eq!(inner.dimension(), 4);
}

// =============================================================================
// Trait bound tests
// =============================================================================

#[test]
fn test_east_coast_clone() {
    let m1 = EastCoastMetric::minkowski_4d();
    let m2 = m1;
    assert_eq!(m1, m2);
}

#[test]
fn test_east_coast_copy() {
    let m1 = EastCoastMetric::minkowski_4d();
    let m2 = m1; // Copy
    assert_eq!(m1, m2);
}

#[test]
fn test_east_coast_debug() {
    let m = EastCoastMetric::minkowski_4d();
    let debug = format!("{:?}", m);
    assert!(debug.contains("EastCoastMetric"));
}

#[test]
fn test_east_coast_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(EastCoastMetric::MINKOWSKI_4D);
    set.insert(EastCoastMetric::MINKOWSKI_4D);
    set.insert(EastCoastMetric::MINKOWSKI_3D);
    assert_eq!(set.len(), 2);
}

// =============================================================================
// Space-dimension refusal
// =============================================================================

#[test]
fn test_east_coast_new_rejects_space_dimension_with_the_wrong_sign() {
    // Time is -1, so the construction clears the first check and is refused only by the loop over
    // the space dimensions. NonEuclidean(4) is (----): e0 is right and e1..e3 are not.
    let err = EastCoastMetric::new(Metric::NonEuclidean(4))
        .expect_err("(----) is not East Coast in its space dimensions");

    assert_eq!(
        err,
        MetricError::sign_convention_mismatch(
            "East Coast convention requires space dimensions to have sign +1"
        ),
        "the refusal must name the space dimensions, not the time dimension"
    );
}

#[test]
fn test_east_coast_new_rejects_a_single_wrong_space_dimension() {
    // Only the last generator is wrong, so a loop that stops early accepts this metric.
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b1001, // e0 = -1 (correct), e3 = -1 (wrong)
        zero_mask: 0,
    };
    assert_eq!(metric.sign_of_sq(0), -1);
    assert_eq!(metric.sign_of_sq(1), 1);
    assert_eq!(metric.sign_of_sq(2), 1);
    assert_eq!(metric.sign_of_sq(3), -1);

    let err = EastCoastMetric::new(metric).expect_err("e3 = -1 is not East Coast");
    assert_eq!(
        err,
        MetricError::sign_convention_mismatch(
            "East Coast convention requires space dimensions to have sign +1"
        )
    );
}

#[test]
fn test_east_coast_new_accepts_a_degenerate_space_dimension() {
    // The loop admits 0 beside +1, so a null space generator is not a convention violation.
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,  // e0 = -1
        zero_mask: 0b1000, // e3 = 0
    };
    let east = EastCoastMetric::new(metric).expect("a null space generator is admitted");
    assert_eq!(east.dimension(), 4);
    assert_eq!(east.signature(), (2, 1, 1));
}
