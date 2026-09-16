/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `deep_causality_calculus`: the API surface
//!
//! Differentiation and integration as values....
//!
//! ```text
//! DifferentiableArrow      run<S: Scalar>(x: S) -> S     a model, written once
//!   .derivative(x)                                       df/dx by forward-mode AD
//!   .value_and_derivative(x)                             both, in one evaluation
//!
//! DifferentiableField<N>   run<S: Scalar>(&[S; N]) -> S  a scalar field
//!   .gradient(&x)                                        ∇f, one seeded coordinate per pass
//!
//! quadrature(f, a, b, n)                                 Simpson's rule, generic over Scalar
//! Euler::new(dt, rate) / Rk4::new(dt, rate)              time integrators as endo-arrows
//! ```
//!
//! The single idea underneath: a model is written **once**, generic over `Scalar`. Evaluated
//! at the working type it is the model; evaluated at `Dual` it is the model together with its
//! derivative.

use deep_causality_algebra::Real;
use deep_causality_calculus::{
    DifferentiableArrow, DifferentiableField, DifferentiateExt, DifferentiateFieldExt, Euler, Rk4,
    Scalar, quadrature,
};
use deep_causality_haft::Arrow;
use deep_causality_num::{Float106, const_scalar_from_float, const_scalar_from_int, lift, lower};
use std::ops::{Add, Mul};

/// Panels for the quadrature.
const PANELS: usize = 256;
/// Steps taken by each integrator, so the two can be compared at equal cost in steps.
const STEPS: usize = 20;

/// The working scalar. Every quantity below carries it.
pub type FloatType = Float106;

/// Small numbers and tolerances, at the working type.
const TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE_TENTH: FloatType = const_scalar_from_float!(FloatType, 0.1);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const THREE: FloatType = const_scalar_from_int!(FloatType, 3);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. A one-input model: f(x) = x^3 - 2x, so f'(x) = 3x^2 - 2.
    // ---------------------------------------------------------------------
    let at = TWO;
    let (value, derivative) = Cubic.value_and_derivative(at);
    let exact = THREE * at * at - TWO;
    print_arrow(lower(at), value, derivative, exact);
    assert_eq!(derivative, exact); // exact on a polynomial: nothing was differenced

    // ---------------------------------------------------------------------
    // 2. A two-input field: g(x, y) = x^2 y + y^3, so ∇g = (2xy, x^2 + 3y^2).
    // ---------------------------------------------------------------------
    let point = [THREE, TWO];
    let grad = Saddle.gradient(&point);
    let exact_grad = [
        TWO * point[0] * point[1],
        point[0] * point[0] + THREE * point[1] * point[1],
    ];
    print_field(&point, &grad, &exact_grad);
    assert_eq!(grad, exact_grad);

    // ---------------------------------------------------------------------
    // 3. Integration: ∫₀¹ (x^3 - 2x) dx = 1/4 - 1 = -3/4.
    // ---------------------------------------------------------------------
    let integral = quadrature(|x: FloatType| Cubic.run(x), ZERO, ONE, PANELS);
    let exact_integral = lift::<FloatType>(-0.75);
    print_quadrature(integral, exact_integral);
    assert!(Real::abs(integral - exact_integral) < TOLERANCE);

    // ---------------------------------------------------------------------
    // 4. Time integration: dy/dt = -y from y(0) = 1, whose solution is e^-t.
    // ---------------------------------------------------------------------
    // `Euler` is first order and `Rk4` fourth. Swapping one for the other changes nothing
    // about the rate field, which is the point of keeping them separate values.
    let dt = ONE_TENTH;
    let decay = |y: &State| State(-y.0);

    let mut euler_state = State(ONE);
    let euler = Euler::new(dt, decay);
    let mut rk4_state = State(ONE);
    let rk4 = Rk4::new(dt, decay);

    for _ in 0..STEPS {
        euler_state = euler.run(euler_state);
        rk4_state = rk4.run(rk4_state);
    }

    let t = dt * lift::<FloatType>(STEPS as f64);
    let exact_decay = Real::exp(-t);
    print_integrators(lower(t), euler_state.0, rk4_state.0, exact_decay);
    // Fourth order beats first order by orders of magnitude at the same step count.
    let euler_err = Real::abs(euler_state.0 - exact_decay);
    let rk4_err = Real::abs(rk4_state.0 - exact_decay);
    assert!(rk4_err < euler_err);

    print_footer();
    Ok(())
}

/// `f(x) = x^3 - 2x`, written once and generic over `Scalar`.
struct Cubic;

impl DifferentiableArrow for Cubic {
    fn run<S: Scalar>(&self, x: S) -> S {
        x * x * x - S::from_f64(2.0).expect("scalar from literal") * x
    }
}

/// `g(x, y) = x^2 y + y^3`, a saddle-shaped scalar field.
struct Saddle;

impl DifferentiableField<2> for Saddle {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        p[0] * p[0] * p[1] + p[1] * p[1] * p[1]
    }
}

/// The integrator state. `Euler` and `Rk4` need a module-valued state, which a bare scalar
/// does not provide, so the value rides in this newtype.
#[derive(Clone, Copy)]
struct State(FloatType);

impl Add for State {
    type Output = State;
    fn add(self, rhs: State) -> State {
        State(self.0 + rhs.0)
    }
}

impl Mul<FloatType> for State {
    type Output = State;
    fn mul(self, s: FloatType) -> State {
        State(self.0 * s)
    }
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== deep_causality_calculus: differentiation and integration ===\n");
    println!("  Precision: {}", core::any::type_name::<FloatType>());
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_arrow(at: f64, value: FloatType, derivative: FloatType, exact: FloatType) {
    println!("\n--- 1. DifferentiableArrow: f(x) = x^3 - 2x ---");
    println!("  at x = {at}");
    println!("  f(x)           = {}", lower(value));
    println!("  f'(x) by AD    = {}", lower(derivative));
    println!("  f'(x) exact    = {}   (3x^2 - 2)", lower(exact));
    println!("  One evaluation returned both; no step size was chosen.");
}

fn print_field(point: &[FloatType; 2], grad: &[FloatType; 2], exact: &[FloatType; 2]) {
    println!("\n--- 2. DifferentiableField<2>: g(x, y) = x^2 y + y^3 ---");
    println!("  at ({}, {})", lower(point[0]), lower(point[1]));
    println!("  grad by AD  = ({}, {})", lower(grad[0]), lower(grad[1]));
    println!(
        "  grad exact  = ({}, {})   (2xy, x^2 + 3y^2)",
        lower(exact[0]),
        lower(exact[1])
    );
}

fn print_quadrature(integral: FloatType, exact: FloatType) {
    println!("\n--- 3. quadrature: the integral of the same model ---");
    println!("  Simpson, {PANELS} panels over [0, 1]");
    println!("  result = {:.12}", lower(integral));
    println!("  exact  = {:.12}   (1/4 - 1)", lower(exact));
}

fn print_integrators(t: f64, euler: FloatType, rk4: FloatType, exact: FloatType) {
    println!("\n--- 4. Euler and Rk4 on dy/dt = -y, y(0) = 1 ---");
    println!("  after {STEPS} steps, t = {t}");
    println!("  {:<10} {:>18} {:>14}", "method", "y(t)", "abs error");
    println!(
        "  {:<10} {:>18.12} {:>14.3e}",
        "Euler",
        lower(euler),
        lower(Real::abs(euler - exact))
    );
    println!(
        "  {:<10} {:>18.12} {:>14.3e}",
        "Rk4",
        lower(rk4),
        lower(Real::abs(rk4 - exact))
    );
    println!("  {:<10} {:>18.12}", "exact", lower(exact));
}

fn print_footer() {
    println!("\n--- Why one model ---");
    println!("  `Cubic` and `Saddle` are written once, generic over `Scalar`. Evaluated at the");
    println!("  working type they are the model; evaluated at `Dual` they are the model and its");
    println!("  derivative. There is no separate hand-written derivative to keep in step, and");
    println!("  no finite difference, so no step size and no truncation error.");
}
