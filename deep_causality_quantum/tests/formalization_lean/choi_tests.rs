/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Lean Choi-application proofs.
//!
//! Mirrors `lean/DeepCausalityFormal/Quantum/Choi.lean`, whose `applyChoi_add`
//! and `applyChoi_smul` establish that the channel action X ↦ applyChoi(J, X)
//! is ℂ-linear. See the module docstring in `partial_trace_tests.rs` for how
//! these `THEOREM_MAP` witnesses feed the `theorem-map` CI gate.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{apply_choi, apply_kraus, choi_from_kraus};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, rows: usize, cols: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![rows, cols]).unwrap()
}

fn max_abs_diff(a: &CausalTensor<C>, b: &CausalTensor<C>) -> f64 {
    assert_eq!(
        a.shape(),
        b.shape(),
        "max_abs_diff shape mismatch: {:?} vs {:?}",
        a.shape(),
        b.shape()
    );
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| ((x.re - y.re).powi(2) + (x.im - y.im).powi(2)).sqrt())
        .fold(0.0, f64::max)
}

fn scale(a: &CausalTensor<C>, s: C) -> CausalTensor<C> {
    let data: Vec<C> = a
        .as_slice()
        .iter()
        .map(|x| c(x.re * s.re - x.im * s.im, x.re * s.im + x.im * s.re))
        .collect();
    CausalTensor::new(data, a.shape().to_vec()).unwrap()
}

/// The qubit depolarizing channel with parameter p as a 4-element Kraus family.
fn depolarizing_kraus(p: f64) -> Vec<CausalTensor<C>> {
    let s0 = (1.0 - 3.0 * p / 4.0).sqrt();
    let s = (p / 4.0_f64).sqrt();
    vec![
        mat(vec![c(s0, 0.), c(0., 0.), c(0., 0.), c(s0, 0.)], 2, 2), // √(1−3p/4)·I
        mat(vec![c(0., 0.), c(s, 0.), c(s, 0.), c(0., 0.)], 2, 2),   // √(p/4)·σx
        mat(vec![c(0., 0.), c(0., -s), c(0., s), c(0., 0.)], 2, 2),  // √(p/4)·σy
        mat(vec![c(s, 0.), c(0., 0.), c(0., 0.), c(-s, 0.)], 2, 2),  // √(p/4)·σz
    ]
}

// THEOREM_MAP: quantum.choi.apply_add
// THEOREM_MAP: quantum.choi.apply_smul
#[test]
fn test_apply_choi_is_linear() {
    // Lean: applyChoi_add, applyChoi_smul — the Choi action X ↦ apply_choi(J, X)
    // is ℂ-linear: apply_choi(J, αX + Y) = α·apply_choi(J, X) + apply_choi(J, Y).
    let kraus = depolarizing_kraus(0.5);
    let j = choi_from_kraus(&kraus).unwrap();
    let apply = |m: &CausalTensor<C>| apply_choi(&j, m, 2, 2).unwrap();

    let x = mat(
        vec![c(0.75, 0.), c(0.25, 0.1), c(0.25, -0.1), c(0.25, 0.)],
        2,
        2,
    );
    let y = mat(vec![c(0.2, 0.), c(0., 0.3), c(0., -0.3), c(0.8, 0.)], 2, 2);
    let alpha = c(0.5, -1.5);

    // applyChoi_add: additive in the operator argument.
    let add_lhs = apply(&(x.clone() + y.clone()));
    let add_rhs = apply(&x) + apply(&y);
    assert!(max_abs_diff(&add_lhs, &add_rhs) < 1e-12, "applyChoi_add");

    // applyChoi_smul: homogeneous under complex scaling.
    let smul_lhs = apply(&scale(&x, alpha));
    let smul_rhs = scale(&apply(&x), alpha);
    assert!(max_abs_diff(&smul_lhs, &smul_rhs) < 1e-12, "applyChoi_smul");

    // Cross-check: the Choi route agrees with the Kraus route it was built from.
    let via_kraus = apply_kraus(&kraus, &x).unwrap();
    assert!(max_abs_diff(&apply(&x), &via_kraus) < 1e-12);
}
