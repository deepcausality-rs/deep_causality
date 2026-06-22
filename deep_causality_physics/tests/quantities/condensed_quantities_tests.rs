/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    BandDrudeWeight, BerryCurvature, Conductance, Mobility, OrbitalAngularMomentum, OrderParameter,
    PhysicsErrorEnum, QuantumMetric, TwistAngle,
};

#[test]
fn test_quantum_metric() {
    let qm = QuantumMetric::<f64>::new(-0.5).unwrap();
    assert_eq!(qm.value(), -0.5);
    let val: f64 = qm.into();
    assert_eq!(val, -0.5);
}

#[test]
fn test_berry_curvature() {
    let bc = BerryCurvature::<f64>::new(1.0).unwrap();
    assert_eq!(bc.value(), 1.0);
    let val: f64 = bc.into();
    assert_eq!(val, 1.0);
}

#[test]
fn test_band_drude_weight() {
    let bdw = BandDrudeWeight::<f64>::new(2.5).unwrap();
    assert_eq!(bdw.value(), 2.5);
    let val: f64 = bdw.into();
    assert_eq!(val, 2.5);
}

#[test]
fn test_orbital_angular_momentum() {
    let oam = OrbitalAngularMomentum::<f64>::new(-3.0).unwrap();
    assert_eq!(oam.value(), -3.0);
    let val: f64 = oam.into();
    assert_eq!(val, -3.0);
}

#[test]
fn test_conductance() {
    let c = Conductance::<f64>::new(0.1).unwrap();
    assert_eq!(c.value(), 0.1);

    let err = Conductance::<f64>::new(-0.1);
    assert!(err.is_err());
    match err.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_mobility() {
    let m = Mobility::<f64>::new(100.0).unwrap();
    assert_eq!(m.value(), 100.0);

    let err = Mobility::<f64>::new(-1.0);
    assert!(err.is_err());
}

#[test]
fn test_twist_angle() {
    let ta = TwistAngle::<f64>::new(1.0); // radians
    assert!(ta.is_ok());

    let deg = TwistAngle::<f64>::from_degrees(180.0);
    assert!((deg.value() - std::f64::consts::PI).abs() < 1e-10);
    assert!((deg.as_degrees() - 180.0).abs() < 1e-10);
}

#[test]
fn test_order_parameter() {
    let op = OrderParameter::new(Complex::new(1.0, 1.0));
    assert_eq!(op.value(), Complex::new(1.0, 1.0));
    assert_eq!(op.magnitude_squared(), 2.0);
}

// ===========================================================================
// new_unchecked tests
// ===========================================================================

#[test]
fn test_conductance_new_unchecked() {
    let c = Conductance::<f64>::new_unchecked(0.1);
    assert_eq!(c.value(), 0.1);
}

#[test]
fn test_mobility_new_unchecked() {
    let m = Mobility::<f64>::new_unchecked(100.0);
    assert_eq!(m.value(), 100.0);
}

// ===========================================================================
// Default trait tests
// ===========================================================================

#[test]
fn test_quantum_metric_default() {
    let qm: QuantumMetric<f64> = Default::default();
    assert_eq!(qm.value(), 0.0);
}

#[test]
fn test_berry_curvature_default() {
    let bc: BerryCurvature<f64> = Default::default();
    assert_eq!(bc.value(), 0.0);
}

#[test]
fn test_band_drude_weight_default() {
    let bdw: BandDrudeWeight<f64> = Default::default();
    assert_eq!(bdw.value(), 0.0);
}

#[test]
fn test_orbital_angular_momentum_default() {
    let oam: OrbitalAngularMomentum<f64> = Default::default();
    assert_eq!(oam.value(), 0.0);
}

#[test]
fn test_conductance_default() {
    let c: Conductance<f64> = Default::default();
    assert_eq!(c.value(), 0.0);
}

#[test]
fn test_mobility_default() {
    let m: Mobility<f64> = Default::default();
    assert_eq!(m.value(), 0.0);
}

#[test]
fn test_twist_angle_default() {
    let t: TwistAngle<f64> = TwistAngle::default();
    assert_eq!(t.value(), 0.0);
}

#[test]
fn test_twist_angle_degrees_roundtrip() {
    let t = TwistAngle::<f64>::from_degrees(45.0);
    assert!((t.as_degrees() - 45.0).abs() < 1e-10);
}

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

// =============================================================================
// From<X> for f64 conversions (condensed/mod.rs:161-163, 196-198, 236-238)
// =============================================================================

#[test]
fn test_conductance_into_f64() {
    let c = Conductance::<f64>::new(3.5).unwrap();
    let val: f64 = c.into();
    assert!((val - 3.5).abs() < 1e-10);
}

#[test]
fn test_mobility_into_f64() {
    let m = Mobility::<f64>::new(0.25).unwrap();
    let val: f64 = m.into();
    assert!((val - 0.25).abs() < 1e-10);
}

#[test]
fn test_twist_angle_into_f64() {
    let ta = TwistAngle::<f64>::new(1.1).unwrap();
    let val: f64 = ta.into();
    assert!((val - 1.1).abs() < 1e-10);
}

// =============================================================================
// Concentration (condensed/mod.rs:346-348 negative branch, 355-357 unchecked)
// =============================================================================

#[test]
fn test_concentration_new_valid() {
    let t = deep_causality_tensor::CausalTensor::new(vec![0.1, 0.2, 0.3], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new(t.clone());
    assert!(c.is_ok());
    assert_eq!(c.unwrap().inner().shape(), t.shape());
}

#[test]
fn test_concentration_new_negative_rejected() {
    let t = deep_causality_tensor::CausalTensor::new(vec![0.1, -0.5, 0.3], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new(t);
    assert!(c.is_err());
    match c.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
        other => panic!("expected PhysicalInvariantBroken, got {other:?}"),
    }
}

#[test]
fn test_concentration_new_unchecked() {
    // new_unchecked bypasses the non-negativity check.
    let t = deep_causality_tensor::CausalTensor::new(vec![-1.0, 0.0, 2.0], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new_unchecked(t.clone());
    assert_eq!(c.inner().shape(), t.shape());
}

// =============================================================================
// VectorPotential Default (condensed/mod.rs:385-387)
// =============================================================================

#[test]
fn test_vector_potential_default_and_new() {
    let vp = deep_causality_physics::VectorPotential::default();
    // Default is a single-component zero multivector.
    assert_eq!(vp.inner().data().len(), 1);

    let mv = deep_causality_multivector::CausalMultiVector::new(
        vec![1.0],
        deep_causality_multivector::Metric::Euclidean(0),
    )
    .unwrap();
    let vp2 = deep_causality_physics::VectorPotential::new(mv.clone());
    assert_eq!(vp2.inner().data(), mv.data());
}
