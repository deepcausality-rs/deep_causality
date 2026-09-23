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

// ============================================================================
// Field Hessian: `hessian(&x)[i][j] = ∂²f / ∂xᵢ∂xⱼ`
// ============================================================================
//
// Every expected entry is the hand-derived closed-form second derivative of the fixture,
// written as a literal (or, for the transcendental fixture, as the analytic expression).
// The fixtures carry distinct, non-zero entries so that a dropped mirror write, a skipped
// diagonal, a wrong `ε` channel, or a wrong seeded coordinate changes the answer. Swapping the
// inner and outer seeds cannot: a C² field has a symmetric Hessian, and each value is written to
// both `(i, j)` and `(j, i)`.

// f(x, y, z) = x²y + 3yz³ + xz
//   H = [[2y, 2x, 1], [2x, 0, 9z²], [1, 9z², 18yz]]
struct Cubic3;
impl DifferentiableField<3> for Cubic3 {
    fn run<S: Scalar>(&self, p: &[S; 3]) -> S {
        let three = S::from_f64(3.0).unwrap();
        p[0] * p[0] * p[1] + three * p[1] * p[2] * p[2] * p[2] + p[0] * p[2]
    }
}

// f(x) = Σ_{i ≤ j} a_ij xᵢ xⱼ with a_ij = 1 + i + 4j (i ≤ j), so
//   H[i][i] = 2·a_ii,  H[i][j] = H[j][i] = a_ij (i < j).
struct UpperQuadratic4;
impl DifferentiableField<4> for UpperQuadratic4 {
    fn run<S: Scalar>(&self, p: &[S; 4]) -> S {
        let mut acc = S::zero();
        for i in 0..4 {
            for j in i..4 {
                let a = S::from_usize(1 + i + 4 * j).unwrap();
                acc += a * p[i] * p[j];
            }
        }
        acc
    }
}

// Rosenbrock f(x, y) = (1 − x)² + 100(y − x²)²
//   H = [[2 − 400(y − x²) + 800x², −400x], [−400x, 200]]
struct Rosenbrock;
impl DifferentiableField<2> for Rosenbrock {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let one = S::one();
        let hundred = S::from_f64(100.0).unwrap();
        let a = one - p[0];
        let b = p[1] - p[0] * p[0];
        a * a + hundred * b * b
    }
}

// f(x, y) = sin(x·y) + x·exp(y)
//   H = [[−y² sin(xy),              cos(xy) − xy sin(xy) + exp(y)],
//        [cos(xy) − xy sin(xy) + exp(y), −x² sin(xy) + x exp(y)]]
struct SinExp;
impl DifferentiableField<2> for SinExp {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        (p[0] * p[1]).sin() + p[0] * p[1].exp()
    }
}

// f(x) = x³ → f''(x) = 6x
struct Cube1;
impl DifferentiableField<1> for Cube1 {
    fn run<S: Scalar>(&self, p: &[S; 1]) -> S {
        p[0] * p[0] * p[0]
    }
}

// The zero-input field: a constant.
struct Constant0;
impl DifferentiableField<0> for Constant0 {
    fn run<S: Scalar>(&self, _p: &[S; 0]) -> S {
        S::from_f64(7.0).unwrap()
    }
}

// f(x, y) = x²y — used for NaN propagation.
struct XSquaredY;
impl DifferentiableField<2> for XSquaredY {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        p[0] * p[0] * p[1]
    }
}

#[test]
fn test_hessian_cubic_distinct_entries() {
    // (x, y, z) = (1, 2, 3): H = [[4, 2, 1], [2, 0, 81], [1, 81, 108]] (closed form above).
    let h = Cubic3.hessian(&[1.0_f64, 2.0, 3.0]);
    assert_eq!(h, [[4.0, 2.0, 1.0], [2.0, 0.0, 81.0], [1.0, 81.0, 108.0]]);
}

#[test]
fn test_hessian_cubic_negative_point() {
    // (x, y, z) = (−1, 2, −3): H = [[4, −2, 1], [−2, 0, 81], [1, 81, −108]] (closed form above).
    let h = Cubic3.hessian(&[-1.0_f64, 2.0, -3.0]);
    assert_eq!(
        h,
        [[4.0, -2.0, 1.0], [-2.0, 0.0, 81.0], [1.0, 81.0, -108.0]]
    );
}

#[test]
fn test_hessian_quadratic_n4_every_position() {
    // a_ij = 1 + i + 4j:
    //   diagonal 2·a_ii = 2, 12, 22, 32;
    //   a_01 = 5, a_02 = 9, a_03 = 13, a_12 = 10, a_13 = 14, a_23 = 15.
    // A quadratic has a constant Hessian, so the point only has to be generic.
    let h = UpperQuadratic4.hessian(&[0.3_f64, -1.7, 2.9, 0.4]);
    let want = [
        [2.0, 5.0, 9.0, 13.0],
        [5.0, 12.0, 10.0, 14.0],
        [9.0, 10.0, 22.0, 15.0],
        [13.0, 14.0, 15.0, 32.0],
    ];
    for i in 0..4 {
        for j in 0..4 {
            assert!(
                (h[i][j] - want[i][j]).abs() < 1e-12,
                "H[{i}][{j}] = {} want {}",
                h[i][j],
                want[i][j]
            );
        }
    }
}

#[test]
fn test_hessian_rosenbrock_at_minimum() {
    // At the minimum (1, 1): H = [[802, −400], [−400, 200]] (closed form above).
    let h = Rosenbrock.hessian(&[1.0_f64, 1.0]);
    assert_eq!(h, [[802.0, -400.0], [-400.0, 200.0]]);
}

#[test]
fn test_hessian_rosenbrock_off_minimum() {
    // At (−1.2, 1): y − x² = −0.44, so H_xx = 2 + 176 + 1152 = 1330, H_xy = 480, H_yy = 200.
    let h = Rosenbrock.hessian(&[-1.2_f64, 1.0]);
    let want = [[1330.0, 480.0], [480.0, 200.0]];
    for i in 0..2 {
        for j in 0..2 {
            assert!(
                (h[i][j] - want[i][j]).abs() < 1e-9,
                "H[{i}][{j}] = {}",
                h[i][j]
            );
        }
    }
}

#[test]
fn test_hessian_transcendental_closed_form() {
    let (x, y) = (0.8_f64, -0.6_f64);
    let h = SinExp.hessian(&[x, y]);
    let (s, c, e) = ((x * y).sin(), (x * y).cos(), y.exp());
    let hxx = -y * y * s;
    let hxy = c - x * y * s + e;
    let hyy = -x * x * s + x * e;
    assert!((h[0][0] - hxx).abs() < 1e-14);
    assert!((h[0][1] - hxy).abs() < 1e-14);
    assert!((h[1][0] - hxy).abs() < 1e-14);
    assert!((h[1][1] - hyy).abs() < 1e-14);
}

#[test]
fn test_hessian_is_exactly_symmetric() {
    // Schwarz: H[i][j] = H[j][i] for a C² field; the returned matrix holds it bit for bit.
    let h = SinExp.hessian(&[1.3_f64, 0.7]);
    assert_eq!(h[0][1].to_bits(), h[1][0].to_bits());
    let h = UpperQuadratic4.hessian(&[0.1_f64, 0.2, 0.3, 0.4]);
    (0..4)
        .flat_map(|i| (0..4).map(move |j| (i, j)))
        .for_each(|(i, j)| assert_eq!(h[i][j].to_bits(), h[j][i].to_bits()));
}

#[test]
fn test_hessian_agrees_with_finite_difference_of_gradient() {
    // Independent algorithm: central differences of the first-order gradient.
    let p = [0.8_f64, -0.6];
    let h = SinExp.hessian(&p);
    let step = 1e-5;
    for j in 0..2 {
        let mut plus = p;
        let mut minus = p;
        plus[j] += step;
        minus[j] -= step;
        let gp = SinExp.gradient(&plus);
        let gm = SinExp.gradient(&minus);
        for i in 0..2 {
            let fd = (gp[i] - gm[i]) / (2.0 * step);
            assert!(
                (h[i][j] - fd).abs() < 1e-7,
                "H[{i}][{j}] = {} fd {fd}",
                h[i][j]
            );
        }
    }
}

#[test]
fn test_hessian_single_input() {
    // f(x) = x³ at x = 2: f'' = 12.
    assert_eq!(Cube1.hessian(&[2.0_f64]), [[12.0]]);
    // At x = −0.5: f'' = −3.
    assert_eq!(Cube1.hessian(&[-0.5_f64]), [[-3.0]]);
}

#[test]
fn test_hessian_zero_inputs() {
    // A `[[f64; 0]; 0]` has no entry to assert on; the test pins that `N = 0` returns without
    // panicking.
    let _h: [[f64; 0]; 0] = Constant0.hessian(&[]);
}

#[test]
fn test_hessian_nan_input_propagates() {
    // A NaN coordinate reaches every entry of x²y's Hessian; nothing is substituted.
    let h = XSquaredY.hessian(&[f64::NAN, 1.0]);
    for row in h {
        for v in row {
            assert!(v.is_nan());
        }
    }
    // A finite point gives [[2y, 2x], [2x, 0]] = [[2, 6], [6, 0]] at (3, 1).
    assert_eq!(XSquaredY.hessian(&[3.0_f64, 1.0]), [[2.0, 6.0], [6.0, 0.0]]);
}

// --- Precision is a parameter: the same field and point at f32 / f64 / Float106 ---
// At (1, 2, 3) every entry of Cubic3's Hessian is a small integer, exact at all three. The `f64`
// case is `test_hessian_cubic_distinct_entries`.

#[test]
fn test_hessian_precision_f32() {
    let h = Cubic3.hessian(&[1.0_f32, 2.0, 3.0]);
    assert_eq!(h, [[4.0, 2.0, 1.0], [2.0, 0.0, 81.0], [1.0, 81.0, 108.0]]);
}

#[test]
fn test_hessian_precision_float106() {
    let f = Float106::from;
    let h = Cubic3.hessian(&[f(1.0), f(2.0), f(3.0)]);
    let want = [
        [f(4.0), f(2.0), f(1.0)],
        [f(2.0), f(0.0), f(81.0)],
        [f(1.0), f(81.0), f(108.0)],
    ];
    assert_eq!(h, want);
}

#[test]
fn test_hessian_precision_float106_transcendental() {
    // Float106 resolves the transcendental Hessian far below f64's ε.
    let f = Float106::from;
    let (x, y) = (f(0.8), f(-0.6));
    let h = SinExp.hessian(&[x, y]);
    let (s, c, e) = ((x * y).sin(), (x * y).cos(), y.exp());
    let hxy = c - x * y * s + e;
    assert!((h[0][1] - hxy).abs() < f(1e-28));
    assert!((h[1][1] - (-x * x * s + x * e)).abs() < f(1e-28));
}
