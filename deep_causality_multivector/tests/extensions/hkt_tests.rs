/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, Monad};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, Metric};

#[test]
fn test_functor_fmap() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1, 2, 3, 4], m).unwrap();

    // fmap: x * 2
    let mapped = CausalMultiVectorWitness::fmap(v, |x| x * 2);

    assert_eq!(mapped.data, vec![2, 4, 6, 8]);
    assert_eq!(mapped.metric, m);
}

#[test]
fn test_applicative_pure() {
    let v = CausalMultiVectorWitness::pure(42);
    assert_eq!(v.data, vec![42]);
    assert_eq!(v.metric.dimension(), 0);
}

#[test]
fn test_applicative_apply_broadcast() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1, 2, 3, 4], m).unwrap();

    // pure function: |x| x + 10
    let func_vec = CausalMultiVectorWitness::pure(|x| x + 10);

    let applied = CausalMultiVectorWitness::apply(func_vec, v);

    assert_eq!(applied.data, vec![11, 12, 13, 14]);
    assert_eq!(applied.metric, m);
}

#[test]
fn test_applicative_apply_zip() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1, 2, 3, 4], m).unwrap();

    // Vector of functions?
    // Construct manually since pure() creates scalar
    let func_data: Vec<fn(i32) -> i32> = vec![|x| x + 1, |x| x + 2, |x| x + 3, |x| x + 4];
    let func_vec = CausalMultiVector::new(func_data, m).unwrap();

    let applied = CausalMultiVectorWitness::apply(func_vec, v);

    assert_eq!(applied.data, vec![2, 4, 6, 8]);
}

#[test]
fn test_monad_bind_tensor_product() {
    // Monad bind implements Tensor Product.
    // A (dim 1, size 2) bind f -> B (dim 1, size 2)
    // Result should be dim 2 (size 4)

    let m_a = Metric::Euclidean(1); // Size 2
    let v_a = CausalMultiVector::new(vec![1.0, 2.0], m_a).unwrap();

    // f: x -> [x, -x] (Euclidean 1)
    let f = |x: f64| {
        let m_b = Metric::Euclidean(1);
        CausalMultiVector::new(vec![x, -x], m_b).unwrap()
    };

    let bound = CausalMultiVectorWitness::bind(v_a, f);

    // Expected:
    // For a=1.0: [1.0, -1.0]
    // For a=2.0: [2.0, -2.0]
    // Result: [1.0, -1.0, 2.0, -2.0]
    // Metric: Euclidean(1) tensor Euclidean(1) = Euclidean(2)

    assert_eq!(bound.data, vec![1.0, -1.0, 2.0, -2.0]);
    assert_eq!(bound.metric.dimension(), 2);
}
