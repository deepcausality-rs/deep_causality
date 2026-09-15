/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A detector error model as a classical causal model in FStoch.
//!
//! The variables are detectors and logical observables, each a bit; the latent parents are error
//! mechanisms, each firing independently with its probability and flipping the variables it names.
//! FStoch is a subcategory of QC (Lorenz & Tull, arXiv:2602.16612, after Example 57), so the model
//! answers queries as a [`QcModel`]: the `Io` query is the distribution over the variables as a
//! morphism from the trivial system with one scalar block per outcome string, and a
//! [`Query::Fault`] on mechanism `k` is the distribution with `k`'s flip pattern applied once more,
//! which is what a Pauli injected at the mechanism's circuit location does to the classical
//! record. The model is carried on a frozen `CausaloidGraph` when it comes from one, and the Stim
//! text format is one constructor behind the `dem` feature. Nothing here decodes.

use crate::QuantumError;
use crate::types::abstraction::fault_set::PauliKind;
use crate::types::abstraction::qc_model::{QcModel, QueryType};
use crate::types::abstraction::query::Query;
use crate::types::circuit_model::{InducedDag, NumericCaps, QcMorphism, WireId};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality::CausableGraph;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// The most mechanisms a model enumerates exactly: `2^20` subsets.
pub const DEM_MAX_MECHANISMS: usize = 20;

/// One error mechanism: its probability and the variables it flips.
#[derive(Debug, Clone, PartialEq)]
pub struct Mechanism {
    /// The firing probability.
    pub probability: f64,
    /// The detectors it flips, ascending.
    pub detectors: Vec<usize>,
    /// The logical observables it flips, ascending.
    pub observables: Vec<usize>,
}

/// Which nodes of a graph play which role.
#[derive(Debug, Clone, PartialEq)]
pub struct DemRoles {
    /// Mechanism nodes with their probabilities, in mechanism order.
    pub mechanisms: Vec<(usize, f64)>,
    /// Detector nodes, in detector order.
    pub detectors: Vec<usize>,
    /// Observable nodes, in observable order.
    pub observables: Vec<usize>,
}

/// A detector error model.
#[derive(Debug, Clone, PartialEq)]
pub struct DemModel {
    mechanisms: Vec<Mechanism>,
    num_detectors: usize,
    num_observables: usize,
}

impl DemModel {
    /// A model from its mechanisms and variable counts.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a mechanism names a variable outside the counts or a
    /// probability outside `[0, 1]`; [`QuantumError::CalculationError`] above
    /// [`DEM_MAX_MECHANISMS`].
    pub fn new(
        mechanisms: Vec<Mechanism>,
        num_detectors: usize,
        num_observables: usize,
    ) -> Result<Self, QuantumError> {
        if mechanisms.len() > DEM_MAX_MECHANISMS {
            return Err(QuantumError::CalculationError(format!(
                "{} mechanisms; the model enumerates at most {DEM_MAX_MECHANISMS}",
                mechanisms.len()
            )));
        }
        let mut sorted = mechanisms;
        for (k, m) in sorted.iter_mut().enumerate() {
            if !(0.0..=1.0).contains(&m.probability) || !m.probability.is_finite() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "mechanism {k} has probability {}, outside [0, 1]",
                    m.probability
                )));
            }
            m.detectors.sort_unstable();
            m.detectors.dedup();
            m.observables.sort_unstable();
            m.observables.dedup();
            if let Some(d) = m.detectors.iter().find(|&&d| d >= num_detectors) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "mechanism {k} flips detector {d}, but the model has {num_detectors} detectors"
                )));
            }
            if let Some(o) = m.observables.iter().find(|&&o| o >= num_observables) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "mechanism {k} flips observable {o}, but the model has {num_observables} observables"
                )));
            }
        }
        Ok(Self {
            mechanisms: sorted,
            num_detectors,
            num_observables,
        })
    }

    /// A model from a frozen causal graph with the roles of its nodes: each mechanism flips the
    /// detectors and observables among its children.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] unless the graph is frozen, as `FactorSupports::from_graph`
    /// requires; [`QuantumError::DimensionMismatch`] on a node outside the graph or a node in two
    /// roles; the errors of [`new`](Self::new).
    pub fn from_graph<T, G>(graph: &G, roles: &DemRoles) -> Result<Self, QuantumError>
    where
        T: Clone,
        G: CausableGraph<T>,
    {
        if !graph.is_frozen() {
            return Err(QuantumError::CalculationError(
                "DemModel::from_graph requires a frozen graph (dense node ids); freeze the graph \
                 before reading the model off it"
                    .into(),
            ));
        }
        let n = graph.number_nodes();
        let mut seen = vec![false; n];
        let mut claim = |node: usize, role: &str| -> Result<(), QuantumError> {
            if node >= n {
                return Err(QuantumError::DimensionMismatch(format!(
                    "{role} node {node} is outside the graph of {n} nodes"
                )));
            }
            if seen[node] {
                return Err(QuantumError::DimensionMismatch(format!(
                    "node {node} is named in two roles"
                )));
            }
            seen[node] = true;
            Ok(())
        };
        for &(m, _) in &roles.mechanisms {
            claim(m, "mechanism")?;
        }
        for &d in &roles.detectors {
            claim(d, "detector")?;
        }
        for &o in &roles.observables {
            claim(o, "observable")?;
        }
        let mechanisms = roles
            .mechanisms
            .iter()
            .map(|&(node, probability)| Mechanism {
                probability,
                detectors: roles
                    .detectors
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| graph.contains_edge(node, **d))
                    .map(|(i, _)| i)
                    .collect(),
                observables: roles
                    .observables
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| graph.contains_edge(node, **o))
                    .map(|(i, _)| i)
                    .collect(),
            })
            .collect();
        Self::new(mechanisms, roles.detectors.len(), roles.observables.len())
    }

    /// Stim's detector-error-model text: `error(p) D… L…`, `detector[(coords)] D…` and
    /// `logical_observable L…` lines, blank lines and `#` comments. Every other line is refused by
    /// its number and text. Detector and observable counts are one past the largest index named.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming an unrecognised line, a malformed probability or
    /// target; the errors of [`new`](Self::new).
    #[cfg(feature = "dem")]
    pub fn from_stim_text(text: &str) -> Result<Self, QuantumError> {
        let mut mechanisms = Vec::new();
        let mut num_detectors = 0usize;
        let mut num_observables = 0usize;
        let refuse = |number: usize, line: &str, why: &str| {
            QuantumError::CalculationError(format!("line {number}: `{line}` {why}"))
        };
        for (i, raw) in text.lines().enumerate() {
            let number = i + 1;
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            // The directive, its parenthesised argument (a probability or detector coordinates,
            // which may contain spaces), and the targets after it.
            let (directive, argument, rest) = match line.find('(') {
                Some(open) => {
                    let close = line
                        .find(')')
                        .ok_or_else(|| refuse(number, raw, "has an unclosed parenthesis"))?;
                    if close < open {
                        return Err(refuse(number, raw, "has an unclosed parenthesis"));
                    }
                    (
                        line[..open].trim(),
                        Some(line[open + 1..close].trim()),
                        &line[close + 1..],
                    )
                }
                None => match line.split_once(char::is_whitespace) {
                    Some((head, rest)) => (head, None, rest),
                    None => (line, None, ""),
                },
            };
            let mut detectors = Vec::new();
            let mut observables = Vec::new();
            for target in rest.split_whitespace() {
                let (kind, index) = target.split_at(1);
                let index: usize = index.parse().map_err(|_| {
                    refuse(number, raw, "has a target that is not `D<n>` or `L<n>`")
                })?;
                match kind {
                    "D" => {
                        num_detectors = num_detectors.max(index + 1);
                        detectors.push(index);
                    }
                    "L" => {
                        num_observables = num_observables.max(index + 1);
                        observables.push(index);
                    }
                    _ => {
                        return Err(refuse(
                            number,
                            raw,
                            "has a target that is not `D<n>` or `L<n>`",
                        ));
                    }
                }
            }
            match directive {
                "error" => {
                    let probability: f64 = argument
                        .ok_or_else(|| refuse(number, raw, "names no probability"))?
                        .parse()
                        .map_err(|_| {
                            refuse(number, raw, "has a probability that is not a number")
                        })?;
                    mechanisms.push(Mechanism {
                        probability,
                        detectors,
                        observables,
                    });
                }
                "detector" | "logical_observable" => {}
                _ => {
                    return Err(refuse(
                        number,
                        raw,
                        "is not an error, detector or logical_observable line",
                    ));
                }
            }
        }
        Self::new(mechanisms, num_detectors, num_observables)
    }

    /// The mechanisms.
    pub fn mechanisms(&self) -> &[Mechanism] {
        &self.mechanisms
    }

    /// The detector count.
    pub fn num_detectors(&self) -> usize {
        self.num_detectors
    }

    /// The observable count.
    pub fn num_observables(&self) -> usize {
        self.num_observables
    }

    /// The variable count, detectors then observables.
    pub fn num_variables(&self) -> usize {
        self.num_detectors + self.num_observables
    }

    /// Adds a mechanism that never fires and flips nothing, the model's stand-in for a circuit
    /// location it does not represent, and returns its index. A fault on it leaves the
    /// distribution unchanged, which is exactly the claim an omitted mechanism makes.
    pub fn push_phantom(&mut self) -> Result<usize, QuantumError> {
        if self.mechanisms.len() >= DEM_MAX_MECHANISMS {
            return Err(QuantumError::CalculationError(format!(
                "the model already holds {DEM_MAX_MECHANISMS} mechanisms"
            )));
        }
        self.mechanisms.push(Mechanism {
            probability: 0.0,
            detectors: Vec::new(),
            observables: Vec::new(),
        });
        Ok(self.mechanisms.len() - 1)
    }

    /// The flip pattern of a mechanism as a variable-index bit mask, detectors first.
    fn pattern(&self, m: &Mechanism) -> usize {
        let mut bits = 0usize;
        for &d in &m.detectors {
            bits |= 1 << (self.num_variables() - 1 - d);
        }
        for &o in &m.observables {
            bits |= 1 << (self.num_variables() - 1 - (self.num_detectors + o));
        }
        bits
    }

    /// The distribution over outcome strings, index `y` with the first variable most significant,
    /// with the flip pattern `shift` applied once more.
    fn distribution(&self, shift: usize) -> Vec<f64> {
        let outcomes = 1usize << self.num_variables();
        let mut p = vec![0.0f64; outcomes];
        let patterns: Vec<usize> = self.mechanisms.iter().map(|m| self.pattern(m)).collect();
        for subset in 0..(1usize << self.mechanisms.len()) {
            let mut weight = 1.0f64;
            let mut flips = shift;
            for (k, m) in self.mechanisms.iter().enumerate() {
                if subset & (1 << k) != 0 {
                    weight *= m.probability;
                    flips ^= patterns[k];
                } else {
                    weight *= 1.0 - m.probability;
                }
            }
            p[flips] += weight;
        }
        p
    }

    /// The shift a query asks for: none for `Io`, mechanism `k`'s pattern for a fault `X` on `k`.
    fn shift_of(&self, query: &Query) -> Result<usize, QuantumError> {
        match query {
            Query::Io => Ok(0),
            Query::Fault(f) => {
                if f.after().is_some() {
                    return Err(QuantumError::CalculationError(
                        "a fault on a detector error model names no node; mechanisms are latent"
                            .into(),
                    ));
                }
                let mut shift = 0usize;
                for &(k, kind) in f.errors() {
                    if kind != PauliKind::X {
                        return Err(QuantumError::CalculationError(format!(
                            "a mechanism fires or does not: the fault on mechanism {k} must be X, not {}",
                            kind.letter()
                        )));
                    }
                    let m = self.mechanisms.get(k).ok_or_else(|| {
                        QuantumError::DimensionMismatch(format!(
                            "the fault names mechanism {k}, but the model has {}",
                            self.mechanisms.len()
                        ))
                    })?;
                    shift ^= self.pattern(m);
                }
                Ok(shift)
            }
            other => Err(QuantumError::CalculationError(format!(
                "a detector error model answers Io and Fault queries, not {}",
                other.kind()
            ))),
        }
    }

    fn digits(&self, y: usize) -> Vec<usize> {
        (0..self.num_variables())
            .map(|v| (y >> (self.num_variables() - 1 - v)) & 1)
            .collect()
    }
}

impl<R> QcModel<R> for DemModel
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn induced_dag(&self) -> InducedDag {
        let m = self.mechanisms.len();
        let mut dag = InducedDag::new(m + self.num_variables());
        for (k, mech) in self.mechanisms.iter().enumerate() {
            for &d in &mech.detectors {
                dag.add_edge(k, m + d);
            }
            for &o in &mech.observables {
                dag.add_edge(k, m + self.num_detectors + o);
            }
        }
        dag
    }

    fn query_type(&self, query: &Query) -> Result<QueryType, QuantumError> {
        self.shift_of(query)?;
        Ok(QueryType {
            quantum_in: Vec::new(),
            classical_in: Vec::new(),
            quantum_out: Vec::new(),
            classical_out: (0..self.num_variables()).collect(),
            quantum_in_dims: Vec::new(),
            classical_in_counts: Vec::new(),
            classical_out_counts: vec![2; self.num_variables()],
        })
    }

    fn numeric_query(
        &self,
        query: &Query,
        _caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError> {
        let shift = self.shift_of(query)?;
        let p = self.distribution(shift);
        let mut out = QcMorphism::new(1, 1, Vec::new(), vec![2; self.num_variables()])?;
        for (y, &weight) in p.iter().enumerate() {
            if weight <= 0.0 {
                continue;
            }
            let amplitude = R::from_f64(weight.sqrt()).ok_or_else(|| {
                QuantumError::CalculationError("the scalar cannot represent the probability".into())
            })?;
            out.push(
                Vec::new(),
                self.digits(y),
                vec![CausalTensor::from_slice(
                    &[Complex::new(amplitude, R::zero())],
                    &[1, 1],
                )],
            )?;
        }
        Ok(out)
    }

    fn wire_cardinality(&self, wire: WireId) -> Option<usize> {
        (wire < self.num_variables()).then_some(2)
    }

    fn is_classical(&self) -> bool {
        true
    }

    fn query_wire_map(&self, query: &Query) -> Result<Vec<(WireId, WireId)>, QuantumError> {
        self.shift_of(query).map(|_| Vec::new())
    }
}

impl core::fmt::Display for DemModel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "detector error model: {} mechanisms over {} detectors and {} observables",
            self.mechanisms.len(),
            self.num_detectors,
            self.num_observables
        )
    }
}

/// A short name for the variables a mechanism flips, `D0 D1 L0`.
pub fn flips_of(m: &Mechanism) -> String {
    let mut parts: Vec<String> = m.detectors.iter().map(|d| format!("D{d}")).collect();
    parts.extend(m.observables.iter().map(|o| format!("L{o}")));
    parts.join(" ")
}
