/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::ops::{Add, Mul};
use deep_causality_algebra::Real;
use deep_causality_calculus::{
    DifferentiableArrow, DifferentiableField, DifferentiateExt, DifferentiateFieldExt, EndoArrow,
    Rk4, Scalar,
};
use deep_causality_num::Float106;

// f(x) = x·sin(x) → f'(x) = sin x + x·cos x, f''(x) = 2·cos x − x·sin x
struct XSinX;
impl DifferentiableArrow for XSinX {
    fn run<S: Scalar>(&self, x: S) -> S {
        x * x.sin()
    }
}

// f(x, y) = x² + y²
struct NormSquared;
impl DifferentiableField<2> for NormSquared {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        p[0] * p[0] + p[1] * p[1]
    }
}

#[test]
fn test_derivative_chain_and_product_rule() {
    let x = 0.7_f64;
    let got = XSinX.derivative(x);
    let want = x.sin() + x * x.cos();
    assert!((got - want).abs() < 1e-12);
}

#[test]
fn test_value_and_derivative_single_pass() {
    let x = 1.2_f64;
    let (v, d) = XSinX.value_and_derivative(x);
    assert!((v - x * x.sin()).abs() < 1e-12);
    assert!((d - (x.sin() + x * x.cos())).abs() < 1e-12);
}

#[test]
fn test_second_derivative_from_the_same_model() {
    let x = 0.9_f64;
    let d2 = XSinX.second_derivative(x);
    let want = 2.0 * x.cos() - x * x.sin();
    assert!((d2 - want).abs() < 1e-10);
}

#[test]
fn test_gradient_and_directional_derivative() {
    let g = NormSquared.gradient(&[3.0_f64, 4.0]);
    assert_eq!(g, [6.0, 8.0]);

    // ∇f(1,1)·(2,0) = 2·2 + 2·0 = 4
    let dd = NormSquared.directional_derivative(&[1.0_f64, 1.0], &[2.0, 0.0]);
    assert!((dd - 4.0).abs() < 1e-12);
}

// --- Precision is a parameter: the SAME model at f32 / f64 / Float106 ---

#[test]
fn test_precision_f32() {
    let x = 0.7_f32;
    let got = XSinX.derivative(x);
    let want = x.sin() + x * x.cos();
    assert!((got - want).abs() < 1e-5);
}

#[test]
fn test_precision_f64() {
    let x = 0.7_f64;
    assert!((XSinX.derivative(x) - (x.sin() + x * x.cos())).abs() < 1e-12);
}

#[test]
fn test_precision_float106() {
    let x = Float106::from(0.7);
    let got = XSinX.derivative(x);
    let want = x.sin() + x * x.cos();
    assert!((got - want).abs() < Float106::from(1e-15));
}

#[test]
fn test_second_derivative_precision_f32() {
    // Nesting (Dual<Dual<f32>>) works at f32 precision.
    let x = 0.5_f32;
    let d2 = XSinX.second_derivative(x);
    let want = 2.0 * x.cos() - x * x.sin();
    assert!((d2 - want).abs() < 1e-4);
}

// ============================================================================
// Worked example (avionics descent): `derivative` differentiated THROUGH the solver.
// ============================================================================
//
// The model is written once over `Scalar`. Integration is the endo-arrow + the EndoArrow
// combinators; the sensitivity of the impact speed to the drag coefficient falls out by
// differentiating the whole pipeline (the tangent functor marches State<Dual<f64>>). No
// `Dual`, `ε`, seeding, or hand-rolled loop appears in the model.

/// Vertical descent state `(altitude h, vertical speed v)` — a 2-component module over `S`.
#[derive(Clone, Copy)]
struct State<S> {
    h: S,
    v: S,
}

impl<T: Add<Output = T>> Add for State<T> {
    type Output = State<T>;
    fn add(self, o: State<T>) -> State<T> {
        State {
            h: self.h + o.h,
            v: self.v + o.v,
        }
    }
}

impl<T, R> Mul<R> for State<T>
where
    T: Mul<R, Output = T>,
    R: Copy,
{
    type Output = State<T>;
    fn mul(self, r: R) -> State<T> {
        State {
            h: self.h * r,
            v: self.v * r,
        }
    }
}

const DT: f64 = 0.05;
const STEPS: usize = 200; // 10 s of fall

// Impact speed after a fixed horizon, as a function of the drag coefficient `cd`.
struct SpeedAfterDescent;
impl DifferentiableArrow for SpeedAfterDescent {
    fn run<S: Scalar>(&self, cd: S) -> S {
        let dt = S::from_f64(DT).unwrap();
        let g = S::from_f64(9.81).unwrap();
        let rho = S::from_f64(1.225).unwrap();
        let area = S::from_f64(12.0).unwrap();
        let mass = S::from_f64(1200.0).unwrap();
        let half = S::from_f64(0.5).unwrap();

        // ḣ = v ;  v̇ = −g + drag/m ,  drag = ½·ρ·cd·A·v²  (decelerates the fall)
        let step = Rk4::new(dt, move |s: &State<S>| {
            let drag = half * rho * cd * area * s.v * s.v;
            State {
                h: s.v,
                v: -g + drag / mass,
            }
        });

        let start = State {
            h: S::from_f64(800.0).unwrap(),
            v: S::from_f64(-3.0).unwrap(),
        };
        step.iterate_n(start, STEPS).v.abs()
    }
}

#[test]
fn test_impact_speed_sensitivity_through_the_solver() {
    let cd0 = 0.9_f64;

    let speed = SpeedAfterDescent.run(cd0);
    assert!(speed > 0.0);

    // Differentiate the WHOLE pipeline: the ε channel is ∂(impact speed)/∂cd.
    let d_speed_d_cd = SpeedAfterDescent.derivative(cd0);

    // Cross-check against a central finite difference of the plain-f64 descent.
    let h = 1e-6;
    let fd = (SpeedAfterDescent.run(cd0 + h) - SpeedAfterDescent.run(cd0 - h)) / (2.0 * h);
    assert!(
        (d_speed_d_cd - fd).abs() / fd.abs() < 1e-4,
        "through-solver AD {d_speed_d_cd} vs finite difference {fd}"
    );

    // More drag decelerates the fall → lower speed after a fixed time → ∂ < 0.
    assert!(d_speed_d_cd < 0.0);
}

#[test]
fn test_descent_reaches_ground_via_iterate_until() {
    // The event mode: march until touchdown (altitude ≤ 0), plain f64.
    let (cd, g, rho, area, mass, half) = (0.9, 9.81, 1.225, 12.0, 1200.0, 0.5);
    let step = Rk4::new(0.05_f64, move |s: &State<f64>| {
        let drag = half * rho * cd * area * s.v * s.v;
        State {
            h: s.v,
            v: -g + drag / mass,
        }
    });
    let start = State { h: 800.0, v: -3.0 };
    let (touchdown, met) = step.iterate_until(start, |s| s.h <= 0.0, 100_000);
    assert!(
        met,
        "descent did not reach the ground within the step bound"
    );
    assert!(touchdown.h <= 0.0);
    assert!(touchdown.v < 0.0); // still moving downward at impact
}
