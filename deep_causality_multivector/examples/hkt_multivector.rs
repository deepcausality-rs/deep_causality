/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, Monad, Pure};
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
// - Monad: Chain operations that can change the dimensionality (e.g., Tensor Products).
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

    // 3. Monad: Tensor Product via Bind
    println!("\n--- Monad (Bind / Tensor Product) ---");
    println!("Geometric Interpretation: Combining dimensions.");

    // Start with a 1D Euclidean vector (size 2: scalar, e1)
    let m1 = Metric::Euclidean(1);
    let v1 = CausalMultiVector::new(vec![1.0, 2.0], m1).unwrap();
    println!("Vector A (1D): {:?}", v1.data());

    // Bind function: For each coefficient in A, create a new 1D vector.
    // This effectively creates a 2D structure (size 4).
    // f(x) -> [x, -x]
    let bind_fn = |x: f64| {
        let m_inner = Metric::Euclidean(1);
        CausalMultiVector::new(vec![x, -x], m_inner).unwrap()
    };

    let tensor_product = CausalMultiVectorWitness::bind(v1, bind_fn);

    println!(
        "Resulting Vector (Tensor Product): {:?}",
        tensor_product.data()
    );
    println!("Resulting Metric: {}", tensor_product.metric());

    // Expected: [1.0, -1.0, 2.0, -2.0]
    // Metric should be Euclidean(2) (1 + 1)
    assert_eq!(tensor_product.data(), &vec![1.0, -1.0, 2.0, -2.0]);
    assert_eq!(tensor_product.metric().dimension(), 2);

    println!("\nAll HKT examples executed successfully.");
}
