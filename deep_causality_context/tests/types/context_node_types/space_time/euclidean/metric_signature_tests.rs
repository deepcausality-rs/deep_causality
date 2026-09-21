/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{EuclideanSpacetime, MetricSignature, TimeScale};
use deep_causality_metric::{Metric, detect_convention, is_lorentzian};

#[test]
fn test_reports_a_four_dimensional_euclidean_signature() {
    let e = EuclideanSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    assert_eq!(e.metric(), Metric::Euclidean(4));
}

#[test]
fn test_the_signature_does_not_depend_on_the_coordinates() {
    // A signature is a property of the manifold, so two events at different places and times in
    // the same spacetime report the same one.
    let here = EuclideanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let there = EuclideanSpacetime::new(2, 9.0, -3.0, 7.5, 42.0, TimeScale::Millisecond);
    assert_eq!(here.metric(), there.metric());
}

#[test]
fn test_it_is_not_lorentzian_and_has_no_sign_convention() {
    // The Newtonian case: an absolute clock, so the time axis squares positive like the spatial
    // ones and neither sign convention applies.
    let e = EuclideanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    assert!(!is_lorentzian(&e.metric()));
    assert_eq!(detect_convention(&e.metric()), None);
}
