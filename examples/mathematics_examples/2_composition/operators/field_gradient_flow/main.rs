/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Operators: the exact gradient of a field, and the arrow that flows down it
//!
//! `DifferentiateFieldExt::gradient` produces `∇f` by forward-mode automatic differentiation:
//! it seeds one coordinate at a time as a `Dual::variable` and reads the `ε` channel of the
//! result. The field is written once, generic over `Scalar`, and the derivative comes from
//! evaluating that same code over `Dual` instead of over the working scalar. No step size is
//! chosen and no difference is taken, so the gradient carries no truncation error at all --
//! the contrast with `extension/manifold_laplacian_stencil`, where the derivative *is* a
//! difference and the step size is the accuracy.
//!
//! `Euler` then treats `-∇f` as a rate field and integrates it. That is gradient descent, and
//! it is the same arrow that steps heat forward in `cubical_heat_diffusion`. The integrator
//! never learns what it is integrating: swapping it for `Rk4` would raise the order here as
//! readily as it does there.
//!
//! Between the two, `CausalTensorWitness::fmap` evaluates the gradient across a whole grid, so
//! the field, its derivative and the flow along it are three composable values.

use deep_causality_algebra::Real;
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Euler, Scalar};
use deep_causality_haft::{Arrow, Functor};
use deep_causality_num::{lift, lift_usize, lower};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use std::ops::{Add, Mul};

/// Grid resolution per axis for the sampled gradient field.
const GRID: usize = 3;
/// Descent steps.
///
/// The slow axis decays by `1 - dt*2 = 0.84` per step, so its error from `x = 4` is
/// `3 * 0.84^n`. Sixty steps puts that under `1e-4`, which is what the closing check asserts.
const STEPS: usize = 60;

/// The working scalar. The field, its gradient and the descent path all carry it.
pub type FloatType = f64;

fn main() {
    print_header();

    // ---------------------------------------------------------------------
    // 1. The gradient at a point, against its closed form.
    // ---------------------------------------------------------------------
    let probe = [lift::<FloatType>(4.0), lift::<FloatType>(1.0)];
    let grad = Bowl.gradient(&probe);
    let exact = closed_form_gradient(&probe);
    print_point_gradient(&probe, &grad, &exact);

    // Forward-mode AD evaluates the same expression tree over `Dual`, so on a polynomial the
    // result is exact to the last bit rather than merely close.
    let residual = Real::abs(grad[0] - exact[0]) + Real::abs(grad[1] - exact[1]);
    print_residual(residual);
    assert_eq!(residual, lift::<FloatType>(0.0));

    // ---------------------------------------------------------------------
    // 2. The gradient across a grid, through the tensor's Functor.
    // ---------------------------------------------------------------------
    // A tensor of points becomes a tensor of gradient magnitudes under one `fmap`. The tensor
    // knows nothing about differentiation; the closure knows nothing about the grid.
    let points: CausalTensor<[FloatType; 2]> = CausalTensor::from_shape_fn(&[GRID, GRID], |idx| {
        [
            lift_usize::<FloatType>(idx[0]),
            lift_usize::<FloatType>(idx[1]) - lift::<FloatType>(3.0),
        ]
    });
    let magnitudes = CausalTensorWitness::fmap(points, |p| {
        let g = Bowl.gradient(&p);
        Real::sqrt(g[0] * g[0] + g[1] * g[1])
    });
    print_gradient_field(magnitudes.as_slice());

    // ---------------------------------------------------------------------
    // 3. `Euler` integrating `-∇f`: gradient descent as an arrow.
    // ---------------------------------------------------------------------
    // The rate field is a pure `Fn(&Point) -> Point`. The integrator is a separate value, and
    // the step size is its `dt`.
    let descent = Euler::new(lift::<FloatType>(0.08), |p: &Point| {
        let g = Bowl.gradient(&p.0);
        Point([-g[0], -g[1]])
    });

    let mut here = Point([lift::<FloatType>(4.0), lift::<FloatType>(1.0)]);
    print_descent_header();
    for step in 0..=STEPS {
        if step % 10 == 0 {
            print_descent_step(step, &here, Bowl.run(&here.0));
        }
        here = descent.run(here);
    }
    print_minimum(&here);

    // Both axes are within 1e-3 of the minimum (1, -2) after STEPS steps; see the note there.
    assert!(Real::abs(here.0[0] - lift::<FloatType>(1.0)) < lift::<FloatType>(1e-3));
    assert!(Real::abs(here.0[1] + lift::<FloatType>(2.0)) < lift::<FloatType>(1e-3));
}

/// An anisotropic bowl: `f(x, y) = (x - 1)^2 + 3 (y + 2)^2`.
///
/// Minimum at `(1, -2)`, with the closed-form gradient `(2(x - 1), 6(y + 2))`. The `3` makes
/// the two axes descend at different rates, which is what makes the path worth printing.
struct Bowl;

impl DifferentiableField<2> for Bowl {
    /// Written once, generic over `Scalar`. Evaluated at `FloatType` it is the field; evaluated
    /// at `Dual<FloatType>` it is the field *and* its derivative.
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let dx = p[0] - S::from_f64(1.0).expect("scalar from literal");
        let dy = p[1] + S::from_f64(2.0).expect("scalar from literal");
        dx * dx + S::from_f64(3.0).expect("scalar from literal") * dy * dy
    }
}

// `DifferentiateFieldExt` is blanket-implemented for every `DifferentiableField`, so declaring
// the field is all it takes to get `gradient`.

/// `∇f` written out by hand, to check the automatic one against.
fn closed_form_gradient(p: &[FloatType; 2]) -> [FloatType; 2] {
    [
        lift::<FloatType>(2.0) * (p[0] - lift::<FloatType>(1.0)),
        lift::<FloatType>(6.0) * (p[1] + lift::<FloatType>(2.0)),
    ]
}

/// The descent state. `Euler` needs a module-valued state (`Add` plus scalar `Mul`), which a
/// bare array does not provide, so the point rides in this newtype.
#[derive(Clone)]
struct Point([FloatType; 2]);

impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point([self.0[0] + rhs.0[0], self.0[1] + rhs.0[1]])
    }
}

impl Mul<FloatType> for Point {
    type Output = Point;
    fn mul(self, s: FloatType) -> Point {
        Point([self.0[0] * s, self.0[1] * s])
    }
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Operators: the exact gradient of a field, and the flow down it ===\n");
    println!("  f(x, y) = (x - 1)^2 + 3 (y + 2)^2,  minimum at (1, -2)");
    println!("  Precision: {}\n", core::any::type_name::<FloatType>());
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_point_gradient(p: &[FloatType; 2], auto: &[FloatType; 2], exact: &[FloatType; 2]) {
    println!("--- 1. The gradient at one point ---");
    println!("  at ({}, {})", lower(p[0]), lower(p[1]));
    println!(
        "  forward-mode AD : ({}, {})",
        lower(auto[0]),
        lower(auto[1])
    );
    println!(
        "  closed form     : ({}, {})",
        lower(exact[0]),
        lower(exact[1])
    );
}

fn print_residual(residual: FloatType) {
    println!(
        "  residual        : {}   (no step size was chosen, so none was paid)",
        lower(residual)
    );
}

fn print_gradient_field(magnitudes: &[FloatType]) {
    println!("\n--- 2. |∇f| across a {GRID}x{GRID} grid, via one fmap ---");
    for row in 0..GRID {
        let cells: Vec<String> = (0..GRID)
            .map(|col| format!("{:>8.3}", lower(magnitudes[row * GRID + col])))
            .collect();
        println!("  [{}]", cells.join(" "));
    }
}

fn print_descent_header() {
    println!("\n--- 3. Euler integrating -∇f: gradient descent ---");
    println!("  step        x          y        f(x, y)");
}

fn print_descent_step(step: usize, p: &Point, value: FloatType) {
    println!(
        "  {step:>4}  {:>9.5}  {:>9.5}  {:>11.6}",
        lower(p.0[0]),
        lower(p.0[1]),
        lower(value)
    );
}

fn print_minimum(p: &Point) {
    println!(
        "\n  arrived at ({:.5}, {:.5}); the bowl's minimum is (1, -2).",
        lower(p.0[0]),
        lower(p.0[1])
    );
    println!("  The same `Euler` steps heat in `cubical_heat_diffusion`; only the rate differs.");
}
