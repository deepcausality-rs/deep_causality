/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Lean Choi-application proofs.
//!
//! Mirrors `lean/DeepCausalityFormal/Quantum/Choi.lean`: `applyChoi_add` and
//! `applyChoi_smul` establish that the channel action X ↦ applyChoi(J, X) is
//! ℂ-linear, and `applyChoi_choiOf` that the action reconstructed from a
//! channel's Choi matrix is the channel. See the module docstring in
//! `partial_trace_tests.rs` for how these `THEOREM_MAP` witnesses feed the
//! `theorem-map` CI gate.

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

/// The qubit amplitude-damping channel with decay probability γ. Its Choi
/// operator couples |0⟩⟨0| ⊗ |0⟩⟨0| with |1⟩⟨1| ⊗ |1⟩⟨1| through √(1−γ), so it is
/// not diagonal.
fn amplitude_damping_kraus(gamma: f64) -> Vec<CausalTensor<C>> {
    let keep = (1.0 - gamma).sqrt();
    let decay = gamma.sqrt();
    vec![
        mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(keep, 0.)], 2, 2),
        mat(vec![c(0., 0.), c(decay, 0.), c(0., 0.), c(0., 0.)], 2, 2),
    ]
}

/// The matrix unit |i⟩⟨j| on a qubit.
fn unit(i: usize, j: usize) -> CausalTensor<C> {
    let mut data = vec![c(0., 0.); 4];
    data[i * 2 + j] = c(1., 0.);
    mat(data, 2, 2)
}

// THEOREM_MAP: quantum.choi.reconstruction
#[test]
fn test_choi_reconstruction_recovers_the_channel() {
    // Lean: applyChoi_choiOf — applyChoi (choiOf E) A = E A. The Choi matrix is
    // built here as Lean's `choiOf` defines it, J (i,k) (j,l) = E(|i⟩⟨j|) (k,l),
    // by applying the channel to each matrix unit; nothing comes from the
    // crate's Choi constructor.
    let kraus = amplitude_damping_kraus(0.3);
    let channel = |m: &CausalTensor<C>| apply_kraus(&kraus, m).unwrap();

    let mut j_data = vec![c(0., 0.); 16];
    for i in 0..2 {
        for j in 0..2 {
            let image = channel(&unit(i, j));
            for k in 0..2 {
                for l in 0..2 {
                    j_data[(i * 2 + k) * 4 + (j * 2 + l)] = image.as_slice()[k * 2 + l];
                }
            }
        }
    }
    let choi = mat(j_data, 4, 4);
    let off_diagonal = choi.as_slice()[3].re.abs();
    assert!(off_diagonal > 0.5, "the Choi matrix must not be diagonal");

    // A generic input: complex, not Hermitian, no zero entry.
    let a = mat(
        vec![c(0.6, 0.1), c(-0.3, 0.4), c(0.2, -0.7), c(0.9, 0.25)],
        2,
        2,
    );
    let reconstructed = apply_choi(&choi, &a, 2, 2).unwrap();
    let direct = channel(&a);
    assert!(
        max_abs_diff(&reconstructed, &direct) < 1e-12,
        "applyChoi(choiOf E) A differs from E A by {}",
        max_abs_diff(&reconstructed, &direct)
    );

    // The crate's Choi constructor agrees with the definition-built matrix.
    let from_kraus = choi_from_kraus(&kraus).unwrap();
    assert!(max_abs_diff(&choi, &from_kraus) < 1e-12);
}
