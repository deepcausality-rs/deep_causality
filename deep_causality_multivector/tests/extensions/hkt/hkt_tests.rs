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

    assert_eq!(
        extended,
        CausalMultiVector::new(vec![10.0, 10.0, 10.0, 10.0], mv.metric()).unwrap()
    );
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

#[test]
fn test_comonad_multivector_context_sensitivity() {
    // Input: [10, 20, 30, 40]
    let mv = CausalMultiVector::new(vec![10.0, 20.0, 30.0, 40.0], Metric::Euclidean(2)).unwrap();

    // Law: "My value becomes the value of the component to my 'right' (cyclic)"
    // Index 0 looks at Index 1.
    // Index 1 looks at Index 2...
    let f = |ctx: &CausalMultiVector<f64>| {
        // ctx.data[0] is "Me"
        // ctx.data[1] is "My Right Neighbor" (because of basis_shift)
        ctx.data()[1]
    };

    let extended = CausalMultiVectorWitness::extend(&mv, f);

    // Logic Trace:
    // i=0: View [10, 20, 30, 40]. Right is 20. -> Result[0] = 20
    // i=1: View [20, 30, 40, 10]. Right is 30. -> Result[1] = 30
    // i=2: View [30, 40, 10, 20]. Right is 40. -> Result[2] = 40
    // i=3: View [40, 10, 20, 30]. Right is 10. -> Result[3] = 10

    assert_eq!(
        extended,
        CausalMultiVector::new(vec![20.0, 30.0, 40.0, 10.0], mv.metric()).unwrap()
    );
}
