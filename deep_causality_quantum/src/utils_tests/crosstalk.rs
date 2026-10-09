/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The crosstalk problem of `qcl_crosstalk` as configured experiments: its three structural
//! candidates as conditional tables, and a response model for its settings.
//!
//! Each qubit is excited with probability 1/10 in every candidate. Under H₁ Q2 is excited with
//! probability 2/5 when Q1 is and 1/15 when it is not, which keeps Q2's own excitation at 1/10;
//! H₂ is the mirror image; under H₃ a bath, on with probability 1/4, excites each qubit with
//! probability 2/5 when on and never when off. Holding Q1 excited then reads Q2 at 0.40, 0.10 and
//! 0.10 under H₁, H₂ and H₃; both read excited together at 0.04 in every candidate; and an echo,
//! which removes a direct coupling and leaves the bath, reads both at 0.01, 0.01 and 0.04.

use crate::QuantumError;
use crate::types::carriers::{Observable, QuantumPlant};
use crate::types::pipeline::evidence_source::ObservedContext;
use crate::types::pipeline::response::{Response, ResponseModel};
use crate::types::qcm::hypothesis::Hypothesis;
use crate::types::qcm::process_factors::{CjFactor, FactorSupports, ProcessFactors};
use crate::types::qgates::operator_linalg::embed_on_legs;
use crate::types::verdict::projection::Projection;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_context_store::ContextSnapshot;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

/// Qubit 1's node and leg.
pub const CROSSTALK_Q1: usize = 0;
/// Qubit 2's node and leg.
pub const CROSSTALK_Q2: usize = 1;
/// The bath's node and leg.
pub const CROSSTALK_BATH: usize = 2;

/// What a crosstalk experiment sets and reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrosstalkSetting {
    /// No intervention; read both qubits excited.
    Passive,
    /// `do(node = |1⟩)`; read `read` excited.
    Hold {
        /// The qubit held excited.
        node: usize,
        /// The qubit read.
        read: usize,
    },
    /// An echo on both qubits, which refocuses a direct coupling and not a fluctuating bath;
    /// read both qubits excited.
    Echo,
}

/// A setting is not a context, so its observations record none.
impl ObservedContext for CrosstalkSetting {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// The response of a structural crosstalk candidate to a setting: holding a qubit replaces its
/// factor by `|1⟩⟨1|` on the qubit and the identity on its parents; an echo replaces a qubit's
/// factor that conditions on the other qubit by the qubit's own excitation, 1/10, whatever the
/// parent; the instrument is `|1⟩⟨1|` on each qubit read and the identity on the candidate's
/// other legs. A mechanism candidate is refused.
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
        let support_space = |node: usize| -> Result<BTreeMap<usize, usize>, QuantumError> {
            let legs = supports.support(node).ok_or_else(|| {
                QuantumError::DimensionMismatch(format!(
                    "candidate '{}' has no node {node}",
                    candidate.name()
                ))
            })?;
            Ok(supports.space_map(&legs.iter().copied().collect()))
        };
        let both = [CROSSTALK_Q1, CROSSTALK_Q2];
        let (factors, read): (Vec<(usize, CjFactor<f64>)>, &[usize]) = match setting {
            CrosstalkSetting::Passive => (Vec::new(), &both),
            CrosstalkSetting::Hold { node, read } => (
                vec![(*node, on_legs(&[0.0, 1.0], *node, &support_space(*node)?)?)],
                core::slice::from_ref(read),
            ),
            CrosstalkSetting::Echo => {
                let mut decoupled = Vec::new();
                for (node, other) in [(CROSSTALK_Q1, CROSSTALK_Q2), (CROSSTALK_Q2, CROSSTALK_Q1)] {
                    let space = support_space(node)?;
                    if space.contains_key(&other) {
                        decoupled.push((node, on_legs(&[0.9, 0.1], node, &space)?));
                    }
                }
                (decoupled, &both)
            }
        };
        let legs = candidate.legs()?;
        let mut instrument: Option<CausalTensor<Complex<f64>>> = None;
        for &qubit in read {
            let excited = on_legs(&[0.0, 1.0], qubit, &legs)?;
            instrument = Some(match instrument {
                None => excited,
                Some(acc) => acc.matmul(&excited).map_err(|e| {
                    QuantumError::CalculationError(format!("instrument product: {e:?}"))
                })?,
            });
        }
        let instrument = instrument.ok_or_else(|| {
            QuantumError::CalculationError("a crosstalk setting reads at least one qubit".into())
        })?;
        Ok(Response::Intervention {
            factors,
            instrument,
        })
    }
}

/// `diag(entries)` on `leg`, the identity on the other legs of `space`.
fn on_legs(
    entries: &[f64; 2],
    leg: usize,
    space: &BTreeMap<usize, usize>,
) -> Result<CjFactor<f64>, QuantumError> {
    embed_on_legs(&diagonal(entries), &BTreeSet::from([leg]), space)
}

/// The three acyclic candidates of `qcl_crosstalk` as conditional tables: H₁ (Q1 → Q2),
/// H₂ (Q2 → Q1) and H₃ (a bath drives both).
///
/// # Errors
///
/// As `Hypothesis::structural`, which these factors pass.
pub fn crosstalk_candidates() -> Result<[Hypothesis<f64>; 3], QuantumError> {
    let (q1, q2, bath) = (CROSSTALK_Q1, CROSSTALK_Q2, CROSSTALK_BATH);
    let (low, high) = (1.0 / 15.0, 0.4);
    // A two-leg factor's diagonal runs over its legs in ascending order, the first most
    // significant: P(child | parent) at (q1, q2) = (0,0), (0,1), (1,0), (1,1) when Q1 < Q2.
    let q2_given_q1 = [1.0 - low, low, 1.0 - high, high];
    let q1_given_q2 = [1.0 - low, 1.0 - high, low, high];
    let given_bath = [1.0, 1.0 - high, 0.0, high];
    Ok([
        structural(
            "H1 Q1->Q2",
            &[(q1, &[q1], &[0.9, 0.1]), (q2, &[q1, q2], &q2_given_q1)],
        )?,
        structural(
            "H2 Q2->Q1",
            &[(q2, &[q2], &[0.9, 0.1]), (q1, &[q1, q2], &q1_given_q2)],
        )?,
        structural(
            "H3 Q1<-B->Q2",
            &[
                (bath, &[bath], &[0.75, 0.25]),
                (q1, &[q1, bath], &given_bath),
                (q2, &[q2, bath], &given_bath),
            ],
        )?,
    ])
}

fn structural(
    name: &str,
    nodes: &[(usize, &[usize], &[f64])],
) -> Result<Hypothesis<f64>, QuantumError> {
    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();
    for &(node, legs, entries) in nodes {
        factors.insert(node, diagonal(entries));
        supports.declare(node, legs);
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

/// The two qubits in `|00⟩`, exposing "qubit 2 excited" as the observable a plant config needs.
///
/// # Errors
///
/// As `QuantumPlant::from_ket` and `Projection::new`, which these pass.
pub fn crosstalk_plant() -> Result<(QuantumPlant<f64>, Observable<f64, 4>), QuantumError> {
    let ground = [1.0, 0.0, 0.0, 0.0].map(|v| Complex::new(v, 0.0));
    let plant = QuantumPlant::from_ket(&CausalTensor::from_slice(&ground, &[4]))?;
    let e2 = Projection::<f64, 4>::new(diagonal(&[0.0, 1.0, 0.0, 1.0]))?;
    Ok((plant, Observable::new("e2", e2)))
}
