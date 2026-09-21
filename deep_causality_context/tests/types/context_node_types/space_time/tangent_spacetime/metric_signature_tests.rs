/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{MetricSignature, MetricTensor4D, TangentSpacetime};
use deep_causality_metric::{Metric, detect_convention, is_lorentzian};

#[test]
fn test_reports_a_four_dimensional_lorentzian_signature() {
    let t = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(t.metric(), Metric::Lorentzian(4));
    assert!(is_lorentzian(&t.metric()));
    assert_eq!(detect_convention(&t.metric()), Some(true));
}

#[test]
fn test_the_signature_survives_replacing_every_tensor_component() {
    // This is the case the type exists for: a numerically evolved metric whose components all
    // change while the signature, which is invariant under continuous evolution, does not.
    let mut t = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    let before = t.metric();

    t.update_metric_tensor([
        [-9.0, 0.5, 0.25, 0.125],
        [0.5, 3.0, 0.75, 0.5],
        [0.25, 0.75, 3.0, 0.25],
        [0.125, 0.5, 0.25, 3.0],
    ]);

    assert_ne!(t.metric_tensor()[0][0], -1.0);
    assert_eq!(t.metric(), before);
    assert_eq!(t.metric(), Metric::Lorentzian(4));
}
