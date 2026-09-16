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
//! adjudication are decided by the same predictions.

use crate::constants::{
    BATH, COST_ECHO, COST_INTERVENTION, COST_PASSIVE, COST_TOMOGRAPHY, COUPLING_ANGLE, ONE,
    OWN_ANGLE, PREDICT_ECHO, PREDICT_HOLD_Q1, PREDICT_HOLD_Q2, PREDICT_PASSIVE, PREDICT_TOMOGRAPHY,
    Q1, Q2, QUBIT_FACTOR, SHOTS, TWO_LEG_FACTOR, ZERO,
};
use crate::{C, FloatType};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, CircuitBox, CircuitModel, Experiment, FactorSupports, Hypothesis, Observable,
    ProcessFactors, Projection, QuantumPlant, QubitOperator, WireType,
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

/// A single-qubit factor of the v1 family.
fn qubit_factor() -> Result<CausalTensor<C>, ModelBuildError> {
    diagonal(&QUBIT_FACTOR)
}

/// A factor on a qubit and one parent, of the v1 family.
fn two_leg_factor() -> Result<CausalTensor<C>, ModelBuildError> {
    diagonal(&TWO_LEG_FACTOR)
}

/// `H₃` as the v1 factorization: a common bath drives both qubits. The circuit form needs a
/// two-output bath node whose dilation exceeds the entry cap; see the module documentation.
pub fn h3_common_bath() -> Result<Hypothesis<FloatType>, ModelBuildError> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();

    for (node, pa) in [(BATH, None), (Q1, Some(BATH)), (Q2, Some(BATH))] {
        let mut legs: Vec<usize> = pa.into_iter().collect();
        legs.push(node);

        let factor = if pa.is_none() {
            qubit_factor()?
        } else {
            two_leg_factor()?
        };

        factors.insert(node, factor);
        supports.declare(node, &legs);
    }

    Hypothesis::structural("H3 Q1<-B->Q2", factors, supports)
        .map_err(|_| ModelBuildError::Factorization("H3"))
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

/// The experiment family with the predicted read-out under `H₁`, `H₂`, `H₃` in that order, the v1
/// example's Table §4.
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
    /// An experiment's predictions are not probabilities.
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
                write!(f, "the experiment {name} does not predict probabilities")
            }
        }
    }
}

impl core::error::Error for ModelBuildError {}
