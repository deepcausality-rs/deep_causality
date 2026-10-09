/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Attributing published measurements among candidates through QCL's control stage.
//!
//! Each measurement is a value and its standard error, entered as effective draws on a fringe
//! (`EvidenceSource::Published`); each candidate is a mechanism whose response to a measurement is
//! the phase of its prediction. The control stage observes every measurement, predicts it in every
//! candidate's world, judges each prediction against the observation and adjudicates the readings
//! together. A prediction printed without an uncertainty carries half a unit of its last printed
//! digit, added to the measurement's standard error in quadrature.

use crate::fringe_qubit::{mechanism, output_port, phase_channel, plant};
use deep_causality_context_store::ContextSnapshot;
use deep_causality_haft::Either;
use deep_causality_quantum::{
    Ambiguity, ConfiguredExperiment, EvidenceSource, Fringe, Hypothesis, ObservedContext,
    QclBuilder, QuantumError, Response, ResponseModel, Survivor,
};
use std::error::Error;

/// The contrast the evidence and the predictions share; no attribution depends on it.
const CONTRAST: f64 = 0.5;
/// The largest phase a prediction or an observation maps to, in rad, where the fringe is linear.
const PHASE_RANGE: f64 = 1e-3;

/// How an adjudication ended.
pub type Outcome = Either<Survivor<f64>, Ambiguity<f64>>;

/// A published measurement and each candidate's prediction for it, in one unit.
pub struct Measurement {
    /// What was measured.
    pub name: String,
    /// The published value.
    pub value: f64,
    /// Its standard error, including any prediction's rounding.
    pub standard_error: f64,
    /// Each candidate's prediction, in the candidates' order.
    pub predictions: Vec<f64>,
}

/// What the evidence leaves: the adjudication, and the candidates whose every reading holds.
pub struct Attribution {
    /// The adjudication; a lone candidate that does not hold reads as [`Ambiguity::NoSurvivor`],
    /// outside the model.
    pub outcome: Outcome,
    /// The candidates that hold.
    pub holding: Vec<String>,
}

/// The adjudication of `candidates` over `measurements`, each judged at `sigmas` standard errors,
/// a pair separated at `floor_bits`.
///
/// # Errors
///
/// A candidate named twice, a measurement without exactly one prediction per candidate, a
/// measurement whose value does not read on the fringe, and the control stage's failures.
pub fn attribute(
    candidates: &[&str],
    measurements: &[Measurement],
    sigmas: f64,
    floor_bits: f64,
) -> Result<Attribution, Box<dyn Error>> {
    if let Some((i, name)) = candidates
        .iter()
        .enumerate()
        .find(|(i, name)| candidates[..*i].contains(name))
    {
        return Err(format!("candidate {i}, '{name}', is named twice").into());
    }
    if let Some(m) = measurements
        .iter()
        .find(|m| m.predictions.len() != candidates.len())
    {
        return Err(format!(
            "'{}' has {} predictions for {} candidates",
            m.name,
            m.predictions.len(),
            candidates.len()
        )
        .into());
    }
    let largest = measurements
        .iter()
        .flat_map(|m| {
            m.predictions
                .iter()
                .copied()
                .chain([m.value.abs() + m.standard_error])
        })
        .fold(0.0f64, |a, b| a.max(b.abs()));
    let scale = PHASE_RANGE / largest.max(f64::MIN_POSITIVE);
    let fringe = Fringe::new(0.5, CONTRAST * scale / 2.0)?;
    let model = Predictions {
        scale,
        candidates: candidates.iter().map(|c| c.to_string()).collect(),
        predictions: measurements.iter().map(|m| m.predictions.clone()).collect(),
    };
    let hypotheses = candidates
        .iter()
        .map(|c| mechanism(c, CONTRAST))
        .collect::<Result<Vec<_>, _>>()?;
    let config = QclBuilder::config::<f64, u64>()
        .over_plant(plant()?, &[output_port()?])
        .mechanisms(&hypotheses)
        .build()?;
    let mut control = QclBuilder::control::<f64, u64, 2, _>(&config).fork();
    for (i, m) in measurements.iter().enumerate() {
        let experiment = ConfiguredExperiment::new(m.name.clone(), 1.0, 1, Index(i), 0)?;
        let source = EvidenceSource::Published(fringe.effective_draws(m.value, m.standard_error)?);
        control = control
            .observe_experiment(&model, &experiment, &source)
            .predict_with(&model, &experiment)
            .compare(sigmas);
    }
    let report = control.adjudicate(floor_bits).finalize()?;
    let holding: Vec<String> = report
        .worlds
        .iter()
        .filter(|w| {
            w.readings()
                .iter()
                .all(|r| r.verdict().is_some_and(|v| v.accepted()))
        })
        .map(|w| w.name().to_string())
        .collect();
    let outcome = match report.adjudication.ok_or("adjudicate ran")?.outcome {
        Either::Right(Ambiguity::Vacuous { worlds }) if holding.is_empty() => {
            Either::Right(Ambiguity::NoSurvivor { worlds })
        }
        outcome => outcome,
    };
    Ok(Attribution { outcome, holding })
}

/// An adjudication, in words.
pub fn describe(outcome: &Outcome) -> String {
    match outcome {
        Either::Left(s) => format!(
            "{} survives, {:.1} bits from its nearest rival",
            s.name, s.separation_bits
        ),
        Either::Right(Ambiguity::NoSurvivor { .. }) => {
            "no candidate holds: outside the model".into()
        }
        Either::Right(Ambiguity::SeveralSurvive { survivors }) => {
            format!(
                "{} candidates survive: {}",
                survivors.len(),
                survivors.join(", ")
            )
        }
        Either::Right(Ambiguity::Unseparated { .. }) => {
            "one candidate holds, unseparated from a rival at the floor".into()
        }
        Either::Right(Ambiguity::NonCommuting { .. }) => {
            "two candidates' verdicts do not commute: no joint verdict".into()
        }
        Either::Right(Ambiguity::Vacuous { .. }) => "one candidate: nothing to discriminate".into(),
    }
}

/// The measurement an experiment reads, by its index.
#[derive(Debug, Clone, Copy)]
struct Index(usize);

impl ObservedContext for Index {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// Each candidate's prediction for each measurement, as the phase `scale · prediction`.
struct Predictions {
    scale: f64,
    candidates: Vec<String>,
    predictions: Vec<Vec<f64>>,
}

impl ResponseModel<f64, Index> for Predictions {
    fn respond(
        &self,
        candidate: &Hypothesis<f64>,
        index: &Index,
    ) -> Result<Response<f64>, QuantumError> {
        let j = self
            .candidates
            .iter()
            .position(|c| c == candidate.name())
            .ok_or_else(|| {
                QuantumError::CalculationError(format!("no candidate '{}'", candidate.name()))
            })?;
        let prediction = self
            .predictions
            .get(index.0)
            .and_then(|p| p.get(j))
            .ok_or_else(|| {
                QuantumError::CalculationError(format!(
                    "no prediction {} for '{}'",
                    index.0,
                    candidate.name()
                ))
            })?;
        Ok(Response::Channel(phase_channel(self.scale * prediction)?))
    }
}
