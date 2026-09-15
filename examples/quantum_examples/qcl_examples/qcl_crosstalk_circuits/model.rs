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

use crate::constants::{BATH, COUPLING_ANGLE, OWN_ANGLE, Q1, Q2, SHOTS};
use crate::{C, FloatType};
use deep_causality_num::lift;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, CircuitBox, CircuitModel, Experiment, FactorSupports, Hypothesis, Observable,
    ProcessFactors, Projection, QuantumPlant, QubitOperator, WireType,
};
use deep_causality_tensor::CausalTensor;

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

/// A single-qubit rotation as a one-operator Kraus box on one wire.
fn rotation(wire: usize, angle: f64) -> CircuitBox<FloatType> {
    CircuitBox::Kraus {
        wires: vec![wire],
        kraus: vec![
            QubitOperator::<FloatType>::rotation(Axis::Y, lift(angle))
                .expect("an angle")
                .matrix()
                .clone(),
        ],
    }
}

/// `H₁` as a circuit: `Q1`'s box first, then `Q2`'s box on the wire it hands on, so the induced
/// DAG is `Q1 → Q2`. Node 0 is `Q1`, node 1 is `Q2`.
pub fn h1_circuit() -> CircuitModel<FloatType> {
    CircuitModel::new(
        vec![WireType::qubit()],
        vec![rotation(0, COUPLING_ANGLE), rotation(0, OWN_ANGLE)],
        vec![vec![0], vec![1]],
        vec![],
        vec![0],
    )
    .expect("a two-node chain")
}

/// `H₂` as a circuit: the same two boxes with `Q2`'s first, grouped so that node 0 is still `Q1`
/// and node 1 still `Q2`; the induced DAG is `Q2 → Q1`.
pub fn h2_circuit() -> CircuitModel<FloatType> {
    CircuitModel::new(
        vec![WireType::qubit()],
        vec![rotation(0, COUPLING_ANGLE), rotation(0, OWN_ANGLE)],
        vec![vec![1], vec![0]],
        vec![],
        vec![0],
    )
    .expect("a two-node chain")
}

/// `H₄` as a circuit: four boxes on one wire grouped as `Q1 → Q2 → B → Q1`, a cycle in the
/// induced DAG, which `build()` refuses.
pub fn h4_cyclic_circuit() -> CircuitModel<FloatType> {
    CircuitModel::new(
        vec![WireType::qubit()],
        vec![
            rotation(0, OWN_ANGLE),
            rotation(0, COUPLING_ANGLE),
            rotation(0, OWN_ANGLE),
            rotation(0, COUPLING_ANGLE),
        ],
        vec![vec![0, 3], vec![1], vec![2]],
        vec![],
        vec![0],
    )
    .expect("the grouping is a partition; the cycle is found at build()")
}

/// A single-qubit factor of the v1 family.
fn qubit_factor() -> CausalTensor<C> {
    diagonal(&[0.9, 0.1])
}

/// A factor on a qubit and one parent, of the v1 family.
fn two_leg_factor() -> CausalTensor<C> {
    diagonal(&[0.85, 0.05, 0.05, 0.05])
}

/// `H₃` as the v1 factorization: a common bath drives both qubits. The circuit form needs a
/// two-output bath node whose dilation exceeds the entry cap; see the module documentation.
pub fn h3_common_bath() -> Hypothesis<FloatType> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();
    for (node, pa) in [(BATH, None), (Q1, Some(BATH)), (Q2, Some(BATH))] {
        let mut legs: Vec<usize> = pa.into_iter().collect();
        legs.push(node);
        factors.insert(
            node,
            if pa.is_none() {
                qubit_factor()
            } else {
                two_leg_factor()
            },
        );
        supports.declare(node, &legs);
    }
    Hypothesis::structural("H3 Q1<-B->Q2", factors, supports).expect("a validated factorization")
}

/// The systems the v1 decomposability check is stated over.
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

/// The experiment family with the predicted read-out under `H₁`, `H₂`, `H₃` in that order, the v1
/// example's Table §4.
pub fn experiments() -> Vec<Experiment<FloatType>> {
    let exp = |name: &str, cost: f64, predictions: [f64; 3]| {
        Experiment::new(
            name,
            lift(cost),
            SHOTS,
            predictions.map(lift::<FloatType>).to_vec(),
        )
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
