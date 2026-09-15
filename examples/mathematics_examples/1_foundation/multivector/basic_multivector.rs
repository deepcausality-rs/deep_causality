/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `CausalMultiVector`: the core products
//!
//! Geometric Algebra unifies complex numbers, quaternions and vector algebra into one
//! framework. Three products carry most of it:
//!
//! ```text
//! geometric  a * b    rotation and scaling together; not commutative
//! outer      a ^ b    the subspace the two span, an area or a volume
//! inner      a . b    projection and metric contraction, a scalar
//! ```
//!
//! In `Cl(2, 0)` a multivector has `2^2 = 4` coefficients, indexed `[1, e1, e2, e12]`.

use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_num::{lift, lower};

/// Blade indices in Cl(2, 0).
const SCALAR: usize = 0;
const E1: usize = 1;
const E2: usize = 2;
const E12: usize = 3;

/// The working scalar. Every coefficient carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metric = Metric::Euclidean(2);
    print_header(&metric.to_string());

    let e1 = unit_blade(E1, metric)?;
    let e2 = unit_blade(E2, metric)?;
    print_created();

    // The geometric product does not commute: swapping the operands flips the sign.
    let e1e2 = e1.clone() * e2.clone();
    let e2e1 = e2.clone() * e1.clone();
    print_product("e1 * e2 (Geometric Product)", blade(&e1e2, E12));
    print_product("e2 * e1 (Geometric Product)", blade(&e2e1, E12));
    assert_eq!(blade(&e1e2, E12), Some(lift::<FloatType>(1.0)));
    assert_eq!(blade(&e2e1, E12), Some(lift::<FloatType>(-1.0)));

    // A Euclidean basis vector squares to +1, which is what the metric's signature says.
    let e1_sq = e1.clone() * e1.clone();
    print_scalar("e1 * e1", blade(&e1_sq, SCALAR));
    assert_eq!(blade(&e1_sq, SCALAR), Some(lift::<FloatType>(1.0)));

    // Outer and inner are the two halves the geometric product splits into.
    let wedge = e1.outer_product(&e2);
    let dot = e1.inner_product(&e1);
    print_product("e1 ^ e2 (Outer Product)", blade(&wedge, E12));
    print_scalar("e1 . e1 (Inner Product)", blade(&dot, SCALAR));
    assert_eq!(blade(&wedge, E12), Some(lift::<FloatType>(1.0)));
    assert_eq!(blade(&dot, SCALAR), Some(lift::<FloatType>(1.0)));

    // (e1e2)^2 = -1, so the inverse of e1e2 is -e1e2, which is e2e1.
    let inverse = e1e2.inverse()?;
    print_product("Inverse of e1e2", blade(&inverse, E12));
    assert_eq!(blade(&inverse, E12), Some(lift::<FloatType>(-1.0)));

    print_footer();
    Ok(())
}

/// A single unit blade in Cl(2, 0).
fn unit_blade(
    index: usize,
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    let mut data = vec![lift::<FloatType>(0.0); 4];
    data[index] = lift::<FloatType>(1.0);
    Ok(CausalMultiVector::new(data, metric)?)
}

/// One coefficient, if the index is in range.
fn blade(mv: &CausalMultiVector<FloatType>, index: usize) -> Option<FloatType> {
    mv.get(index).copied()
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header(metric: &str) {
    println!("--- CausalMultiVector Basic Usage ---");
    println!("Metric: {metric}");
}

fn print_created() {
    println!("e1 created");
    println!("e2 created");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_product(label: &str, value: Option<FloatType>) {
    println!("{label}:");
    if let Some(v) = value {
        println!("  Component e1e2 (idx {E12}): {}", lower(v));
    }
}

fn print_scalar(label: &str, value: Option<FloatType>) {
    println!("{label}:");
    if let Some(v) = value {
        println!("  Scalar component (idx {SCALAR}): {}", lower(v));
    }
}

fn print_footer() {
    println!("--- All examples passed ---");
}
