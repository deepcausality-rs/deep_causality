/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::QuantumMetric;

#[test]
fn test_quantum_metric() {
    let qm = QuantumMetric::<f64>::new(-0.5).unwrap();
    assert_eq!(qm.value(), -0.5);
    let val: f64 = qm.into();
    assert_eq!(val, -0.5);
}

#[test]
fn test_quantum_metric_default() {
    let qm: QuantumMetric<f64> = Default::default();
    assert_eq!(qm.value(), 0.0);
}
