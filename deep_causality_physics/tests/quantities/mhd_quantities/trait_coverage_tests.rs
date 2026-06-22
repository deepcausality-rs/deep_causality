/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AlfvenSpeed, Conductivity, DebyeLength, Diffusivity, LarmorRadius, MagneticPressure,
    PlasmaBeta, PlasmaFrequency,
};

#[test]
fn test_mhd_scalars_into_f64() {
    let v: f64 = AlfvenSpeed::<f64>::new(100.0).unwrap().into();
    assert_eq!(v, 100.0);

    let v: f64 = PlasmaBeta::<f64>::new(0.5).unwrap().into();
    assert_eq!(v, 0.5);

    let v: f64 = MagneticPressure::<f64>::new(1000.0).unwrap().into();
    assert_eq!(v, 1000.0);

    let v: f64 = LarmorRadius::<f64>::new(1.0).unwrap().into();
    assert_eq!(v, 1.0);

    let v: f64 = DebyeLength::<f64>::new(1e-6).unwrap().into();
    assert_eq!(v, 1e-6);

    let v: f64 = PlasmaFrequency::<f64>::new(1e9).unwrap().into();
    assert_eq!(v, 1e9);

    let v: f64 = Conductivity::<f64>::new(1e7).unwrap().into();
    assert_eq!(v, 1e7);

    let v: f64 = Diffusivity::<f64>::new(1.0).unwrap().into();
    assert_eq!(v, 1.0);
}

#[test]
fn test_mhd_scalars_traits() {
    let a = AlfvenSpeed::<f64>::new(1.0).unwrap();
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a, a.clone());
    assert!(a < AlfvenSpeed::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", a);

    let pb = PlasmaBeta::<f64>::new(0.5).unwrap();
    assert!(pb < PlasmaBeta::<f64>::new(1.0).unwrap());
    let _ = format!("{:?}", pb);

    let mp = MagneticPressure::<f64>::new(100.0).unwrap();
    assert!(mp < MagneticPressure::<f64>::new(200.0).unwrap());
    let _ = format!("{:?}", mp);

    let lr = LarmorRadius::<f64>::new(1.0).unwrap();
    assert!(lr < LarmorRadius::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", lr);

    let dl = DebyeLength::<f64>::new(1.0).unwrap();
    assert!(dl < DebyeLength::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", dl);

    let pf = PlasmaFrequency::<f64>::new(1.0).unwrap();
    assert!(pf < PlasmaFrequency::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", pf);

    let c = Conductivity::<f64>::new(1.0).unwrap();
    assert!(c < Conductivity::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", c);

    let d = Diffusivity::<f64>::new(1.0).unwrap();
    assert!(d < Diffusivity::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", d);
}
