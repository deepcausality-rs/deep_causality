/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{SpacetimeInterval, SpacetimeVector};

// =============================================================================
// SpacetimeInterval Tests
// =============================================================================

#[test]
fn test_spacetime_interval_new_valid() {
    let interval = SpacetimeInterval::new(100.0);
    assert!(interval.is_ok());
    assert!((interval.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_spacetime_interval_new_negative() {
    // Negative intervals are spacelike
    let interval = SpacetimeInterval::new(-25.0);
    assert!(interval.is_ok());
    assert!((interval.unwrap().value() - (-25.0)).abs() < 1e-10);
}

#[test]
fn test_spacetime_interval_default() {
    let interval = SpacetimeInterval::default();
    assert!((interval.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_spacetime_interval_into_f64() {
    let interval = SpacetimeInterval::new_unchecked(42.0);
    let val: f64 = interval.into();
    assert!((val - 42.0).abs() < 1e-10);
}

// =============================================================================
// SpacetimeVector Tests
// =============================================================================

#[test]
fn test_spacetime_vector_default() {
    let vec = SpacetimeVector::default();
    // Default uses Minkowski(0) metric
    assert_eq!(vec.inner().metric(), Metric::Minkowski(0));
}

#[test]
fn test_spacetime_vector_new_and_accessors() {
    let mv = CausalMultiVector::new(
        vec![
            1.0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let sv = SpacetimeVector::new(mv.clone());

    assert_eq!(sv.inner().data(), mv.data());

    let inner = sv.into_inner();
    assert_eq!(inner.data(), mv.data());
}
