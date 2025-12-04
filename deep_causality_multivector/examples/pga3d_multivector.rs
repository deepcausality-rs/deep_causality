/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::MultiVector;
use deep_causality_multivector::alias::PGA3DMultiVector;

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// Projective Geometric Algebra (PGA) is the modern standard for Computer Graphics and Robotics.
// Unlike standard Linear Algebra, PGA represents points, lines, and planes uniformly and
// handles rigid body motions (translations + rotations) using a single "Motor" algebra.
//
// This example demonstrates:
// 1. Representing Points as dual tri-vectors.
// 2. Creating a Translator (Motor) for movement.
// 3. Applying the transformation efficiently.
//
// This simplifies kinematic chains in robotics and collision detection in physics engines.
// -----------------------------------------------------------------------------------------

fn main() {
    println!("=== 3D Projective Geometric Algebra (PGA) Example ===");

    // 1. Create a point at (1, 2, 3)
    // In PGA, points are dual tri-vectors.
    let p = PGA3DMultiVector::new_point(1.0, 2.0, 3.0);
    println!("\nOriginal Point P:");
    // We can inspect the coefficients.
    // e123 (14) is the homogeneous coordinate w.
    println!("  w (e123): {:.2}", p.get(14).unwrap_or(&0.0));
    println!("  x (e032): {:.2}", p.get(13).unwrap_or(&0.0));
    println!("  y (e013): {:.2}", p.get(11).unwrap_or(&0.0));
    println!("  z (e021): {:.2}", p.get(7).unwrap_or(&0.0));

    // 2. Create a translator (motor)
    // Move by vector d = (2, 0, 0) -> Shift x by +2.
    let t = PGA3DMultiVector::translator(2.0, 0.0, 0.0);
    println!("\nTranslator T (dx=2, dy=0, dz=0):");
    println!("  Scalar: {:.2}", t.get(0).unwrap_or(&0.0));
    println!("  e01: {:.2}", t.get(3).unwrap_or(&0.0));

    // 3. Apply the transformation
    // In PGA, points transform via the sandwich product (or just geometric product for single-sided if normalized?)
    // Standard motor transformation: P' = T * P * ~T
    // However, for simple translators acting on points, T * P is often sufficient to see the shift in the dual space representation
    // if we interpret it correctly, but let's stick to the sandwich product P' = T P ~T for general motors.

    let t_rev = t.reversion();
    let p_prime = t * p.clone() * t_rev;

    println!("\nTransformed Point P' = T * P * ~T:");
    println!("  w (e123): {:.2}", p_prime.get(14).unwrap_or(&0.0));
    println!("  x (e032): {:.2}", p_prime.get(13).unwrap_or(&0.0));
    println!("  y (e013): {:.2}", p_prime.get(11).unwrap_or(&0.0));
    println!("  z (e021): {:.2}", p_prime.get(7).unwrap_or(&0.0));

    // Expected result: (3, 2, 3)
    // x should be 1 + 2 = 3.
    // y should be 2.
    // z should be 3.

    let x_prime = *p_prime.get(13).unwrap_or(&0.0);
    let y_prime = *p_prime.get(11).unwrap_or(&0.0);
    let z_prime = *p_prime.get(7).unwrap_or(&0.0);

    println!(
        "\nResult Coordinates: ({:.2}, {:.2}, {:.2})",
        x_prime, y_prime, z_prime
    );

    assert!((x_prime - 3.0).abs() < 1e-6, "X coordinate should be 3.0");
    assert!((y_prime - 2.0).abs() < 1e-6, "Y coordinate should be 2.0");
    assert!((z_prime - 3.0).abs() < 1e-6, "Z coordinate should be 3.0");

    println!("\nPGA3D example executed successfully.");
}
