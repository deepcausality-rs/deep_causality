/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::Functor;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    ChemicalPotentialGradient, Concentration, Mobility, OrderParameter, cahn_hilliard_flux_kernel,
    ginzburg_landau_free_energy_kernel,
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
