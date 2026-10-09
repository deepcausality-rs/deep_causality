/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The four structural candidates of the crosstalk problem, two of them as circuits.
//!
//! A `CircuitModel` carries its causal structure in its wiring: a wire leaving one node's box and
//! entering the next node's box is an edge of the induced DAG (Lorenz & Tull, Example 61), and the
//! dilation turns every node into a normalised factor `ρ_{A|Pa(A)}` on legs of dimension
//! `(d_in · d_out)²` per node. `H₁` and `H₂` are one wire with two single-qubit boxes in either
//! order, so each node's leg has dimension 16 and the conditional factor 256; they dilate and
//! screen at once. `H₃` needs a bath node with two output wires: its node dimension is
//! `d_in · d_out = 16` and its leg `d² = 256`, each single-wire child has a leg of 16, a child's
//! conditional factor on the legs `{bath, child}` is `4096 × 4096`, `2^24` entries, at the dilation
//! cap, and the Markov union over the three legs is `65536 × 65536`, `2^32` entries. It is kept as
//! the v1 factorization, which is a legal QCM by construction, for that cost and not by choice. `H₄` is the
//! same one-wire chain with its boxes grouped into a cycle, and `build()` refuses it.
//!
//! The plant, the observables and the experiment family are the v1 example's, so the plan and the
//! adjudication are decided by the same predictions. A circuit's dilation acts on one wire's boxes,
//! not on the two qubits the experiments hold and read, so no response model reads an experiment
//! off its factors. Each candidate's predictions are therefore computed from the factorization it
//! stands for, the `qcl_crosstalk` conditional tables: each qubit excited with probability
//! `EXCITED`, a driven qubit with probability `DRIVEN` when its driver is excited or on.

use crate::constants::{
    BATH, COST_ECHO, COST_INTERVENTION, COST_PASSIVE, COUPLING_ANGLE, DRIVEN, EXCITED, ONE,
    OWN_ANGLE, Q1, Q2, SHOTS, ZERO,
};
use crate::{C, FloatType};
use deep_causality_context_store::ContextSnapshot;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, CircuitBox, CircuitModel, CjFactor, ConfiguredExperiment, Experiment, FactorSupports,
    Hypothesis, Observable, ObservedContext, ProcessFactors, Projection, QuantumError,
    QuantumPlant, QubitOperator, Response, ResponseModel, WireType, embed_on_legs,
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

/// A single-qubit rotation as a one-operator Kraus box on one wire.
fn rotation(wire: usize, angle: FloatType) -> Result<CircuitBox<FloatType>, ModelBuildError> {
    let operator = QubitOperator::<FloatType>::rotation(Axis::Y, angle)
        .map_err(|_| ModelBuildError::Rotation)?;

    Ok(CircuitBox::Kraus {
        wires: vec![wire],
        kraus: vec![operator.matrix().clone()],
    })
}

/// `H₁` as a circuit: `Q1`'s box first, then `Q2`'s box on the wire it hands on, so the induced
/// DAG is `Q1 → Q2`. Node 0 is `Q1`, node 1 is `Q2`.
pub fn h1_circuit() -> Result<CircuitModel<FloatType>, ModelBuildError> {
    CircuitModel::new(
        vec![WireType::qubit()],
        vec![rotation(0, COUPLING_ANGLE)?, rotation(0, OWN_ANGLE)?],
        vec![vec![0], vec![1]],
        vec![],
        vec![0],
    )
    .map_err(|_| ModelBuildError::Circuit("H1"))
}

/// `H₂` as a circuit: the same two boxes with `Q2`'s first, grouped so that node 0 is still `Q1`
/// and node 1 still `Q2`; the induced DAG is `Q2 → Q1`.
pub fn h2_circuit() -> Result<CircuitModel<FloatType>, ModelBuildError> {
    CircuitModel::new(
        vec![WireType::qubit()],
        vec![rotation(0, COUPLING_ANGLE)?, rotation(0, OWN_ANGLE)?],
        vec![vec![1], vec![0]],
        vec![],
        vec![0],
    )
    .map_err(|_| ModelBuildError::Circuit("H2"))
}

/// `H₄` as a circuit: four boxes on one wire grouped as `Q1 → Q2 → B → Q1`, a cycle in the
/// induced DAG, which `build()` refuses.
pub fn h4_cyclic_circuit() -> Result<CircuitModel<FloatType>, ModelBuildError> {
    CircuitModel::new(
        vec![WireType::qubit()],
        vec![
            rotation(0, OWN_ANGLE)?,
            rotation(0, COUPLING_ANGLE)?,
            rotation(0, OWN_ANGLE)?,
            rotation(0, COUPLING_ANGLE)?,
        ],
        vec![vec![0, 3], vec![1], vec![2]],
        vec![],
        vec![0],
    )
    // The grouping is a legal partition, so the model itself is well formed. The cycle it induces
    // is what `build()` refuses, which is the point of the candidate.
    .map_err(|_| ModelBuildError::Circuit("H4"))
}

/// A qubit's excitation without its driver: what keeps its own excitation at `EXCITED`.
fn undriven() -> FloatType {
    EXCITED * (ONE - DRIVEN) / (ONE - EXCITED)
}

/// A structural candidate from `(node, support, diagonal of P(node | parents))` triples. A
/// diagonal runs over the support's legs in ascending order, the first most significant.
fn structural(
    name: &'static str,
    nodes: &[(usize, &[usize], Vec<FloatType>)],
) -> Result<Hypothesis<FloatType>, ModelBuildError> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();

    for (node, legs, entries) in nodes {
        factors.insert(*node, diagonal(entries)?);
        supports.declare(*node, legs);
    }

    Hypothesis::structural(name, factors, supports)
        .map_err(|_| ModelBuildError::Factorization(name))
}

/// A parentless qubit.
fn alone() -> Vec<FloatType> {
    vec![ONE - EXCITED, EXCITED]
}

/// The child's table when the child's leg comes second, over (parent, child).
fn child_second(off: FloatType, on: FloatType) -> Vec<FloatType> {
    vec![ONE - off, off, ONE - on, on]
}

/// The child's table when the child's leg comes first, over (child, parent).
fn child_first(off: FloatType, on: FloatType) -> Vec<FloatType> {
    vec![ONE - off, ONE - on, off, on]
}

/// `H₁` as the factorization its circuit stands for: Q1 drives Q2.
pub fn h1_factorization() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural(
        "H1 Q1->Q2",
        &[
            (Q1, &[Q1], alone()),
            (Q2, &[Q1, Q2], child_second(undriven(), DRIVEN)),
        ],
    )
}

/// `H₂` as the factorization its circuit stands for: Q2 drives Q1.
pub fn h2_factorization() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    structural(
        "H2 Q2->Q1",
        &[
            (Q2, &[Q2], alone()),
            (Q1, &[Q1, Q2], child_first(undriven(), DRIVEN)),
        ],
    )
}

/// `H₃` as the v1 factorization: a common bath, on with probability `EXCITED / DRIVEN`, drives
/// both qubits. The circuit form needs a two-output bath node whose dilation exceeds the entry cap;
/// see the module documentation.
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

/// The systems the v1 decomposability check is stated over.
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

/// What a setting does in a factorization's world: holding a qubit replaces its factor by
/// `|1⟩⟨1|` on the qubit and the identity on its parents; an echo replaces the factor of a qubit
/// driven by the other qubit by its own excitation; the instrument is `|1⟩⟨1|` on each qubit read
/// and the identity on the factorization's other legs.
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
pub fn configured_experiments()
-> Result<Vec<ConfiguredExperiment<FloatType, Setting>>, ModelBuildError> {
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

/// The experiment family as probes for the circuit-derived candidates: each experiment's
/// predicted read-out under `H₁`, `H₂`, `H₃` in that order, computed from the factorizations the
/// candidates stand for.
pub fn experiments() -> Result<Vec<Experiment<FloatType>>, Box<dyn std::error::Error>> {
    let twins = [h1_factorization()?, h2_factorization()?, h3_common_bath()?];
    let plant = plant()?;
    configured_experiments()?
        .iter()
        .map(|e| {
            let predictions = twins
                .iter()
                .map(|h| e.predict::<_, 4>(&CrosstalkModel, h, &plant, &[]))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Experiment::new(e.name(), e.cost(), e.shots(), predictions)?)
        })
        .collect()
}

// =============================================================================
// Errors
// =============================================================================

/// What can go wrong assembling the problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelBuildError {
    /// An operator could not be formed as a square matrix.
    Operator,
    /// A rotation angle was rejected.
    Rotation,
    /// A circuit's wiring is not a legal model.
    Circuit(&'static str),
    /// A factorization was rejected.
    Factorization(&'static str),
    /// The plant state could not be formed.
    Plant,
    /// An observable's operator is not a projector.
    Projector(&'static str),
    /// An experiment's cost or shot count was refused.
    Experiment(&'static str),
}

impl core::fmt::Display for ModelBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModelBuildError::Operator => write!(f, "an operator is not a square matrix"),
            ModelBuildError::Rotation => write!(f, "a rotation angle was rejected"),
            ModelBuildError::Circuit(name) => write!(f, "{name}'s wiring is not a legal model"),
            ModelBuildError::Factorization(name) => {
                write!(f, "{name} is not a valid factorization")
            }
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
