/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Length, RayAngle, Wavelength, grating_equation_kernel, single_slit_irradiance_kernel,
};
use std::f64::consts::PI;

#[test]
fn test_single_slit() {
    let i0 = 1.0;
    let a = Length::new(1.0).unwrap();
    let lambda = Wavelength::new(1.0).unwrap();

    // Theta = 0 -> beta = 0 -> sinc(0)=1 -> I=I0
    let theta0 = RayAngle::new(0.0).unwrap();
    let res0 = single_slit_irradiance_kernel(i0, a, theta0, lambda);
    assert!((res0.unwrap() - 1.0).abs() < 1e-10);

    // First min: a sin theta = lambda -> sin theta = 1 -> theta = pi/2
    let theta_min = RayAngle::new(PI / 2.0).unwrap();
    let res_min = single_slit_irradiance_kernel(i0, a, theta_min, lambda);
    assert!(res_min.unwrap().abs() < 1e-10);
}

#[test]
fn test_grating() {
    // d sin theta = m lambda
    let d = Length::new(2.0).unwrap();
    let m = 1;
    let lambda = Wavelength::new(1.0).unwrap();
    let inc = RayAngle::new(0.0).unwrap();

    let res = grating_equation_kernel(d, m, inc, lambda);
    assert!(res.is_ok());
    let angle = res.unwrap();

    // sin theta = 1/2 = 0.5 -> theta = 30 deg = pi/6
    assert!((angle.value() - PI / 6.0).abs() < 1e-10);
}

#[test]
fn test_single_slit_errors() {
    let a = Length::new(1.0).unwrap();
    let theta = RayAngle::new(0.0).unwrap();
    let lambda_valid = Wavelength::new(1.0).unwrap();

    // i0 < 0
    assert!(single_slit_irradiance_kernel(-1.0, a, theta, lambda_valid).is_err());

    // lambda = 0
    let lambda_zero = Wavelength::new_unchecked(0.0);
    assert!(single_slit_irradiance_kernel(1.0, a, theta, lambda_zero).is_err());
}

#[test]
fn test_grating_errors() {
    let pitch_valid = Length::new(1.0).unwrap();
    let lambda = Wavelength::new(1.0).unwrap();
    let inc = RayAngle::new(0.0).unwrap();

    // pitch <= 0
    let pitch_invalid = Length::new_unchecked(0.0);
    assert!(grating_equation_kernel(pitch_invalid, 1, inc, lambda).is_err());

    // sin_theta_m > 1
    // m*lambda/d = 2*1/1 = 2
    assert!(grating_equation_kernel(pitch_valid, 2, inc, lambda).is_err());
}
