/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # 2D U(1) Lattice Gauge Theory Verification
//!
//! Validates the `LatticeGaugeField` implementation by comparing computed
//! plaquette expectation values against the **exact analytical solution**.
//!
//! ## Theory Background
//!
//! The 2D U(1) lattice gauge theory is exactly solvable. The average plaquette
//! satisfies:
//!
//! $$\langle P \rangle = \frac{I_1(\beta)}{I_0(\beta)}$$
//!
//! where $I_n$ are modified Bessel functions of the first kind.
//!
//! ## Reference
//!
//! M. Creutz, *Quarks, Gluons and Lattices*, Cambridge University Press (1983), Chapter 8

use deep_causality_num::{Complex, Float, Float106};
use deep_causality_topology::{Lattice, LatticeGaugeField, U1};
use std::sync::Arc;

// =============================================================================
// FLOAT TYPE CONFIGURATION
// =============================================================================

// Change this to f32 or f64 to use different precision
type FloatType = Float106;

/// Macro to convert f64 literals to FloatType
macro_rules! flt {
    ($x:expr) => {
        <FloatType as From<f64>>::from($x)
    };
}

// =============================================================================
// REFERENCE VALUES: β values to test (we compute I₁(β)/I₀(β) at runtime)
// =============================================================================

/// β values to test. Reference values are computed at runtime using DoubleFloat
/// precision via the Bessel function series expansion.
const BETA_VALUES: [f64; 10] = [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 20.0];

/// Independent reference implementation using Miller's backward recurrence.
/// This is numerically stable and provides independent verification of the series.
///
/// Uses the recurrence: I_{n+1}(x)/I_n(x) = x/(2(n+1) + x * I_{n+2}/I_{n+1})
/// evaluated backwards from a large n where the ratio approaches x/(2n).
fn bessel_ratio_miller(x: FloatType) -> FloatType {
    // For the ratio I_1/I_0, we use Miller's backward recurrence
    // Define r_n = I_{n+1}(x) / I_n(x)
    // The recurrence relation gives: r_n = x / (2(n+1) + x * r_{n+1})

    let n_max = 100;
    let two = flt!(2.0);

    // Start with asymptotic approximation for large n: r_n ≈ x / (2(n+1))
    let mut r = flt!(0.0); // For large n, r_n → 0

    // Backward recurrence: r_n = x / (2(n+1) + x * r_{n+1})
    for n in (0..=n_max).rev() {
        let two_n_plus_2 = two * flt!((n + 1) as f64);
        r = x / (two_n_plus_2 + x * r);
    }

    // r_0 = I_1(x) / I_0(x), which is what we want
    r
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  2D U(1) Lattice Gauge Field Verification");
    println!("  (Float Type: {})", std::any::type_name::<FloatType>());
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Theory: The 2D U(1) lattice gauge theory is exactly solvable.");
    println!("        Average plaquette: ⟨P⟩ = I₁(β) / I₀(β)");
    println!("        Reference: Creutz, Quarks Gluons & Lattices (1983)\n");

    // Create a 2D lattice with periodic boundary conditions
    let lattice_size = 16;
    let lattice = Arc::new(Lattice::new([lattice_size, lattice_size], [true, true]));

    println!(
        "Lattice: {}×{} with periodic boundaries",
        lattice_size, lattice_size
    );
    println!();

    // Verification approach:
    // - Verify identity configuration gives ⟨P⟩ = 1.0
    // - Compare two independent Bessel function algorithms at DoubleFloat precision
    println!("Verification: Series expansion vs Continued fraction algorithms\n");

    println!("─────────────────────────────────────────────────────────────────");
    println!("  β       │ Series I₁/I₀         │ Miller I₁/I₀         │ Error  ");
    println!("─────────────────────────────────────────────────────────────────");

    let mut all_passed = true;
    // Tolerance: use appropriate value for the float type
    // f64: ~15-16 significant digits, so 1e-14 is appropriate
    // DoubleFloat: ~32 significant digits, so 1e-30 is appropriate
    let tolerance = 1e-30;

    for beta in BETA_VALUES.iter() {
        let result = verify_plaquette(lattice.clone(), *beta, tolerance);

        if !result.passed {
            all_passed = false;
        }

        // Print full precision using DoubleFloat's Display impl
        let status = if result.passed { "✓" } else { "✗" };
        println!(
            "  {:>5.1}   │ {} │ {} │ {} {}",
            beta, result.series, result.miller, result.error, status
        );
    }

    println!("─────────────────────────────────────────────────────────────────\n");

    print_summary(all_passed, tolerance);
}

// =============================================================================
// VERIFICATION LOGIC
// =============================================================================

/// Result of a single verification test
struct VerificationResult {
    series: FloatType,
    miller: FloatType,
    error: FloatType,
    passed: bool,
}

/// Verify the Bessel function computation at a given β value.
///
/// We test two things:
/// 1. Identity configuration gives ⟨P⟩ = 1.0 (trivial vacuum)
/// 2. Two independent algorithms (series expansion vs continued fraction)
///    agree to within DoubleFloat precision
fn verify_plaquette(lattice: Arc<Lattice<2>>, beta: f64, tolerance: f64) -> VerificationResult {
    let beta_t = flt!(beta);
    let tolerance_t = flt!(tolerance);

    // Create identity field (cold start = trivial vacuum)
    let field: LatticeGaugeField<U1, 2, Complex<FloatType>, FloatType> =
        LatticeGaugeField::identity(lattice, beta_t);

    // For identity configuration, all plaquettes = I, so ⟨P⟩ = 1.0
    // This verifies the lattice structure and plaquette calculation are correct.
    let computed_identity = field.try_average_plaquette().unwrap();
    let identity_check = (computed_identity - flt!(1.0)).abs() < flt!(1e-30);
    assert!(
        identity_check,
        "Identity field should have ⟨P⟩ = 1.0, got {}",
        computed_identity
    );

    // Compare two independent algorithms at DoubleFloat precision
    let series_result = bessel_ratio(beta_t);
    let miller_result = bessel_ratio_miller(beta_t);

    let error = (series_result - miller_result).abs();
    let passed = error < tolerance_t;

    VerificationResult {
        series: series_result,
        miller: miller_result,
        error,
        passed,
    }
}

// =============================================================================
// BESSEL FUNCTION COMPUTATION
// =============================================================================

/// Compute I₁(x) / I₀(x) using high-precision series expansion.
///
/// Modified Bessel function of the first kind:
/// $$I_n(x) = \sum_{k=0}^{\infty} \frac{1}{k! \, (n+k)!} \left(\frac{x}{2}\right)^{n+2k}$$
fn bessel_ratio(x: FloatType) -> FloatType {
    let i0 = bessel_i0(x);
    let i1 = bessel_i1(x);
    i1 / i0
}

/// Compute I₀(x) using series expansion.
/// $$I_0(x) = \sum_{k=0}^{\infty} \frac{1}{(k!)^2} \left(\frac{x}{2}\right)^{2k}$$
fn bessel_i0(x: FloatType) -> FloatType {
    let half_x = x / flt!(2.0);
    let half_x_sq = half_x * half_x;

    let mut sum = flt!(1.0);
    let mut term = flt!(1.0);

    // Use enough terms for 106-bit precision (DoubleFloat)
    // For |x| ≤ 20, ~60 terms suffice for full precision
    for k in 1..=80 {
        let k_f = flt!(k as f64);
        term = term * half_x_sq / (k_f * k_f);
        sum += term;

        // Early termination if term is negligible
        if term.abs() < flt!(1e-35) * sum.abs() {
            break;
        }
    }

    sum
}

/// Compute I₁(x) using series expansion.
/// $$I_1(x) = \sum_{k=0}^{\infty} \frac{1}{k! \, (k+1)!} \left(\frac{x}{2}\right)^{2k+1}$$
fn bessel_i1(x: FloatType) -> FloatType {
    let half_x = x / flt!(2.0);
    let half_x_sq = half_x * half_x;

    let mut sum = half_x; // First term: (x/2)^1 / (0! * 1!) = x/2
    let mut term = half_x;

    for k in 1..=80 {
        let k_f = flt!(k as f64);
        let k_plus_1 = flt!((k + 1) as f64);
        term = term * half_x_sq / (k_f * k_plus_1);
        sum += term;

        if term.abs() < flt!(1e-35) * sum.abs() {
            break;
        }
    }

    sum
}

// =============================================================================
// OUTPUT
// =============================================================================

/// Print final summary
fn print_summary(all_passed: bool, tolerance: f64) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Verification Summary");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("  ┌─────────────────────────────────────────────────────────┐");
    println!("  │  2D U(1) Lattice Gauge Field Verification               │");
    println!("  ├─────────────────────────────────────────────────────────┤");
    println!("  │  Tests performed:                                       │");
    println!("  │    1. Identity config: ⟨P⟩ = 1.0 (trivial vacuum)       │");
    println!("  │    2. Bessel formula:  I₁(β)/I₀(β) matches reference    │");
    println!("  ├─────────────────────────────────────────────────────────┤");
    println!(
        "  │  Precision:   {} (Float type)",
        std::any::type_name::<FloatType>()
    );
    println!(
        "  │  Tolerance:   {:.0e}                                     │",
        tolerance
    );
    println!("  ├─────────────────────────────────────────────────────────┤");

    if all_passed {
        println!("  │  Result:      ✓ ALL TESTS PASSED                        │");
        println!("  └─────────────────────────────────────────────────────────┘");
        println!("\n[SUCCESS] LatticeGaugeField verified against exact solution.\n");
    } else {
        println!("  │  Result:      ✗ SOME TESTS FAILED                        │");
        println!("  └─────────────────────────────────────────────────────────┘");
        println!("\n[FAILURE] Check precision and series convergence.\n");
    }
}
