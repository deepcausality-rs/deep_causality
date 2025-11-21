/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num::Complex;

// Helper function to create a basic CausalMultiVector for testing
fn create_test_multivector(dim: usize, metric: Metric) -> CausalMultiVector<Complex<f64>> {
    let size = 1 << dim;
    let data = vec![Complex::new(1.0, 0.0); size];
    CausalMultiVector::new(data, metric).unwrap()
}

#[test]
fn test_new_spin10_success() {
    let data = vec![Complex::new(1.0, 0.0); 1 << 10]; // 2^10 elements for Cl(0,10)
    let state = HilbertState::new_spin10(data.clone()).unwrap();
    assert_eq!(state.mv().data(), &data);
    assert_eq!(state.mv().metric(), Metric::NonEuclidean(10));
}

#[test]
fn test_new_spin10_error_data_length_mismatch() {
    let data = vec![Complex::new(1.0, 0.0); 10]; // Incorrect length for Cl(0,10)
    let err = HilbertState::new_spin10(data).unwrap_err();
    assert_eq!(
        format!("{}", err),
        "Data length mismatch: expected 1024, found 10"
    );
}

#[test]
fn test_new_success() {
    let dim = 2;
    let size = 1 << dim;
    let data = vec![Complex::new(1.0, 0.0); size];
    let metric = Metric::Euclidean(dim);
    let state = HilbertState::new(data.clone(), metric).unwrap();
    assert_eq!(state.mv().data(), &data);
    assert_eq!(state.mv().metric(), metric);
}

#[test]
fn test_new_error_data_length_mismatch() {
    let dim = 2;
    let data = vec![Complex::new(1.0, 0.0); 3]; // Incorrect length for dim 2
    let metric = Metric::Euclidean(dim);
    let err = HilbertState::new(data, metric).unwrap_err();
    assert_eq!(
        format!("{}", err),
        "Data length mismatch: expected 4, found 3"
    );
}

#[test]
fn test_new_unchecked() {
    let dim = 2;
    let size = 1 << dim;
    let data = vec![Complex::new(1.0, 0.0); size];
    let metric = Metric::Euclidean(dim);
    let state = HilbertState::new_unchecked(data.clone(), metric);
    assert_eq!(state.mv().data(), &data);
    assert_eq!(state.mv().metric(), metric);
}

#[test]
fn test_from_multivector() {
    let dim = 2;
    let metric = Metric::Euclidean(dim);
    let mv = create_test_multivector(dim, metric);
    let state = HilbertState::from(mv.clone());
    assert_eq!(state.mv().data(), mv.data());
    assert_eq!(state.mv().metric(), mv.metric());
}

#[test]
fn test_into_inner() {
    let dim = 2;
    let metric = Metric::Euclidean(dim);
    let mv = create_test_multivector(dim, metric);
    let state = HilbertState::from(mv.clone());
    let inner_mv = state.into_inner();
    assert_eq!(inner_mv.data(), mv.data());
    assert_eq!(inner_mv.metric(), mv.metric());
}

#[test]
fn test_as_inner() {
    let dim = 2;
    let metric = Metric::Euclidean(dim);
    let mv = create_test_multivector(dim, metric);
    let state = HilbertState::from(mv.clone());
    let inner_mv_ref = state.as_inner();
    assert_eq!(inner_mv_ref.data(), mv.data());
    assert_eq!(inner_mv_ref.metric(), mv.metric());
}

#[test]
fn test_mv() {
    let dim = 2;
    let metric = Metric::Euclidean(dim);
    let mv = create_test_multivector(dim, metric);
    let state = HilbertState::from(mv.clone());
    let mv_ref = state.mv();
    assert_eq!(mv_ref.data(), mv.data());
    assert_eq!(mv_ref.metric(), mv.metric());
}

#[test]
fn test_add_hilbert_states() {
    let dim = 2;
    let size = 1 << dim;
    let metric = Metric::Euclidean(dim);

    let data1 = vec![Complex::new(1.0, 2.0); size];
    let data2 = vec![Complex::new(3.0, 4.0); size];

    let state1 = HilbertState::new(data1.clone(), metric).unwrap();
    let state2 = HilbertState::new(data2.clone(), metric).unwrap();

    let result_state = state1 + state2;

    let expected_data: Vec<Complex<f64>> = data1
        .iter()
        .zip(data2.iter())
        .map(|(&c1, &c2)| c1 + c2)
        .collect();

    assert_eq!(result_state.mv().data(), &expected_data);
    assert_eq!(result_state.mv().metric(), metric);
}

#[test]
#[should_panic(expected = "Dimension mismatch in addition")]
fn test_add_hilbert_states_metric_mismatch_panics() {
    let dim = 1;
    let size1 = 1 << dim;
    let metric1 = Metric::Euclidean(dim);
    let data1 = vec![Complex::new(1.0, 0.0); size1];
    let state1 = HilbertState::new(data1, metric1).unwrap();

    let dim = 2; // Different dimension
    let size2 = 1 << dim;
    let metric2 = Metric::Euclidean(dim);
    let data2 = vec![Complex::new(1.0, 0.0); size2];
    let state2 = HilbertState::new(data2, metric2).unwrap();

    // This should panic due to metric mismatch in CausalMultiVector::add
    let _ = state1 + state2;
}

#[test]
fn test_mul_complex_scalar() {
    let dim = 2;
    let size = 1 << dim;
    let metric = Metric::Euclidean(dim);
    let data = vec![Complex::new(1.0, 1.0); size];
    let state = HilbertState::new(data.clone(), metric).unwrap();

    let scalar = Complex::new(2.0, 0.0);
    let result_state = state * scalar;

    let expected_data: Vec<Complex<f64>> = data.iter().map(|&c| c * scalar).collect();

    assert_eq!(result_state.mv().data(), &expected_data);
    assert_eq!(result_state.mv().metric(), metric);

    let data_complex = vec![Complex::new(1.0, 1.0); size];
    let state_complex = HilbertState::new(data_complex.clone(), metric).unwrap();
    let scalar_complex = Complex::new(0.0, 1.0); // i
    let result_state_complex = state_complex * scalar_complex;
    let expected_data_complex: Vec<Complex<f64>> =
        data_complex.iter().map(|&c| c * scalar_complex).collect();
    assert_eq!(result_state_complex.mv().data(), &expected_data_complex);
}
