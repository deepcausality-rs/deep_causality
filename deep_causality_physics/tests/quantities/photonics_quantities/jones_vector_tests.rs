/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_physics::JonesVector;
use deep_causality_tensor::CausalTensor;

#[test]
fn test_jones_vector() {
    // Right-circular polarisation: the two components differ in both parts, so a wrapper that
    // reordered or zeroed them cannot look the same as one that carried them through. The shape
    // alone could not tell those apart.
    let data = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 1.0)];
    let t = CausalTensor::new(data.clone(), vec![2]).unwrap();
    let j = JonesVector::<f64>::new(t);
    assert_eq!(j.inner().shape(), vec![2]);
    assert_eq!(j.inner().as_slice(), data.as_slice());
}

#[test]
fn test_jones_vector_default() {
    let j: JonesVector<f64> = Default::default();
    assert!(j.inner().is_empty());
}
