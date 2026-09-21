/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{LorentzianSpacetime, MetricSignature, TimeScale};
use deep_causality_metric::{Metric, detect_convention, is_lorentzian};

#[test]
fn test_reports_a_four_dimensional_lorentzian_signature() {
    let l = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    assert_eq!(l.metric(), Metric::Lorentzian(4));
}

#[test]
fn test_the_signature_does_not_depend_on_the_coordinates() {
    let here = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let there = LorentzianSpacetime::new(2, 9.0, -3.0, 7.5, 42.0, TimeScale::Nanoseconds);
    assert_eq!(here.metric(), there.metric());
}

#[test]
fn test_it_is_lorentzian_in_the_east_coast_convention() {
    // (−,+,+,+): the convention this type documents and its interval computes.
    let l = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    assert!(is_lorentzian(&l.metric()));
    assert_eq!(detect_convention(&l.metric()), Some(true));
}

#[test]
fn test_it_differs_from_the_newtonian_signature() {
    use deep_causality_context::EuclideanSpacetime;

    let relativistic = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let newtonian = EuclideanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    assert_ne!(relativistic.metric(), newtonian.metric());
}
