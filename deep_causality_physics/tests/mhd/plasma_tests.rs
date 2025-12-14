/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Mass, PhysicalField, Speed, Temperature, debye_length_kernel, larmor_radius_kernel,
};

#[test]
fn test_debye_length() {
    let t = Temperature::new(100.0).unwrap();
    let n = 1e18;
    let eps0 = 8.854e-12;
    let e = 1.602e-19;

    let res = debye_length_kernel(t, n, eps0, e);
    assert!(res.is_ok());
    assert!(res.unwrap().value() > 0.0);
}

#[test]
fn test_larmor_radius() {
    let m = Mass::new(1.0).unwrap();
    let v = Speed::new(10.0).unwrap();
    let q = 1.0;

    let b_vec = CausalMultiVector::new(vec![0.0, 5.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b = PhysicalField::new(b_vec);

    let res = larmor_radius_kernel(m, v, q, &b);
    assert!(res.is_ok());
    // r = mv/qB = 1*10 / 1*5 = 2
    assert!((res.unwrap().value() - 2.0).abs() < 1e-10);
}
