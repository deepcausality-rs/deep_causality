/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};

#[test]
fn test_data_getter() {
    let metric = Metric::Euclidean(2);
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let mv = CausalMultiVector::new(data.clone(), metric).unwrap();

    assert_eq!(mv.data(), &data);
}

#[test]
fn test_metric_getter() {
    let metric = Metric::Euclidean(3);
    let data = vec![1.0; 8];
    let mv = CausalMultiVector::new(data, metric).unwrap();

    assert_eq!(mv.metric(), metric);
}
