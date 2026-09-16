/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The algebra tower: one signature for every scalar
//!
//! A function bounded on `RealField` is written once and runs at every supported precision.
//!
//! ```text
//! Real       analytic: sqrt, exp, ln, sin, ordering, rounding, constants
//! Field      invertible: a total multiplicative inverse, so division is defined
//! RealField  Real + Field + ToPrimitive
//! ```
//!
//! The split between `Real` and `Field` is the part that matters. A dual number
//! `a + b·ε` with `ε² = 0` has every elementary function by the chain rule, so it is analytic —
//! but `ε` is a zero divisor, so `Field`'s promise that every non-zero element has an inverse
//! fails, and `Dual` is not a `RealField`. Bound a function on `RealField` and automatic
//! differentiation cannot pass through it. Bound the same function on `Real` and it can, at no
//! cost to the concrete scalars, because every `RealField` is a `Real`.
//!
//! That decides whether a model is differentiable.

use core::ops::Div;
use deep_causality_algebra::{Real, RealField};
use deep_causality_num::FromPrimitive;
use deep_causality_num::{BFloat16, Float106, const_scalar_from_int, lift, lift_usize, lower};
use deep_causality_num_dual::Dual;

/// Terms of the series, chosen so `f32` runs out of significand before the series runs out
/// of terms.
const TERMS: usize = 25;

/// The working scalar of the *display* path only. Each computation below picks its own, which
/// is the entire point of the example.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

fn main() {
    print_header();

    // ---------------------------------------------------------------------
    // 1. One function, four precisions.
    // ---------------------------------------------------------------------
    // `exp_series` is written once against `RealField`. Nothing in its body names a concrete
    // type, so each row below is the same code with a different scalar substituted.
    let at = 1.0;
    print_precision_table(
        lower(exp_series::<f32>(lift(at))),
        lower(exp_series::<f64>(lift(at))),
        lower(exp_series::<BFloat16>(lift(at))),
        lower(exp_series::<Float106>(lift(at))),
        core::f64::consts::E,
    );

    // ---------------------------------------------------------------------
    // 2. The same body, bounded on `Real` instead, admits a dual number.
    // ---------------------------------------------------------------------
    // `Dual` is analytic but not a field, so it satisfies `Real` and fails `RealField`.
    // Bounding on the weaker trait is what lets the derivative flow through.
    // The bounds, checked by the compiler rather than asserted in prose.
    requires_real_field::<f64>();
    requires_real::<f64>();
    requires_real::<Dual<f64>>();
    // requires_real_field::<Dual<f64>>();  // rejected: Dual is not a Field

    let seeded = Dual::variable(ONE);
    let evaluated = exp_series_analytic(seeded);
    print_dual(evaluated.re, evaluated.du);

    // d/dx exp(x) = exp(x), so the value and the derivative of this series agree.
    let gap = lower(evaluated.re) - lower(evaluated.du);
    print_derivative_check(gap);
    assert!(gap.abs() < 1e-12);

    print_footer();
}

/// `exp(x)` by its Taylor series, over any real field.
///
/// Nothing in this body names a concrete type. `RealField` supplies the arithmetic and
/// `FromPrimitive` the crossing from the loop counter into the scalar.
fn exp_series<T: RealField + FromPrimitive>(x: T) -> T {
    let mut term = T::one();
    let mut sum = T::one();
    for k in 1..=TERMS {
        term = term * x / lift_usize::<T>(k);
        sum += term;
    }
    sum
}

/// The same series, one bound weaker: `Real` instead of `RealField`.
///
/// The body is identical. What changed is the promise: `Real` asks for the analytic surface
/// and a division that works, not for `Field`'s guarantee that *every* non-zero element has an
/// inverse. `Dual` can honour the first and not the second -- dividing by a pure-ε dual has no
/// answer -- so it satisfies this signature and fails the one above.
fn exp_series_analytic<T: Real + FromPrimitive + Div<Output = T>>(x: T) -> T {
    let mut term = T::one();
    let mut sum = T::one();
    for k in 1..=TERMS {
        term = term * x / lift_usize::<T>(k);
        sum += term;
    }
    sum
}

/// Accepts any real field. `f64` passes; `Dual<f64>` does not, because it is not a `Field`.
fn requires_real_field<T: RealField>() {}

/// Accepts any analytic scalar, field or not. Both `f64` and `Dual<f64>` pass.
fn requires_real<T: Real>() {}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== The algebra tower: one signature, every scalar ===\n");
    println!("  exp(1) by Taylor series, {TERMS} terms, written once against `RealField`.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_precision_table(f32_v: f64, f64_v: f64, bf16_v: f64, f106_v: f64, exact: f64) {
    println!("--- 1. The same body at four precisions ---");
    println!("  {:<10} {:>22} {:>12}", "scalar", "exp(1)", "abs error");
    for (name, value) in [
        ("BFloat16", bf16_v),
        ("f32", f32_v),
        ("f64", f64_v),
        ("Float106", f106_v),
    ] {
        println!(
            "  {:<10} {:>22.17} {:>12.2e}",
            name,
            value,
            (value - exact).abs()
        );
    }
    println!("  {:<10} {:>22.17}", "exact", exact);
    println!("\n  One `impl`, no `match` on a type, no duplicated body.");
}

fn print_dual(value: FloatType, derivative: FloatType) {
    println!("\n--- 2. The same series over `Dual`, bounded on `Real` ---");
    println!("  `Dual` is analytic but not a field, so it is a `Real` and not a `RealField`.");
    println!("  value      = {:.12}", lower(value));
    println!("  derivative = {:.12}", lower(derivative));
}

fn print_derivative_check(gap: f64) {
    println!("  d/dx exp(x) = exp(x), so the two agree to {gap:.2e}.");
}

fn print_footer() {
    println!("\n--- What the bound decided ---");
    println!("  Both bodies divide. The difference is what they promise: `Field` guarantees");
    println!("  that *every* non-zero element has an inverse, and `Dual` cannot -- dividing by");
    println!("  a pure-epsilon dual has no answer. `Real` asks only for the analytic surface,");
    println!("  so the same series admits the type that carries a derivative.");
    println!("  That choice is made once, in a signature, long before any caller shows up.");
}
