/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The model: a two-node causal graph `0 → 1` and its Choi–Jamiołkowski factor store, both
//! factors on the shared Hilbert leg `0`. Configuration only; the stages run in `main.rs`.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!` and
//! `const_scalar_from_float!`, so the compiler resolves them against the alias in `main`.

use crate::{C, FloatType};
use deep_causality::{BaseCausaloid, CausableGraph, Causaloid, CausaloidGraph, PropagatingEffect};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{FactorSupports, ProcessFactors};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// The small numbers the operators are written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

/// The two diagonal entries of the commuting partner, `diag(3, −1)`.
///
/// Any two diagonal operators commute, so the values carry no weight beyond being distinct: a
/// multiple of the identity would commute with everything and prove nothing about diagonality.
pub const DIAGONAL_FIRST: FloatType = const_scalar_from_int!(FloatType, 3);
pub const DIAGONAL_SECOND: FloatType = const_scalar_from_int!(FloatType, -1);

/// The observation level each node reports against. It plays no part in the commutativity check;
/// a node has to compute something, and this is what these compute.
pub const DETECTION_THRESHOLD: FloatType = const_scalar_from_float!(FloatType, 0.55);

/// The Hilbert leg both factors are declared on, and the two nodes of the graph.
pub const SHARED_LEG: usize = 0;
pub const SOURCE_NODE: usize = 0;
pub const TARGET_NODE: usize = 1;
pub const SOURCE_ID: u64 = 0;
pub const TARGET_ID: u64 = 1;

// =============================================================================
// Operators
// =============================================================================

/// A real entry of an operator matrix.
fn real(value: FloatType) -> C {
    Complex::new(value, ZERO)
}

/// A single-qubit operator from its four entries, in row-major order.
fn operator(data: Vec<C>) -> Result<CausalTensor<C>, ModelBuildError> {
    CausalTensor::new(data, vec![2, 2]).map_err(|_| ModelBuildError::Operator)
}

/// Pauli-X, the bit flip.
pub fn sigma_x() -> Result<CausalTensor<C>, ModelBuildError> {
    operator(vec![real(ZERO), real(ONE), real(ONE), real(ZERO)])
}

/// Pauli-Z, the phase flip. It anticommutes with `sigma_x`, so their commutator is `2i σ_y` and
/// its norm is as far from zero as a single-qubit commutator gets.
pub fn sigma_z() -> Result<CausalTensor<C>, ModelBuildError> {
    operator(vec![real(ONE), real(ZERO), real(ZERO), real(-ONE)])
}

/// The diagonal operator `diag(3, −1)`, which commutes with any other diagonal factor.
pub fn diagonal() -> Result<CausalTensor<C>, ModelBuildError> {
    operator(vec![
        real(DIAGONAL_FIRST),
        real(ZERO),
        real(ZERO),
        real(DIAGONAL_SECOND),
    ])
}

// =============================================================================
// The graph
// =============================================================================

/// What each node does: report whether an observation passed the detection threshold.
///
/// It is written out here rather than taken from the engine's test helpers, which fix their value
/// type at `f64`. Borrowing them would pin the graph to one precision while the factors followed
/// the alias, which is how a precision parameter quietly stops being one.
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

/// A two-node graph `0 → 1`, frozen or dynamic.
pub fn two_node_graph(
    frozen: bool,
) -> Result<CausaloidGraph<BaseCausaloid<FloatType, bool>>, ModelBuildError> {
    let mut graph = CausaloidGraph::new(0);

    let source = graph
        .add_causaloid(detector(SOURCE_ID))
        .map_err(|_| ModelBuildError::Node(SOURCE_NODE))?;
    let target = graph
        .add_causaloid(detector(TARGET_ID))
        .map_err(|_| ModelBuildError::Node(TARGET_NODE))?;

    graph
        .add_edge(source, target)
        .map_err(|_| ModelBuildError::Edge(SOURCE_NODE, TARGET_NODE))?;

    if frozen {
        graph.freeze();
    }

    Ok(graph)
}

/// The factor store and support registry for two factors on the shared leg.
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

// =============================================================================
// Errors
// =============================================================================

/// What can go wrong assembling the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelBuildError {
    /// An operator could not be formed as a 2x2 matrix.
    Operator,
    /// A node could not be added to the graph.
    Node(usize),
    /// An edge could not be added between two nodes.
    Edge(usize, usize),
}

impl core::fmt::Display for ModelBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModelBuildError::Operator => write!(f, "an operator is not a 2x2 matrix"),
            ModelBuildError::Node(n) => write!(f, "node {n} could not be added"),
            ModelBuildError::Edge(a, b) => write!(f, "the edge {a} -> {b} could not be added"),
        }
    }
}

impl core::error::Error for ModelBuildError {}
