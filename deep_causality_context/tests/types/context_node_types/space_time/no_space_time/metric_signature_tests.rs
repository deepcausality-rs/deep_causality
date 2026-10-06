/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{Coordinate, FloatType, MetricSignature, NoSpaceTime};
use deep_causality_metric::{Metric, detect_convention, is_lorentzian};

#[test]
fn test_reports_a_zero_dimensional_signature() {
    // No axes, so nothing to give a sign to. The dimension agrees with the coordinate system.
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert_eq!(empty.metric(), Metric::Euclidean(0));
    assert_eq!(empty.metric().dimension(), empty.dimension());
}

#[test]
fn test_it_is_neither_lorentzian_nor_of_any_convention() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert!(!is_lorentzian(&empty.metric()));
    assert_eq!(detect_convention(&empty.metric()), None);
}
