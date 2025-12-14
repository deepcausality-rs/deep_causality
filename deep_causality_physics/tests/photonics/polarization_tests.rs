/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    JonesVector, RayAngle, StokesVector, degree_of_polarization_kernel, jones_rotation_kernel,
    stokes_from_jones_kernel,
};
use deep_causality_tensor::CausalTensor;
use std::f64::consts::PI;

#[test]
fn test_jones_rotation() {
    // Horizontal H = [1, 0]
    // Rotate 90 deg -> Vertical V = [0, 1]
    // But kernel rotates the MATRIX operator, not vector.
    // Let's test rotating an Identity operator? R(-t) I R(t) = I.

    let id_data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
    ];
    let id = CausalTensor::new(id_data, vec![2, 2]).unwrap();

    let angle = RayAngle::new(PI / 2.0).unwrap();
    let res = jones_rotation_kernel(&id, angle);
    assert!(res.is_ok());
    let rot = res.unwrap();
    // Should still be Identity
    let d = rot.data();
    assert!((d[0].re - 1.0).abs() < 1e-10);
    assert!((d[3].re - 1.0).abs() < 1e-10);
}

#[test]
fn test_stokes_from_jones() {
    // H = [1, 0]. Stokes = [1, 1, 0, 0]
    let j_data = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let j_tensor = CausalTensor::new(j_data, vec![2]).unwrap();
    let jones = JonesVector::new(j_tensor);

    let res = stokes_from_jones_kernel(&jones);
    assert!(res.is_ok());
    let s = res.unwrap();
    let d = s.inner().data();

    assert!((d[0] - 1.0).abs() < 1e-10); // S0
    assert!((d[1] - 1.0).abs() < 1e-10); // S1
    assert!((d[2] - 0.0).abs() < 1e-10);
    assert!((d[3] - 0.0).abs() < 1e-10);
}

#[test]
fn test_dop() {
    // Fully polarized [1, 1, 0, 0]
    let s_data = vec![1.0, 1.0, 0.0, 0.0];
    let s_tensor = CausalTensor::new(s_data, vec![4]).unwrap();
    let stokes = StokesVector::new(s_tensor).unwrap();

    let res = degree_of_polarization_kernel(&stokes);
    assert!(res.is_ok());
    assert!((res.unwrap().value() - 1.0).abs() < 1e-10);

    // Unpolarized [1, 0, 0, 0]
    let s_unpol =
        StokesVector::new(CausalTensor::new(vec![1.0, 0.0, 0.0, 0.0], vec![4]).unwrap()).unwrap();
    let res2 = degree_of_polarization_kernel(&s_unpol);
    assert!((res2.unwrap().value() - 0.0).abs() < 1e-10);
}
