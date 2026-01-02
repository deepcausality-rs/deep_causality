/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Pure};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, Metric};

#[test]
fn test_applicative_apply_broadcast() {
    // Case 1: Broadcast (Scalar Function applied to Vector)
    // f_ab is a CausalMultiVector containing a single function (Dimension 0, size 1)

    let euclidean_2 = Metric::Euclidean(2);
    let vector_a = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], euclidean_2).unwrap();

    // Create a scalar MV containing a closure
    // The closure takes f64 -> f64
    let func = |x: f64| x * 2.0;

    // Use pure to wrap the function in a CausalMultiVector (dim 0)
    let f_ab = CausalMultiVectorWitness::pure(func);

    let result = CausalMultiVectorWitness::apply(f_ab, vector_a);

    // Expect: [2, 4, 6, 8]
    assert_eq!(result.data(), &vec![2.0, 4.0, 6.0, 8.0]);
    assert_eq!(result.metric(), euclidean_2);
}

#[test]
fn test_applicative_apply_zip() {
    // Case 2: Element-wise (Zip)
    // f_ab has same length as f_a

    let euclidean_2 = Metric::Euclidean(2);
    // 4 elements
    let vector_a = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], euclidean_2).unwrap();

    // Create a vector of functions.
    // To properly simulate Applicative Zip, we need valid functions for each slot.
    // e.g., f1(x)=x+1, f2(x)=x+2, ...

    let f1 = |x: f64| x + 1.0;
    let f2 = |x: f64| x + 2.0;
    let f3 = |x: f64| x + 3.0;
    let f4 = |x: f64| x + 4.0;

    let f_ab = CausalMultiVector::new(vec![f1, f2, f3, f4], euclidean_2).unwrap();

    let result = CausalMultiVectorWitness::apply(f_ab, vector_a);

    // Expect: [1+1, 2+2, 3+3, 4+4] = [2, 4, 6, 8]
    assert_eq!(result.data(), &vec![2.0, 4.0, 6.0, 8.0]);
}

#[test]
#[should_panic(expected = "Applicative::apply shape mismatch")]
fn test_applicative_apply_mismatch_panic() {
    // Case 3: Mismatch
    let m1 = Metric::Euclidean(2); // size 4 (2^2)
    let m2 = Metric::Euclidean(1); // size 2 (2^1)

    let vector_a = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m1).unwrap();

    let f1 = |x: f64| x;
    let f2 = |x: f64| x;

    let f_ab = CausalMultiVector::new(vec![f1, f2], m2).unwrap();

    CausalMultiVectorWitness::apply(f_ab, vector_a);
}
