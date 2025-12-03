/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{
    Applicative, BoundedAdjunction, BoundedComonad, Foldable, Functor, HKT, Monad,
};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

// --- HKT Tests ---

#[test]
fn test_hkt_causal_tensor_witness() {
    let value: <CausalTensorWitness as HKT>::Type<i32> =
        CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    assert_eq!(value.as_slice(), &[1, 2, 3]);

    let empty_value: <CausalTensorWitness as HKT>::Type<f64> =
        CausalTensor::new(vec![], vec![0]).unwrap();
    assert!(empty_value.is_empty());
}

// --- Applicative Tests ---

#[test]
fn test_applicative_causal_tensor_pure() {
    let tensor = CausalTensorWitness::pure(42);
    assert_eq!(tensor.as_slice(), &[42]);
    assert_eq!(tensor.shape(), &[] as &[usize]); // Scalar tensor
}

#[test]
fn test_applicative_causal_tensor_apply_scalar_func() {
    let f_tensor = CausalTensor::new(vec![|x: i32| x * 2], vec![]).unwrap(); // Scalar function, added type annotation
    let a_tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result_tensor = CausalTensorWitness::apply(f_tensor, a_tensor);
    assert_eq!(result_tensor.as_slice(), &[2, 4, 6]);
    assert_eq!(result_tensor.shape(), &[3]);
}

#[test]
fn test_applicative_causal_tensor_apply_non_scalar_func() {
    // Create a non-scalar function tensor (e.g., a vector of functions)
    let f_tensor = CausalTensor::new(vec![|x: i32| x * 2, |x: i32| x * 3], vec![2]).unwrap();
    let a_tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result_tensor = CausalTensorWitness::apply(f_tensor, a_tensor);
    // Expect an empty tensor as per the updated implementation
    assert!(result_tensor.is_empty());
    assert_eq!(result_tensor.shape(), &[0]);
}

// --- Functor Tests ---

#[test]
fn test_functor_causal_tensor() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let f = |x| x * 2;
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert_eq!(mapped_tensor.as_slice(), &[2, 4, 6]);
    assert_eq!(mapped_tensor.shape(), &[3]);
}

#[test]
fn test_functor_causal_tensor_empty() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let f = |x| x * 2;
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert!(mapped_tensor.is_empty());
    assert_eq!(mapped_tensor.shape(), &[0]);
}

#[test]
fn test_functor_causal_tensor_type_change() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let f = |x: i32| x.to_string(); // Added type annotation for x
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert_eq!(
        mapped_tensor.as_slice(),
        &["1".to_string(), "2".to_string(), "3".to_string()]
    );
    assert_eq!(mapped_tensor.shape(), &[3]);
}

// --- Foldable Tests ---

#[test]
fn test_foldable_causal_tensor_sum() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5], vec![5]).unwrap();
    let sum = CausalTensorWitness::fold(tensor, 0, |acc, x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn test_foldable_causal_tensor_empty() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let sum = CausalTensorWitness::fold(tensor, 0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn test_foldable_causal_tensor_string_concat() {
    let tensor =
        CausalTensor::new(vec!["hello".to_string(), "world".to_string()], vec![2]).unwrap();
    let concatenated = CausalTensorWitness::fold(tensor, String::new(), |mut acc, x| {
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(&x);
        acc
    });
    assert_eq!(concatenated, "hello world");
}

// --- Monad Tests ---

#[test]
fn test_monad_causal_tensor_bind() {
    let tensor = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let f = |x: i32| CausalTensor::new(vec![x, x * 10], vec![2]).unwrap(); // Added type annotation for x
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert_eq!(bound_tensor.as_slice(), &[1, 10, 2, 20]);
    assert_eq!(bound_tensor.shape(), &[4]); // Flattened to 1D
}

#[test]
fn test_monad_causal_tensor_bind_empty() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let f = |x: i32| CausalTensor::new(vec![x, x * 10], vec![2]).unwrap(); // Added type annotation for x
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert!(bound_tensor.is_empty());
    assert_eq!(bound_tensor.shape(), &[0]);
}

#[test]
fn test_monad_causal_tensor_bind_to_empty() {
    let tensor = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let f = |_x: i32| CausalTensor::<i32>::new(vec![], vec![0]).unwrap(); // Explicitly specify <i32>
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert!(bound_tensor.is_empty());
    assert_eq!(bound_tensor.shape(), &[0]);
}

// ---CoMonad Tests ---

#[test]
fn test_comonad_causal_tensor_extract_scalar() {
    let scalar_tensor = CausalTensor::new(vec![10.0], vec![]).unwrap();
    let extracted = CausalTensorWitness::extract(&scalar_tensor);
    assert_eq!(extracted, 10.0);
}

#[test]
#[should_panic(expected = "CoMonad::extract cannot be called on an empty CausalTensor.")]
fn test_comonad_causal_tensor_extract_empty_panics() {
    let empty_tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    CausalTensorWitness::extract(&empty_tensor);
}

#[test]
fn test_comonad_causal_tensor_extract_non_scalar_first_element() {
    let vector_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let extracted = CausalTensorWitness::extract(&vector_tensor);
    // Arbitrary choice, should extract the first element
    assert_eq!(extracted, 1.0);
}

#[test]
fn test_comonad_causal_tensor_extend_scalar() {
    let scalar_tensor = CausalTensor::new(vec![5.0], vec![]).unwrap();
    // Function that observes the context (the scalar tensor) and returns a new value
    let f = |ct: &CausalTensor<f64>| ct.data()[0] * 2.0;
    let extended = CausalTensorWitness::extend(&scalar_tensor, f);
    assert_eq!(extended, CausalTensor::new(vec![10.0], vec![]).unwrap());
}

#[test]
fn test_comonad_causal_tensor_extend_non_scalar() {
    let vector_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    // Function that observes the context (the vector tensor) and returns a summary value
    // Requires T: Add for sum()
    let f = |ct: &CausalTensor<f64>| ct.data().iter().cloned().sum::<f64>(); // Added .cloned() for sum
    let extended = CausalTensorWitness::extend(&vector_tensor, f);
    // The result should be a scalar tensor containing the sum of the vector elements
    assert_eq!(
        extended,
        CausalTensor::new(vec![6.0, 6.0, 6.0], vec![3]).unwrap()
    );
}

#[test]
fn test_comonad_causal_tensor_extend_topology_check() {
    let vector_tensor = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();

    // The Law: "My value plus the value of the element to my right"
    // This relies on the Shifted View putting 'Me' at 0 and 'Neighbor' at 1.
    let f = |ct: &CausalTensor<f64>| {
        let me = ct.data()[0];
        // If strictly 1D, neighbor is at 1.
        // Ideally verify size > 1 to avoid panic, but for test we know input is len 3.
        let neighbor = ct.data()[1];
        me + neighbor
    };

    let extended = CausalTensorWitness::extend(&vector_tensor, f);

    // Expected: [10+20, 20+30, 30+10] -> [30, 50, 40]
    let expected = CausalTensor::new(vec![30.0, 50.0, 40.0], vec![3]).unwrap();

    assert_eq!(
        extended, expected,
        "Topology check failed: Shift/Wrap-around logic is incorrect"
    );
}

// --- BoundedAdjunction Tests ---

#[test]
fn test_bounded_adjunction_causal_tensor_unit_scalar() {
    let ctx = vec![]; // Scalar shape context
    let a = 42;
    let t_t_a = CausalTensorWitness::unit(&ctx, a);

    // Expected: CausalTensor<CausalTensor<i32>> where inner is CausalTensor<i32> scalar
    // and outer is CausalTensor<CausalTensor<i32>> scalar.
    assert_eq!(t_t_a.num_dim(), 0); // Outer is scalar
    assert_eq!(t_t_a.len(), 1);

    let inner_tensor = &t_t_a.data()[0];
    assert_eq!(inner_tensor.num_dim(), 0); // Inner is scalar
    assert_eq!(inner_tensor.as_slice(), &[42]);
}

#[test]
#[should_panic(
    expected = "BoundedAdjunction::unit for CausalTensor requires an empty shape vector (Scalar). Provided shape: [1]"
)]
fn test_bounded_adjunction_causal_tensor_unit_non_scalar_panics() {
    let ctx = vec![1]; // Non-scalar shape context (volume is 1, but it's not empty, which means it's a vector of length 1)
    let a = 42;
    CausalTensorWitness::unit(&ctx, a);
}

#[test]
fn test_bounded_adjunction_causal_tensor_counit() {
    let ctx = vec![]; // Context doesn't matter for counit, but pass it anyway.
    let inner_tensor = CausalTensor::new(vec![100], vec![]).unwrap();
    let lrb = CausalTensor::new(vec![inner_tensor], vec![]).unwrap(); // CausalTensor<CausalTensor<i32>>

    let b = CausalTensorWitness::counit(&ctx, lrb);
    assert_eq!(b, 100);
}

#[test]
fn test_bounded_adjunction_causal_tensor_left_adjunct() {
    let ctx = vec![]; // Scalar context for 'unit' within left_adjunct
    let a = 5;
    // f: CausalTensor<A> -> B
    let f = |ct_a: CausalTensor<i32>| ct_a.as_slice()[0] * 2; // Assumes scalar input tensor

    let result_tensor = CausalTensorWitness::left_adjunct(&ctx, a, f);

    // Expected: CausalTensor<B> (a scalar tensor containing 10)
    assert_eq!(result_tensor.num_dim(), 0);
    assert_eq!(result_tensor.as_slice(), &[10]);
}

#[test]
fn test_bounded_adjunction_causal_tensor_right_adjunct() {
    let ctx = vec![]; // Context for 'counit' within right_adjunct
    let la = CausalTensor::new(vec![3], vec![]).unwrap(); // L(A) = CausalTensor<A>
    // f: A -> CausalTensor<B>
    let f = |val_a: i32| CausalTensor::new(vec![val_a + 7], vec![]).unwrap(); // Returns scalar tensor

    let b = CausalTensorWitness::right_adjunct(&ctx, la, f);

    // Expected: B (the extracted value from the resulting CausalTensor<B>)
    assert_eq!(b, 10);
}
