/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Algebraic Scanner: Automated Theory Search
//!
//! Scans Clifford Algebras to find dimensions admitting complex structure (I² = -1).
//!
//! ## Key Concepts
//! - **Pseudoscalar (I)**: Highest-grade element of the algebra
//! - **Complex Structure**: When I² = -1, enabling quantum mechanics formulations
//! - **Metric Signatures**: Euclidean Cl(n,0) vs Minkowski Cl(p,q)
//!
//! ## APIs Demonstrated
//! - `CausalMultiVector::new()` - Create multivector with metric
//! - `geometric_product()` - Compute algebraic products
//! - `Metric::Euclidean(n)`, `Metric::Minkowski(n)` - Signature selection

use deep_causality_algebra::Real;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::{const_scalar_from_int, lower};

/// Highest dimension scanned. The algebra has 2^n coefficients, so this grows fast.
const MAX_DIM: usize = 9;

/// The working scalar. The pseudoscalar's coefficients carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const EIGHT: FloatType = const_scalar_from_int!(FloatType, 8);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    for dim in 1..=MAX_DIM {
        for (metric, name) in [
            (Metric::Euclidean(dim), "Euclidean"),
            (Metric::Minkowski(dim), "Minkowski"),
        ] {
            if let Some(i_sq) = complex_structure(dim, metric)? {
                print_match(dim, name, i_sq);
            }
        }
    }

    Ok(())
}

/// `I^2` when the algebra admits a complex structure, `None` otherwise.
///
/// The pseudoscalar `I` is the highest-grade element, which in this basis ordering is the last
/// coefficient. Squaring it gives a scalar, and that scalar being `-1` is what lets the algebra
/// stand in for the complex numbers.
fn complex_structure(
    dim: usize,
    metric: Metric,
) -> Result<Option<FloatType>, Box<dyn std::error::Error>> {
    let size = 1 << dim;
    let mut coefficients = vec![ZERO; size];
    coefficients[size - 1] = ONE;

    let i = CausalMultiVector::new(coefficients, metric)?;
    let scalar_part = i.geometric_product(&i).data()[0];

    // The tolerance is a multiple of the working type's epsilon, so it moves with the alias.
    let tol = EIGHT * <FloatType as Real>::epsilon();
    Ok((Real::abs(scalar_part + ONE) < tol).then_some(scalar_part))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Automated Theory Search (Algebraic Scanner) ===");
    println!("Scanning Clifford Algebras Cl(p, q) for Complex Structure (I^2 = -1)...\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_match(dim: usize, name: &str, i_sq: FloatType) {
    println!(
        "[MATCH] Dimension {}: {} signature Cl({}, {}) admits Complex Structure. I^2 = {:.4}",
        dim,
        name,
        dim,
        0,
        lower(i_sq)
    );
}
