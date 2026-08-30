/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};

#[test]
fn test_normalize() {
    let data = vec![3.0, 4.0, 0.0, 0.0];
    let mv = CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap();

    let normalized = mv.normalize();
    // Magnitude should be 5. Normalized should be 3/5, 4/5.
    let d = normalized.data();
    assert!((d[0] - 0.6_f64).abs() < 1e-6);
    assert!((d[1] - 0.8_f64).abs() < 1e-6);

    // Normalizing zero vector
    let zero: CausalMultiVector<f64> = CausalMultiVector::zero(Metric::Euclidean(2));
    let norm_zero = zero.normalize();
    assert!((norm_zero.data()[0]).abs() < 1e-6);
}

#[test]
fn test_geometric_product_general() {
    // 2D Euclidean: e1*e1 = 1, e2*e2=1, e1*e2 = e12
    let metric = Metric::Euclidean(2);
    // mv1 = 2*e1
    let mv1 = CausalMultiVector::new(vec![0.0, 2.0, 0.0, 0.0], metric).unwrap();
    // mv2 = 3*e2
    let mv2 = CausalMultiVector::new(vec![0.0, 0.0, 3.0, 0.0], metric).unwrap();

    // e1 * e2 = e12. Result should be 6*e12.
    // e12 is usually index 3 in usual ordering (1, e1, e2, e12)

    let res = mv1.geometric_product_general(&mv2);
    let d = res.data();
    assert_eq!(d[3], 6.0);

    // Test non-commutativity if possible, or at least that it runs.
    // e2 * e1 = -e12
    let res2 = mv2.geometric_product_general(&mv1);
    let d2 = res2.data();
    assert_eq!(d2[3], -6.0);
}
