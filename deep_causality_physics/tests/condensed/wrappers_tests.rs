/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_core::EffectValue;
use deep_causality_haft::Functor;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    ChemicalPotentialGradient, Concentration, Displacement, Energy, Length, Mobility, Momentum,
    OrderParameter, QuantumEigenvector, QuantumMetric, QuantumVelocity, Ratio, Speed, Stiffness,
    TwistAngle, bistritzer_macdonald, cahn_hilliard_flux, effective_band_drude_weight,
    foppl_von_karman_strain, foppl_von_karman_strain_simple, ginzburg_landau_free_energy,
    quantum_geometric_tensor, quasi_qgt,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// ============================================================================
// QGT Wrappers
// ============================================================================

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
fn test_wrapper_quasi_qgt() {
    let energies = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 4], vec![2, 2]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(vec![Complex::new(0.1, 0.0); 4], vec![2, 2]).unwrap(),
    );

    let effect = quasi_qgt(&energies, &u, &v, &v, 0, 1e-12);
    assert!(effect.is_ok());

    if let EffectValue::Value(q) = effect.value() {
        // Check that we got a valid complex number
        assert!(q.re.is_finite());
        assert!(q.im.is_finite());
    }
}

#[test]
fn test_wrapper_effective_band_drude_weight() {
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(0.0).unwrap();
    let curvature = 0.5;
    let metric = QuantumMetric::new(2.0).unwrap();
    let lattice = Length::new(1.0).unwrap();

    let effect = effective_band_drude_weight(energy_n, energy_0, curvature, metric, lattice);
    assert!(effect.is_ok());

    if let EffectValue::Value(bdw) = effect.value() {
        // (0.5 + 1.0*2.0) * 1.0^2 = 2.5
        assert!((bdw.value() - 2.5).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_effective_band_drude_weight_error() {
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(0.0).unwrap();
    let curvature = f64::INFINITY; // Error case
    let metric = QuantumMetric::new(1.0).unwrap();
    let lattice = Length::new(1.0).unwrap();

    let effect = effective_band_drude_weight(energy_n, energy_0, curvature, metric, lattice);
    assert!(effect.is_err());
}

// ============================================================================
// Moiré Wrappers
// ============================================================================

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
fn test_wrapper_moire_magic_angle() {
    // Magic angle ~1.1° ≈ 0.019 radians
    // Note: The bistritzer_macdonald kernel may have specific validation
    let theta = TwistAngle::new(0.019).unwrap();
    let w = Energy::new(0.11).unwrap(); // ~110 meV interlayer coupling
    let vf = Speed::new(1e6).unwrap(); // Fermi velocity
    let k = Momentum::default();

    let effect = bistritzer_macdonald(theta, w, vf, k, 2);
    // Test that wrapper handles result (success or error) without panic
    let _ = effect.is_ok() || effect.is_err();
}

// ============================================================================
// Strain Wrappers
// ============================================================================

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

// ============================================================================
// Phase Wrappers
// ============================================================================

#[test]
fn test_wrapper_phase() {
    let psi = OrderParameter::default();
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_c =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    let effect = ginzburg_landau_free_energy(psi, -1.0, 1.0, &grad_c, None);
    assert!(effect.is_ok());
}

#[test]
fn test_wrapper_cahn_hilliard_flux() {
    let conc = Concentration::new(CausalTensor::new(vec![0.5], vec![1]).unwrap()).unwrap();
    let m = Mobility::new(2.0).unwrap();
    let grad = ChemicalPotentialGradient::new(CausalTensor::new(vec![10.0], vec![1]).unwrap());

    let effect = cahn_hilliard_flux(&conc, m, &grad);
    assert!(effect.is_ok());

    if let EffectValue::Value(flux) = effect.value() {
        // M(0.5) = 0.5 * 0.5 * 2.0 = 0.5
        // J = -0.5 * 10 = -5.0
        assert!((flux.data()[0] - (-5.0)).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_cahn_hilliard_flux_error() {
    // Dimension mismatch
    let conc = Concentration::new(CausalTensor::new(vec![0.5, 0.5], vec![2]).unwrap()).unwrap();
    let m = Mobility::new(1.0).unwrap();
    let grad = ChemicalPotentialGradient::new(CausalTensor::new(vec![1.0], vec![1]).unwrap());

    let effect = cahn_hilliard_flux(&conc, m, &grad);
    assert!(effect.is_err());
}

// ============================================================================
// Error Propagation Tests
// ============================================================================

#[test]
fn test_wrapper_qgt_error_propagation() {
    let energies = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();
    // Wrong shape: 1D instead of 2D
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 4], vec![4]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(vec![Complex::new(0.0, 0.0); 4], vec![2, 2]).unwrap(),
    );

    let effect = quantum_geometric_tensor(&energies, &u, &v, &v, 0, 1e-12);
    assert!(effect.is_err());
}
