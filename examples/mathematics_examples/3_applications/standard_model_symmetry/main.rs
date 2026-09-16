/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Standard Model symmetry in the Dixon algebra
//!
//! High-energy particle physics encodes its gauge group `SU(3) × SU(2) × U(1)` in algebraic
//! structure. The Dixon algebra `Cl_C(6)` carries that structure using octonions and complex
//! numbers, which is the setting C. Furey and others use for Standard Model constructions.
//!
//! `Cl_C(6)` has `2^6 = 64` coefficients, each a complex number. This example builds two basis
//! vectors in it, checks that they anticommute, and reads the magnitude back out.
//!
//! `DixonAlgebra` is an alias for `CausalMultiVector<Complex64>`, so the element type is
//! complex and the component scalar is `f64`.

use deep_causality_multivector::{DixonAlgebra, MultiVector};
use deep_causality_num::{lift, lower};
use deep_causality_num_complex::Complex;

/// `Cl_C(6)` holds `2^6` coefficients.
const COEFFICIENTS: usize = 64;
/// `e1` sits at `2^0`, `e2` at `2^1`, and their product at `1 ^ 2`.
const E1: usize = 1;
const E2: usize = 2;
const E1E2: usize = 3;

/// The working scalar: the real and imaginary components of every coefficient carry it.
pub type FloatType = f64;

fn main() {
    // The zero element, which fixes the shape of the algebra.
    let zero = DixonAlgebra::new_dixon_state_space(vec![complex(0.0, 0.0); COEFFICIENTS]);
    print_properties(
        &zero.metric().to_string(),
        zero.metric().dimension(),
        zero.data().len(),
    );

    // Two basis vectors.
    let e1 = basis(E1);
    let e2 = basis(E2);
    print_basis(e1.data()[E1], e2.data()[E2]);

    // Anticommutation: e1 e2 = -e2 e1, so the two products sum to zero.
    let e1e2 = e1.clone() * e2.clone();
    let e2e1 = e2.clone() * e1.clone();
    let sum = e1e2.clone() + e2e1.clone();
    print_product(e1e2.data()[E1E2], e2e1.data()[E1E2], sum.data()[E1E2]);

    // The coefficients are complex, so a complex scalar multiplies straight through.
    let i = complex(0.0, 1.0);
    let ie1 = e1.clone() * i;
    print_scalar_mult(ie1.data()[E1]);

    // The magnitude is the scalar part of A ~A, and it stays complex because the coefficients
    // are. `Cl_C(6)` carries a NonEuclidean signature, so e1 squares to -1 here.
    print_magnitude(e1.squared_magnitude());
}

/// A complex coefficient in the working scalar.
fn complex(re: f64, im: f64) -> Complex<FloatType> {
    Complex::new(lift::<FloatType>(re), lift::<FloatType>(im))
}

/// The unit basis vector at one coefficient index.
fn basis(index: usize) -> DixonAlgebra {
    let mut data = vec![complex(0.0, 0.0); COEFFICIENTS];
    data[index] = complex(1.0, 0.0);
    DixonAlgebra::new_dixon_state_space(data)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// Renders a complex coefficient. The display boundary: `f64` appears here and nowhere else.
fn shown(z: Complex<FloatType>) -> Complex<f64> {
    Complex::new(lower(z.re), lower(z.im))
}

fn print_properties(metric: &str, dimension: usize, coefficients: usize) {
    println!("=== Dixon Algebra (Cl_C(6)) Example ===");
    println!("This algebra operates on Octonions in particle physics models (e.g. C. Furey).");
    println!("\nAlgebra Properties:");
    println!("  Precision: {}", core::any::type_name::<FloatType>());
    println!("  Metric: {metric}");
    println!("  Dimension (N): {dimension}");
    println!("  Total Coefficients: {coefficients}");
}

fn print_basis(e1: Complex<FloatType>, e2: Complex<FloatType>) {
    println!("\nBasis Vectors:");
    println!("  e1 coeff at index {E1}: {}", shown(e1));
    println!("  e2 coeff at index {E2}: {}", shown(e2));
}

fn print_product(e1e2: Complex<FloatType>, e2e1: Complex<FloatType>, sum: Complex<FloatType>) {
    println!("\nGeometric Product:");
    println!("  e1 * e2 (index {E1E2}): {}", shown(e1e2));
    println!("  e2 * e1 (index {E1E2}): {}", shown(e2e1));
    println!("  (e1 * e2) + (e2 * e1) (should be 0): {}", shown(sum));
}

fn print_scalar_mult(ie1: Complex<FloatType>) {
    println!("\nComplex Scalar Multiplication:");
    println!("  i * e1 (index {E1}): {}", shown(ie1));
}

fn print_magnitude(magnitude: Complex<FloatType>) {
    println!("\nSquared Magnitude of e1: {}", shown(magnitude));
    println!("\nDixon Algebra example executed successfully.");
}
