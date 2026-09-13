/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::abstraction::fault_set::Fault;
use crate::types::circuit_model::{InducedDag, NodeId, WireId};
use alloc::format;
use alloc::vec::Vec;

/// A query on a compositional model, Lorenz & Tull, arXiv:2602.16612, §3.2 and §7.2.
///
/// `Open(S)` is the quantum generalisation of the abstract Do-query: the mechanisms of `S` are
/// deleted and their wires become inputs. It is the mechanism-level intervention QCL v1 names
/// `intervene_mechanism`; the wrapper adds a name and changes nothing. `Inc(S₁, …, Sₙ)` is defined
/// only on pairwise disjoint sets each of which is parallelisable, which a [`QuerySignature`] checks
/// at construction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Query {
    /// Declared inputs to declared outputs.
    Io,
    /// Delete the mechanisms of the named nodes and make their wires inputs.
    Open(Vec<NodeId>),
    /// Interchange: feed the named sets from separate copies of the model.
    Inc(Vec<Vec<NodeId>>),
    /// Measure the named output wires in the computational basis.
    Observe(Vec<WireId>),
    /// Insert a Pauli error at a location of the circuit: a comb that is not a Do-query.
    Fault(Fault),
}

impl Query {
    /// A short name for reports.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Open(_) => "open",
            Self::Inc(_) => "interchange",
            Self::Observe(_) => "observe",
            Self::Fault(_) => "fault",
        }
    }
}

/// A set of queries validated against the DAG they will be asked of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuerySignature {
    queries: Vec<Query>,
}

impl QuerySignature {
    /// The signature, with every `Inc` checked to be on pairwise disjoint, each parallelisable, sets
    /// of existing nodes, and every `Open` on existing nodes.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NotParallelisable`] naming the directed path that joins two members of one
    /// interchange set; [`QuantumError::DimensionMismatch`] on a node out of range or two sets
    /// sharing a node.
    pub fn new(dag: &InducedDag, queries: Vec<Query>) -> Result<Self, QuantumError> {
        for q in &queries {
            match q {
                Query::Open(nodes) => {
                    for &n in nodes {
                        if n >= dag.num_vertices() {
                            return Err(QuantumError::DimensionMismatch(format!(
                                "Open names node {n}, but the model has {} nodes",
                                dag.num_vertices()
                            )));
                        }
                    }
                }
                Query::Inc(sets) => {
                    for (i, set) in sets.iter().enumerate() {
                        for &a in set {
                            if a >= dag.num_vertices() {
                                return Err(QuantumError::DimensionMismatch(format!(
                                    "Inc names node {a}, but the model has {} nodes",
                                    dag.num_vertices()
                                )));
                            }
                            for &b in set {
                                if a != b && dag.reaches(a, b) {
                                    return Err(QuantumError::NotParallelisable(i, a, b));
                                }
                            }
                        }
                        for other in &sets[i + 1..] {
                            if let Some(shared) = set.iter().find(|n| other.contains(n)) {
                                return Err(QuantumError::DimensionMismatch(format!(
                                    "node {shared} appears in two interchange sets"
                                )));
                            }
                        }
                    }
                }
                Query::Fault(fault) => {
                    if let Some(n) = fault.after()
                        && n >= dag.num_vertices()
                    {
                        return Err(QuantumError::DimensionMismatch(format!(
                            "the fault follows node {n}, but the model has {} nodes",
                            dag.num_vertices()
                        )));
                    }
                }
                Query::Io | Query::Observe(_) => {}
            }
        }
        Ok(Self { queries })
    }

    /// The queries, in order.
    pub fn queries(&self) -> &[Query] {
        &self.queries
    }

    /// The query count.
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Whether the signature is empty.
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }
}
