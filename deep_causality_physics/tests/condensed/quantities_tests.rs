/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    BandDrudeWeight, BerryCurvature, Conductance, Mobility, OrbitalAngularMomentum, OrderParameter,
    PhysicsErrorEnum, QuantumMetric, TwistAngle,
};

#[test]
fn test_quantum_metric() {
    let qm = QuantumMetric::new(-0.5).unwrap();
    assert_eq!(qm.value(), -0.5);
    let val: f64 = qm.into();
    assert_eq!(val, -0.5);
}

#[test]
fn test_berry_curvature() {
    let bc = BerryCurvature::new(1.0).unwrap();
    assert_eq!(bc.value(), 1.0);
    let val: f64 = bc.into();
    assert_eq!(val, 1.0);
}

#[test]
fn test_band_drude_weight() {
    let bdw = BandDrudeWeight::new(2.5).unwrap();
    assert_eq!(bdw.value(), 2.5);
    let val: f64 = bdw.into();
    assert_eq!(val, 2.5);
}

#[test]
fn test_orbital_angular_momentum() {
    let oam = OrbitalAngularMomentum::new(-3.0).unwrap();
    assert_eq!(oam.value(), -3.0);
    let val: f64 = oam.into();
    assert_eq!(val, -3.0);
}

#[test]
fn test_conductance() {
    let c = Conductance::new(0.1).unwrap();
    assert_eq!(c.value(), 0.1);

    let err = Conductance::new(-0.1);
    assert!(err.is_err());
    match err.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_mobility() {
    let m = Mobility::new(100.0).unwrap();
    assert_eq!(m.value(), 100.0);

    let err = Mobility::new(-1.0);
    assert!(err.is_err());
}

#[test]
fn test_twist_angle() {
    let ta = TwistAngle::new(1.0); // radians
    assert!(ta.is_ok());

    let deg = TwistAngle::from_degrees(180.0);
    assert!((deg.value() - std::f64::consts::PI).abs() < 1e-10);
    assert!((deg.as_degrees() - 180.0).abs() < 1e-10);
}

#[test]
fn test_order_parameter() {
    let op = OrderParameter::new(Complex::new(1.0, 1.0));
    assert_eq!(op.value(), Complex::new(1.0, 1.0));
    assert_eq!(op.magnitude_squared(), 2.0);
}
