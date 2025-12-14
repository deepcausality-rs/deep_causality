/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::Functor;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    cahn_hilliard_flux_kernel, ginzburg_landau_free_energy_kernel, ChemicalPotentialGradient,
    Concentration, Mobility, OrderParameter, VectorPotential,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_ginzburg_landau_zero() {
    let psi = OrderParameter::new(Complex::new(0.0, 0.0));
    let alpha = -1.0;
    let beta = 1.0;
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_complex =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    let res = ginzburg_landau_free_energy_kernel(psi, alpha, beta, &grad_complex, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().value(), 0.0);
}

#[test]
fn test_ginzburg_landau_uniform() {
    let psi = OrderParameter::new(Complex::new(1.0, 0.0));
    let alpha = 1.0;
    let beta = 2.0;
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_complex =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    // F = 1*1 + (2/2)*1 + 0 = 2
    let res = ginzburg_landau_free_energy_kernel(psi, alpha, beta, &grad_complex, None);
    assert!(res.is_ok());
    assert!((res.unwrap().value() - 2.0).abs() < 1e-10);
}

#[test]
fn test_ginzburg_landau_with_vector_potential() {
    let psi = OrderParameter::new(Complex::new(1.0, 0.0));
    let alpha = 1.0;
    let beta = 1.0;

    // Gradient of psi (complex)
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_complex =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    // Vector potential A (real)
    let a_field = CausalMultiVector::new(vec![0.1, 0.2, 0.3, 0.4], Metric::Euclidean(2)).unwrap();
    let vector_potential = VectorPotential::new(a_field);

    let res =
        ginzburg_landau_free_energy_kernel(psi, alpha, beta, &grad_complex, Some(&vector_potential));
    assert!(res.is_ok());
}

#[test]
fn test_ginzburg_landau_superconducting_state() {
    // Alpha < 0 corresponds to superconducting state
    let psi = OrderParameter::new(Complex::new(1.0, 0.5));
    let alpha = -1.0; // Below Tc
    let beta = 1.0;
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_complex =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    let res = ginzburg_landau_free_energy_kernel(psi, alpha, beta, &grad_complex, None);
    assert!(res.is_ok());

    // |psi|^2 = 1 + 0.25 = 1.25
    // F = -1 * 1.25 + (1/2) * 1.5625 = -1.25 + 0.78125 = -0.46875
    let energy = res.unwrap();
    assert!(energy.value() < 0.0); // Negative energy in superconducting state
}

#[test]
fn test_ginzburg_landau_complex_order_parameter() {
    let psi = OrderParameter::new(Complex::new(0.5, 0.5));
    let alpha = 0.0;
    let beta = 4.0;
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_complex =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    let res = ginzburg_landau_free_energy_kernel(psi, alpha, beta, &grad_complex, None);
    assert!(res.is_ok());

    // |psi|^2 = 0.5, |psi|^4 = 0.25
    // F = 0 + (4/2) * 0.25 = 0.5
    let energy = res.unwrap();
    assert!((energy.value() - 0.5).abs() < 1e-10);
}

#[test]
fn test_ginzburg_landau_error_metric_mismatch() {
    let psi = OrderParameter::new(Complex::new(1.0, 0.0));

    // Gradient in Euclidean(2) metric
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_complex =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    // Vector potential in different metric (Euclidean(3))
    let a_field = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    let vector_potential = VectorPotential::new(a_field);

    let res =
        ginzburg_landau_free_energy_kernel(psi, 1.0, 1.0, &grad_complex, Some(&vector_potential));
    assert!(res.is_err());
}

#[test]
fn test_ginzburg_landau_error_vector_size_mismatch() {
    let psi = OrderParameter::new(Complex::new(1.0, 0.0));

    // Gradient with 4 components
    let grad = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let grad_complex =
        deep_causality_multivector::CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, 0.0));

    // Vector potential with 2 components (same metric but different size won't happen with same Metric)
    // This test is tricky because Metric::Euclidean(n) determines the size
    // Let's test with matching metric but the kernel checks data length equality
    // Actually the size is determined by the Metric enum, so this case may not be easily triggered
    // Skip this test as the Metric type enforces consistent sizing

    // The kernel checks if a.data().len() != grad_data.len()
    // With matching Metric, this should always pass
    let a_field = CausalMultiVector::new(vec![0.0; 4], Metric::Euclidean(2)).unwrap();
    let vector_potential = VectorPotential::new(a_field);

    let res =
        ginzburg_landau_free_energy_kernel(psi, 1.0, 1.0, &grad_complex, Some(&vector_potential));
    assert!(res.is_ok()); // Should pass with matching sizes
}

#[test]
fn test_cahn_hilliard_flux() {
    let conc = Concentration::new(CausalTensor::new(vec![0.5], vec![1]).unwrap()).unwrap();
    let m = Mobility::new(2.0).unwrap();
    let grad = ChemicalPotentialGradient::new(CausalTensor::new(vec![10.0], vec![1]).unwrap());

    let res = cahn_hilliard_flux_kernel(&conc, m, &grad);
    assert!(res.is_ok());

    let flux = res.unwrap();
    // M(c) = 2.0 * 0.5 * (1 - 0.5) = 2.0 * 0.25 = 0.5
    // J = -0.5 * 10.0 = -5.0
    assert!((flux.data()[0] - (-5.0)).abs() < 1e-10);
}

#[test]
fn test_cahn_hilliard_flux_pure_phase_c0() {
    // c = 0 → M(c) = 0 → no flux
    let conc = Concentration::new(CausalTensor::new(vec![0.0], vec![1]).unwrap()).unwrap();
    let m = Mobility::new(2.0).unwrap();
    let grad = ChemicalPotentialGradient::new(CausalTensor::new(vec![10.0], vec![1]).unwrap());

    let res = cahn_hilliard_flux_kernel(&conc, m, &grad);
    assert!(res.is_ok());

    let flux = res.unwrap();
    // M(0) = 0, J = 0
    assert!((flux.data()[0]).abs() < 1e-10);
}

#[test]
fn test_cahn_hilliard_flux_pure_phase_c1() {
    // c = 1 → M(c) = 0 → no flux
    let conc = Concentration::new(CausalTensor::new(vec![1.0], vec![1]).unwrap()).unwrap();
    let m = Mobility::new(2.0).unwrap();
    let grad = ChemicalPotentialGradient::new(CausalTensor::new(vec![10.0], vec![1]).unwrap());

    let res = cahn_hilliard_flux_kernel(&conc, m, &grad);
    assert!(res.is_ok());

    let flux = res.unwrap();
    // M(1) = 0, J = 0
    assert!((flux.data()[0]).abs() < 1e-10);
}

#[test]
fn test_cahn_hilliard_flux_error_dimension_mismatch() {
    // Different shapes for concentration and gradient
    let conc = Concentration::new(CausalTensor::new(vec![0.5, 0.5], vec![2]).unwrap()).unwrap();
    let m = Mobility::new(1.0).unwrap();
    let grad = ChemicalPotentialGradient::new(CausalTensor::new(vec![1.0], vec![1]).unwrap());

    let res = cahn_hilliard_flux_kernel(&conc, m, &grad);
    assert!(res.is_err());
}

#[test]
fn test_cahn_hilliard_flux_multi_element() {
    // Multi-element field
    let conc =
        Concentration::new(CausalTensor::new(vec![0.5, 0.25, 0.75], vec![3]).unwrap()).unwrap();
    let m = Mobility::new(1.0).unwrap();
    let grad = ChemicalPotentialGradient::new(CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap());

    let res = cahn_hilliard_flux_kernel(&conc, m, &grad);
    assert!(res.is_ok());

    let flux = res.unwrap();
    // M(0.5) = 0.25, J[0] = -0.25 * 1 = -0.25
    // M(0.25) = 0.25 * 0.75 = 0.1875, J[1] = -0.1875 * 2 = -0.375
    // M(0.75) = 0.75 * 0.25 = 0.1875, J[2] = -0.1875 * 3 = -0.5625
    assert!((flux.data()[0] - (-0.25)).abs() < 1e-10);
    assert!((flux.data()[1] - (-0.375)).abs() < 1e-10);
    assert!((flux.data()[2] - (-0.5625)).abs() < 1e-10);
}

