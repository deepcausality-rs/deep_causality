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

use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Automated Theory Search (Algebraic Scanner) ===");
    println!("Scanning Clifford Algebras Cl(p, q) for Complex Structure (I^2 = -1)...\n");

    let max_dim = 9; // Limit to 9 to keep it fast (exponential growth 2^n)

    for dim in 1..=max_dim {
        check_signature(dim, Metric::Euclidean(dim), "Euclidean")?;
        check_signature(dim, Metric::Minkowski(dim), "Minkowski")?;
    }

    Ok(())
}

fn check_signature(
    dim: usize,
    metric: Metric,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = 1 << dim;
    let mut i_components = vec![0.0; size];

    // The Pseudoscalar I is the highest grade element.
    // In our basis ordering, this corresponds to the last index.
    i_components[size - 1] = 1.0;

    let i = CausalMultiVector::new(i_components, metric)?;

    // Calculate I^2
    let i_sq = i.geometric_product(&i);

    // The scalar part is at index 0
    let scalar_part: f64 = i_sq.data()[0];

    // Check if I^2 approx -1
    if (scalar_part + 1.0).abs() < 1e-6 {
        println!(
            "[MATCH] Dimension {}: {} signature Cl({}, {}) admits Complex Structure. I^2 = {:.4}",
            dim, name, dim, 0, scalar_part
        );
    } else {
        // println!("[----] Dimension {}: {} signature. I^2 = {:.4}", dim, name, scalar_part);
    }

    Ok(())
}
