/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The process-operator factorization `σ = ∏ᵢ ρ_{Aᵢ|Pa(Aᵢ)}` as an external,
//! node-index-keyed store — the operator analogue of the edge-keyed
//! `LambdaEdges` (R3). `σ` is **static freeze-time decoration**, consulted only
//! at the freeze boundary; it never rides the runtime STATE channel.

use crate::QuantumError;
use deep_causality::CausableGraph;
use deep_causality_algebra::RealField;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;
use std::collections::{BTreeMap, BTreeSet};

/// A single Choi–Jamiołkowski factor `ρ_{Aᵢ|Pa(Aᵢ)}` — a complex matrix on the
/// composite Hilbert space of node `Aᵢ` and its parents.
pub type CjFactor<R> = CausalTensor<Complex<R>>;

/// The node-keyed store of CJ factors. Keys are intrinsic graph node indices
/// (never enumeration/insertion order); an absent key denotes a node with no
/// operator factor. This is an external parameter to the freeze, mirroring
/// `LambdaEdges<V>`; it is never carried on the runtime arity-5 STATE channel.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProcessFactors<R: RealField> {
    factors: BTreeMap<usize, CjFactor<R>>,
}

impl<R: RealField> ProcessFactors<R> {
    /// An empty store.
    pub fn new() -> Self {
        Self {
            factors: BTreeMap::new(),
        }
    }

    /// Inserts (or replaces) the factor for node `node`.
    pub fn insert(&mut self, node: usize, factor: CjFactor<R>) -> &mut Self {
        self.factors.insert(node, factor);
        self
    }

    /// Borrows the factor for `node`, if any (`CausalTensor` is not `Copy`).
    pub fn get(&self, node: usize) -> Option<&CjFactor<R>> {
        self.factors.get(&node)
    }

    /// The node indices that carry a factor, ascending.
    pub fn nodes(&self) -> impl Iterator<Item = usize> + '_ {
        self.factors.keys().copied()
    }

    /// The number of factors.
    pub fn len(&self) -> usize {
        self.factors.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.factors.is_empty()
    }
}

/// The Hilbert support of each factor: the ordered leg-ids it acts on
/// (ascending, the row-major factor layout) plus each leg's dimension. For the
/// flat QCM the leg-id of a single-system node is the node index itself, and
/// `support(Aᵢ) = {Aᵢ} ∪ Pa(Aᵢ)`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FactorSupports {
    /// node index → ascending leg-ids the factor acts on.
    supports: BTreeMap<usize, Vec<usize>>,
    /// leg-id → Hilbert dimension.
    leg_dims: BTreeMap<usize, usize>,
}

impl FactorSupports {
    /// An empty support registry.
    pub fn new() -> Self {
        Self {
            supports: BTreeMap::new(),
            leg_dims: BTreeMap::new(),
        }
    }

    /// Declares a node's support as an explicit ordered leg list. The legs are
    /// sorted ascending (the required row-major factor layout); each leg's
    /// dimension defaults to `2` (a qubit) unless already registered or set via
    /// [`Self::set_leg_dim`].
    pub fn declare(&mut self, node: usize, legs: &[usize]) -> &mut Self {
        let mut legs = legs.to_vec();
        legs.sort_unstable();
        legs.dedup();
        for &l in &legs {
            self.leg_dims.entry(l).or_insert(2);
        }
        self.supports.insert(node, legs);
        self
    }

    /// Overrides a leg's Hilbert dimension (default `2`).
    pub fn set_leg_dim(&mut self, leg: usize, dim: usize) -> &mut Self {
        self.leg_dims.insert(leg, dim);
        self
    }

    /// The ascending leg-ids of `node`, if declared.
    pub fn support(&self, node: usize) -> Option<&[usize]> {
        self.supports.get(&node).map(|v| v.as_slice())
    }

    /// The dimension of `leg` (default `2` if unset).
    pub fn leg_dim(&self, leg: usize) -> usize {
        self.leg_dims.get(&leg).copied().unwrap_or(2)
    }

    /// The product of the leg dimensions on `node`'s support — the expected
    /// matrix dimension of that node's factor.
    pub fn support_dim(&self, node: usize) -> Option<usize> {
        // Checked product: an oversized `set_leg_dim` must not overflow (debug
        // panic / release wrap). `None` here means "undeclared or overflowing".
        self.supports.get(&node).and_then(|legs| {
            legs.iter()
                .try_fold(1usize, |acc, &l| acc.checked_mul(self.leg_dim(l)))
        })
    }

    /// The leg → dimension map restricted to the given legs (the space passed
    /// to `embed_on_legs`).
    pub fn space_map(&self, legs: &BTreeSet<usize>) -> BTreeMap<usize, usize> {
        legs.iter().map(|&l| (l, self.leg_dim(l))).collect()
    }

    /// Builds the support registry from a **frozen** causal graph, using the
    /// single-system-per-node convention: `support(Aᵢ) = {Aᵢ} ∪ Pa(Aᵢ)`, with
    /// the parent set derived from `contains_edge(parent, node)`. Only nodes
    /// carrying a factor are registered. Every leg defaults to a qubit
    /// (dimension `2`); override with [`Self::set_leg_dim`].
    ///
    /// The graph **must be frozen**: only the static representation guarantees a
    /// dense node-id space `0..number_nodes()`. On a dynamic graph a
    /// `remove_node` tombstones a slot without compacting, so a live node can
    /// have an id `≥ number_nodes()` and its parent edges would be silently
    /// dropped. An unfrozen graph is therefore rejected rather than
    /// mis-derived.
    ///
    /// # Errors
    /// Returns [`QuantumError::CalculationError`] if the graph is not frozen.
    pub fn from_graph<T, G, R>(graph: &G, factors: &ProcessFactors<R>) -> Result<Self, QuantumError>
    where
        T: Clone,
        G: CausableGraph<T>,
        R: RealField,
    {
        if !graph.is_frozen() {
            return Err(QuantumError::CalculationError(
                "FactorSupports::from_graph requires a frozen graph (dense node ids); \
                 freeze the graph before deriving supports"
                    .into(),
            ));
        }
        let n = graph.number_nodes();
        let mut me = Self::new();
        for node in factors.nodes() {
            // A factor keyed outside the graph's node range would otherwise be
            // declared as a lone qubit, detaching the factorization from `G`.
            if node >= n {
                return Err(QuantumError::CalculationError(format!(
                    "factor keyed by node {} but the frozen graph has {} nodes (valid ids 0..{})",
                    node, n, n
                )));
            }
            let mut legs: Vec<usize> = (0..n).filter(|&p| graph.contains_edge(p, node)).collect();
            legs.push(node);
            me.declare(node, &legs);
        }
        Ok(me)
    }

    /// Validates that every factor's matrix dimension equals the product of its
    /// declared support leg dimensions, and that each factor is square.
    pub fn validate<R: RealField>(&self, factors: &ProcessFactors<R>) -> Result<(), QuantumError> {
        for node in factors.nodes() {
            let factor = factors.get(node).expect("node came from factors.nodes()");
            let shape = factor.shape();
            if shape.len() != 2 || shape[0] != shape[1] {
                return Err(QuantumError::DimensionMismatch(format!(
                    "factor at node {} is not a square matrix (shape {:?})",
                    node, shape
                )));
            }
            if shape[0] == 0 {
                return Err(QuantumError::DimensionMismatch(format!(
                    "factor at node {} is an empty (0×0) matrix",
                    node
                )));
            }
            let legs = self.supports.get(&node).ok_or_else(|| {
                QuantumError::DimensionMismatch(format!(
                    "node {} has a factor but no declared support",
                    node
                ))
            })?;
            let expected = legs
                .iter()
                .try_fold(1usize, |acc, &l| acc.checked_mul(self.leg_dim(l)))
                .ok_or_else(|| {
                    QuantumError::DimensionMismatch(format!(
                        "node {} support leg dimensions overflow usize",
                        node
                    ))
                })?;
            if shape[0] != expected {
                return Err(QuantumError::DimensionMismatch(format!(
                    "factor at node {} has dim {} but its support implies {}",
                    node, shape[0], expected
                )));
            }
        }
        Ok(())
    }
}
