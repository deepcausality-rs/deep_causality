/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A downward abstraction, Lorenz & Tull, arXiv:2602.16612, Definition 15: a type alignment
//! `(π, τ)` and a query map `π : Q_H → Q_L` sending each high-level query to a low-level query of
//! type `π(X) → π(Y)`. Upward abstractions are derived by composing with low-level sharp states
//! (Proposition 18) and are not stored.

use crate::QuantumError;
use crate::types::abstraction::alignment_structure::{
    AlignmentStructure, StructureScope, check_alignment_structure,
};
use crate::types::abstraction::qc_model::QcModel;
use crate::types::abstraction::query::{Query, QuerySignature};
use crate::types::abstraction::type_alignment::{AlignmentSide, TypeAlignment};
use crate::types::circuit_model::{NumericCaps, QcMorphism};
use alloc::format;
use alloc::vec::Vec;
use core::marker::PhantomData;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;

/// A downward abstraction from a low-level model `L` to a high-level model `H`.
#[derive(Debug, Clone)]
pub struct Abstraction<R: RealField, L, H> {
    low: L,
    high: H,
    alignment: TypeAlignment<R>,
    signature: QuerySignature,
    query_map: Vec<(Query, Query)>,
    _r: PhantomData<R>,
}

impl<R, L, H> Abstraction<R, L, H>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    L: QcModel<R>,
    H: QcModel<R>,
{
    /// An abstraction from its two models, its alignment and its query map, given as pairs
    /// `(high-level query, low-level query)`. The high-level queries form the signature, which is
    /// validated against `H`'s DAG; each low-level query is validated against `L`'s.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if a high-level query is mapped twice; the signature's
    /// errors on either side.
    pub fn new(
        low: L,
        high: H,
        alignment: TypeAlignment<R>,
        query_map: Vec<(Query, Query)>,
    ) -> Result<Self, QuantumError> {
        let high_queries: Vec<Query> = query_map.iter().map(|(h, _)| h.clone()).collect();
        for (i, q) in high_queries.iter().enumerate() {
            if high_queries[i + 1..].contains(q) {
                return Err(QuantumError::CalculationError(format!(
                    "the high-level query {q:?} is mapped twice"
                )));
            }
        }
        let signature = QuerySignature::new(&high.induced_dag(), high_queries)?;
        let low_dag = low.induced_dag();
        for (_, q) in &query_map {
            QuerySignature::new(&low_dag, alloc::vec![q.clone()])?;
        }
        Ok(Self {
            low,
            high,
            alignment,
            signature,
            query_map,
            _r: PhantomData,
        })
    }

    /// The low-level model.
    pub fn low(&self) -> &L {
        &self.low
    }

    /// The high-level model.
    pub fn high(&self) -> &H {
        &self.high
    }

    /// The type alignment.
    pub fn alignment(&self) -> &TypeAlignment<R> {
        &self.alignment
    }

    /// The high-level signature.
    pub fn signature(&self) -> &QuerySignature {
        &self.signature
    }

    /// The query map, as `(high, low)` pairs.
    pub fn query_map(&self) -> &[(Query, Query)] {
        &self.query_map
    }

    /// The low-level query a high-level query maps to.
    pub fn image(&self, high: &Query) -> Option<&Query> {
        self.query_map
            .iter()
            .find(|(h, _)| h == high)
            .map(|(_, l)| l)
    }

    /// Definition 49's predicates for a partition of the low-level nodes by high-level node,
    /// `partition[X]` the low-level nodes of `X`. The high-level inputs are the roots of `H`'s
    /// DAG. The scope reads `Equivalent` when both models are classical and `Necessary` otherwise,
    /// which is what Theorem 51 licenses.
    ///
    /// # Errors
    ///
    /// The partition's shape errors.
    pub fn check_alignment_structure(
        &self,
        partition: &[Vec<usize>],
    ) -> Result<AlignmentStructure, QuantumError> {
        let high = self.high.induced_dag();
        let low = self.low.induced_dag();
        let inputs: Vec<usize> = (0..high.num_vertices())
            .filter(|&v| high.parents(v).is_empty())
            .collect();
        let scope = if self.low.is_classical() && self.high.is_classical() {
            StructureScope::Equivalent
        } else {
            StructureScope::Necessary
        };
        check_alignment_structure(&low, &high, partition, &inputs, scope)
    }

    /// The two sides of the naturality square for one high-level query, as morphisms from the
    /// low-level input type to the high-level output type: `τ_out ∘ ⟦π(Q)⟧_L` and `⟦Q⟧_H ∘ τ_in`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if the query is not in the signature or a type is not
    /// aligned; the semantics' caps.
    pub fn square(
        &self,
        high: &Query,
        caps: &NumericCaps,
    ) -> Result<(QcMorphism<R>, QcMorphism<R>), QuantumError> {
        let low_q = self.image(high).ok_or_else(|| {
            QuantumError::CalculationError(format!("the query {high:?} is not in the signature"))
        })?;
        self.square_with(high, low_q, caps)
    }

    /// The two sides of the square for a high-level query against an explicit low-level query.
    /// Neither need be in the signature: the high-level query is any query well formed on `H`,
    /// the low-level one any query well formed on `L`. The fault-tolerance check asks the
    /// high-level `Io` against every faulted low-level query.
    ///
    /// # Errors
    ///
    /// The queries' construction errors on their models, as [`QuerySignature::new`] reports them;
    /// [`QuantumError::CalculationError`] if a type is not aligned or the square is ill-typed; the
    /// semantics' caps.
    pub fn square_with(
        &self,
        high: &Query,
        low_q: &Query,
        caps: &NumericCaps,
    ) -> Result<(QcMorphism<R>, QcMorphism<R>), QuantumError> {
        QuerySignature::new(&self.high.induced_dag(), alloc::vec![high.clone()])?;
        QuerySignature::new(&self.low.induced_dag(), alloc::vec![low_q.clone()])?;
        let th = self.high.query_type(high)?;
        let tl = self.low.query_type(low_q)?;
        let alignment = self.alignment.extended(
            &self.high.query_wire_map(high)?,
            &self.low.query_wire_map(low_q)?,
            high,
        )?;
        for (side, high_wires, low_wires) in [
            (AlignmentSide::Input, &th.quantum_in, &tl.quantum_in),
            (AlignmentSide::Output, &th.quantum_out, &tl.quantum_out),
        ] {
            let expected = alignment.low_for_side(high_wires, side)?;
            if &expected != low_wires {
                return Err(QuantumError::CalculationError(format!(
                    "the square for {high:?} is ill-typed on the {side:?} side: π of the high-level \
                     wires {high_wires:?} is {expected:?}, but the low-level query has {low_wires:?}"
                )));
            }
        }
        let tau_in = tau_with_classical(
            &alignment,
            AlignmentSide::Input,
            &th.quantum_in,
            &th.classical_in_counts,
            &tl.classical_in_counts,
            caps,
        )?;
        let tau_out = tau_with_classical(
            &alignment,
            AlignmentSide::Output,
            &th.quantum_out,
            &th.classical_out_counts,
            &tl.classical_out_counts,
            caps,
        )?;
        let low_m = self.low.numeric_query(low_q, caps)?;
        let high_m = self.high.numeric_query(high, caps)?;
        let left = low_m.then(&tau_out, caps)?;
        let right = tau_in.then(&high_m, caps)?;
        Ok((left, right))
    }

    /// The concrete, upward form of an `Open` query (Proposition 18): both sides of the square,
    /// each a morphism from the low-level input type, composed with the low-level sharp state
    /// `E(s)`, computed on demand. `state` is a ket on the quantum part of the high-level input
    /// type of the opened query; its low-level image is the section applied to it, and on the
    /// right side `τ ∘ E(s) = s` recovers the high-level concrete query `⟦Q⟧_H ∘ s`. The classical
    /// inputs of the opened query stay free: both sides carry them as classical inputs.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] unless `high` is an `Open` query in the signature;
    /// [`QuantumError::DimensionMismatch`] if the ket does not fit the opened type.
    pub fn concrete_do(
        &self,
        high: &Query,
        state: &[Complex<R>],
        caps: &NumericCaps,
    ) -> Result<(QcMorphism<R>, QcMorphism<R>), QuantumError> {
        let Query::Open(_) = high else {
            return Err(QuantumError::CalculationError(format!(
                "{high:?} is not an Open query; the concrete form is defined for openings"
            )));
        };
        let (left, right) = self.square(high, caps)?;
        let th = self.high.query_type(high)?;
        let sharp = QcMorphism::state(state)?;
        // The high-level input is the opened type alone when the model had no inputs; otherwise
        // the state must cover the whole high-level input type.
        let d_high_in = th.d_in();
        if sharp.d_out() != d_high_in {
            return Err(QuantumError::DimensionMismatch(format!(
                "the state has dimension {} but the opened high-level type has {d_high_in}",
                sharp.d_out()
            )));
        }
        let low_q = self.image(high).expect("checked by square");
        let alignment = self.alignment.extended(
            &self.high.query_wire_map(high)?,
            &self.low.query_wire_map(low_q)?,
            high,
        )?;
        let mut low_state = sharp.then(
            &alignment.section_for_side(&th.quantum_in, AlignmentSide::Input, caps)?,
            caps,
        )?;
        if !th.classical_in_counts.is_empty() {
            low_state = low_state.tensor(
                &QcMorphism::classical_identity(&th.classical_in_counts)?,
                caps,
            )?;
        }
        Ok((low_state.then(&left, caps)?, low_state.then(&right, caps)?))
    }
}

/// `τ` on a type with classical wires carried as the identity; the classical outcome counts must
/// agree across the square.
fn tau_with_classical<R>(
    alignment: &TypeAlignment<R>,
    side: AlignmentSide,
    high_quantum: &[usize],
    high_classical: &[usize],
    low_classical: &[usize],
    caps: &NumericCaps,
) -> Result<QcMorphism<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    if high_classical != low_classical {
        return Err(QuantumError::CalculationError(format!(
            "classical wires differ across the square: high {high_classical:?} against low {low_classical:?}"
        )));
    }
    let quantum = alignment.tau_for_side(high_quantum, side, caps)?;
    if high_classical.is_empty() {
        return Ok(quantum);
    }
    quantum.tensor(&QcMorphism::classical_identity(high_classical)?, caps)
}
