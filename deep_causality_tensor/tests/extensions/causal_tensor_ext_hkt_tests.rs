/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, HKT, Monad};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

// --- HKT Tests ---

#[test]
fn test_hkt_causal_tensor_witness() {
    let value: <CausalTensorWitness as HKT>::Type<f64> =
        CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert_eq!(value.as_slice(), &[1.0, 2.0, 3.0]);

    let empty_value: <CausalTensorWitness as HKT>::Type<f64> =
        CausalTensor::new(vec![], vec![0]).unwrap();
    assert!(empty_value.is_empty());
}

// --- Applicative Tests ---

#[test]
fn test_applicative_causal_tensor_pure() {
    let tensor = CausalTensorWitness::pure(42.0);
    assert_eq!(tensor.as_slice(), &[42.0]);
    assert_eq!(tensor.shape(), &[] as &[usize]); // Scalar tensor
}

#[test]
fn test_applicative_causal_tensor_apply_scalar_func() {
    // Explicitly type the function as a function pointer `fn(f64) -> f64`
    // This satisfies Satisfies<TensorConstraint>
    let f: fn(f64) -> f64 = |x: f64| x * 2.0;
    let f_tensor = CausalTensor::new(vec![f], vec![]).unwrap();
    let a_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let result_tensor = CausalTensorWitness::apply(f_tensor, a_tensor);
    assert_eq!(result_tensor.as_slice(), &[2.0, 4.0, 6.0]);
    assert_eq!(result_tensor.shape(), &[3]);
}

#[test]
fn test_applicative_causal_tensor_apply_non_scalar_func() {
    // For hetero closures or just to demonstrate Box support:
    let f1: Box<dyn Fn(f64) -> f64> = Box::new(|x: f64| x * 2.0);
    let f2: Box<dyn Fn(f64) -> f64> = Box::new(|x: f64| x * 3.0);

    // Create vector of Boxed functions
    let f_tensor = CausalTensor::from_vec(vec![f1, f2], &[2]);

    // Use matching shape (2 elements)
    let a_tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let result_tensor = CausalTensorWitness::apply(f_tensor, a_tensor);

    // Expect correct application: [1.0 * 2.0, 2.0 * 3.0] = [2.0, 6.0]
    assert_eq!(result_tensor.as_slice(), &[2.0, 6.0]);
    assert_eq!(result_tensor.shape(), &[2]);
}

// --- Functor Tests ---

#[test]
fn test_functor_causal_tensor() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let f = |x| x * 2.0; // Closure is fine here as it's not stored in Tensor
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert_eq!(mapped_tensor.as_slice(), &[2.0, 4.0, 6.0]);
    assert_eq!(mapped_tensor.shape(), &[3]);
}

#[test]
fn test_functor_causal_tensor_empty() {
    let tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    let f = |x| x * 2.0;
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert!(mapped_tensor.is_empty());
    assert_eq!(mapped_tensor.shape(), &[0]);
}

#[test]
fn test_functor_causal_tensor_type_change() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let f = |x: f64| x as f32; // Change type from f64 to f32
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert_eq!(mapped_tensor.as_slice(), &[1.0f32, 2.0f32, 3.0f32]);
    assert_eq!(mapped_tensor.shape(), &[3]);
}

// --- Foldable Tests ---

#[test]
fn test_foldable_causal_tensor_sum() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let sum = CausalTensorWitness::fold(tensor, 0.0, |acc, x| acc + x);
    assert_eq!(sum, 15.0);
}

#[test]
fn test_foldable_causal_tensor_empty() {
    let tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    let sum = CausalTensorWitness::fold(tensor, 0.0, |acc, x| acc + x);
    assert_eq!(sum, 0.0);
}

// --- Monad Tests ---

#[test]
fn test_monad_causal_tensor_bind() {
    let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let f = |x: f64| CausalTensor::new(vec![x, x * 10.0], vec![2]).unwrap();
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert_eq!(bound_tensor.as_slice(), &[1.0, 10.0, 2.0, 20.0]);
    assert_eq!(bound_tensor.shape(), &[4]); // Flattened to 1D
}

#[test]
fn test_monad_causal_tensor_bind_empty() {
    let tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    let f = |x: f64| CausalTensor::new(vec![x, x * 10.0], vec![2]).unwrap();
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert!(bound_tensor.is_empty());
    assert_eq!(bound_tensor.shape(), &[0]);
}

#[test]
fn test_monad_causal_tensor_bind_to_empty() {
    let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let f = |_x: f64| CausalTensor::<f64>::new(vec![], vec![0]).unwrap();
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert!(bound_tensor.is_empty());
    assert_eq!(bound_tensor.shape(), &[0]);
}

#[test]
fn test_monad_causal_tensor_nesting() {
    // Nested tensor test.
    // CausalTensor<CausalTensor<f64>> satisfies Constraint.
    let inner = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let tensor = CausalTensor::new(vec![inner], vec![1]).unwrap(); // Tensor of Tensor

    // Bind: Flatten
    let flattened = CausalTensorWitness::bind(tensor, |x| x);
    assert_eq!(flattened.as_slice(), &[1.0, 2.0]);
}

// --- CoMonad Tests ---

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
    assert_eq!(extracted, 1.0);
}

#[test]
fn test_comonad_causal_tensor_extend_scalar() {
    let scalar_tensor = CausalTensor::new(vec![5.0], vec![]).unwrap();
    let f = |ct: &CausalTensor<f64>| ct.data()[0] * 2.0;
    let extended = CausalTensorWitness::extend(&scalar_tensor, f);
    assert_eq!(extended, CausalTensor::new(vec![10.0], vec![]).unwrap());
}

#[test]
fn test_comonad_causal_tensor_extend_non_scalar() {
    let vector_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let f = |ct: &CausalTensor<f64>| ct.data().iter().cloned().sum::<f64>();
    let extended = CausalTensorWitness::extend(&vector_tensor, f);
    assert_eq!(
        extended,
        CausalTensor::new(vec![6.0, 6.0, 6.0], vec![3]).unwrap()
    );
}

#[test]
fn test_comonad_causal_tensor_extend_topology_check() {
    let vector_tensor = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let f = |ct: &CausalTensor<f64>| {
        let me = ct.data()[0];
        let neighbor = ct.data()[1];
        me + neighbor
    };
    let extended = CausalTensorWitness::extend(&vector_tensor, f);
    let expected = CausalTensor::new(vec![30.0, 50.0, 40.0], vec![3]).unwrap();

    assert_eq!(
        extended, expected,
        "Topology check failed: Shift/Wrap-around logic is incorrect"
    );
}
