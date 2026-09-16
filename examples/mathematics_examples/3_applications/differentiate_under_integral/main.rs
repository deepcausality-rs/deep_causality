/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Differentiate Under the Integral: the Leibniz bridge
//!
//! The two `arrow-calculus` primitives compose. Evaluate `quadrature` over `Dual` and one sweep
//! returns a definite integral together with its sensitivity to a parameter. This is the
//! naturality of the tangent functor through the quadrature fold, `T(∫f) = ∫(Tf)`; it is
//! differentiation under the integral sign, obtained for free.
//!
//! Take `I(θ) = ∫₀¹ sin(θ·x) dx`. Seed `θ` as a `Dual::variable` and integrate over `Dual`. The
//! real part of the result is the integral `I(θ)`; the infinitesimal (`ε`) part is `dI/dθ`.

use deep_causality_algebra::Real;
use deep_causality_calculus::quadrature;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lower};
use deep_causality_num_dual::Dual;

/// The parameter the integral is differentiated with respect to.
const THETA: FloatType = const_scalar_from_float!(FloatType, 1.3);
/// Panels for the Simpson sweep.
const PANELS: usize = 128;

/// The working scalar. The integral, its sensitivity and both closed forms carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

fn main() {
    let theta = THETA;

    // Seed θ as the differentiation variable, then run a single quadrature sweep over `Dual`.
    let seeded = Dual::variable(theta);
    let sweep = quadrature(
        |x: Dual<FloatType>| (seeded * x).sin(),
        Dual::constant(ZERO),
        Dual::constant(ONE),
        PANELS,
    );

    // One sweep carries both answers. The value is I(θ); the ε channel is dI/dθ.
    let integral = sweep.value();
    let sensitivity = sweep.derivative();

    print_header(theta);
    print_integral(integral, analytic_integral(theta));
    print_sensitivity(sensitivity, analytic_sensitivity(theta));
    print_footer();
}

/// `I(θ) = (1 − cos θ) / θ`.
fn analytic_integral(theta: FloatType) -> FloatType {
    (ONE - Real::cos(theta)) / theta
}

/// `dI/dθ = [θ·sin θ − (1 − cos θ)] / θ²`.
fn analytic_sensitivity(theta: FloatType) -> FloatType {
    (theta * Real::sin(theta) - (ONE - Real::cos(theta))) / (theta * theta)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn print_header(theta: FloatType) {
    println!("=== Differentiate Under the Integral (the Leibniz bridge) ===\n");
    println!("I(θ) = ∫₀¹ sin(θ·x) dx   at θ = {}", lower(theta));
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_integral(integral: FloatType, analytic: FloatType) {
    println!("  integral (real part):   {:.10}", lower(integral));
    println!("  analytic (1−cosθ)/θ:    {:.10}", lower(analytic));
    println!(
        "  abs error:              {:.2e}\n",
        lower(Real::abs(integral - analytic))
    );
}

fn print_sensitivity(sensitivity: FloatType, analytic: FloatType) {
    println!("dI/dθ, read from the same sweep with no second pass:");
    println!("  sensitivity (ε part):   {:.10}", lower(sensitivity));
    println!("  analytic:               {:.10}", lower(analytic));
    println!(
        "  abs error:              {:.2e}\n",
        lower(Real::abs(sensitivity - analytic))
    );
}

fn print_footer() {
    println!("One quadrature over Dual returns both the integral and dI/dθ.");
}
