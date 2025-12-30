/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{DixonAlgebra, MultiVector};
use deep_causality_num::Complex;

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// High-energy particle physics (Standard Model) relies on complex symmetries (SU(3)xSU(2)xU(1)).
// Dixon Algebra (Cl_C(6)) provides a mathematical structure that naturally encodes these
// symmetries using Octonions and Complex numbers.
//
// This example demonstrates how to construct and manipulate these high-dimensional
// algebraic structures. This capability is essential for researchers modeling
// Quantum Field Theory (QFT) or exploring Beyond Standard Model (BSM) physics
// within a computational framework.
// -----------------------------------------------------------------------------------------
fn main() {
    println!("=== Dixon Algebra (Cl_C(6)) Example ===");
    println!("This algebra operates on Octonions in particle physics models (e.g. C. Furey).");

    // 1. Create the Zero Vector in Cl_C(6)
    // The dimension is 2^6 = 64.
    // Coefficients are Complex64.
    let data = vec![Complex::new(0.0, 0.0); 64];
    let d = DixonAlgebra::new_dixon_algebra_left(data);

    println!("\nAlgebra Properties:");
    println!("  Metric: {}", d.metric());
    println!("  Dimension (N): {}", d.metric().dimension());
    println!("  Total Coefficients: {}", d.data().len());

    // 2. Construct Basis Vectors e1 and e2
    // e1 corresponds to index 2^0 = 1
    // e2 corresponds to index 2^1 = 2
    let mut e1_data = vec![Complex::new(0.0, 0.0); 64];
    e1_data[1] = Complex::new(1.0, 0.0);
    let e1 = DixonAlgebra::new_dixon_algebra_left(e1_data);

    let mut e2_data = vec![Complex::new(0.0, 0.0); 64];
    e2_data[2] = Complex::new(1.0, 0.0);
    let e2 = DixonAlgebra::new_dixon_algebra_left(e2_data);

    println!("\nBasis Vectors:");
    println!("  e1 coeff at index 1: {}", e1.data()[1]);
    println!("  e2 coeff at index 2: {}", e2.data()[2]);

    // 3. Verify Anti-Commutation: e1 * e2 = - e2 * e1
    let e1e2 = e1.clone() * e2.clone();
    let e2e1 = e2.clone() * e1.clone();

    // e1e2 should have coefficient 1.0 at index 1^2 = 3
    // e2e1 should have coefficient -1.0 at index 3
    println!("\nGeometric Product:");
    println!("  e1 * e2 (index 3): {}", e1e2.data()[3]);
    println!("  e2 * e1 (index 3): {}", e2e1.data()[3]);

    let sum = e1e2.clone() + e2e1.clone();
    println!("  (e1 * e2) + (e2 * e1) (should be 0): {}", sum.data()[3]);

    // 4. Complex Coefficients
    // Let's multiply e1 by i
    let i = Complex::new(0.0, 1.0);
    let ie1 = e1.clone() * i;
    println!("\nComplex Scalar Multiplication:");
    println!("  i * e1 (index 1): {}", ie1.data()[1]);

    // 5. Squared Magnitude
    // In Euclidean metric, e1*e1 = 1.
    // Magnitude squared is scalar part of A * ~A
    let mag_sq = e1.squared_magnitude();
    println!("\nSquared Magnitude of e1: {}", mag_sq);

    println!("\nDixon Algebra example executed successfully.");
}
