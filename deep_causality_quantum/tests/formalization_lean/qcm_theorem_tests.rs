/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Lean quantum-causal-model theorems.
//!
//! Mirrors three files under `lean/DeepCausalityFormal/Quantum/`:
//! `Markov.lean` (factors on disjoint legs commute; a pairwise-commuting product
//! does not depend on its order), `ClassicalEmbedding.lean` (diagonal factors
//! commute; the Kronecker product of diagonals is diagonal) and
//! `NoInfluence.lean` (the D marginal factors through the partial trace over A
//! exactly when it depends on the A input only through its trace).
//!
//! Each witness runs on the crate's own operations: `embed_on_legs` is the
//! alignment the freeze check uses, and `partial_trace` is the crate's partial
//! trace. Legs are numbered in factor order, leg 0 first.

use std::collections::{BTreeMap, BTreeSet};

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{embed_on_legs, frobenius_norm, matrix_commutator, partial_trace};
use deep_causality_tensor::{CausalTensor, Tensor};

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

fn max_abs_diff(a: &CausalTensor<C>, b: &CausalTensor<C>) -> f64 {
    assert_eq!(a.shape(), b.shape());
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| ((x.re - y.re).powi(2) + (x.im - y.im).powi(2)).sqrt())
        .fold(0.0, f64::max)
}

fn dagger(a: &CausalTensor<C>) -> CausalTensor<C> {
    let d = a.shape()[0];
    let s = a.as_slice();
    let data = (0..d * d)
        .map(|n| {
            let x = s[(n % d) * d + n / d];
            c(x.re, -x.im)
        })
        .collect();
    mat(data, d)
}

fn mul(a: &CausalTensor<C>, b: &CausalTensor<C>) -> CausalTensor<C> {
    a.matmul(b).unwrap()
}

fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0., 0.), c(1., 0.), c(1., 0.), c(0., 0.)], 2)
}

fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(-1., 0.)], 2)
}

fn diag(entries: &[C]) -> CausalTensor<C> {
    let d = entries.len();
    let mut data = vec![c(0., 0.); d * d];
    for (i, &x) in entries.iter().enumerate() {
        data[i * d + i] = x;
    }
    mat(data, d)
}

/// A factor on `legs` of a space of qubit legs `0..n`, through the freeze check's alignment.
fn on_legs(op: &CausalTensor<C>, legs: &[usize], n: usize) -> CausalTensor<C> {
    let op_legs: BTreeSet<usize> = legs.iter().copied().collect();
    let space: BTreeMap<usize, usize> = (0..n).map(|l| (l, 2)).collect();
    embed_on_legs(op, &op_legs, &space).unwrap()
}

// THEOREM_MAP: quantum.markov_commutativity
#[test]
fn test_factors_on_disjoint_legs_commute() {
    // Lean: kron_one_commute_one_kron — (A ⊗ 1)(1 ⊗ B) = (1 ⊗ B)(A ⊗ 1), for A
    // and B that do not commute with each other on a shared leg.
    let a = mat(vec![c(1., 0.), c(2., 1.), c(0., -1.), c(-1., 0.)], 2);
    let b = mat(vec![c(0., 1.), c(3., 0.), c(1., 1.), c(2., 0.)], 2);
    assert!(frobenius_norm(&matrix_commutator(&a, &b).unwrap()) > 1.0);

    let a_on_0 = on_legs(&a, &[0], 2);
    let b_on_1 = on_legs(&b, &[1], 2);
    let commutator = matrix_commutator(&a_on_0, &b_on_1).unwrap();
    assert!(frobenius_norm(&commutator) < 1e-12);
}

// THEOREM_MAP: quantum.markov_commutativity
#[test]
fn test_a_pairwise_commuting_product_does_not_depend_on_order() {
    // Lean: pairwise_commute_prod_perm. Three factors on three disjoint legs
    // commute pairwise, so all orders of the product agree.
    let f0 = on_legs(&sigma_x(), &[0], 3);
    let f1 = on_legs(
        &mat(vec![c(1., 0.), c(0., 2.), c(0., 0.), c(3., 0.)], 2),
        &[1],
        3,
    );
    let f2 = on_legs(&sigma_z(), &[2], 3);
    let reference = mul(&mul(&f0, &f1), &f2);
    for order in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
        let fs = [&f0, &f1, &f2];
        let product = mul(&mul(fs[order[0]], fs[order[1]]), fs[order[2]]);
        assert!(
            max_abs_diff(&product, &reference) < 1e-12,
            "order {order:?}"
        );
    }

    // Negative control: σx and σz on the same leg do not commute, and their
    // product depends on the order.
    let x = on_legs(&sigma_x(), &[0], 1);
    let z = on_legs(&sigma_z(), &[0], 1);
    assert!(max_abs_diff(&mul(&x, &z), &mul(&z, &x)) > 1.0);
}

// THEOREM_MAP: quantum.classical_embedding
#[test]
fn test_diagonal_factors_commute_and_their_kronecker_product_is_diagonal() {
    // Lean: diagonal_commute, diagonal_prod_perm, kron_diagonal.
    let d1 = diag(&[c(0.9, 0.), c(0.1, 0.3), c(-0.4, 0.2)]);
    let d2 = diag(&[c(0.2, -0.5), c(1.5, 0.), c(0.7, 0.7)]);
    assert!(frobenius_norm(&matrix_commutator(&d1, &d2).unwrap()) < 1e-12);

    // The Kronecker product of diag(a) and diag(b) is diag(a_i · b_j) on the pair index.
    let a = [c(0.9, 0.), c(0.1, 0.3)];
    let b = [c(0.2, -0.5), c(1.5, 0.)];
    let product = diag(&a).kronecker(&diag(&b)).unwrap();
    let expected: Vec<C> = a
        .iter()
        .flat_map(|&x| b.iter().map(move |&y| x * y))
        .collect();
    assert!(max_abs_diff(&product, &diag(&expected)) < 1e-12);
}

/// `U ρ U†` on two qubits.
fn conjugate(u: &CausalTensor<C>, rho: &CausalTensor<C>) -> CausalTensor<C> {
    mul(&mul(u, rho), &dagger(u))
}

/// The D marginal: trace out the output's first leg, C.
fn d_marginal(op: &CausalTensor<C>) -> CausalTensor<C> {
    partial_trace(op, &[2, 2], &[0]).unwrap()
}

/// The swap of two qubits, which carries the input A to the output D.
fn swap() -> CausalTensor<C> {
    let mut data = vec![c(0., 0.); 16];
    for i in 0..2 {
        for j in 0..2 {
            data[(j * 2 + i) * 4 + (i * 2 + j)] = c(1., 0.);
        }
    }
    mat(data, 4)
}

// THEOREM_MAP: quantum.no_influence
#[test]
fn test_a_product_unitary_has_no_influence_from_a_to_d() {
    // Lean: noInfluence_iff. Under U = V ⊗ W the D marginal is W · Tr_A(ρ) · W†.
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let v = mat(vec![c(s, 0.), c(s, 0.), c(s, 0.), c(-s, 0.)], 2);
    let w = mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 1.)], 2);
    let u = v.kronecker(&w).unwrap();

    // Product inputs whose A parts differ but share a trace give the same D marginal.
    let x = mat(vec![c(0.7, 0.), c(0.2, 0.1), c(0.2, -0.1), c(0.3, 0.)], 2);
    let x_prime = mat(vec![c(0.1, 0.), c(0., -0.4), c(0., 0.4), c(0.9, 0.)], 2);
    let y = mat(vec![c(0.6, 0.), c(0.1, 0.2), c(0.1, -0.2), c(0.4, 0.)], 2);
    let out = d_marginal(&conjugate(&u, &x.kronecker(&y).unwrap()));
    let out_prime = d_marginal(&conjugate(&u, &x_prime.kronecker(&y).unwrap()));
    assert!(max_abs_diff(&out, &out_prime) < 1e-12);

    // The factorization holds on an entangled input too: Tr_C(U ρ U†) = W Tr_A(ρ) W†
    // for the Bell projector, where no product form exists.
    let h = 0.5;
    let bell = mat(
        vec![
            c(h, 0.),
            c(0., 0.),
            c(0., 0.),
            c(h, 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(h, 0.),
            c(0., 0.),
            c(0., 0.),
            c(h, 0.),
        ],
        4,
    );
    let lhs = d_marginal(&conjugate(&u, &bell));
    let rhs = conjugate(&w, &partial_trace(&bell, &[2, 2], &[0]).unwrap());
    assert!(max_abs_diff(&lhs, &rhs) < 1e-12);
}

// THEOREM_MAP: quantum.no_influence
#[test]
fn test_the_swap_carries_a_to_d() {
    // Lean: noInfluence_iff, the failing side. The swap sends X ⊗ Y to Y ⊗ X, so the
    // D marginal is Tr(Y) · X: two A inputs of equal trace give different marginals.
    let x = mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 0.)], 2); // |0⟩⟨0|
    let x_prime = mat(vec![c(0., 0.), c(0., 0.), c(0., 0.), c(1., 0.)], 2); // |1⟩⟨1|
    let y = mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 0.)], 2);
    let out = d_marginal(&conjugate(&swap(), &x.kronecker(&y).unwrap()));
    let out_prime = d_marginal(&conjugate(&swap(), &x_prime.kronecker(&y).unwrap()));
    assert!(max_abs_diff(&out, &x) < 1e-12);
    assert!(max_abs_diff(&out_prime, &x_prime) < 1e-12);
    assert!(max_abs_diff(&out, &out_prime) > 0.5);
}
