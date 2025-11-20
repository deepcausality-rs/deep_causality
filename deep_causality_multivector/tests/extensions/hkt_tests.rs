/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::BoundedComonad;
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, Metric};

#[test]
fn test_comonad_causal_multi_vector_extract_scalar() {
    let scalar_mv = CausalMultiVector::scalar(10.0, Metric::Euclidean(0));
    let extracted = CausalMultiVectorWitness::extract(&scalar_mv);
    assert_eq!(extracted, 10.0);
}

#[test]
fn test_comonad_causal_multi_vector_extract_non_scalar_first_element() {
    let mv = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], Metric::Euclidean(2)).unwrap();
    // In a multivector, data[0] is the scalar part, which is the natural focus.
    let extracted = CausalMultiVectorWitness::extract(&mv);
    assert_eq!(extracted, 1.0);
}

#[test]
fn test_comonad_causal_multi_vector_extend_scalar() {
    let scalar_mv = CausalMultiVector::scalar(5.0, Metric::Euclidean(0));
    // Function that observes the context (the scalar MV) and returns a new value
    let f = |mv: &CausalMultiVector<f64>| mv.data()[0] * 2.0;
    let extended = CausalMultiVectorWitness::extend(&scalar_mv, f);
    assert_eq!(
        extended,
        CausalMultiVector::scalar(10.0, Metric::Euclidean(0))
    );
}

#[test]
fn test_comonad_causal_multi_vector_extend_non_scalar_to_scalar() {
    let mv = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], Metric::Euclidean(2)).unwrap();
    // Function that observes the context (the MV) and returns a summary value (sum of all coeffs)
    let f = |mv_ctx: &CausalMultiVector<f64>| mv_ctx.data().iter().sum::<f64>();
    let extended = CausalMultiVectorWitness::extend(&mv, f);
    // The result should be a scalar MV containing the sum of the input MV coefficients
    assert_eq!(extended, CausalMultiVector::scalar(10.0, mv.metric()));
}

#[test]
fn test_comonad_multivector_preserves_metric() {
    // Create a Complex MultiVector (Metric: NonEuclidean(1) -> Cl(0,1))
    // Data: [Real, Imaginary] -> [1.0, 2.0]
    let mv = CausalMultiVector::new(vec![1.0, 2.0], Metric::NonEuclidean(1)).unwrap();

    // Law: "Double the coefficient"
    let f = |m: &CausalMultiVector<f64>| m.data()[0] * 2.0;

    let result = CausalMultiVectorWitness::extend(&mv, f);

    // 1. Check Data
    // i=0: [1, 2] -> 1*2 = 2
    // i=1: [2, 1] (Basis Shift Logic) -> 2*2 = 4
    // (Assuming basis_shift implementation swaps them)
    // Result data should be [2.0, 4.0]
    assert_eq!(result.data(), &[2.0, 4.0]);

    // 2. Check Metric (Critical for Physics)
    assert_eq!(result.metric(), Metric::NonEuclidean(1));
}
