/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    BeamWaist, FocalLength, NumericalAperture, OpticalPower, RayAngle, RayHeight, Wavelength,
};

#[test]
fn test_photonics_scalars_traits() {
    let f = FocalLength::<f64>::new(1.0).unwrap();
    assert_eq!(f, f.clone());
    let _ = format!("{:?}", f);

    let p = OpticalPower::<f64>::new(2.0).unwrap();
    assert_eq!(p, p.clone());
    let _ = format!("{:?}", p);

    let w = Wavelength::<f64>::new(500e-9).unwrap();
    assert_eq!(w, w.clone());
    assert!(w < Wavelength::<f64>::new(600e-9).unwrap());
    let _ = format!("{:?}", w);

    let na = NumericalAperture::<f64>::new(0.5).unwrap();
    assert_eq!(na, na.clone());
    let _ = format!("{:?}", na);

    let bw = BeamWaist::<f64>::new(1e-6).unwrap();
    assert_eq!(bw, bw.clone());
    let _ = format!("{:?}", bw);

    let rh = RayHeight::<f64>::new(0.01).unwrap();
    assert_eq!(rh, rh.clone());
    let _ = format!("{:?}", rh);

    let ra = RayAngle::<f64>::new(0.1).unwrap();
    assert_eq!(ra, ra.clone());
    let _ = format!("{:?}", ra);
}
