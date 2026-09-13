/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The three query constructions of Lorenz & Tull §7.2 as rewired circuits: opening deletes a
//! node's boxes and hands its wires in as fresh inputs, observing appends computational-basis
//! measurements, and interchanging runs a second copy of the model up to the named nodes and swaps
//! its state onto the main lines where those nodes would have acted. Each returns a new model, so
//! the numeric semantics of a query is the numeric semantics of the rewired model.

use crate::QuantumError;
use crate::types::abstraction::fault_set::Fault;
use crate::types::carriers::Channel;
use crate::types::circuit_model::circuit_box::CircuitBox;
use crate::types::circuit_model::model::CircuitModel;
use crate::types::circuit_model::wire::{BoxId, NodeId, WireId, WireType};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// A box with its quantum wires renamed through `f`; classical wires through `g`.
fn rewire<R: RealField>(
    bx: &CircuitBox<R>,
    f: &dyn Fn(WireId) -> WireId,
    g: &dyn Fn(WireId) -> WireId,
) -> CircuitBox<R> {
    match bx {
        CircuitBox::Encoder {
            input,
            outputs,
            states,
        } => CircuitBox::Encoder {
            input: g(*input),
            outputs: outputs.iter().map(|&w| f(w)).collect(),
            states: states.clone(),
        },
        CircuitBox::Unitary { wires, program } => CircuitBox::Unitary {
            wires: wires.iter().map(|&w| f(w)).collect(),
            program: program.clone(),
        },
        CircuitBox::Channel { wires, channel } => CircuitBox::Channel {
            wires: wires.iter().map(|&w| f(w)).collect(),
            channel: channel.clone(),
        },
        CircuitBox::Kraus { wires, kraus } => CircuitBox::Kraus {
            wires: wires.iter().map(|&w| f(w)).collect(),
            kraus: kraus.clone(),
        },
        CircuitBox::Instrument {
            wires,
            outcome,
            kraus,
        } => CircuitBox::Instrument {
            wires: wires.iter().map(|&w| f(w)).collect(),
            outcome: g(*outcome),
            kraus: kraus.clone(),
        },
        CircuitBox::Measurement { wires, outcome } => CircuitBox::Measurement {
            wires: wires.iter().map(|&w| f(w)).collect(),
            outcome: g(*outcome),
        },
    }
}

impl<R> CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The model opened at `nodes` (§7.2, `Open(S)`): their boxes are deleted, each of their
    /// quantum wires is cut at that point and continues as a fresh declared input, the parent's
    /// output on the cut wire is discarded, and any classical wire they wrote disappears.
    ///
    /// The fresh wires are appended in the order the opened wires are first met, so a caller can
    /// read them back through [`opened_inputs`](Self::opened_inputs).
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a node is out of range; the errors of
    /// [`new`](Self::new) on the rewired model, which include a classical output no box writes
    /// any more.
    pub fn opened(&self, nodes: &[NodeId]) -> Result<Self, QuantumError> {
        self.opened_with_map(nodes).map(|(m, _)| m)
    }

    /// [`opened`](Self::opened) together with the renaming it performed: `(original wire, fresh
    /// input)` for every cut wire, in the order of first encounter.
    ///
    /// # Errors
    ///
    /// As [`opened`](Self::opened).
    pub fn opened_with_map(
        &self,
        nodes: &[NodeId],
    ) -> Result<(Self, Vec<(WireId, WireId)>), QuantumError> {
        for &n in nodes {
            if n >= self.nodes().len() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "node {n} does not exist: the model has {} nodes",
                    self.nodes().len()
                )));
            }
        }
        let deleted: BTreeSet<BoxId> = self.boxes_of_nodes(nodes).into_iter().collect();
        let mut wires = self.wires().to_vec();
        // Current name of each original wire: a cut wire is renamed to its fresh continuation.
        let mut current: Vec<WireId> = (0..wires.len()).collect();
        let mut fresh_inputs: Vec<WireId> = Vec::new();
        let mut renamed: Vec<(WireId, WireId)> = Vec::new();
        let mut boxes: Vec<CircuitBox<R>> = Vec::new();
        let mut kept_box_node: Vec<NodeId> = Vec::new();
        for (b, bx) in self.boxes().iter().enumerate() {
            if deleted.contains(&b) {
                for &w in bx.quantum_wires() {
                    // Cut once per original wire: the first deleted box to touch it opens it.
                    if current[w] == w {
                        wires.push(wires[w]);
                        let fresh = wires.len() - 1;
                        current[w] = fresh;
                        fresh_inputs.push(fresh);
                        renamed.push((w, fresh));
                    }
                }
                continue;
            }
            let cur = current.clone();
            boxes.push(rewire(bx, &|w| cur[w], &|w| w));
            kept_box_node.push(self.node_of(b).expect("validated"));
        }
        // Nodes: the kept boxes keep their grouping, renumbered densely.
        let mut node_map: Vec<Option<NodeId>> = vec![None; self.nodes().len()];
        let mut new_nodes: Vec<Vec<BoxId>> = Vec::new();
        for (nb, &old_node) in kept_box_node.iter().enumerate() {
            let target = match node_map[old_node] {
                Some(t) => t,
                None => {
                    new_nodes.push(Vec::new());
                    node_map[old_node] = Some(new_nodes.len() - 1);
                    new_nodes.len() - 1
                }
            };
            new_nodes[target].push(nb);
        }
        let mut inputs: Vec<WireId> = self.inputs().to_vec();
        inputs.extend(&fresh_inputs);
        let written: BTreeSet<WireId> = boxes.iter().filter_map(|b| b.classical_write()).collect();
        let outputs: Vec<WireId> = self
            .outputs()
            .iter()
            .filter_map(|&w| {
                if wires[w].is_quantum() {
                    Some(current[w])
                } else if written.contains(&w) {
                    Some(w)
                } else {
                    None
                }
            })
            .collect();
        Self::new(wires, boxes, new_nodes, inputs, outputs).map(|m| (m, renamed))
    }

    /// The fresh input wires an opening at `nodes` would create, in order: one per quantum wire
    /// of the opened boxes, numbered from the current wire count.
    pub fn opened_inputs(&self, nodes: &[NodeId]) -> Vec<WireId> {
        let deleted: BTreeSet<BoxId> = self.boxes_of_nodes(nodes).into_iter().collect();
        let mut seen: BTreeSet<WireId> = BTreeSet::new();
        let mut count = 0usize;
        let mut out = Vec::new();
        for (b, bx) in self.boxes().iter().enumerate() {
            if !deleted.contains(&b) {
                continue;
            }
            for &w in bx.quantum_wires() {
                if seen.insert(w) {
                    out.push(self.wires().len() + count);
                    count += 1;
                }
            }
        }
        out
    }

    /// The model with the quantum wires `observed` measured in the computational basis at the end
    /// (§7.2, `Observe(O)`): one measurement box and one classical outcome wire per observed wire,
    /// the outcome wires replacing the observed wires among the outputs.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a wire is not a quantum wire of the model or not
    /// one of its declared outputs.
    pub fn observed(&self, observed: &[WireId]) -> Result<Self, QuantumError> {
        let mut wires = self.wires().to_vec();
        let mut boxes = self.boxes().to_vec();
        let mut nodes = self.nodes().to_vec();
        let mut outputs: Vec<WireId> = self.outputs().to_vec();
        for &w in observed {
            match self.wires().get(w) {
                Some(t) if t.is_quantum() => {}
                _ => {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "wire {w} is not a quantum wire of the model"
                    )));
                }
            }
            if !self.outputs().contains(&w) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "wire {w} is not a declared output of the model; Observe measures output wires"
                )));
            }
            wires.push(WireType::Classical {
                outcomes: self.wires()[w].cardinality(),
            });
            let outcome = wires.len() - 1;
            boxes.push(CircuitBox::Measurement {
                wires: vec![w],
                outcome,
            });
            nodes.push(vec![boxes.len() - 1]);
            outputs.retain(|&o| o != w);
            outputs.push(outcome);
        }
        Self::new(wires, boxes, nodes, self.inputs().to_vec(), outputs)
    }

    /// The faulted model: the fault's Pauli program as a unitary box on its wires, inserted after
    /// the last box of the named node, or before the first box when no node is named, as a node of
    /// its own. Wires, inputs and outputs are unchanged.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a fault wire is not a quantum wire or the node does
    /// not exist; the errors of [`new`](Self::new).
    pub fn faulted(&self, fault: &Fault) -> Result<Self, QuantumError> {
        for w in fault.wires() {
            match self.wires().get(w) {
                Some(t) if t.is_quantum() => {}
                _ => {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "fault wire {w} is not a quantum wire of the model"
                    )));
                }
            }
        }
        let position = match fault.after() {
            None => 0,
            Some(n) => {
                let members = self.nodes().get(n).ok_or_else(|| {
                    QuantumError::DimensionMismatch(format!(
                        "the fault follows node {n}, but the model has {} nodes",
                        self.nodes().len()
                    ))
                })?;
                members.iter().copied().max().map_or(0, |b| b + 1)
            }
        };
        let mut boxes = self.boxes().to_vec();
        boxes.insert(
            position,
            CircuitBox::Unitary {
                wires: fault.wires(),
                program: fault.program(),
            },
        );
        let mut nodes: Vec<Vec<BoxId>> = self
            .nodes()
            .iter()
            .map(|members| {
                members
                    .iter()
                    .map(|&b| if b >= position { b + 1 } else { b })
                    .collect()
            })
            .collect();
        nodes.push(vec![position]);
        Self::new(
            self.wires().to_vec(),
            boxes,
            nodes,
            self.inputs().to_vec(),
            self.outputs().to_vec(),
        )
    }

    /// The interchange model (§7.2, `Inc(S₁, …, Sₙ)`): for each set, a copy of every box that is an
    /// ancestor of, or in, its nodes runs on fresh wires from fresh copies of the model's inputs,
    /// and in the main circuit the set's boxes are replaced by swaps that move the copy's state onto
    /// the main lines. The result's inputs are the model's followed by one copy of them per set.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NotParallelisable`] if two nodes of one set are joined by a directed path;
    /// [`QuantumError::DimensionMismatch`] on a node out of range, a node in two sets, or a node
    /// whose box writes a classical wire, since the copy would write its private outcome wire and
    /// the main wire would stay unwritten; the errors of [`new`](Self::new).
    pub fn interchanged(&self, sets: &[Vec<NodeId>]) -> Result<Self, QuantumError> {
        self.interchanged_with_map(sets).map(|(m, _)| m)
    }

    /// [`interchanged`](Self::interchanged) together with the renaming of the model's inputs to
    /// their copies: `(original input, copy)` for each set in order.
    ///
    /// # Errors
    ///
    /// As [`interchanged`](Self::interchanged).
    pub fn interchanged_with_map(
        &self,
        sets: &[Vec<NodeId>],
    ) -> Result<(Self, Vec<(WireId, WireId)>), QuantumError> {
        let dag = self.induced_dag();
        let mut named: BTreeSet<NodeId> = BTreeSet::new();
        for (i, set) in sets.iter().enumerate() {
            for &n in set {
                if n >= self.nodes().len() {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "node {n} does not exist: the model has {} nodes",
                        self.nodes().len()
                    )));
                }
                if !named.insert(n) {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "node {n} appears in two interchange sets"
                    )));
                }
                for &b in &self.nodes()[n] {
                    if let Some(w) = self.boxes()[b].classical_write() {
                        return Err(QuantumError::DimensionMismatch(format!(
                            "node {n} holds box {b} ({}), which writes classical wire {w}; an \
                             interchange set holds quantum-only nodes",
                            self.boxes()[b].kind()
                        )));
                    }
                }
            }
            for &a in set {
                for &b in set {
                    if a != b && dag.reaches(a, b) {
                        return Err(QuantumError::NotParallelisable(i, a, b));
                    }
                }
            }
        }
        let mut wires = self.wires().to_vec();
        let mut boxes: Vec<CircuitBox<R>> = Vec::new();
        let mut nodes: Vec<Vec<BoxId>> = Vec::new();
        let mut inputs: Vec<WireId> = self.inputs().to_vec();
        let mut renamed: Vec<(WireId, WireId)> = Vec::new();
        // Swaps to insert at the position of a replaced box: (main wire, copy wire). One swap per
        // wire of a replaced node, placed at the node's last box on that wire.
        let mut swaps_at: Vec<Vec<(WireId, WireId)>> = vec![Vec::new(); self.boxes().len()];
        let mut replaced: BTreeSet<BoxId> = BTreeSet::new();
        for set in sets {
            let members: BTreeSet<NodeId> = set.iter().copied().collect();
            let prefix: BTreeSet<NodeId> = (0..self.nodes().len())
                .filter(|&n| members.contains(&n) || members.iter().any(|&m| dag.reaches(n, m)))
                .collect();
            // Fresh wire per original wire the prefix touches, including classical ones.
            let mut copy: Vec<Option<WireId>> = vec![None; self.wires().len()];
            let mut name = |w: WireId, wires: &mut Vec<WireType>| -> WireId {
                if let Some(c) = copy[w] {
                    return c;
                }
                wires.push(wires[w]);
                copy[w] = Some(wires.len() - 1);
                wires.len() - 1
            };
            let mut copy_boxes: Vec<(BoxId, CircuitBox<R>)> = Vec::new();
            // Per (node, wire): the last box of the node on that wire and the wire's copy.
            let mut last_on_wire: BTreeMap<(NodeId, WireId), (BoxId, WireId)> = BTreeMap::new();
            for (b, bx) in self.boxes().iter().enumerate() {
                let node = self.node_of(b).expect("validated");
                if !prefix.contains(&node) {
                    continue;
                }
                let mut named: Vec<(WireId, WireId)> = Vec::new();
                for &w in bx.quantum_wires() {
                    let c = name(w, &mut wires);
                    named.push((w, c));
                }
                for w in bx.classical_read().into_iter().chain(bx.classical_write()) {
                    let c = name(w, &mut wires);
                    named.push((w, c));
                }
                let lookup = |w: WireId| {
                    named
                        .iter()
                        .find(|(o, _)| *o == w)
                        .map(|(_, c)| *c)
                        .unwrap_or(w)
                };
                copy_boxes.push((b, rewire(bx, &lookup, &lookup)));
                if members.contains(&node) {
                    replaced.insert(b);
                    for &w in bx.quantum_wires() {
                        last_on_wire.insert((node, w), (b, lookup(w)));
                    }
                }
            }
            for ((_, w), (b, c)) in last_on_wire {
                swaps_at[b].push((w, c));
            }
            for &w in self.inputs() {
                if let Some(c) = copy[w] {
                    inputs.push(c);
                    renamed.push((w, c));
                }
            }
            // The copy's boxes go first, each its own node.
            for (_, bx) in copy_boxes {
                boxes.push(bx);
                nodes.push(vec![boxes.len() - 1]);
            }
        }
        // The main circuit, with the replaced boxes turned into swaps.
        let mut node_map: Vec<Option<NodeId>> = vec![None; self.nodes().len()];
        for (b, bx) in self.boxes().iter().enumerate() {
            if replaced.contains(&b) {
                for &(main, copy) in &swaps_at[b] {
                    let d = self.wires()[main].cardinality();
                    boxes.push(CircuitBox::Channel {
                        wires: vec![main, copy],
                        channel: swap_channel::<R>(d)?,
                    });
                    nodes.push(vec![boxes.len() - 1]);
                }
                continue;
            }
            boxes.push(bx.clone());
            let old_node = self.node_of(b).expect("validated");
            let target = match node_map[old_node] {
                Some(t) => t,
                None => {
                    nodes.push(Vec::new());
                    node_map[old_node] = Some(nodes.len() - 1);
                    nodes.len() - 1
                }
            };
            nodes[target].push(boxes.len() - 1);
        }
        Self::new(wires, boxes, nodes, inputs, self.outputs().to_vec()).map(|m| (m, renamed))
    }
}

/// The swap of two `d`-dimensional systems as a channel.
pub fn swap_channel<R>(d: usize) -> Result<Channel<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let n = d * d;
    let mut data = vec![Complex::new(R::zero(), R::zero()); n * n];
    for a in 0..d {
        for b in 0..d {
            data[(b * d + a) * n + (a * d + b)] = Complex::new(R::one(), R::zero());
        }
    }
    Channel::from_kraus(&[CausalTensor::from_slice(&data, &[n, n])])
}
