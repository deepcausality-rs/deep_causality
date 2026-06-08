/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_calculus::{Diff, DifferentiableArrow, Scalar};
use deep_causality_haft::{Arrow, Lift};
use deep_causality_num::Dual;

struct Square;
impl DifferentiableArrow for Square {
    fn run<S: Scalar>(&self, x: S) -> S {
        x * x
    }
}
// Value view: the model is also a concrete Arrow<f64, f64>.
impl Arrow for Square {
    type In = f64;
    type Out = f64;
    fn run(&self, x: f64) -> f64 {
        DifferentiableArrow::run(self, x)
    }
}

#[test]
fn test_model_is_a_plain_arrow() {
    assert_eq!(Arrow::run(&Square, 3.0_f64), 9.0);
}

#[test]
fn test_diff_is_a_concrete_dual_arrow() {
    let d: Diff<Square, f64> = Diff::new(Square);
    let y = d.run(Dual::variable(3.0)); // x² and its derivative at 3
    assert_eq!(y.value(), 9.0);
    assert_eq!(y.derivative(), 6.0);
}

#[test]
fn test_diff_composes_with_the_strength_algebra() {
    // Diff<Square> ∘ negate  ⇒  d/dx (−x²) = −2x; at x = 3 → −6.
    let pipeline = Diff::<Square, f64>::new(Square).compose(Lift::new(|d: Dual<f64>| -d));
    let y = pipeline.run(Dual::variable(3.0));
    assert_eq!(y.value(), -9.0);
    assert_eq!(y.derivative(), -6.0);
}
