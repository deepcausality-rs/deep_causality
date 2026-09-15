/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::MultiVector;
use deep_causality_multivector::PGA3DMultiVector;
use deep_causality_num::lower;

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

/// Coefficient indices in the 16-slot `Cl(3, 0, 1)` layout. `x` and `z` carry a sign flip
/// because the dual basis reverses their orientation.
const W_E123: usize = 14;
const X_E032: usize = 13;
const Y_E013: usize = 11;
const Z_E021: usize = 7;
const SCALAR: usize = 0;
const E01: usize = 3;

/// The working scalar.
///
/// `PGA3DMultiVector` is an alias for `CausalMultiVector<f64>` and `new_point` / `translator`
/// take `f64` directly, so this one type pins its precision at the crate rather than leaving
/// it to the caller. The alias below names what that pin resolves to.
pub type FloatType = f64;

fn main() {
    print_header();

    // A point at (1, 2, 3). In PGA points are dual tri-vectors, and e123 is the homogeneous
    // coordinate w.
    let p = PGA3DMultiVector::new_point(1.0, 2.0, 3.0);
    print_point("\nOriginal Point P:", &p);

    // A translator is a motor: the rigid motion that shifts x by +2.
    let t = PGA3DMultiVector::translator(2.0, 0.0, 0.0);
    print_translator(coeff(&t, SCALAR), coeff(&t, E01));

    // Points transform by the sandwich product P' = T P ~T.
    let t_rev = t.reversion();
    let p_prime = t * p.clone() * t_rev;
    print_point("\nTransformed Point P' = T * P * ~T:", &p_prime);

    let (x, y, z) = (
        -coeff(&p_prime, X_E032),
        coeff(&p_prime, Y_E013),
        -coeff(&p_prime, Z_E021),
    );
    print_result(x, y, z);

    // The translation moved x from 1 to 3 and left y and z alone.
    assert!((x - 3.0).abs() < 1e-6, "X coordinate should be 3.0");
    assert!((y - 2.0).abs() < 1e-6, "Y coordinate should be 2.0");
    assert!((z - 3.0).abs() < 1e-6, "Z coordinate should be 3.0");

    print_footer();
}

/// One coefficient, defaulting to zero when the slot is not stored.
fn coeff(mv: &PGA3DMultiVector, index: usize) -> FloatType {
    mv.get(index).copied().unwrap_or(0.0)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn print_header() {
    println!("=== 3D Projective Geometric Algebra (PGA) Example ===");
}

fn print_point(title: &str, p: &PGA3DMultiVector) {
    println!("{title}");
    println!("  w (e123): {:.2}", lower(coeff(p, W_E123)));
    println!("  x (e032): {:.2}", lower(-coeff(p, X_E032)));
    println!("  y (e013): {:.2}", lower(coeff(p, Y_E013)));
    println!("  z (e021): {:.2}", lower(-coeff(p, Z_E021)));
}

fn print_translator(scalar: FloatType, e01: FloatType) {
    println!("\nTranslator T (dx=2, dy=0, dz=0):");
    println!("  Scalar: {:.2}", lower(scalar));
    println!("  e01: {:.2}", lower(e01));
}

fn print_result(x: FloatType, y: FloatType, z: FloatType) {
    println!(
        "\nResult Coordinates: ({:.2}, {:.2}, {:.2})",
        lower(x),
        lower(y),
        lower(z)
    );
}

fn print_footer() {
    println!("\nPGA3D example executed successfully.");
}
