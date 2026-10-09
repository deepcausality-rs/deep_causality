/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The problem: two qubits whose errors are correlated beyond independence, and four structural
//! candidates for why. Configuration only; the pipeline runs in `main.rs`.
//!
//! Each candidate is a factorization `σ = ∏ ρ_{A|Pa(A)}` of conditional tables. Under the flat
//! convention `support(A) = {A} ∪ Pa(A)`, the supports carry the causal structure: a leg of a
//! node's support that is itself a factor node is one of its parents. The factors are diagonal, so
//! every candidate is a legal QCM and the whole weight of the discrimination rests on the
//! interventions.
//!
//! Each qubit is excited with probability `EXCITED` in every candidate, so passive data cannot tell
//! them apart. Under H₁, Q2 is excited with probability `DRIVEN` when Q1 is, and with the
//! probability that keeps its own excitation at `EXCITED` when Q1 is not; H₂ is the mirror image;
//! under H₃ a bath, on with probability `EXCITED / DRIVEN`, excites each qubit with probability
//! `DRIVEN` when on and never when off.
//!
//! The experiments are configured, and a response model computes what each does in each
//! candidate's world: holding a qubit excited replaces its factor, and an echo, which refocuses a
//! quasi-static direct coupling and not a fluctuating bath, replaces the driven qubit's factor by
//! its own excitation.

use crate::constants::{
    BATH, COST_ECHO, COST_INTERVENTION, COST_PASSIVE, DRIVEN, EXCITED, ONE, Q1, Q2, SHOTS, ZERO,
};
use crate::{C, FloatType};
use deep_causality_context_store::ContextSnapshot;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CjFactor, ConfiguredExperiment, FactorSupports, Hypothesis, Observable, ObservedContext,
    ProcessFactors, Projection, QuantumError, QuantumPlant, Response, ResponseModel, embed_on_legs,
};
use deep_causality_tensor::{CausalTensor, Tensor};
use std::collections::{BTreeMap, BTreeSet};

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

/// A qubit's excitation without its driver: what keeps its own excitation at `EXCITED`.
fn undriven() -> FloatType {
    EXCITED * (ONE - DRIVEN) / (ONE - EXCITED)
}

/// A structural candidate from `(node, support, diagonal of P(node | parents))` triples. A
/// diagonal runs over the support's legs in ascending order, the first most significant. Every
/// entry is a probability, so constants that derive one outside `[0, 1]` are refused here.
fn structural(
    name: &str,
    nodes: &[(usize, &[usize], Vec<FloatType>)],
) -> Result<Hypothesis<FloatType>, ModelBuildError> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();

    for (node, legs, entries) in nodes {
        if let Some(&entry) = entries.iter().find(|&&p| !(ZERO..=ONE).contains(&p)) {
            return Err(ModelBuildError::Probability {
                candidate: name.to_string(),
                node: *node,
                entry,
            });
        }
        factors.insert(*node, diagonal(entries)?);
        supports.declare(*node, legs);
    }

    Hypothesis::structural(name, factors, supports)
        .map_err(|_| ModelBuildError::Factorization(name.to_string()))
}

/// A parentless qubit.
fn alone() -> Vec<FloatType> {
    vec![ONE - EXCITED, EXCITED]
}

/// The child's table when the child's leg comes second: `(0,0), (0,1), (1,0), (1,1)` over
/// (parent, child).
fn child_second(off: FloatType, on: FloatType) -> Vec<FloatType> {
    vec![ONE - off, off, ONE - on, on]
}

/// The child's table when the child's leg comes first: over (child, parent).
fn child_first(off: FloatType, on: FloatType) -> Vec<FloatType> {
    vec![ONE - off, ONE - on, off, on]
}

/// H₁: Q1 drives Q2.
pub fn h1_direct_q1_to_q2() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural(
        "H1 Q1->Q2",
        &[
            (Q1, &[Q1], alone()),
            (Q2, &[Q1, Q2], child_second(undriven(), DRIVEN)),
        ],
    )
}

/// H₂: Q2 drives Q1.
pub fn h2_direct_q2_to_q1() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural(
        "H2 Q2->Q1",
        &[
            (Q2, &[Q2], alone()),
            (Q1, &[Q1, Q2], child_first(undriven(), DRIVEN)),
        ],
    )
}

/// H₃: a common bath drives both.
pub fn h3_common_bath() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    let on = EXCITED / DRIVEN;
    structural(
        "H3 Q1<-B->Q2",
        &[
            (BATH, &[BATH], vec![ONE - on, on]),
            (Q1, &[Q1, BATH], child_first(ZERO, DRIVEN)),
            (Q2, &[Q2, BATH], child_first(ZERO, DRIVEN)),
        ],
    )
}

/// H₄: a cycle, Q1 → Q2 → B → Q1. Out of v1's scope by decision, and refused at `build()`.
pub fn h4_cyclic() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural(
        "H4 Q1->Q2->B->Q1",
        &[
            (Q1, &[Q1, BATH], child_first(undriven(), DRIVEN)),
            (Q2, &[Q1, Q2], child_second(undriven(), DRIVEN)),
            (BATH, &[Q2, BATH], child_second(undriven(), DRIVEN)),
        ],
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

// =============================================================================
// The experiments and the response model
// =============================================================================

/// What an experiment sets and reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    /// No intervention; read both qubits excited.
    Passive,
    /// `do(node = |1⟩)`; read `read` excited.
    Hold { node: usize, read: usize },
    /// An echo on both qubits; read both qubits excited.
    Echo,
}

/// A setting is not a context, so its observations record none.
impl ObservedContext for Setting {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// What a setting does in each candidate's world: holding a qubit replaces its factor by `|1⟩⟨1|`
/// on the qubit and the identity on its parents; an echo replaces the factor of a qubit driven by
/// the other qubit by its own excitation, whatever its driver does; the instrument is `|1⟩⟨1|` on
/// each qubit read and the identity on the candidate's other legs.
pub struct CrosstalkModel;

impl ResponseModel<FloatType, Setting> for CrosstalkModel {
    fn respond(
        &self,
        candidate: &Hypothesis<FloatType>,
        setting: &Setting,
    ) -> Result<Response<FloatType>, QuantumError> {
        let supports = candidate.supports().ok_or_else(|| {
            QuantumError::CalculationError(format!("'{}' is not structural", candidate.name()))
        })?;
        let support_space = |node: usize| -> Result<BTreeMap<usize, usize>, QuantumError> {
            let legs = supports.support(node).ok_or_else(|| {
                QuantumError::DimensionMismatch(format!(
                    "'{}' has no node {node}",
                    candidate.name()
                ))
            })?;
            Ok(supports.space_map(&legs.iter().copied().collect()))
        };
        let both = [Q1, Q2];
        let (factors, read): (Vec<(usize, CjFactor<FloatType>)>, &[usize]) = match setting {
            Setting::Passive => (Vec::new(), &both),
            Setting::Hold { node, read } => (
                vec![(*node, on_leg([ZERO, ONE], *node, &support_space(*node)?)?)],
                std::slice::from_ref(read),
            ),
            Setting::Echo => {
                let mut decoupled = Vec::new();
                for (node, driver) in [(Q1, Q2), (Q2, Q1)] {
                    let space = support_space(node)?;
                    if space.contains_key(&driver) {
                        decoupled.push((node, on_leg([ONE - EXCITED, EXCITED], node, &space)?));
                    }
                }
                (decoupled, &both)
            }
        };
        let legs = candidate.legs()?;
        let mut instrument: Option<CausalTensor<C>> = None;
        for &qubit in read {
            let excited = on_leg([ZERO, ONE], qubit, &legs)?;
            instrument = Some(match instrument {
                None => excited,
                Some(acc) => acc.matmul(&excited).map_err(|e| {
                    QuantumError::CalculationError(format!("instrument product: {e:?}"))
                })?,
            });
        }
        let instrument = instrument
            .ok_or_else(|| QuantumError::CalculationError("a setting reads a qubit".into()))?;
        Ok(Response::Intervention {
            factors,
            instrument,
        })
    }
}

/// `diag(entries)` on `leg`, the identity on the other legs of `space`.
fn on_leg(
    entries: [FloatType; 2],
    leg: usize,
    space: &BTreeMap<usize, usize>,
) -> Result<CjFactor<FloatType>, QuantumError> {
    let d = CausalTensor::from_slice(
        &[real(entries[0]), real(ZERO), real(ZERO), real(entries[1])],
        &[2, 2],
    );
    embed_on_legs(&d, &BTreeSet::from([leg]), space)
}

/// The experiment family: passive, hold each qubit and read the other, and the echo.
pub fn experiments() -> Result<Vec<ConfiguredExperiment<FloatType, Setting>>, ModelBuildError> {
    let exp = |name: &'static str, cost: FloatType, setting: Setting| {
        ConfiguredExperiment::new(name, cost, SHOTS, setting, 0)
            .map_err(|_| ModelBuildError::Experiment(name))
    };

    Ok(vec![
        exp("E0 passive P(e1,e2)", COST_PASSIVE, Setting::Passive)?,
        exp(
            "E1 do(Q1=|1>) P(e2)",
            COST_INTERVENTION,
            Setting::Hold { node: Q1, read: Q2 },
        )?,
        exp(
            "E2 do(Q2=|1>) P(e1)",
            COST_INTERVENTION,
            Setting::Hold { node: Q2, read: Q1 },
        )?,
        exp("E3 echo both P(e1,e2)", COST_ECHO, Setting::Echo)?,
    ])
}

// =============================================================================
// Errors
// =============================================================================

/// What can go wrong assembling the problem.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelBuildError {
    /// An operator could not be formed as a square matrix.
    Operator,
    /// A factorization was rejected.
    Factorization(String),
    /// A conditional table's entry is not a probability.
    Probability {
        candidate: String,
        node: usize,
        entry: FloatType,
    },
    /// The plant state could not be formed.
    Plant,
    /// An observable's operator is not a projector.
    Projector(&'static str),
    /// An experiment's cost or shots were refused.
    Experiment(&'static str),
}

impl core::fmt::Display for ModelBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModelBuildError::Operator => write!(f, "an operator is not a square matrix"),
            ModelBuildError::Factorization(name) => {
                write!(f, "{name} is not a valid factorization")
            }
            ModelBuildError::Probability {
                candidate,
                node,
                entry,
            } => write!(
                f,
                "{candidate}: node {node}'s table has the entry {entry}, outside [0, 1]"
            ),
            ModelBuildError::Plant => write!(f, "the plant state could not be formed"),
            ModelBuildError::Projector(name) => {
                write!(f, "the observable {name} is not a projector")
            }
            ModelBuildError::Experiment(name) => {
                write!(f, "the experiment {name} has an invalid cost or shot count")
            }
        }
    }
}

impl core::error::Error for ModelBuildError {}
