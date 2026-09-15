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
use deep_causality_num_dual::Dual;

fn main() {
    println!("=== Differentiate Under the Integral (the Leibniz bridge) ===\n");

    let theta_val = 1.3_f64;

    // Seed θ as the differentiation variable, then run a single quadrature sweep over `Dual`.
    let theta = Dual::variable(theta_val);
    let sweep = quadrature(
        |x: Dual<f64>| (theta * x).sin(),
        Dual::constant(0.0),
        Dual::constant(1.0),
        128,
    );

    // One sweep carries both answers. The value is I(θ); the ε channel is dI/dθ.
    let integral = sweep.value();
    let sensitivity = sweep.derivative();

    // Closed forms: I(θ) = (1 − cos θ)/θ, and dI/dθ = [θ·sin θ − (1 − cos θ)] / θ².
    let analytic_i = (1.0 - theta_val.cos()) / theta_val;
    let analytic_di =
        (theta_val * theta_val.sin() - (1.0 - theta_val.cos())) / (theta_val * theta_val);

    println!("I(θ) = ∫₀¹ sin(θ·x) dx   at θ = {theta_val}\n");
    println!("  integral (real part):   {integral:.10}");
    println!("  analytic (1−cosθ)/θ:    {analytic_i:.10}");
    println!(
        "  abs error:              {:.2e}\n",
        (integral - analytic_i).abs()
    );
    println!("dI/dθ, read from the same sweep with no second pass:");
    println!("  sensitivity (ε part):   {sensitivity:.10}");
    println!("  analytic:               {analytic_di:.10}");
    println!(
        "  abs error:              {:.2e}\n",
        (sensitivity - analytic_di).abs()
    );

    println!("One quadrature over Dual returns both the integral and dI/dθ.");
}
