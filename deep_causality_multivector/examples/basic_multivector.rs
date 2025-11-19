/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- CausalMultiVector Basic Usage ---");

    // 1. Define a Metric (e.g., Euclidean 2D)
    let metric = Metric::Euclidean(2);
    println!("Metric: {}", metric);

    // 2. Create Basis Vectors
    // Size is 2^2 = 4. Indices: 0=scalar, 1=e1, 2=e2, 3=e1e2
    let mut data_e1 = vec![0.0; 4];
    data_e1[1] = 1.0; // e1
    let e1 = CausalMultiVector::new(data_e1, metric)?;

    let mut data_e2 = vec![0.0; 4];
    data_e2[2] = 1.0; // e2
    let e2 = CausalMultiVector::new(data_e2, metric)?;

    println!("e1 created");
    println!("e2 created");

    // 3. Geometric Product: e1 * e2 = e1e2
    let e1e2 = e1.clone() * e2.clone();
    println!("e1 * e2 (Geometric Product):");
    // Expect index 3 (binary 11) to be 1.0
    if let Some(val) = e1e2.get(3) {
        println!("  Component e1e2 (idx 3): {}", val);
        assert_eq!(*val, 1.0);
    }

    // 4. Geometric Product: e2 * e1 = -e1e2
    let e2e1 = e2.clone() * e1.clone();
    println!("e2 * e1 (Geometric Product):");
    if let Some(val) = e2e1.get(3) {
        println!("  Component e1e2 (idx 3): {}", val);
        assert_eq!(*val, -1.0);
    }

    // 5. Squaring: e1 * e1 = 1 (Euclidean)
    let e1_sq = e1.clone() * e1.clone();
    println!("e1 * e1:");
    if let Some(val) = e1_sq.get(0) {
        println!("  Scalar component (idx 0): {}", val);
        assert_eq!(*val, 1.0);
    }

    // 6. Outer Product: e1 ^ e2 = e1e2
    let e1_wedge_e2 = e1.outer_product(&e2);
    println!("e1 ^ e2 (Outer Product):");
    if let Some(val) = e1_wedge_e2.get(3) {
        println!("  Component e1e2 (idx 3): {}", val);
        assert_eq!(*val, 1.0);
    }

    // 7. Inner Product: e1 . e1 = 1
    let e1_dot_e1 = e1.inner_product(&e1);
    println!("e1 . e1 (Inner Product):");
    if let Some(val) = e1_dot_e1.get(0) {
        println!("  Scalar component (idx 0): {}", val);
        assert_eq!(*val, 1.0);
    }

    // 8. Inverse
    // (e1e2)^2 = -1, so inverse should be -(e1e2) = e2e1
    let inv_e1e2 = e1e2.inverse()?;
    println!("Inverse of e1e2:");
    if let Some(val) = inv_e1e2.get(3) {
        println!("  Component e1e2 (idx 3): {}", val);
        assert_eq!(*val, -1.0);
    }

    println!("--- All examples passed ---");
    Ok(())
}
