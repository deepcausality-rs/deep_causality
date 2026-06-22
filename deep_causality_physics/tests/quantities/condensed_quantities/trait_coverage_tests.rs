/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    BandDrudeWeight, BerryCurvature, Conductance, Mobility, OrbitalAngularMomentum, QuantumMetric,
    TwistAngle,
};

#[test]
fn test_condensed_scalars_traits() {
    let qm = QuantumMetric::<f64>::new(1.0).unwrap();
    assert_eq!(qm, qm.clone());
    let _ = format!("{:?}", qm);

    let bc = BerryCurvature::<f64>::new(1.0).unwrap();
    assert_eq!(bc, bc.clone());
    let _ = format!("{:?}", bc);

    let bdw = BandDrudeWeight::<f64>::new(1.0).unwrap();
    assert_eq!(bdw, bdw.clone());
    let _ = format!("{:?}", bdw);

    let oam = OrbitalAngularMomentum::<f64>::new(1.0).unwrap();
    assert_eq!(oam, oam.clone());
    let _ = format!("{:?}", oam);

    let c = Conductance::<f64>::new(1.0).unwrap();
    assert_eq!(c, c.clone());
    assert!(c < Conductance::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", c);

    let m = Mobility::<f64>::new(1.0).unwrap();
    assert_eq!(m, m.clone());
    let _ = format!("{:?}", m);

    let ta = TwistAngle::<f64>::new(0.5).unwrap();
    assert_eq!(ta, ta.clone());
    let _ = format!("{:?}", ta);
}
