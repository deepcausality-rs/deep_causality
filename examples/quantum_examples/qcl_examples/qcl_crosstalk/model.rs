/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The problem: two qubits whose errors are correlated beyond independence, and four structural
//! candidates for why. Configuration only; the pipeline runs in `main.rs`.
//!
//! Each candidate is a factorization. Under the flat convention `support(A) = {A} ∪ Pa(A)`, the
//! supports carry the causal structure: a leg of a node's support that is itself a factor node is
//! one of its parents. The factors are diagonal so that every candidate is a legal QCM, which puts
//! the whole weight of the discrimination on the interventions, where the note says it belongs.
//!
//! The experiments and their predicted read-outs are the note's Table §4: holding Q1 excited
//! exposes a ZZ shift on Q2 under H₁ only, holding Q2 excited exposes it on Q1 under H₂ only, an
//! echo refocuses a quasi-static coupling under H₁ and H₂ but not a fluctuating bath under H₃,
//! and process tomography resolves everything at a hundred times the price.

use crate::constants::{BATH, Q1, Q2, SHOTS};
use crate::{C, FloatType};
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Experiment, FactorSupports, Hypothesis, Observable, ProcessFactors, Projection, QuantumPlant,
};
use deep_causality_tensor::CausalTensor;

/// A configuration literal, lifted once into the working type. This is the boundary at which
/// `f64` may appear: the model is written against `FloatType` from here on.
pub fn lift(x: f64) -> FloatType {
    <FloatType as FromPrimitive>::from_f64(x).expect("a configuration literal is representable")
}

fn c(re: f64) -> C {
    Complex::new(lift(re), lift(0.0))
}

/// A diagonal operator of the given dimension, entries in order.
fn diagonal(entries: &[f64]) -> CausalTensor<C> {
    let d = entries.len();
    let mut data = vec![c(0.0); d * d];
    for (i, &e) in entries.iter().enumerate() {
        data[i * d + i] = c(e);
    }
    CausalTensor::new(data, vec![d, d]).expect("a square matrix")
}

/// A single-qubit factor.
fn qubit_factor() -> CausalTensor<C> {
    diagonal(&[0.9, 0.1])
}

/// A factor on a qubit and one parent.
fn two_leg_factor() -> CausalTensor<C> {
    diagonal(&[0.85, 0.05, 0.05, 0.05])
}

/// A structural candidate from its parent lists, one entry per factor node.
fn structural(name: &str, parents: &[(usize, &[usize])]) -> Hypothesis<FloatType> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();
    for &(node, pa) in parents {
        let mut legs: Vec<usize> = pa.to_vec();
        legs.push(node);
        factors.insert(
            node,
            if pa.is_empty() {
                qubit_factor()
            } else {
                assert_eq!(pa.len(), 1, "one parent per node in this family");
                two_leg_factor()
            },
        );
        supports.declare(node, &legs);
    }
    Hypothesis::structural(name, factors, supports).expect("a validated factorization")
}

/// H₁: Q1 drives Q2.
pub fn h1_direct_q1_to_q2() -> Hypothesis<FloatType> {
    structural("H1 Q1->Q2", &[(Q1, &[]), (Q2, &[Q1])])
}

/// H₂: Q2 drives Q1.
pub fn h2_direct_q2_to_q1() -> Hypothesis<FloatType> {
    structural("H2 Q2->Q1", &[(Q2, &[]), (Q1, &[Q2])])
}

/// H₃: a common bath drives both.
pub fn h3_common_bath() -> Hypothesis<FloatType> {
    structural("H3 Q1<-B->Q2", &[(BATH, &[]), (Q1, &[BATH]), (Q2, &[BATH])])
}

/// H₄: a cycle, Q1 → Q2 → B → Q1. Out of v1's scope by decision, and refused at `build()`.
pub fn h4_cyclic() -> Hypothesis<FloatType> {
    structural(
        "H4 Q1->Q2->B->Q1",
        &[(Q1, &[BATH]), (Q2, &[Q1]), (BATH, &[Q2])],
    )
}

/// The declared input and output systems: every node is both, under the flat convention.
pub fn systems() -> Vec<usize> {
    vec![Q1, Q2, BATH]
}

/// The two-qubit plant in `|00⟩`.
pub fn plant() -> QuantumPlant<FloatType> {
    let ket = CausalTensor::from_slice(&[c(1.0), c(0.0), c(0.0), c(0.0)], &[4]);
    QuantumPlant::from_ket(&ket).expect("a state")
}

/// The projector onto "qubit 2 excited", `|01⟩⟨01| + |11⟩⟨11|` in `|q1 q2⟩` order.
pub fn e2_projector() -> Observable<FloatType, 4> {
    let p = diagonal(&[0.0, 1.0, 0.0, 1.0]);
    Observable::new(
        "e2",
        Projection::<FloatType, 4>::new(p).expect("a projector"),
    )
}

/// The projector onto "qubit 1 excited", `|10⟩⟨10| + |11⟩⟨11|`.
pub fn e1_projector() -> Observable<FloatType, 4> {
    let p = diagonal(&[0.0, 0.0, 1.0, 1.0]);
    Observable::new(
        "e1",
        Projection::<FloatType, 4>::new(p).expect("a projector"),
    )
}

/// The experiment family, with the predicted read-out under each of H₁, H₂, H₃ in that order.
pub fn experiments() -> Vec<Experiment<FloatType>> {
    let exp = |name: &str, cost: f64, predictions: [f64; 3]| {
        Experiment::new(name, lift(cost), SHOTS, predictions.map(lift).to_vec())
            .expect("a probability triple")
    };
    vec![
        exp("E0 passive P(e1,e2)", 1.0, [0.04, 0.04, 0.04]),
        exp("E1 do(Q1=|1>) P(e2)", 1.0, [0.40, 0.10, 0.10]),
        exp("E2 do(Q2=|1>) P(e1)", 1.0, [0.10, 0.40, 0.10]),
        exp("E3 echo both P(e1,e2)", 2.0, [0.01, 0.01, 0.04]),
        exp("E4 process tomography", 200.0, [0.90, 0.50, 0.10]),
    ]
}
