/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The quantum causal model: a two-node causal graph and its Choi–Jamiołkowski
//! factor store. Configuration only — no execution — so the freeze scenarios in
//! `main.rs` read cleanly.

use crate::FloatType;
use crate::constants::{
    C, DETECTION_THRESHOLD, DIAGONAL_FIRST, DIAGONAL_SECOND, ONE, SHARED_LEG, SOURCE_ID,
    SOURCE_NODE, TARGET_ID, TARGET_NODE, ZERO,
};
use deep_causality::{BaseCausaloid, CausableGraph, Causaloid, CausaloidGraph, PropagatingEffect};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{FactorSupports, ProcessFactors};
use deep_causality_tensor::CausalTensor;

/// A real entry of an operator matrix.
fn real(value: FloatType) -> C {
    Complex::new(value, ZERO)
}

/// A single-qubit operator from its four entries, in row-major order.
fn operator(data: Vec<C>) -> Result<CausalTensor<C>, TopologyBuildError> {
    CausalTensor::new(data, vec![2, 2]).map_err(|_| TopologyBuildError::Operator)
}

/// Pauli-X, the bit flip.
pub fn sigma_x() -> Result<CausalTensor<C>, TopologyBuildError> {
    operator(vec![real(ZERO), real(ONE), real(ONE), real(ZERO)])
}

/// Pauli-Z, the phase flip. It does not commute with `sigma_x`: the two anticommute, so their
/// commutator is `2 i σ_y` and its norm is as far from zero as a single-qubit commutator gets.
pub fn sigma_z() -> Result<CausalTensor<C>, TopologyBuildError> {
    operator(vec![real(ONE), real(ZERO), real(ZERO), real(-ONE)])
}

/// The diagonal operator `diag(3, −1)`, which commutes with any other diagonal factor.
pub fn diagonal() -> Result<CausalTensor<C>, TopologyBuildError> {
    operator(vec![
        real(DIAGONAL_FIRST),
        real(ZERO),
        real(ZERO),
        real(DIAGONAL_SECOND),
    ])
}

/// What each node does: report whether an observation passed the detection threshold.
///
/// The node's own rule is incidental to the check this example is about — the commutativity
/// condition is a statement about the factors, not about what the nodes compute. It is written out
/// here rather than borrowed from the library's test helpers, because those are fixed at `f64` and
/// would pin the graph to one precision while the factors followed the alias.
fn passes_threshold(observation: FloatType) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(observation >= DETECTION_THRESHOLD)
}

/// One node of the graph.
fn detector(id: u64) -> BaseCausaloid<FloatType, bool> {
    Causaloid::new(
        id,
        passes_threshold,
        "reports whether an observation passed the detection threshold",
    )
}

/// A fresh two-node causal graph `0 → 1` whose nodes each carry one qubit.
pub fn two_node_graph() -> Result<CausaloidGraph<BaseCausaloid<FloatType, bool>>, TopologyBuildError>
{
    let mut graph = CausaloidGraph::new(0);

    let source = graph
        .add_causaloid(detector(SOURCE_ID))
        .map_err(|_| TopologyBuildError::Node(SOURCE_NODE))?;
    let target = graph
        .add_causaloid(detector(TARGET_ID))
        .map_err(|_| TopologyBuildError::Node(TARGET_NODE))?;

    graph
        .add_edge(source, target)
        .map_err(|_| TopologyBuildError::Edge(SOURCE_NODE, TARGET_NODE))?;

    Ok(graph)
}

/// The node-keyed factor store and the single-qubit support registry for the
/// two factors, both living on the shared Hilbert leg `0`.
pub fn factors_on_shared_leg(
    factor0: CausalTensor<C>,
    factor1: CausalTensor<C>,
) -> (ProcessFactors<FloatType>, FactorSupports) {
    let mut factors = ProcessFactors::new();
    factors.insert(SOURCE_NODE, factor0);
    factors.insert(TARGET_NODE, factor1);

    let mut supports = FactorSupports::new();
    supports.declare(SOURCE_NODE, &[SHARED_LEG]);
    supports.declare(TARGET_NODE, &[SHARED_LEG]);

    (factors, supports)
}

/// What can go wrong assembling the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyBuildError {
    /// An operator could not be formed as a 2x2 matrix.
    Operator,
    /// A node could not be added to the graph.
    Node(usize),
    /// An edge could not be added between two nodes.
    Edge(usize, usize),
}

impl core::fmt::Display for TopologyBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TopologyBuildError::Operator => write!(f, "an operator is not a 2x2 matrix"),
            TopologyBuildError::Node(n) => write!(f, "node {n} could not be added"),
            TopologyBuildError::Edge(a, b) => write!(f, "the edge {a} -> {b} could not be added"),
        }
    }
}

impl core::error::Error for TopologyBuildError {}
