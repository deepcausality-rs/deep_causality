/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The quantum causal model: a two-node causal graph, the context its nodes read, and its
//! Choi–Jamiołkowski factor store. Configuration only — no execution — so the freeze scenarios in
//! `main.rs` read cleanly.

use crate::FloatType;
use crate::constants::{
    C, DIAGONAL_FIRST, DIAGONAL_SECOND, ONE, SHARED_LEG, SOURCE_ID, SOURCE_NODE, TARGET_ID,
    TARGET_NODE, ZERO,
};
use deep_causality::{
    CausableGraph, CausalEffect, CausalityError, Causaloid, CausaloidGraph, PropagatingProcess,
};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_num::const_scalar_from_float;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{FactorSupports, ProcessFactors};
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

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

/// What the nodes read: the observation level each one reports against.
///
/// The check needs no position, clock or event, so the spatial, temporal and spacetime slots are
/// the absent ones. The data node is at the working type, so the threshold follows the alias with
/// the factors.
pub type DetectorContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// One node of the graph: a detector reading its threshold from the shared [`DetectorContext`].
pub type Detector = Causaloid<FloatType, bool, (), Arc<DetectorContext>>;

/// Contextoid id: the detection threshold an observation must reach, in the observation's unit.
const DETECTION_THRESHOLD: ContextoidId = 1;

/// The detectors' context: one `Data` node holding the detection threshold, `0.55`.
///
/// The threshold plays no part in the commutativity check; a node has to compute something, and
/// this is what these compute.
fn detector_context() -> Result<DetectorContext, ContextIndexError> {
    let facts = [(
        DETECTION_THRESHOLD,
        const_scalar_from_float!(FloatType, 0.55),
    )];
    let mut context = Context::with_capacity(1, "detection threshold", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// The detection threshold, read off the context.
fn detection_threshold(context: &DetectorContext) -> Result<FloatType, CausalityError> {
    context.get_data_by_id(DETECTION_THRESHOLD).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "the detector context holds no detection threshold at contextoid id \
             {DETECTION_THRESHOLD}"
        ))
    })
}

/// What each node does: report whether an observation passed the detection threshold the context
/// holds.
///
/// The node's own rule is incidental to the check this example is about — the commutativity
/// condition is a statement about the factors, not about what the nodes compute. It is written out
/// here rather than borrowed from the library's test helpers, because those are fixed at `f64` and
/// would pin the graph to one precision while the factors followed the alias.
fn passes_threshold(
    observation: CausalEffect<FloatType>,
    _state: (),
    context: Option<Arc<DetectorContext>>,
) -> PropagatingProcess<bool, (), Arc<DetectorContext>> {
    let Some(context) = context else {
        return PropagatingProcess::from_error(CausalityError::MissingContext());
    };
    let threshold = match detection_threshold(&context) {
        Ok(threshold) => threshold,
        Err(error) => return PropagatingProcess::from_error(error),
    };
    match observation.into_value() {
        Some(observation) => PropagatingProcess::pure(observation >= threshold),
        None => PropagatingProcess::from_error(CausalityError::ValueNotAvailable()),
    }
}

/// One node of the graph.
fn detector(id: u64, context: &Arc<DetectorContext>) -> Detector {
    Causaloid::new_with_context(
        id,
        passes_threshold,
        Arc::clone(context),
        "reports whether an observation passed the detection threshold",
    )
}

/// A fresh two-node causal graph `0 → 1` whose nodes each carry one qubit and share one
/// [`DetectorContext`].
pub fn two_node_graph() -> Result<CausaloidGraph<Detector>, TopologyBuildError> {
    let context = Arc::new(detector_context().map_err(TopologyBuildError::Context)?);
    let mut graph = CausaloidGraph::new(0);

    let source = graph
        .add_causaloid(detector(SOURCE_ID, &context))
        .map_err(|_| TopologyBuildError::Node(SOURCE_NODE))?;
    let target = graph
        .add_causaloid(detector(TARGET_ID, &context))
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
#[derive(Debug)]
pub enum TopologyBuildError {
    /// An operator could not be formed as a 2x2 matrix.
    Operator,
    /// The detectors' context could not be built; carries the context's error naming the cause.
    Context(ContextIndexError),
    /// A node could not be added to the graph.
    Node(usize),
    /// An edge could not be added between two nodes.
    Edge(usize, usize),
}

impl core::fmt::Display for TopologyBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TopologyBuildError::Operator => write!(f, "an operator is not a 2x2 matrix"),
            TopologyBuildError::Context(error) => {
                write!(f, "the detector context could not be built: {error}")
            }
            TopologyBuildError::Node(n) => write!(f, "node {n} could not be added"),
            TopologyBuildError::Edge(a, b) => write!(f, "the edge {a} -> {b} could not be added"),
        }
    }
}

impl core::error::Error for TopologyBuildError {}
