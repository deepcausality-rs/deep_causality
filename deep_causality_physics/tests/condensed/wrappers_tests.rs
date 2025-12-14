/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::Functor;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    Displacement, Energy, Momentum, OrderParameter, QuantumEigenvector, QuantumVelocity, Ratio,
    Speed, Stiffness, TwistAngle, bistritzer_macdonald, foppl_von_karman_strain,
    foppl_von_karman_strain_simple, ginzburg_landau_free_energy, quantum_geometric_tensor,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

#[test]
fn test_wrapper_qgt() {
    let energies = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 4], vec![2, 2]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(vec![Complex::new(0.0, 0.0); 4], vec![2, 2]).unwrap(),
    );

    let effect = quantum_geometric_tensor(&energies, &u, &v, &v, 0, 1e-12);
    assert!(effect.is_ok());
}

#[test]
fn test_wrapper_moire() {
    let theta = TwistAngle::new(0.1).unwrap();
    let w = Energy::new(0.1).unwrap();
    let vf = Speed::new(1e5).unwrap();
    let k = Momentum::default();

    let effect = bistritzer_macdonald(theta, w, vf, k, 1);
    assert!(effect.is_ok());
}

#[test]
fn test_wrapper_strain_simple() {
    let eps = Displacement::new(CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap());
    let e = Stiffness::new(100.0).unwrap();
    let nu = Ratio::new(0.3).unwrap();

    let effect = foppl_von_karman_strain_simple(&eps, e, nu);
    assert!(effect.is_ok());
}

fn create_flat_manifold() -> Manifold<f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn test_wrapper_strain_full() {
    let man = create_flat_manifold();
    let e = Stiffness::new(100.0).unwrap();
    let nu = Ratio::new(0.3).unwrap();

    let effect = foppl_von_karman_strain(&man, &man, e, nu);
    assert!(effect.is_ok());
}

#[test]
fn test_wrapper_phase() {
    let psi = OrderParameter::default();
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_c =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    let effect = ginzburg_landau_free_energy(psi, -1.0, 1.0, &grad_c, None);
    assert!(effect.is_ok());
}
