/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructor, rejection-branch, arithmetic, and Rk4-arrow tests for
//! `VelocityOneForm` — the DEC solver's marching state.

use deep_causality_calculus::Rk4;
use deep_causality_haft::Arrow;
use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_physics::VelocityOneForm;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn unit_manifold<R>(n: usize) -> Manifold<LatticeComplex<2, R>, R>
where
    R: RealField + FromPrimitive + Default + PartialEq + core::fmt::Debug + core::fmt::Display,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn edge_tensor(manifold: &Manifold<LatticeComplex<2, f64>, f64>, fill: f64) -> CausalTensor<f64> {
    let n1 = manifold.complex().num_cells(1);
    CausalTensor::new(vec![fill; n1], vec![n1]).unwrap()
}

#[test]
fn constructs_with_valid_field_and_exposes_getters() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let v = VelocityOneForm::new(edge_tensor(&manifold, 1.5), &manifold).unwrap();
    assert_eq!(v.len(), n1);
    assert!(!v.is_empty());
    assert_eq!(v.as_tensor().as_slice()[0], 1.5);
}

#[test]
fn rejects_length_mismatch() {
    let manifold = unit_manifold::<f64>(3);
    let bad = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let err = VelocityOneForm::new(bad, &manifold).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("VelocityOneForm") && msg.contains("Dimension Mismatch"));
}

#[test]
fn rejects_nan_coefficient() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let mut data = vec![0.0; n1];
    data[2] = f64::NAN;
    let err =
        VelocityOneForm::new(CausalTensor::new(data, vec![n1]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("non-finite coefficient at index 2"));
}

#[test]
fn rejects_positive_infinity_coefficient() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let mut data = vec![0.0; n1];
    data[0] = f64::INFINITY;
    let err =
        VelocityOneForm::new(CausalTensor::new(data, vec![n1]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("Numerical Instability"));
}

#[test]
fn rejects_negative_infinity_coefficient() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let mut data = vec![0.0; n1];
    data[n1 - 1] = f64::NEG_INFINITY;
    let err =
        VelocityOneForm::new(CausalTensor::new(data, vec![n1]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("Numerical Instability"));
}

#[test]
fn add_is_elementwise_and_mul_scales() {
    let manifold = unit_manifold::<f64>(3);
    let a = VelocityOneForm::new(edge_tensor(&manifold, 1.0), &manifold).unwrap();
    let b = VelocityOneForm::new(edge_tensor(&manifold, 2.5), &manifold).unwrap();

    let sum = a.clone() + b;
    for v in sum.as_tensor().as_slice() {
        assert_eq!(*v, 3.5);
    }
    let scaled = a * 4.0;
    for v in scaled.as_tensor().as_slice() {
        assert_eq!(*v, 4.0);
    }
}

#[test]
#[should_panic(expected = "matching edge counts")]
fn add_panics_on_mismatched_lattices() {
    let m3 = unit_manifold::<f64>(3);
    let m4 = unit_manifold::<f64>(4);
    let n1_4 = m4.complex().num_cells(1);
    let a = VelocityOneForm::new(edge_tensor(&m3, 1.0), &m3).unwrap();
    let b =
        VelocityOneForm::new(CausalTensor::new(vec![1.0; n1_4], vec![n1_4]).unwrap(), &m4).unwrap();
    let _ = a + b;
}

#[test]
fn derives_debug_clone_partial_eq() {
    let manifold = unit_manifold::<f64>(3);
    let a = VelocityOneForm::new(edge_tensor(&manifold, 1.0), &manifold).unwrap();
    let b = a.clone();
    assert_eq!(a, b);
    assert!(format!("{a:?}").contains("VelocityOneForm"));
}

/// The whole-field state rides the `Rk4` arrow: linear decay `du/dt = −λu`
/// reproduces the analytic exponential at every precision backend.
fn assert_rk4_decay<R>(tol: R)
where
    R: RealField + FromPrimitive + Default + PartialEq + core::fmt::Debug + core::fmt::Display,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(3);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let n1 = manifold.complex().num_cells(1);
    let one = R::one();
    let field = CausalTensor::new(vec![one; n1], vec![n1]).unwrap();
    let mut state = VelocityOneForm::new(field, &manifold).unwrap();

    let lambda = R::from_f64(0.5).expect("lifts");
    let dt = R::from_f64(0.01).expect("lifts");
    let steps = 100usize;

    let rk4 = Rk4::new(dt, |s: &VelocityOneForm<R>| {
        s.clone() * (R::zero() - lambda)
    });
    for _ in 0..steps {
        state = rk4.run(state);
    }

    // u(1) = e^{−0.5}; RK4 global error O(dt⁴).
    let expected = (R::zero() - lambda).exp();
    for v in state.as_tensor().as_slice() {
        assert!(
            (*v - expected).abs() < tol,
            "decay mismatch: got {v}, expected {expected}"
        );
    }
}

#[test]
fn rides_rk4_arrow_f64() {
    assert_rk4_decay::<f64>(1e-8);
}

#[test]
fn rides_rk4_arrow_f32() {
    assert_rk4_decay::<f32>(1e-4_f32);
}

#[test]
fn rides_rk4_arrow_float106() {
    assert_rk4_decay::<Float106>(Float106::from_f64(1e-9));
}
