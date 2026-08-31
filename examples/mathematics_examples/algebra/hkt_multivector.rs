/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, Pure};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, Metric};

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// Modern software engineering relies on composable abstractions. Higher-Kinded Types (HKT)
// allow us to define generic operations (Map, Apply, Bind) that work across different
// data structures (Vectors, Tensors, Trees).
//
// This example demonstrates how `CausalMultiVector` implements functional patterns:
// - Functor: Safely transform coefficients without changing geometry.
// - Applicative: Broadcast functions across vector structures.
//
// There is deliberately no Monad. A multivector holds exactly 2^dim coefficients, where dim comes
// from its Metric, so an operation that changes the coefficient count changes the algebra it lives
// in. `bind` cannot do that lawfully: its two identity laws demand the metric come from opposite
// places, and `pure` has no metric of its own to reconcile them. The tensor product below is that
// dimension-changing operation, written directly.
//
// This enables "Algebraic Programming" where complex physics pipelines are built from
// small, verifiable, and reusable functional blocks.
// -----------------------------------------------------------------------------------------

fn main() {
    println!("=== Higher-Kinded Types (HKT) with CausalMultiVector ===");

    // 1. Functor: Mapping over coefficients
    println!("\n--- Functor (Map) ---");
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    println!("Original Vector: {:?}", v.data());

    // Scale by 2.0 using fmap
    let scaled = CausalMultiVectorWitness::fmap(v.clone(), |x| x * 2.0);
    println!("Scaled Vector (x2): {:?}", scaled.data());
    assert_eq!(scaled.data(), &vec![2.0, 4.0, 6.0, 8.0]);

    // 2. Applicative: Broadcasting a function
    println!("\n--- Applicative (Apply/Broadcast) ---");
    // Create a "pure" function wrapped in a scalar multivector
    let pure_fn = CausalMultiVectorWitness::pure(|x: f64| x + 10.0);

    // Apply it to our vector
    let shifted = CausalMultiVectorWitness::apply(pure_fn, v.clone());
    println!("Shifted Vector (+10): {:?}", shifted.data());
    assert_eq!(shifted.data(), &vec![11.0, 12.0, 13.0, 14.0]);

    // 3. Tensor product: an operation that changes the algebra
    println!("\n--- Tensor Product (dimension-changing) ---");
    println!("Geometric Interpretation: Combining dimensions.");

    // Start with a 1D Euclidean vector (size 2: scalar, e1)
    let m1 = Metric::Euclidean(1);
    let v1 = CausalMultiVector::new(vec![1.0, 2.0], m1).unwrap();
    println!("Vector A (1D): {:?}", v1.data());

    // For each coefficient x of A, produce [x, -x], then concatenate. Two coefficients each from
    // two coefficients gives four, so the result lives in Cl(2) rather than Cl(1).
    //
    // This is why it is not `bind`. A monadic bind would have to pick one metric for the result,
    // and neither the input's Cl(1) nor the closure's Cl(1) is the answer: the answer is Cl(2),
    // which neither operand carries. The dimension is a property of the operation, so the operation
    // states it.
    let expanded: Vec<f64> = v1.data().iter().flat_map(|&x| [x, -x]).collect();
    let tensor_product = CausalMultiVector::new(expanded, Metric::Euclidean(2))
        .expect("four coefficients is exactly 2^2, the dimension of Cl(2)");

    println!(
        "Resulting Vector (Tensor Product): {:?}",
        tensor_product.data()
    );
    println!("Resulting Metric: {}", tensor_product.metric());

    assert_eq!(tensor_product.data(), &vec![1.0, -1.0, 2.0, -2.0]);
    assert_eq!(tensor_product.metric().dimension(), 2);

    println!("\nAll HKT examples executed successfully.");
}
