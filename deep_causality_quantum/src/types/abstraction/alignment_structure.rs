/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The structural precheck of a partition, Lorenz & Tull, arXiv:2602.16612, Definition 49 and
//! Theorem 51.
//!
//! For a high-level vertex `X` with `π(X)` its low-level block, `α(X)` is the set of low-level
//! vertices with a directed path to `π(X)` that does not pass through `π(Pa(X))`, where a path
//! passes through a blocked vertex when any vertex on it, its start included, is blocked; the
//! paper's Example 54 has `α(X) = {X, Z}` with `W` excluded on exactly that reading, and
//! `π(X) ⊆ α(X)` through the empty path. The partition is *simple* when `α(X) ∩ π(Y) = ∅`,
//! *extra-simple* when `α(X) ∩ α(Y) = ∅`, for all `X ≠ Y`, and *full* when every vertex of
//! `π(Pa(X) ∖ V^in_H)` has a directed path to `π(X)`.
//!
//! Theorem 51 makes these the exact conditions for a constructive abstraction between classical
//! causal models to extend to the mechanism level, one per structure type. Quantum compositional
//! models of DAGs have no copy maps, and the paper proves no such characterisation for them; there
//! the predicates are a necessary condition by Remark 56, and the report says `Necessary`.

use crate::QuantumError;
use crate::types::circuit_model::{InducedDag, NodeId};
use crate::types::decision::{Check, CheckItem, CheckReport};
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;

/// What the predicates license.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructureScope {
    /// Both models are classical causal models: Theorem 51, the predicates decide mechanism-level
    /// abstraction.
    Equivalent,
    /// The low-level model is quantum: the predicates are a necessary condition (Remark 56).
    Necessary,
}

/// What `check_alignment_structure` computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentStructure {
    /// `α(X) ∩ π(Y) = ∅` for all `X ≠ Y`.
    pub simple: bool,
    /// `α(X) ∩ α(Y) = ∅` for all `X ≠ Y`.
    pub extra_simple: bool,
    /// Every vertex of `π(Pa(X) ∖ V^in_H)` reaches `π(X)`, for all `X`.
    pub full: bool,
    /// The first pair `(X, Y)` at which simplicity failed: `α(X)` meets `π(Y)`.
    pub simple_witness: Option<(NodeId, NodeId)>,
    /// The first pair `(X, Y)` at which extra-simplicity failed: `α(X)` meets `α(Y)`.
    pub extra_simple_witness: Option<(NodeId, NodeId)>,
    /// The first `(X, y)` at which fullness failed: `y ∈ π(Pa(X))` does not reach `π(X)`.
    pub full_witness: Option<(NodeId, NodeId)>,
    /// `α(X)` for each high-level vertex.
    pub alpha: Vec<BTreeSet<NodeId>>,
    /// What the predicates license.
    pub scope: StructureScope,
    /// Ordered pairs `(X, Y)` examined.
    pub pairs_examined: usize,
}

impl AlignmentStructure {
    /// The decision as a report: one record per ordered pair for simplicity, one for
    /// extra-simplicity, one per parent vertex for fullness, each `0` against a threshold of `0`
    /// when it held and `1` when it did not.
    pub fn report<R: RealField>(&self) -> CheckReport<R> {
        let mut checks = Vec::new();
        let n = self.alpha.len();
        for x in 0..n {
            for y in 0..n {
                if x == y {
                    continue;
                }
                let simple_failed = self.simple_witness == Some((x, y));
                let extra_failed = self.extra_simple_witness == Some((x, y));
                checks.push(Check::new(
                    CheckItem::Pair(x, y),
                    if simple_failed || extra_failed {
                        R::one()
                    } else {
                        R::zero()
                    },
                    R::zero(),
                ));
            }
        }
        if let Some((x, y)) = self.full_witness {
            checks.push(Check::new(CheckItem::Pair(x, y), R::one(), R::zero()));
        }
        CheckReport::new(checks, self.pairs_examined)
    }
}

/// Definition 49's predicates on a partition. `partition[X]` lists the low-level vertices of the
/// high-level vertex `X`; `high_inputs` are the high-level input vertices, whose blocks fullness
/// does not ask about.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if the partition has other than one block per high-level
/// vertex, a block is empty, a low-level vertex is named twice or out of range, or an input is
/// out of range. A low-level vertex in no block is allowed: `π` need not be onto.
pub fn check_alignment_structure(
    low: &InducedDag,
    high: &InducedDag,
    partition: &[Vec<NodeId>],
    high_inputs: &[NodeId],
    scope: StructureScope,
) -> Result<AlignmentStructure, QuantumError> {
    let n = high.num_vertices();
    if partition.len() != n {
        return Err(QuantumError::DimensionMismatch(format!(
            "the partition has {} blocks for {n} high-level vertices",
            partition.len()
        )));
    }
    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    let mut blocks: Vec<BTreeSet<NodeId>> = Vec::with_capacity(n);
    for (x, b) in partition.iter().enumerate() {
        if b.is_empty() {
            return Err(QuantumError::DimensionMismatch(format!(
                "block {x} holds no low-level vertex"
            )));
        }
        let block: BTreeSet<NodeId> = b.iter().copied().collect();
        if block.len() != b.len() {
            return Err(QuantumError::DimensionMismatch(format!(
                "block {x} names a low-level vertex twice: {b:?}"
            )));
        }
        for &z in &block {
            if z >= low.num_vertices() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "block {x} names low-level vertex {z}, but the low-level model has {} vertices",
                    low.num_vertices()
                )));
            }
            if !seen.insert(z) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "low-level vertex {z} lies in two blocks; π must be a disjoint partition"
                )));
            }
        }
        blocks.push(block);
    }
    if let Some(&bad) = high_inputs.iter().find(|&&i| i >= n) {
        return Err(QuantumError::DimensionMismatch(format!(
            "high-level input {bad} is out of range for {n} vertices"
        )));
    }
    let inputs: BTreeSet<NodeId> = high_inputs.iter().copied().collect();

    let mut alpha: Vec<BTreeSet<NodeId>> = Vec::with_capacity(n);
    for x in 0..n {
        let blocked: BTreeSet<NodeId> = high
            .parents(x)
            .into_iter()
            .flat_map(|p| blocks[p].iter().copied())
            .collect();
        let a: BTreeSet<NodeId> = (0..low.num_vertices())
            .filter(|&z| low.reaches_avoiding(z, &blocks[x], &blocked))
            .collect();
        alpha.push(a);
    }

    let mut simple_witness = None;
    let mut extra_simple_witness = None;
    let mut pairs_examined = 0usize;
    for x in 0..n {
        for y in 0..n {
            if x == y {
                continue;
            }
            pairs_examined += 1;
            if simple_witness.is_none() && !alpha[x].is_disjoint(&blocks[y]) {
                simple_witness = Some((x, y));
            }
            if extra_simple_witness.is_none() && !alpha[x].is_disjoint(&alpha[y]) {
                extra_simple_witness = Some((x, y));
            }
        }
    }
    let mut full_witness = None;
    'full: for x in 0..n {
        for p in high.parents(x) {
            if inputs.contains(&p) {
                continue;
            }
            for &y in &blocks[p] {
                if !blocks[x].iter().any(|&t| low.reaches(y, t)) {
                    full_witness = Some((x, y));
                    break 'full;
                }
            }
        }
    }
    Ok(AlignmentStructure {
        simple: simple_witness.is_none(),
        extra_simple: extra_simple_witness.is_none(),
        full: full_witness.is_none(),
        simple_witness,
        extra_simple_witness,
        full_witness,
        alpha,
        scope,
        pairs_examined,
    })
}
