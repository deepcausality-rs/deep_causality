/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Trait coverage: Debug / Clone / Copy / PartialEq / PartialOrd across all dynamics scalars.

use deep_causality_physics::{
    Acceleration, Area, Force, Frequency, Length, Mass, MomentOfInertia, Speed, Torque, Volume,
};

#[test]
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_dynamics_scalars_traits() {
    let m1 = Mass::<f64>::new(1.0).unwrap();
    let m2 = m1;
    let m3 = m1.clone();
    assert_eq!(m1, m2);
    assert_eq!(m1, m3);
    assert!(m1 < Mass::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", m1);

    let s = Speed::<f64>::new(3.0).unwrap();
    assert!(s < Speed::<f64>::new(4.0).unwrap());
    let _ = format!("{:?}", s);

    let a = Acceleration::<f64>::new(-1.0).unwrap();
    let _ = format!("{:?}", a);
    assert_eq!(a.clone(), a);

    let f = Force::<f64>::new(-5.0).unwrap();
    let _ = format!("{:?}", f);
    assert_eq!(f, f.clone());

    let t = Torque::<f64>::new(2.0).unwrap();
    let _ = format!("{:?}", t);
    assert_eq!(t, t.clone());

    let l = Length::<f64>::new(2.0).unwrap();
    assert!(l < Length::<f64>::new(3.0).unwrap());
    let _ = format!("{:?}", l);

    let ar = Area::<f64>::new(4.0).unwrap();
    assert!(ar < Area::<f64>::new(5.0).unwrap());
    let _ = format!("{:?}", ar);

    let v = Volume::<f64>::new(1.0).unwrap();
    assert!(v < Volume::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", v);

    let i = MomentOfInertia::<f64>::new(0.5).unwrap();
    assert!(i < MomentOfInertia::<f64>::new(1.0).unwrap());
    let _ = format!("{:?}", i);

    let fr = Frequency::<f64>::new(60.0).unwrap();
    assert!(fr < Frequency::<f64>::new(120.0).unwrap());
    let _ = format!("{:?}", fr);
}
