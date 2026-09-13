/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A compositional model in QC: typed wires, boxes, a grouping of boxes into nodes, and declared
//! inputs and outputs. The induced open DAG is derived from it and never stored beside it.
//!
//! Lorenz & Tull, *Causal and Compositional Abstraction*, arXiv:2602.16612: a compositional model
//! is a signature of boxes and wires with a semantics functor into a category (Definition 7); a
//! quantum circuit with encoders, unitaries and measurements is one such model in QC (Examples 58
//! and 61), and grouping its gates into the nodes of a coarser DAG is a strict component-level
//! abstraction (Example 63). This type is the signature and the grouping; the semantics functors
//! are `numeric_semantics` and `exact_program`.

use crate::QuantumError;
use crate::types::carriers::Channel;
use crate::types::circuit_model::circuit_box::CircuitBox;
use crate::types::circuit_model::induced_dag::InducedDag;
use crate::types::circuit_model::wire::{BoxId, NodeId, WireId, WireType};
use crate::types::decision::Tolerance;
use crate::types::qgates::operator_linalg::{frobenius_norm, identity_matrix};
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::{CausalTensor, Tensor};

/// A compositional model in QC. See the module documentation.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitModel<R: RealField> {
    wires: Vec<WireType>,
    boxes: Vec<CircuitBox<R>>,
    nodes: Vec<Vec<BoxId>>,
    box_node: Vec<NodeId>,
    inputs: Vec<WireId>,
    outputs: Vec<WireId>,
}

impl<R> CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// A model from its wires, its boxes in application order, a grouping of the boxes into nodes,
    /// the quantum wires whose initial state is free (the model's quantum inputs; every other
    /// quantum wire starts in `|0⟩`), and the wires kept as outputs (quantum wires not listed are
    /// discarded at the end).
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] naming the wire and the box when a box names a wire
    /// outside the model, of the wrong kind, or of a dimension that disagrees with the box's
    /// operators, when a box names a wire twice, when two boxes write one classical wire, or when a
    /// declared input is prepared by an encoder. [`QuantumError::CalculationError`] naming the box
    /// when the grouping is not a partition of the boxes, or when an instrument is not jointly
    /// trace-preserving.
    pub fn new(
        wires: Vec<WireType>,
        boxes: Vec<CircuitBox<R>>,
        nodes: Vec<Vec<BoxId>>,
        inputs: Vec<WireId>,
        outputs: Vec<WireId>,
    ) -> Result<Self, QuantumError> {
        for (id, w) in wires.iter().enumerate() {
            w.validate(id)?;
        }
        let mut classical_writer: Vec<Option<BoxId>> = vec![None; wires.len()];
        let mut encoded: BTreeSet<WireId> = BTreeSet::new();
        // The first box to touch each quantum wire: an encoder prepares fresh lines, so a wire an
        // earlier box touched cannot be one of its outputs.
        let mut first_touch: Vec<Option<BoxId>> = vec![None; wires.len()];
        for (b, bx) in boxes.iter().enumerate() {
            check_box(&wires, b, bx)?;
            if let CircuitBox::Encoder { outputs, .. } = bx
                && let Some((w, first)) = outputs
                    .iter()
                    .find_map(|&w| first_touch.get(w).copied().flatten().map(|f| (w, f)))
            {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (encoder) prepares wire {w}, which box {first} touched before it; an \
                     encoder prepares fresh lines"
                )));
            }
            for &w in bx.quantum_wires() {
                if first_touch[w].is_none() {
                    first_touch[w] = Some(b);
                }
            }
            if let Some(w) = bx.classical_write() {
                if let Some(first) = classical_writer[w] {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "classical wire {w} is written by both box {first} and box {b}"
                    )));
                }
                classical_writer[w] = Some(b);
            }
            if let CircuitBox::Encoder { outputs, .. } = bx {
                encoded.extend(outputs.iter().copied());
            }
        }
        for &w in &inputs {
            match wires.get(w) {
                Some(t) if t.is_quantum() => {}
                _ => {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "declared input {w} is not a quantum wire of the model"
                    )));
                }
            }
            if encoded.contains(&w) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "declared input {w} is prepared by an encoder and cannot be free"
                )));
            }
        }
        let input_set: BTreeSet<WireId> = inputs.iter().copied().collect();
        if input_set.len() != inputs.len() {
            return Err(QuantumError::DimensionMismatch(
                "a wire is declared as input twice".into(),
            ));
        }
        let mut seen_out = BTreeSet::new();
        for &w in &outputs {
            let Some(t) = wires.get(w) else {
                return Err(QuantumError::DimensionMismatch(format!(
                    "declared output {w} is not a wire of the model"
                )));
            };
            if !t.is_quantum() && classical_writer[w].is_none() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "declared classical output {w} is written by no box"
                )));
            }
            if !seen_out.insert(w) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "wire {w} is declared as output twice"
                )));
            }
        }
        let mut box_node = vec![usize::MAX; boxes.len()];
        for (n, members) in nodes.iter().enumerate() {
            if members.is_empty() {
                return Err(QuantumError::CalculationError(format!(
                    "node {n} of the grouping holds no box"
                )));
            }
            for &b in members {
                if b >= boxes.len() {
                    return Err(QuantumError::CalculationError(format!(
                        "node {n} names box {b}, but the model has {} boxes",
                        boxes.len()
                    )));
                }
                if box_node[b] != usize::MAX {
                    return Err(QuantumError::CalculationError(format!(
                        "box {b} is grouped into node {} and node {n}",
                        box_node[b]
                    )));
                }
                box_node[b] = n;
            }
        }
        if let Some(b) = box_node.iter().position(|&n| n == usize::MAX) {
            return Err(QuantumError::CalculationError(format!(
                "box {b} is grouped into no node"
            )));
        }
        Ok(Self {
            wires,
            boxes,
            nodes,
            box_node,
            inputs,
            outputs,
        })
    }

    /// A model with every box its own node, the finest grouping.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new).
    pub fn ungrouped(
        wires: Vec<WireType>,
        boxes: Vec<CircuitBox<R>>,
        inputs: Vec<WireId>,
        outputs: Vec<WireId>,
    ) -> Result<Self, QuantumError> {
        let nodes = (0..boxes.len()).map(|b| vec![b]).collect();
        Self::new(wires, boxes, nodes, inputs, outputs)
    }
}

impl<R> CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// Two models glued along wires: `joins` pairs an output wire of `self` with a declared input
    /// wire of `other`, so `other`'s boxes follow `self`'s on a shared line. The result's inputs are
    /// `self`'s and `other`'s unjoined inputs; its outputs are `self`'s unjoined outputs and
    /// `other`'s. A composite of two circuits is a circuit, and its dilation is the induced
    /// factorization a composite of two marginals could not construct.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a joined wire is not a quantum output of `self` or a
    /// quantum declared input of `other`, if the two wires' types differ, or if a wire is joined
    /// twice; and the errors of [`new`](Self::new) on the composite.
    pub fn glue(&self, other: &Self, joins: &[(WireId, WireId)]) -> Result<Self, QuantumError> {
        let mut map: Vec<Option<WireId>> = vec![None; other.wires.len()];
        for &(mine, theirs) in joins {
            if !self.outputs.contains(&mine)
                || !self.wires.get(mine).is_some_and(WireType::is_quantum)
            {
                return Err(QuantumError::DimensionMismatch(format!(
                    "wire {mine} is not a quantum output of the first model"
                )));
            }
            if !other.inputs.contains(&theirs) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "wire {theirs} is not a declared input of the second model"
                )));
            }
            if self.wires[mine] != other.wires[theirs] {
                return Err(QuantumError::DimensionMismatch(format!(
                    "wires {mine} and {theirs} have different types and cannot be joined"
                )));
            }
            if map[theirs].is_some() || joins.iter().filter(|(m, _)| *m == mine).count() > 1 {
                return Err(QuantumError::DimensionMismatch(format!(
                    "wire {mine} or {theirs} is joined more than once"
                )));
            }
            map[theirs] = Some(mine);
        }
        let mut wires = self.wires.clone();
        for (w, slot) in map.iter_mut().enumerate() {
            if slot.is_none() {
                wires.push(other.wires[w]);
                *slot = Some(wires.len() - 1);
            }
        }
        let m = |w: WireId| map[w].expect("every wire mapped");
        let mut boxes = self.boxes.clone();
        for bx in &other.boxes {
            boxes.push(match bx {
                CircuitBox::Encoder {
                    input,
                    outputs,
                    states,
                } => CircuitBox::Encoder {
                    input: m(*input),
                    outputs: outputs.iter().map(|&w| m(w)).collect(),
                    states: states.clone(),
                },
                CircuitBox::Unitary { wires, program } => CircuitBox::Unitary {
                    wires: wires.iter().map(|&w| m(w)).collect(),
                    program: program.clone(),
                },
                CircuitBox::Channel { wires, channel } => CircuitBox::Channel {
                    wires: wires.iter().map(|&w| m(w)).collect(),
                    channel: channel.clone(),
                },
                CircuitBox::Kraus { wires, kraus } => CircuitBox::Kraus {
                    wires: wires.iter().map(|&w| m(w)).collect(),
                    kraus: kraus.clone(),
                },
                CircuitBox::Instrument {
                    wires,
                    outcome,
                    kraus,
                } => CircuitBox::Instrument {
                    wires: wires.iter().map(|&w| m(w)).collect(),
                    outcome: m(*outcome),
                    kraus: kraus.clone(),
                },
                CircuitBox::Measurement { wires, outcome } => CircuitBox::Measurement {
                    wires: wires.iter().map(|&w| m(w)).collect(),
                    outcome: m(*outcome),
                },
            });
        }
        let offset = self.boxes.len();
        let mut nodes = self.nodes.clone();
        nodes.extend(
            other
                .nodes
                .iter()
                .map(|members| members.iter().map(|&b| b + offset).collect::<Vec<_>>()),
        );
        let joined_mine: BTreeSet<WireId> = joins.iter().map(|(a, _)| *a).collect();
        let joined_theirs: BTreeSet<WireId> = joins.iter().map(|(_, b)| *b).collect();
        let mut inputs = self.inputs.clone();
        inputs.extend(
            other
                .inputs
                .iter()
                .filter(|w| !joined_theirs.contains(w))
                .map(|&w| m(w)),
        );
        let mut outputs: Vec<WireId> = self
            .outputs
            .iter()
            .copied()
            .filter(|w| !joined_mine.contains(w))
            .collect();
        outputs.extend(other.outputs.iter().map(|&w| m(w)));
        Self::new(wires, boxes, nodes, inputs, outputs)
    }
}

impl<R: RealField> CircuitModel<R> {
    /// The wires.
    pub fn wires(&self) -> &[WireType] {
        &self.wires
    }

    /// The boxes, in application order.
    pub fn boxes(&self) -> &[CircuitBox<R>] {
        &self.boxes
    }

    /// The grouping: the boxes of each node.
    pub fn nodes(&self) -> &[Vec<BoxId>] {
        &self.nodes
    }

    /// The node a box belongs to.
    pub fn node_of(&self, b: BoxId) -> Option<NodeId> {
        self.box_node.get(b).copied()
    }

    /// The declared quantum inputs.
    pub fn inputs(&self) -> &[WireId] {
        &self.inputs
    }

    /// The declared outputs.
    pub fn outputs(&self) -> &[WireId] {
        &self.outputs
    }

    /// The quantum wires, ascending.
    pub fn quantum_wires(&self) -> Vec<WireId> {
        (0..self.wires.len())
            .filter(|&w| self.wires[w].is_quantum())
            .collect()
    }

    /// The number of quantum wires.
    pub fn num_qubits(&self) -> usize {
        self.quantum_wires().len()
    }

    /// The classical wires read by some box and written by none: the model's classical inputs.
    pub fn classical_inputs(&self) -> Vec<WireId> {
        let written: BTreeSet<WireId> = self
            .boxes
            .iter()
            .filter_map(|b| b.classical_write())
            .collect();
        let mut reads: Vec<WireId> = self
            .boxes
            .iter()
            .filter_map(|b| b.classical_read())
            .filter(|w| !written.contains(w))
            .collect();
        reads.sort_unstable();
        reads.dedup();
        reads
    }

    /// The induced open DAG on the nodes (Example 61): an edge from node `a` to node `b` whenever a
    /// box of `b` is the next box to touch a quantum wire after a box of `a`, or reads a classical
    /// wire a box of `a` wrote. Derived on each call.
    pub fn induced_dag(&self) -> InducedDag {
        let mut dag = InducedDag::new(self.nodes.len());
        let mut last: Vec<Option<BoxId>> = vec![None; self.wires.len()];
        for (b, bx) in self.boxes.iter().enumerate() {
            let node_b = self.box_node[b];
            for &w in bx.quantum_wires() {
                if let Some(a) = last[w] {
                    let node_a = self.box_node[a];
                    if node_a != node_b {
                        dag.add_edge(node_a, node_b);
                    }
                }
                last[w] = Some(b);
            }
            if let Some(w) = bx.classical_read()
                && let Some(a) = last[w]
                && self.box_node[a] != node_b
            {
                dag.add_edge(self.box_node[a], node_b);
            }
            if let Some(w) = bx.classical_write() {
                last[w] = Some(b);
            }
        }
        dag
    }

    /// The boxes of the named nodes, in application order, for opening queries.
    pub fn boxes_of_nodes(&self, nodes: &[NodeId]) -> Vec<BoxId> {
        let mut out: Vec<BoxId> = nodes
            .iter()
            .filter_map(|&n| self.nodes.get(n))
            .flatten()
            .copied()
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The nodes whose boxes touch a given wire.
    pub fn nodes_on_wire(&self, w: WireId) -> Vec<NodeId> {
        let mut out: Vec<NodeId> = self
            .boxes
            .iter()
            .enumerate()
            .filter(|(_, bx)| {
                bx.quantum_wires().contains(&w)
                    || bx.classical_read() == Some(w)
                    || bx.classical_write() == Some(w)
            })
            .map(|(b, _)| self.box_node[b])
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// One box against the wires it names.
fn check_box<R>(wires: &[WireType], b: BoxId, bx: &CircuitBox<R>) -> Result<(), QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let quantum_dim = |ids: &[WireId]| -> Result<usize, QuantumError> {
        let mut seen = BTreeSet::new();
        let mut dim = 1usize;
        for &w in ids {
            let Some(t) = wires.get(w) else {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} ({}) names wire {w}, but the model has {} wires",
                    bx.kind(),
                    wires.len()
                )));
            };
            if !t.is_quantum() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} ({}) names wire {w} as quantum, but it is classical",
                    bx.kind()
                )));
            }
            if !seen.insert(w) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} ({}) names wire {w} more than once",
                    bx.kind()
                )));
            }
            dim = dim.checked_mul(t.cardinality()).ok_or_else(|| {
                QuantumError::DimensionMismatch(format!(
                    "box {b}: the wire dimensions overflow usize"
                ))
            })?;
        }
        Ok(dim)
    };
    let classical = |w: WireId| -> Result<usize, QuantumError> {
        match wires.get(w) {
            Some(WireType::Classical { outcomes }) => Ok(*outcomes),
            Some(_) => Err(QuantumError::DimensionMismatch(format!(
                "box {b} ({}) names wire {w} as classical, but it is quantum",
                bx.kind()
            ))),
            None => Err(QuantumError::DimensionMismatch(format!(
                "box {b} ({}) names wire {w}, but the model has {} wires",
                bx.kind(),
                wires.len()
            ))),
        }
    };
    match bx {
        CircuitBox::Encoder {
            input,
            outputs,
            states,
        } => {
            let d = quantum_dim(outputs)?;
            let outcomes = classical(*input)?;
            if states.len() != outcomes {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (encoder) prepares {} states but wire {input} has {outcomes} values",
                    states.len()
                )));
            }
            if let Some(bad) = states.iter().position(|s| s.as_slice().len() != d) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (encoder): state {bad} has {} amplitudes, its wires {outputs:?} imply {d}",
                    states[bad].as_slice().len()
                )));
            }
            let tol = Tolerance::<R>::state()
                .threshold(d, R::one())
                .unwrap_or_else(|| R::epsilon().sqrt());
            for (i, s) in states.iter().enumerate() {
                if s.as_slice()
                    .iter()
                    .any(|a| !a.re.is_finite() || !a.im.is_finite())
                {
                    return Err(QuantumError::NonFiniteValue(format!(
                        "box {b} (encoder): state {i} has a non-finite amplitude"
                    )));
                }
                let norm = s
                    .as_slice()
                    .iter()
                    .fold(R::zero(), |acc, a| acc + a.re * a.re + a.im * a.im)
                    .sqrt();
                if (norm - R::one()).abs() > tol {
                    return Err(QuantumError::NormalizationError(format!(
                        "box {b} (encoder): state {i} has norm {norm:?}, not one within {tol:?}"
                    )));
                }
            }
        }
        CircuitBox::Unitary { wires: ws, program } => {
            quantum_dim(ws)?;
            if let Some(w) = ws.iter().find(|&&w| wires[w].cardinality() != 2) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (unitary) names wire {w} of dimension {}, but a gate program acts on qubits",
                    wires[*w].cardinality()
                )));
            }
            for op in program {
                if let Some(q) = op.qubits().into_iter().find(|&q| q >= ws.len()) {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "box {b} (unitary): gate {op:?} names local qubit {q}, but the box has {} wires",
                        ws.len()
                    )));
                }
            }
        }
        CircuitBox::Channel { wires: ws, channel } => {
            let d = quantum_dim(ws)?;
            if channel.d_in() != d || channel.d_out() != d {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (channel) is {} → {} but its wires {ws:?} imply {d}",
                    channel.d_in(),
                    channel.d_out()
                )));
            }
        }
        CircuitBox::Kraus { wires: ws, kraus } => {
            let d = quantum_dim(ws)?;
            if kraus.is_empty() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (kraus) has no Kraus operators"
                )));
            }
            if let Some(k) = kraus.iter().find(|k| k.shape() != [d, d]) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (kraus) has an operator of shape {:?} but its wires {ws:?} imply {d} × {d}",
                    k.shape()
                )));
            }
            check_trace_preserving(b, "kraus", kraus.iter(), d)?;
        }
        CircuitBox::Instrument {
            wires: ws,
            outcome,
            kraus,
        } => {
            let d = quantum_dim(ws)?;
            let outcomes = classical(*outcome)?;
            if kraus.len() != outcomes {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (instrument) has {} outcome families but wire {outcome} has {outcomes} values",
                    kraus.len()
                )));
            }
            check_trace_preserving(b, "instrument", kraus.iter().flatten(), d)?;
        }
        CircuitBox::Measurement { wires: ws, outcome } => {
            let d = quantum_dim(ws)?;
            let outcomes = classical(*outcome)?;
            if outcomes != d {
                return Err(QuantumError::DimensionMismatch(format!(
                    "box {b} (measurement) writes wire {outcome} of {outcomes} values but measures dimension {d}"
                )));
            }
        }
    }
    Ok(())
}

/// Trace preservation of a Kraus family, `Σ K†K = I` within the state tolerance, over every
/// operator of a Kraus box or every outcome family of an instrument. Non-finite entries are
/// refused first, since a non-finite defect passes no comparison.
fn check_trace_preserving<'a, R>(
    b: BoxId,
    kind: &str,
    kraus: impl Iterator<Item = &'a CausalTensor<deep_causality_num_complex::Complex<R>>>,
    d: usize,
) -> Result<(), QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug + 'a,
{
    let mut sum = identity_matrix::<R>(d) - identity_matrix::<R>(d);
    for k in kraus {
        let shape = k.shape();
        if shape.len() != 2 || shape[0] != d || shape[1] != d {
            return Err(QuantumError::DimensionMismatch(format!(
                "box {b} ({kind}): a Kraus operator has shape {shape:?}, its wires imply {d} × {d}"
            )));
        }
        if k.as_slice()
            .iter()
            .any(|a| !a.re.is_finite() || !a.im.is_finite())
        {
            return Err(QuantumError::NonFiniteValue(format!(
                "box {b} ({kind}): a Kraus operator has a non-finite entry"
            )));
        }
        let kk = k
            .dagger()
            .and_then(|kd| kd.matmul(k))
            .map_err(|e| QuantumError::CalculationError(format!("matmul: {e:?}")))?;
        sum = sum + kk;
    }
    let defect = frobenius_norm(&(sum - identity_matrix::<R>(d)));
    let tol = Tolerance::<R>::state()
        .threshold(d, R::one())
        .unwrap_or_else(|| R::epsilon().sqrt());
    if !defect.is_finite() {
        return Err(QuantumError::NonFiniteValue(format!(
            "box {b} ({kind}): ‖Σ K†K − I‖_F is not finite"
        )));
    }
    if defect > tol {
        let jointly = if kind == "instrument" { "jointly " } else { "" };
        return Err(QuantumError::CalculationError(format!(
            "box {b} ({kind}) is not {jointly}trace-preserving: ‖Σ K†K − I‖_F = {defect:?} exceeds {tol:?}"
        )));
    }
    Ok(())
}

/// An identity channel on `d` dimensions, the box a noiseless wire carries.
pub fn identity_channel<R>(d: usize) -> Result<Channel<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    Channel::from_kraus(&[identity_matrix::<R>(d)])
}
