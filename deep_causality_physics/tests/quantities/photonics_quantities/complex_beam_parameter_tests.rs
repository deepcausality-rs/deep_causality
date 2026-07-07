/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_physics::ComplexBeamParameter;

#[test]
fn test_complex_beam_parameter() {
    let q = ComplexBeamParameter::<f64>::new(Complex::new(1.0, 2.0)).unwrap();
    assert_eq!(q.value(), Complex::new(1.0, 2.0));

    // Im(q) must be positive
    let err = ComplexBeamParameter::<f64>::new(Complex::new(1.0, -1.0));
    assert!(err.is_err());
}

#[test]
fn test_complex_beam_parameter_new_unchecked() {
    let q = ComplexBeamParameter::<f64>::new_unchecked(Complex::new(1.0, 2.0));
    assert_eq!(q.value(), Complex::new(1.0, 2.0));
}

#[test]
fn test_complex_beam_parameter_default() {
    let q: ComplexBeamParameter<f64> = Default::default();
    assert_eq!(q.value(), Complex::new(0.0, 0.0));
}
