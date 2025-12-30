/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{BoundedAdjunction, BoundedComonad};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, Metric};

// --- BoundedComonad Tests ---

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

// --- BoundedAdjunction Tests ---

#[test]
fn test_bounded_adjunction_causal_multi_vector_unit() {
    let ctx = Metric::Euclidean(1); // Context is a 2D Vector space (e0, e1)
    let a = 42;
    let mv_mv_a = CausalMultiVectorWitness::unit(&ctx, a);

    // Expected: Outer MV is scalar, inner MV has the metric of the context.
    assert_eq!(mv_mv_a.metric(), Metric::Euclidean(0));
    assert_eq!(mv_mv_a.data().len(), 1);

    let inner_mv = &mv_mv_a.data()[0];
    assert_eq!(inner_mv.metric(), ctx);
    assert_eq!(inner_mv.data().len(), 1);
    assert_eq!(inner_mv.data()[0], 42);
}

#[test]
fn test_bounded_adjunction_causal_multi_vector_counit() {
    let ctx = Metric::Euclidean(0); // Context is not used, but required by signature.
    // Create a MV<MV<B>>
    let inner_mv = CausalMultiVector::new(vec![100], Metric::Euclidean(0)).unwrap();
    let lrb = CausalMultiVector::new(vec![inner_mv], Metric::Euclidean(0)).unwrap();

    let b = CausalMultiVectorWitness::counit(&ctx, lrb);

    // counit should flatten and extract the scalar value.
    assert_eq!(b, 100);
}

#[test]
fn test_bounded_adjunction_causal_multi_vector_left_adjunct() {
    let ctx = Metric::Euclidean(1); // A 2-element vector space
    let a = 5;

    // f: MV<A> -> B
    // A function that takes a multivector and extracts its scalar part.
    let f = |mv_a: CausalMultiVector<i32>| mv_a.data()[0];

    let result_mv = CausalMultiVectorWitness::left_adjunct(&ctx, a, f);

    // Expected: A scalar MV containing the result of f(unit(a)).
    // unit(a) creates a MV with data=[5] and metric=Euclidean(1).
    // fmap applies f to this inner MV, so f([5]) -> 5.
    // The resulting outer MV is a scalar MV containing 5.
    assert_eq!(result_mv.metric(), Metric::Euclidean(0));
    assert_eq!(result_mv.data(), &[5]);
}

#[test]
fn test_bounded_adjunction_causal_multi_vector_right_adjunct() {
    let ctx = Metric::Euclidean(0); // For counit part.
    // la: L(A) -> MV<A>
    let la = CausalMultiVector::new(vec![3, 4], Metric::Euclidean(1)).unwrap();

    // f: A -> R(B) -> A -> MV<B>
    // A function that takes a value and wraps it in a scalar MV.
    let f = |val_a: i32| CausalMultiVector::new(vec![val_a * 2], Metric::Euclidean(0)).unwrap();

    // right_adjunct = counit(fmap(la, f))
    // 1. fmap(la, f) applies f to each element of la:
    //    f(3) -> MV([6])
    //    f(4) -> MV([8])
    //    This results in an outer MV containing two inner MVs: MV<MV<i32>>
    //    The data is [MV([6]), MV([8])]
    // 2. counit flattens this via `bind` and then `extract`s the first element.
    //    bind flattens [MV([6]), MV([8])] -> MV([6, 8], metric=...)
    //    extract gets the first element of the flattened data, which is 6.
    let b = CausalMultiVectorWitness::right_adjunct(&ctx, la, f);

    assert_eq!(b, 6);
}
