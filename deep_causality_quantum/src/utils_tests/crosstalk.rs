/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The crosstalk problem of `qcl_crosstalk` as configured experiments: its three structural
//! candidates with the example's factors, and a response model for holding one qubit excited and
//! reading another.

use crate::QuantumError;
use crate::types::pipeline::evidence_source::ObservedContext;
use crate::types::pipeline::response::{Response, ResponseModel};
use crate::types::qcm::hypothesis::Hypothesis;
use crate::types::qcm::process_factors::{CjFactor, FactorSupports, ProcessFactors};
use crate::types::qgates::operator_linalg::embed_on_legs;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_context_store::ContextSnapshot;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// Qubit 1's node and leg.
pub const CROSSTALK_Q1: usize = 0;
/// Qubit 2's node and leg.
pub const CROSSTALK_Q2: usize = 1;
/// The bath's node and leg.
pub const CROSSTALK_BATH: usize = 2;

/// What a crosstalk experiment sets: nothing, or one qubit held excited, and the qubit it reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrosstalkSetting {
    /// No intervention; read `read` excited.
    Passive {
        /// The qubit read.
        read: usize,
    },
    /// `do(node = |1⟩)`; read `read` excited.
    Hold {
        /// The qubit held excited.
        node: usize,
        /// The qubit read.
        read: usize,
    },
}

/// A setting is not a context, so its observations record none.
impl ObservedContext for CrosstalkSetting {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// The response of a structural crosstalk candidate to a setting: holding a qubit replaces its
/// factor by `|1⟩⟨1|` on the qubit and the identity on its parents, and the instrument is
/// `|1⟩⟨1|` on the qubit read and the identity on the candidate's other legs. A mechanism
/// candidate is refused.
#[derive(Debug, Clone, Copy, Default)]
pub struct CrosstalkModel;

impl ResponseModel<f64, CrosstalkSetting> for CrosstalkModel {
    fn respond(
        &self,
        candidate: &Hypothesis<f64>,
        setting: &CrosstalkSetting,
    ) -> Result<Response<f64>, QuantumError> {
        let supports = candidate.supports().ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "the crosstalk model answers structural candidates, and '{}' is a mechanism",
                candidate.name()
            ))
        })?;
        let (held, read) = match *setting {
            CrosstalkSetting::Passive { read } => (None, read),
            CrosstalkSetting::Hold { node, read } => (Some(node), read),
        };
        let factors = match held {
            None => Vec::new(),
            Some(node) => {
                let legs = supports.support(node).ok_or_else(|| {
                    QuantumError::DimensionMismatch(format!(
                        "candidate '{}' has no node {node} to hold",
                        candidate.name()
                    ))
                })?;
                let space = supports.space_map(&legs.iter().copied().collect());
                vec![(node, excited_on(node, &space)?)]
            }
        };
        let instrument = excited_on(read, &candidate.legs()?)?;
        Ok(Response::Intervention {
            factors,
            instrument,
        })
    }
}

/// `|1⟩⟨1|` on `leg`, the identity on the other legs of `space`.
fn excited_on(leg: usize, space: &BTreeMap<usize, usize>) -> Result<CjFactor<f64>, QuantumError> {
    let excited = CausalTensor::from_slice(
        &[
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
        ],
        &[2, 2],
    );
    embed_on_legs(&excited, &BTreeSet::from([leg]), space)
}

/// The three acyclic candidates of `qcl_crosstalk`, H₁ (Q1 → Q2), H₂ (Q2 → Q1) and H₃ (a bath
/// drives both), with its factors: `diag(0.9, 0.1)` on a parentless qubit and
/// `diag(0.85, 0.05, 0.05, 0.05)` on a qubit and its parent.
///
/// # Errors
///
/// As `Hypothesis::structural`, which these factors pass.
pub fn crosstalk_candidates() -> Result<[Hypothesis<f64>; 3], QuantumError> {
    let (q1, q2, bath) = (CROSSTALK_Q1, CROSSTALK_Q2, CROSSTALK_BATH);
    Ok([
        structural("H1 Q1->Q2", &[(q1, &[]), (q2, &[q1])])?,
        structural("H2 Q2->Q1", &[(q2, &[]), (q1, &[q2])])?,
        structural("H3 Q1<-B->Q2", &[(bath, &[]), (q1, &[bath]), (q2, &[bath])])?,
    ])
}

fn structural(name: &str, parents: &[(usize, &[usize])]) -> Result<Hypothesis<f64>, QuantumError> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();
    for &(node, pa) in parents {
        let mut legs = pa.to_vec();
        legs.push(node);
        let entries: &[f64] = if pa.is_empty() {
            &[0.9, 0.1]
        } else {
            &[0.85, 0.05, 0.05, 0.05]
        };
        factors.insert(node, diagonal(entries));
        supports.declare(node, &legs);
    }
    Hypothesis::structural(name, factors, supports)
}

fn diagonal(entries: &[f64]) -> CjFactor<f64> {
    let d = entries.len();
    let mut data = vec![Complex::new(0.0, 0.0); d * d];
    for (i, &entry) in entries.iter().enumerate() {
        data[i * d + i] = Complex::new(entry, 0.0);
    }
    CausalTensor::from_slice(&data, &[d, d])
}
