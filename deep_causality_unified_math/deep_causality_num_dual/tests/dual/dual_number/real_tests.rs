/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::Real;
use deep_causality_num_dual::Dual;

const TOL: f64 = 1e-9;

#[test]
fn test_exp_derivative() {
    let y = Dual::variable(1.5_f64).exp();
    assert!((y.value() - 1.5_f64.exp()).abs() < TOL);
    assert!((y.derivative() - 1.5_f64.exp()).abs() < TOL); // d/dx eˣ = eˣ
}

#[test]
fn test_ln_derivative() {
    let y = Dual::variable(2.0_f64).ln();
    assert!((y.value() - 2.0_f64.ln()).abs() < TOL);
    assert!((y.derivative() - 0.5).abs() < TOL); // 1/2
}

#[test]
fn test_sqrt_derivative() {
    let y = Dual::variable(4.0_f64).sqrt();
    assert!((y.value() - 2.0).abs() < TOL);
    assert!((y.derivative() - 0.25).abs() < TOL); // 1/(2·√4)
}

#[test]
fn test_sin_and_cos_derivatives() {
    let s = Dual::variable(0.5_f64).sin();
    assert!((s.derivative() - 0.5_f64.cos()).abs() < TOL);
    let c = Dual::variable(0.5_f64).cos();
    assert!((c.derivative() - (-0.5_f64.sin())).abs() < TOL);
}

#[test]
fn test_tan_derivative() {
    let y = Dual::variable(0.3_f64).tan();
    let sec2 = 1.0 / (0.3_f64.cos() * 0.3_f64.cos());
    assert!((y.derivative() - sec2).abs() < TOL);
}

#[test]
fn test_inverse_trig_and_hyperbolic_derivatives() {
    let a = 0.4_f64;
    let d = (1.0 - a * a).sqrt();
    assert!((Dual::variable(a).asin().derivative() - 1.0 / d).abs() < TOL);
    assert!((Dual::variable(a).acos().derivative() - (-1.0 / d)).abs() < TOL);
    assert!((Dual::variable(a).atan().derivative() - 1.0 / (1.0 + a * a)).abs() < TOL);
    assert!((Dual::variable(a).sinh().derivative() - a.cosh()).abs() < TOL);
    assert!((Dual::variable(a).cosh().derivative() - a.sinh()).abs() < TOL);
    assert!((Dual::variable(a).tanh().derivative() - (1.0 - a.tanh() * a.tanh())).abs() < TOL);
}

#[test]
fn test_log_base_derivative() {
    // d/dx log₂(x) = 1/(x·ln2); d/dx log₁₀(x) = 1/(x·ln10)
    let x = 8.0_f64;
    assert!((Dual::variable(x).log2().derivative() - 1.0 / (x * 2.0_f64.ln())).abs() < TOL);
    assert!((Dual::variable(x).log10().derivative() - 1.0 / (x * 10.0_f64.ln())).abs() < TOL);
    // log with an arbitrary (constant) base
    let y = Dual::variable(x).log(Dual::constant(2.0));
    assert!((y.value() - x.log(2.0)).abs() < TOL);
    assert!((y.derivative() - 1.0 / (x * 2.0_f64.ln())).abs() < TOL);
}

#[test]
fn test_chain_rule_through_sin_times_exp() {
    // f(x) = sin(x)·exp(x); f'(x) = cos(x)·exp(x) + sin(x)·exp(x)
    let x0 = 0.7_f64;
    let y = Dual::variable(x0).sin() * Dual::variable(x0).exp();
    let expected = x0.cos() * x0.exp() + x0.sin() * x0.exp();
    assert!((y.derivative() - expected).abs() < TOL);
}

#[test]
fn test_powf_derivative() {
    // f(x) = x^2.5 at x = 3 → f'(x) = 2.5·x^1.5
    let y = Dual::variable(3.0_f64).powf(Dual::constant(2.5));
    assert!((y.value() - 3.0_f64.powf(2.5)).abs() < 1e-6);
    assert!((y.derivative() - 2.5 * 3.0_f64.powf(1.5)).abs() < 1e-6);
}

#[test]
fn test_abs_derivative_is_sign() {
    assert_eq!(Dual::variable(2.0_f64).abs().derivative(), 1.0);
    let n = Dual::variable(-2.0_f64).abs();
    assert_eq!(n.value(), 2.0);
    assert_eq!(n.derivative(), -1.0);
}

#[test]
fn test_nonsmooth_ops_have_zero_derivative() {
    assert_eq!(Dual::variable(2.7_f64).floor().value(), 2.0);
    assert_eq!(Dual::variable(2.7_f64).floor().derivative(), 0.0);
    assert_eq!(Dual::variable(2.1_f64).ceil().value(), 3.0);
    assert_eq!(Dual::variable(2.1_f64).ceil().derivative(), 0.0);
    assert_eq!(Dual::variable(2.5_f64).round().derivative(), 0.0);
}

#[test]
fn test_constants_and_predicates() {
    assert_eq!(<Dual<f64> as Real>::pi().value(), core::f64::consts::PI);
    assert_eq!(<Dual<f64> as Real>::pi().derivative(), 0.0);
    assert_eq!(<Dual<f64> as Real>::e().value(), core::f64::consts::E);
    assert!(<Dual<f64> as Real>::nan().is_nan());
    assert!(Dual::variable(1.0_f64).is_finite());
    assert!(Dual::new(f64::INFINITY, 0.0).is_infinite());
    assert!(!Dual::new(f64::INFINITY, 0.0).is_finite());
}

#[test]
fn test_clamp() {
    let lo = Dual::constant(0.0_f64);
    let hi = Dual::constant(1.0_f64);
    assert_eq!(Dual::variable(0.5_f64).clamp(lo, hi).value(), 0.5);
    assert_eq!(Dual::variable(0.5_f64).clamp(lo, hi).derivative(), 1.0); // passthrough
    assert_eq!(Dual::variable(2.0_f64).clamp(lo, hi).value(), 1.0); // clamped to hi
    assert_eq!(Dual::variable(2.0_f64).clamp(lo, hi).derivative(), 0.0); // bound's derivative
}

#[test]
fn test_clamp_to_lower_bound() {
    let lo = Dual::constant(0.0_f64);
    let hi = Dual::constant(1.0_f64);
    // A value below the lower bound clamps to `min`, carrying the bound's (zero) derivative.
    let r = Dual::variable(-1.0_f64).clamp(lo, hi);
    assert_eq!(r.value(), 0.0);
    assert_eq!(r.derivative(), 0.0);
}

#[test]
fn test_atan2_derivative() {
    // atan2(y, x) with y the variable and x constant: d/dy = x / (x² + y²).
    let y = 1.0_f64;
    let x = 2.0_f64;
    let r = Dual::variable(y).atan2(Dual::constant(x));
    assert!((r.value() - y.atan2(x)).abs() < TOL);
    assert!((r.derivative() - x / (x * x + y * y)).abs() < TOL);
}

#[test]
fn test_epsilon_is_a_constant() {
    assert_eq!(<Dual<f64> as Real>::epsilon().value(), f64::EPSILON);
    assert_eq!(<Dual<f64> as Real>::epsilon().derivative(), 0.0);
}

#[test]
fn test_dual_is_a_real_scalar_and_nests() {
    fn assert_real<T: Real>() {}
    assert_real::<Dual<f64>>();
    assert_real::<Dual<f32>>();
    assert_real::<Dual<Dual<f64>>>(); // duals nest: Dual<Dual<T>> is also Real
}

#[test]
fn test_nested_duals_give_second_derivative() {
    // f(x) = x⁴ at x = 2:  f'(x) = 4x³ = 32,  f''(x) = 12x² = 48
    let x = Dual::variable(Dual::variable(2.0_f64));
    let y = x * x * x * x;
    assert_eq!(y.derivative().value(), 32.0); // first derivative
    assert_eq!(y.derivative().derivative(), 48.0); // second derivative
}

// --- cbrt ---------------------------------------------------------------------------------
//
// d/dx x^(1/3) = 1/(3·x^(2/3)). Every expected derivative below is that closed form evaluated
// by hand at a point where it is a ratio of small integers, written as a literal. The
// central-difference test is an independent numerical check on the same quantity.

#[test]
fn test_cbrt_derivative_at_a_perfect_cube() {
    // x = 8: cbrt = 2, and 1/(3·8^(2/3)) = 1/(3·4) = 1/12.
    let y = Dual::variable(8.0_f64).cbrt();
    assert!((y.value() - 2.0).abs() < TOL);
    assert!((y.derivative() - 1.0 / 12.0).abs() < TOL);

    // x = 27: cbrt = 3, and 1/(3·27^(2/3)) = 1/(3·9) = 1/27.
    let y = Dual::variable(27.0_f64).cbrt();
    assert!((y.value() - 3.0).abs() < TOL);
    assert!((y.derivative() - 1.0 / 27.0).abs() < TOL);

    // x = 1: cbrt = 1, derivative 1/3.
    let y = Dual::variable(1.0_f64).cbrt();
    assert!((y.value() - 1.0).abs() < TOL);
    assert!((y.derivative() - 1.0 / 3.0).abs() < TOL);

    // x = 1/8: cbrt = 1/2, and 1/(3·(1/8)^(2/3)) = 1/(3·(1/4)) = 4/3.
    let y = Dual::variable(0.125_f64).cbrt();
    assert!((y.value() - 0.5).abs() < TOL);
    assert!((y.derivative() - 4.0 / 3.0).abs() < TOL);
}

#[test]
fn test_cbrt_derivative_at_a_negative_argument() {
    // x = -8: cbrt = -2. The derivative is 1/(3·(-8)^(2/3)) = 1/(3·4) = 1/12, positive —
    // cbrt increases through the negative half too.
    let y = Dual::variable(-8.0_f64).cbrt();
    assert!((y.value() + 2.0).abs() < TOL, "value was {}", y.value());
    assert!(
        (y.derivative() - 1.0 / 12.0).abs() < TOL,
        "derivative was {}",
        y.derivative()
    );

    // x = -27: cbrt = -3, derivative 1/27.
    let y = Dual::variable(-27.0_f64).cbrt();
    assert!((y.value() + 3.0).abs() < TOL);
    assert!((y.derivative() - 1.0 / 27.0).abs() < TOL);
    assert!(y.derivative() > 0.0, "cbrt is increasing at x = -27");
}

/// The reason `powf(1/3)` cannot serve as the dual cube root: it is `NaN` on the negative half,
/// so both components are lost.
#[test]
fn test_cbrt_negative_argument_is_not_nan_where_powf_is() {
    let y = Dual::variable(-8.0_f64).cbrt();
    assert!(!y.value().is_nan());
    assert!(!y.derivative().is_nan());

    let viapowf = Dual::variable(-8.0_f64).powf(Dual::constant(1.0 / 3.0));
    assert!(
        viapowf.value().is_nan(),
        "powf(1/3) is real on the negative half"
    );
}

/// An independent check on the derivative: a central difference of the value channel, which
/// shares no code with the ε channel.
#[test]
fn test_cbrt_derivative_matches_a_central_difference() {
    for x in [-27.0_f64, -3.5, -0.5, 0.5, 3.5, 27.0, 1000.0] {
        let h = 1e-6 * x.abs();
        let numeric = (Real::cbrt(x + h) - Real::cbrt(x - h)) / (2.0 * h);
        let dual = Dual::variable(x).cbrt().derivative();
        assert!(
            (dual - numeric).abs() <= 1e-6 * numeric.abs(),
            "at x = {x}: dual {dual}, central difference {numeric}"
        );
    }
}

/// The crate's convention is to let the arithmetic produce the infinity rather than intercept
/// it. `sqrt` already does this at the same point, and is asserted alongside so the two cannot
/// drift apart.
#[test]
fn test_cbrt_at_zero_yields_an_infinite_derivative() {
    let y = Dual::variable(0.0_f64).cbrt();
    assert_eq!(y.value(), 0.0);
    assert!(
        y.derivative().is_infinite(),
        "derivative was {}",
        y.derivative()
    );
    assert!(y.derivative() > 0.0);

    let s = Dual::variable(0.0_f64).sqrt();
    assert!(s.derivative().is_infinite(), "sqrt sets the convention");
}

/// A constant at zero divides 0 by 0, so the ε channel is NaN rather than zero. That is not a
/// special case of cbrt: `sqrt` does the same at the same point, for the same reason, and both
/// are asserted here so the shared convention stays visible.
#[test]
fn test_cbrt_of_a_constant_zero_gives_nan_as_sqrt_does() {
    let y = Dual::constant(0.0_f64).cbrt();
    assert_eq!(y.value(), 0.0);
    assert!(y.derivative().is_nan(), "derivative was {}", y.derivative());

    let s = Dual::constant(0.0_f64).sqrt();
    assert!(s.derivative().is_nan(), "sqrt sets the convention");
}

/// The chain rule composes: d/dx cbrt(u(x)) = u'(x)/(3·cbrt(u)²). At u = x², x = 3:
/// u = 9, cbrt(9) = 9^(1/3), u' = 6, so the derivative is 6/(3·9^(2/3)) = 2/9^(2/3).
#[test]
fn test_cbrt_composes_under_the_chain_rule() {
    let x = Dual::variable(3.0_f64);
    let y = (x * x).cbrt();
    assert!((y.value() - 9.0_f64.cbrt()).abs() < TOL);
    let want = 2.0 / 9.0_f64.powf(2.0 / 3.0);
    assert!(
        (y.derivative() - want).abs() < TOL,
        "derivative was {}",
        y.derivative()
    );
}

/// cbrt is odd on duals as it is on reals, in both components.
#[test]
fn test_cbrt_is_odd_on_duals() {
    for x in [0.5_f64, 2.0, 8.0, 100.0] {
        let pos = Dual::variable(x).cbrt();
        let neg = Dual::variable(-x).cbrt();
        assert!(
            (neg.value() + pos.value()).abs() < TOL,
            "value oddness at {x}"
        );
        // The derivative is even, being a function of x^(2/3).
        assert!(
            (neg.derivative() - pos.derivative()).abs() < TOL,
            "derivative parity at {x}"
        );
    }
}

/// Duals nest, so the second derivative is available. d²/dx² x^(1/3) = -(2/9)·x^(-5/3).
/// At x = 8: -(2/9)·8^(-5/3) = -(2/9)·(1/32) = -1/144.
#[test]
fn test_cbrt_second_derivative_through_nested_duals() {
    let x = Dual::variable(Dual::variable(8.0_f64));
    let y = x.cbrt();
    assert!((y.value().value() - 2.0).abs() < TOL);
    assert!((y.value().derivative() - 1.0 / 12.0).abs() < TOL);
    assert!(
        (y.derivative().derivative() + 1.0 / 144.0).abs() < TOL,
        "second derivative was {}",
        y.derivative().derivative()
    );
}
