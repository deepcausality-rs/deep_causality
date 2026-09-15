/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, Pure};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, Metric};
use deep_causality_num::{lift, lower};

/// The working scalar. Every coefficient of every multivector below carries it.
pub type FloatType = f64;

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
    // 1. Functor: mapping over coefficients leaves the geometry alone.
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(
        vec![lift::<FloatType>(1.0), lift(2.0), lift(3.0), lift(4.0)],
        m,
    )
    .expect("four coefficients is exactly 2^2");
    let scaled = CausalMultiVectorWitness::fmap(v.clone(), |x| x * lift::<FloatType>(2.0));
    print_functor(v.data(), scaled.data());
    assert_eq!(
        scaled.data(),
        &vec![lift::<FloatType>(2.0), lift(4.0), lift(6.0), lift(8.0)]
    );

    // 2. Applicative: a scalar multivector holding a function broadcasts across the whole vector.
    let pure_fn = CausalMultiVectorWitness::pure(|x: FloatType| x + lift::<FloatType>(10.0));
    let shifted = CausalMultiVectorWitness::apply(pure_fn, v.clone());
    print_applicative(shifted.data());
    assert_eq!(
        shifted.data(),
        &vec![lift::<FloatType>(11.0), lift(12.0), lift(13.0), lift(14.0)]
    );

    // 3. Tensor product: an operation that changes the algebra, so it is written directly.
    //
    // For each coefficient x of A, produce [x, -x], then concatenate. Two coefficients each from
    // two coefficients gives four, so the result lives in Cl(2) rather than Cl(1).
    //
    // This is why it is not `bind`. A monadic bind would have to pick one metric for the result,
    // and neither the input's Cl(1) nor the closure's Cl(1) is the answer: the answer is Cl(2),
    // which neither operand carries. The dimension is a property of the operation, so the operation
    // states it.
    let v1 = CausalMultiVector::new(
        vec![lift::<FloatType>(1.0), lift(2.0)],
        Metric::Euclidean(1),
    )
    .expect("two coefficients is exactly 2^1");
    let expanded: Vec<FloatType> = v1.data().iter().flat_map(|&x| [x, -x]).collect();
    let tensor_product = CausalMultiVector::new(expanded, Metric::Euclidean(2))
        .expect("four coefficients is exactly 2^2, the dimension of Cl(2)");

    print_tensor_product(
        v1.data(),
        tensor_product.data(),
        &tensor_product.metric().to_string(),
    );
    assert_eq!(
        tensor_product.data(),
        &vec![lift::<FloatType>(1.0), lift(-1.0), lift(2.0), lift(-2.0)]
    );
    assert_eq!(tensor_product.metric().dimension(), 2);

    print_footer();
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn shown(coefficients: &[FloatType]) -> Vec<f64> {
    coefficients.iter().map(|&x| lower(x)).collect()
}

fn print_functor(original: &[FloatType], scaled: &[FloatType]) {
    println!("=== Higher-Kinded Types (HKT) with CausalMultiVector ===");
    println!("\n--- Functor (Map) ---");
    println!("Original Vector: {:?}", shown(original));
    println!("Scaled Vector (x2): {:?}", shown(scaled));
}

fn print_applicative(shifted: &[FloatType]) {
    println!("\n--- Applicative (Apply/Broadcast) ---");
    println!("Shifted Vector (+10): {:?}", shown(shifted));
}

fn print_tensor_product(a: &[FloatType], product: &[FloatType], metric: &str) {
    println!("\n--- Tensor Product (dimension-changing) ---");
    println!("Geometric Interpretation: Combining dimensions.");
    println!("Vector A (1D): {:?}", shown(a));
    println!("Resulting Vector (Tensor Product): {:?}", shown(product));
    println!("Resulting Metric: {metric}");
}

fn print_footer() {
    println!("\nAll HKT examples executed successfully.");
}
