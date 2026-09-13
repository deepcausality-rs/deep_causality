/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::abstraction::query::Query;
use crate::types::circuit_model::{CircuitModel, InducedDag, NumericCaps, QcMorphism, WireId};
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The wires a query's morphism runs between, in the order the morphism's legs take them: quantum
/// wires ascending, classical wires in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryType {
    /// Quantum input wires, ascending.
    pub quantum_in: Vec<WireId>,
    /// Classical input wires, ascending.
    pub classical_in: Vec<WireId>,
    /// Quantum output wires, ascending.
    pub quantum_out: Vec<WireId>,
    /// Classical output wires, in output order.
    pub classical_out: Vec<WireId>,
    /// The dimensions of the quantum input wires, in the query model.
    pub quantum_in_dims: Vec<usize>,
    /// The outcome counts of the classical input wires, in the query model.
    pub classical_in_counts: Vec<usize>,
    /// The outcome counts of the classical output wires, in the query model.
    pub classical_out_counts: Vec<usize>,
}

impl QueryType {
    /// The quantum input dimension.
    pub fn d_in(&self) -> usize {
        self.quantum_in_dims.iter().product::<usize>().max(1)
    }
}

/// A compositional model in QC that can answer queries: the low-level or high-level side of an
/// [`Abstraction`](crate::Abstraction). Static dispatch: each model type implements it directly.
pub trait QcModel<R: RealField> {
    /// The induced open DAG.
    fn induced_dag(&self) -> InducedDag;

    /// The wire type of a query's morphism.
    ///
    /// # Errors
    ///
    /// The query's own construction errors.
    fn query_type(&self, query: &Query) -> Result<QueryType, QuantumError>;

    /// The numeric semantics of a query.
    ///
    /// # Errors
    ///
    /// The query's construction errors and the numeric semantics' caps.
    fn numeric_query(
        &self,
        query: &Query,
        caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError>;

    /// The cardinality of a wire, for building alignments.
    fn wire_cardinality(&self, wire: WireId) -> Option<usize>;

    /// Whether the model is a classical causal model: every wire classical. Theorem 51's
    /// characterisation applies to a pair of classical models and is a necessary condition otherwise.
    fn is_classical(&self) -> bool;

    /// The wires a query renamed to fresh inputs, as `(original, fresh)` pairs; empty for queries
    /// that add no inputs.
    ///
    /// # Errors
    ///
    /// The query's construction errors.
    fn query_wire_map(&self, query: &Query) -> Result<Vec<(WireId, WireId)>, QuantumError>;
}

impl<R> CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The rewired model a query evaluates on.
    ///
    /// # Errors
    ///
    /// The rewiring's errors.
    pub fn query_model(&self, query: &Query) -> Result<CircuitModel<R>, QuantumError> {
        match query {
            Query::Io => Ok(self.clone()),
            Query::Open(nodes) => self.opened(nodes),
            Query::Inc(sets) => self.interchanged(sets),
            Query::Observe(wires) => self.observed(wires),
            Query::Fault(fault) => self.faulted(fault),
        }
    }
}

impl<R> QcModel<R> for CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn induced_dag(&self) -> InducedDag {
        CircuitModel::induced_dag(self)
    }

    fn query_type(&self, query: &Query) -> Result<QueryType, QuantumError> {
        let model = self.query_model(query)?;
        let mut quantum_in = model.inputs().to_vec();
        quantum_in.sort_unstable();
        quantum_in.dedup();
        let classical_in = model.classical_inputs();
        let mut quantum_out: Vec<WireId> = model
            .outputs()
            .iter()
            .copied()
            .filter(|&w| model.wires()[w].is_quantum())
            .collect();
        quantum_out.sort_unstable();
        quantum_out.dedup();
        let classical_out: Vec<WireId> = model
            .outputs()
            .iter()
            .copied()
            .filter(|&w| !model.wires()[w].is_quantum())
            .collect();
        let card = |w: &WireId| model.wires()[*w].cardinality();
        Ok(QueryType {
            quantum_in_dims: quantum_in.iter().map(card).collect(),
            classical_in_counts: classical_in.iter().map(card).collect(),
            classical_out_counts: classical_out.iter().map(card).collect(),
            quantum_in,
            classical_in,
            quantum_out,
            classical_out,
        })
    }

    fn numeric_query(
        &self,
        query: &Query,
        caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError> {
        self.query_model(query)?.numeric_semantics(caps)
    }

    fn wire_cardinality(&self, wire: WireId) -> Option<usize> {
        self.wires().get(wire).map(|t| t.cardinality())
    }

    fn is_classical(&self) -> bool {
        self.wires().iter().all(|w| !w.is_quantum())
    }

    fn query_wire_map(&self, query: &Query) -> Result<Vec<(WireId, WireId)>, QuantumError> {
        match query {
            Query::Io => Ok(Vec::new()),
            Query::Observe(_) | Query::Fault(_) => self.query_model(query).map(|_| Vec::new()),
            Query::Open(nodes) => self.opened_with_map(nodes).map(|(_, m)| m),
            Query::Inc(sets) => self.interchanged_with_map(sets).map(|(_, m)| m),
        }
    }
}
