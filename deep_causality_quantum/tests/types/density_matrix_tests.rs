/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{DensityMatrix, QuantumErrorEnum, choi_from_kraus, identity_matrix};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

#[test]
fn test_valid_pure_state_accepted() {
    // |+⟩⟨+|
    let rho = mat(
        vec![c(0.5, 0.), c(0.5, 0.), c(0.5, 0.), c(0.5, 0.)],
        2,
    );
    let dm = DensityMatrix::new(rho).unwrap();
    assert_eq!(dm.dim(), 2);
    assert!((dm.purity() - 1.0).abs() < 1e-12);
    assert!(dm.is_pure(1e-9));
}

#[test]
fn test_maximally_mixed_state_accepted() {
    let rho = mat(vec![c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)], 2);
    let dm = DensityMatrix::new(rho).unwrap();
    assert!((dm.purity() - 0.5).abs() < 1e-12);
    assert!(!dm.is_pure(1e-9));
}

#[test]
fn test_non_hermitian_rejected() {
    let rho = mat(vec![c(0.5, 0.), c(0.5, 0.2), c(0.5, 0.2), c(0.5, 0.)], 2);
    let err = DensityMatrix::new(rho).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NonPositiveOperator(_)));
}

#[test]
fn test_negative_eigenvalue_rejected() {
    // Hermitian, unit trace, but spectrum {1.5, −0.5}.
    let rho = mat(vec![c(1.5, 0.), c(0., 0.), c(0., 0.), c(-0.5, 0.)], 2);
    let err = DensityMatrix::new(rho).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NonPositiveOperator(_)));
}

#[test]
fn test_non_unit_trace_rejected() {
    let rho = mat(vec![c(0.9, 0.), c(0., 0.), c(0., 0.), c(0.9, 0.)], 2);
    let err = DensityMatrix::new(rho).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NonUnitTrace(_)));
}

#[test]
fn test_non_finite_rejected() {
    let rho = mat(vec![c(f64::NAN, 0.), c(0., 0.), c(0., 0.), c(1.0, 0.)], 2);
    let err = DensityMatrix::new(rho).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NonFiniteValue(_)));
}

#[test]
fn test_non_square_rejected() {
    let rect = CausalTensor::new(vec![c(1., 0.); 6], vec![2, 3]).unwrap();
    assert!(DensityMatrix::new(rect).is_err());
}

#[test]
fn test_from_ket_ray_normalizes() {
    // An unnormalized ket still yields a valid pure state (kets are rays).
    let ket = CausalTensor::new(vec![c(3., 0.), c(0., 4.)], vec![2, 1]).unwrap();
    let dm = DensityMatrix::from_ket(&ket).unwrap();
    assert!(dm.is_pure(1e-9));
    // ⟨0|ρ|0⟩ = 9/25, ⟨1|ρ|1⟩ = 16/25.
    let m = dm.matrix().as_slice();
    assert!((m[0].re - 0.36).abs() < 1e-12);
    assert!((m[3].re - 0.64).abs() < 1e-12);
}

#[test]
fn test_from_ket_rejects_zero_and_nonfinite() {
    let zero = CausalTensor::new(vec![c(0., 0.); 2], vec![2, 1]).unwrap();
    assert!(DensityMatrix::from_ket(&zero).is_err());
    let nan = CausalTensor::new(vec![c(f64::NAN, 0.), c(1., 0.)], vec![2, 1]).unwrap();
    assert!(DensityMatrix::from_ket(&nan).is_err());
}

#[test]
fn test_from_ensemble_mixes() {
    let k0 = CausalTensor::new(vec![c(1., 0.), c(0., 0.)], vec![2, 1]).unwrap();
    let k1 = CausalTensor::new(vec![c(0., 0.), c(1., 0.)], vec![2, 1]).unwrap();
    let dm = DensityMatrix::from_ensemble(&[(0.5, k0), (0.5, k1)]).unwrap();
    // The 50/50 basis mixture is the maximally mixed state.
    assert!((dm.purity() - 0.5).abs() < 1e-12);
}

#[test]
fn test_from_ensemble_rejects_bad_weights() {
    let k0 = CausalTensor::new(vec![c(1., 0.), c(0., 0.)], vec![2, 1]).unwrap();
    let k1 = CausalTensor::new(vec![c(0., 0.), c(1., 0.)], vec![2, 1]).unwrap();
    assert!(DensityMatrix::from_ensemble(&[(0.7, k0.clone()), (0.7, k1.clone())]).is_err());
    assert!(DensityMatrix::from_ensemble(&[(-0.5, k0), (1.5, k1)]).is_err());
    assert!(DensityMatrix::<f64>::from_ensemble(&[]).is_err());
}

#[test]
fn test_from_choi_is_the_normalized_choi_state() {
    // The identity channel's Choi state J/d is a valid density matrix with
    // purity 1 (the maximally entangled pure state).
    let j = choi_from_kraus(&[identity_matrix::<f64>(2)]).unwrap();
    let dm = DensityMatrix::from_choi(&j).unwrap();
    assert_eq!(dm.dim(), 4);
    assert!(dm.is_pure(1e-9));
}

#[test]
fn test_from_choi_rejects_non_psd() {
    // Hermitian with a negative eigenvalue and positive trace.
    let m = mat(vec![c(2., 0.), c(0., 0.), c(0., 0.), c(-0.5, 0.)], 2);
    assert!(DensityMatrix::from_choi(&m).is_err());
}
