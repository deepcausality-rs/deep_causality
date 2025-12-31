/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{
    EastCoastMetric, LorentzianMetric, Metric, WestCoastMetric, detect_convention, east_to_west,
    is_lorentzian, west_to_east,
};

// =============================================================================
// Conversion function tests
// =============================================================================

#[test]
fn test_west_to_east() {
    let west = Metric::Minkowski(4);
    let east = west_to_east(&west).unwrap();

    assert!(east.is_east_coast());
    assert_eq!(east.time_sign(), -1);
}

#[test]
fn test_east_to_west() {
    let east = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    let west = east_to_west(&east).unwrap();

    assert!(west.is_west_coast());
    assert_eq!(west.time_sign(), 1);
}

// =============================================================================
// detect_convention tests
// =============================================================================

#[test]
fn test_detect_east_coast() {
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    assert_eq!(detect_convention(&metric), Some(true));
}

#[test]
fn test_detect_west_coast() {
    let metric = Metric::Minkowski(4);
    assert_eq!(detect_convention(&metric), Some(false));
}

#[test]
fn test_detect_euclidean() {
    let metric = Metric::Euclidean(4);
    assert_eq!(detect_convention(&metric), None);
}

#[test]
fn test_detect_small_dim() {
    let metric = Metric::Euclidean(1);
    assert_eq!(detect_convention(&metric), None);
}

// =============================================================================
// is_lorentzian tests
// =============================================================================

#[test]
fn test_is_lorentzian_minkowski() {
    assert!(is_lorentzian(&Metric::Minkowski(4)));
}

#[test]
fn test_is_lorentzian_east_coast() {
    let metric = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    assert!(is_lorentzian(&metric));
}

#[test]
fn test_is_not_lorentzian_euclidean() {
    assert!(!is_lorentzian(&Metric::Euclidean(4)));
}

#[test]
fn test_is_not_lorentzian_pga() {
    // PGA has a degenerate direction, not a proper Lorentzian
    assert!(!is_lorentzian(&Metric::PGA(4)));
}

// =============================================================================
// Cross-convention tests
// =============================================================================

#[test]
fn test_east_west_roundtrip() {
    let original = EastCoastMetric::minkowski_4d();
    let west = WestCoastMetric::from_east_coast(original.into_metric()).unwrap();
    let back = EastCoastMetric::from_west_coast(west.into_metric()).unwrap();

    assert_eq!(original.signature(), back.signature());
}

#[test]
fn test_generic_function_with_trait() {
    fn process_metric<M: LorentzianMetric>(m: M) -> i32 {
        m.time_sign() + m.space_sign()
    }

    // East Coast: -1 + 1 = 0
    assert_eq!(process_metric(EastCoastMetric::minkowski_4d()), 0);

    // West Coast: 1 + (-1) = 0
    assert_eq!(process_metric(WestCoastMetric::minkowski_4d()), 0);
}
