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

use crate::constants::{
    BATH, COST_ECHO, COST_INTERVENTION, COST_PASSIVE, COST_TOMOGRAPHY, ONE, PREDICT_ECHO,
    PREDICT_HOLD_Q1, PREDICT_HOLD_Q2, PREDICT_PASSIVE, PREDICT_TOMOGRAPHY, Q1, Q2, QUBIT_FACTOR,
    SHOTS, TWO_LEG_FACTOR, ZERO,
};
use crate::{C, FloatType};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Experiment, FactorSupports, Hypothesis, Observable, ProcessFactors, Projection, QuantumPlant,
};
use deep_causality_tensor::CausalTensor;

/// A real entry of an operator matrix.
fn real(value: FloatType) -> C {
    Complex::new(value, ZERO)
}

/// A diagonal operator of the given dimension, entries in order.
fn diagonal(entries: &[FloatType]) -> Result<CausalTensor<C>, ModelBuildError> {
    let d = entries.len();
    let mut data = vec![real(ZERO); d * d];

    for (i, &entry) in entries.iter().enumerate() {
        data[i * d + i] = real(entry);
    }

    CausalTensor::new(data, vec![d, d]).map_err(|_| ModelBuildError::Operator)
}

/// A single-qubit factor.
fn qubit_factor() -> Result<CausalTensor<C>, ModelBuildError> {
    diagonal(&QUBIT_FACTOR)
}

/// A factor on a qubit and one parent.
fn two_leg_factor() -> Result<CausalTensor<C>, ModelBuildError> {
    diagonal(&TWO_LEG_FACTOR)
}

/// A structural candidate from its parent lists, one entry per factor node.
fn structural(
    name: &str,
    parents: &[(usize, &[usize])],
) -> Result<Hypothesis<FloatType>, ModelBuildError> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();

    for &(node, pa) in parents {
        let mut legs: Vec<usize> = pa.to_vec();
        legs.push(node);

        let factor = match pa.len() {
            0 => qubit_factor()?,
            1 => two_leg_factor()?,
            n => return Err(ModelBuildError::TooManyParents(node, n)),
        };

        factors.insert(node, factor);
        supports.declare(node, &legs);
    }

    Hypothesis::structural(name, factors, supports)
        .map_err(|_| ModelBuildError::Factorization(name.to_string()))
}

/// H₁: Q1 drives Q2.
pub fn h1_direct_q1_to_q2() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural("H1 Q1->Q2", &[(Q1, &[]), (Q2, &[Q1])])
}

/// H₂: Q2 drives Q1.
pub fn h2_direct_q2_to_q1() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural("H2 Q2->Q1", &[(Q2, &[]), (Q1, &[Q2])])
}

/// H₃: a common bath drives both.
pub fn h3_common_bath() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural("H3 Q1<-B->Q2", &[(BATH, &[]), (Q1, &[BATH]), (Q2, &[BATH])])
}

/// H₄: a cycle, Q1 → Q2 → B → Q1. Out of v1's scope by decision, and refused at `build()`.
pub fn h4_cyclic() -> Result<Hypothesis<FloatType>, ModelBuildError> {
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
pub fn plant() -> Result<QuantumPlant<FloatType>, ModelBuildError> {
    let ket = CausalTensor::from_slice(&[real(ONE), real(ZERO), real(ZERO), real(ZERO)], &[4]);

    QuantumPlant::from_ket(&ket).map_err(|_| ModelBuildError::Plant)
}

/// The projector onto "qubit 2 excited", `|01⟩⟨01| + |11⟩⟨11|` in `|q1 q2⟩` order.
pub fn e2_projector() -> Result<Observable<FloatType, 4>, ModelBuildError> {
    let p = diagonal(&[ZERO, ONE, ZERO, ONE])?;

    Ok(Observable::new(
        "e2",
        Projection::<FloatType, 4>::new(p).map_err(|_| ModelBuildError::Projector("e2"))?,
    ))
}

/// The projector onto "qubit 1 excited", `|10⟩⟨10| + |11⟩⟨11|`.
pub fn e1_projector() -> Result<Observable<FloatType, 4>, ModelBuildError> {
    let p = diagonal(&[ZERO, ZERO, ONE, ONE])?;

    Ok(Observable::new(
        "e1",
        Projection::<FloatType, 4>::new(p).map_err(|_| ModelBuildError::Projector("e1"))?,
    ))
}

/// The experiment family, with the predicted read-out under each of H₁, H₂, H₃ in that order.
pub fn experiments() -> Result<Vec<Experiment<FloatType>>, ModelBuildError> {
    let exp = |name: &'static str, cost: FloatType, predictions: [FloatType; 3]| {
        Experiment::new(name, cost, SHOTS, predictions.to_vec())
            .map_err(|_| ModelBuildError::Experiment(name))
    };

    Ok(vec![
        exp("E0 passive P(e1,e2)", COST_PASSIVE, PREDICT_PASSIVE)?,
        exp("E1 do(Q1=|1>) P(e2)", COST_INTERVENTION, PREDICT_HOLD_Q1)?,
        exp("E2 do(Q2=|1>) P(e1)", COST_INTERVENTION, PREDICT_HOLD_Q2)?,
        exp("E3 echo both P(e1,e2)", COST_ECHO, PREDICT_ECHO)?,
        exp("E4 process tomography", COST_TOMOGRAPHY, PREDICT_TOMOGRAPHY)?,
    ])
}

// =============================================================================
// Errors
// =============================================================================

/// What can go wrong assembling the problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelBuildError {
    /// An operator could not be formed as a square matrix.
    Operator,
    /// A node in this family was given more than one parent.
    TooManyParents(usize, usize),
    /// A factorization was rejected.
    Factorization(String),
    /// The plant state could not be formed.
    Plant,
    /// An observable's operator is not a projector.
    Projector(&'static str),
    /// An experiment's predictions are not probabilities.
    Experiment(&'static str),
}

impl core::fmt::Display for ModelBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModelBuildError::Operator => write!(f, "an operator is not a square matrix"),
            ModelBuildError::TooManyParents(node, n) => {
                write!(f, "node {node} has {n} parents; this family allows one")
            }
            ModelBuildError::Factorization(name) => {
                write!(f, "{name} is not a valid factorization")
            }
            ModelBuildError::Plant => write!(f, "the plant state could not be formed"),
            ModelBuildError::Projector(name) => {
                write!(f, "the observable {name} is not a projector")
            }
            ModelBuildError::Experiment(name) => {
                write!(f, "the experiment {name} does not predict probabilities")
            }
        }
    }
}

impl core::error::Error for ModelBuildError {}
