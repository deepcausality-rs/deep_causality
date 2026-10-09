/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::{Observable, QuantumPlant};
use crate::types::design::experiment_design::check_cost_and_shots;
use crate::types::pipeline::response::{Response, ResponseModel};
use crate::types::qcm::hypothesis::Hypothesis;
use alloc::format;
use alloc::string::String;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// An experiment that names a configuration and an observable, whose predictions a
/// [`ResponseModel`] computes: the counterpart of an [`Experiment`](crate::Experiment) with typed
/// predictions.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfiguredExperiment<R, C> {
    name: String,
    cost: R,
    shots: u64,
    configuration: C,
    observable: usize,
}

impl<R, C> ConfiguredExperiment<R, C>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// An experiment at `cost` taking `shots`, that sets `configuration` and reads the plant's
    /// observable at index `observable`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] on a non-finite or negative cost;
    /// [`QuantumError::NormalizationError`] on zero shots. The same rules as
    /// [`Experiment::new`](crate::Experiment::new).
    pub fn new(
        name: impl Into<String>,
        cost: R,
        shots: u64,
        configuration: C,
        observable: usize,
    ) -> Result<Self, QuantumError> {
        check_cost_and_shots(cost, shots)?;
        Ok(Self {
            name: name.into(),
            cost,
            shots,
            configuration,
            observable,
        })
    }

    /// The same experiment at `shots`, as a plan priced in time sizes it.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NormalizationError`] on zero shots.
    pub fn with_shots(mut self, shots: u64) -> Result<Self, QuantumError> {
        check_cost_and_shots(self.cost, shots)?;
        self.shots = shots;
        Ok(self)
    }

    /// The name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The cost.
    pub fn cost(&self) -> R {
        self.cost
    }

    /// The shots it takes.
    pub fn shots(&self) -> u64 {
        self.shots
    }

    /// The configuration it sets.
    pub fn configuration(&self) -> &C {
        &self.configuration
    }

    /// The index of the observable it reads.
    pub fn observable(&self) -> usize {
        self.observable
    }

    /// The read-out `candidate` predicts for this experiment under `model`.
    ///
    /// For a mechanism candidate, the model answers with a channel, and the prediction is the
    /// Born read-out of the observable on `plant` evolved by the candidate's channel and then by
    /// the configuration's. For a structural candidate, it answers with interventions and an
    /// instrument, and the prediction is the candidate under the interventions evaluated against
    /// the instrument, as `Hypothesis::predict` evaluates one. Either is a read-out probability,
    /// and one outside `[0, 1]` is refused here, so every stage that reads a prediction reads a
    /// probability.
    ///
    /// # Errors
    ///
    /// The model's refusal; [`QuantumError::CalculationError`] when it answers a mechanism with
    /// interventions or a structural candidate with a channel;
    /// [`QuantumError::DimensionMismatch`] when the observable is not among `observables`;
    /// [`QuantumError::NormalizationError`] naming the candidate and the experiment when the
    /// prediction is not a finite probability in `[0, 1]`; and the errors of the evolution, the
    /// read-out, the interventions or the evaluation.
    pub fn predict<M, const D: usize>(
        &self,
        model: &M,
        candidate: &Hypothesis<R>,
        plant: &QuantumPlant<R>,
        observables: &[Observable<R, D>],
    ) -> Result<R, QuantumError>
    where
        M: ResponseModel<R, C>,
    {
        let prediction = match (
            model.respond(candidate, &self.configuration)?,
            candidate.channel(),
        ) {
            (Response::Channel(configured), Some(own)) => {
                let observable = observables.get(self.observable).ok_or_else(|| {
                    QuantumError::DimensionMismatch(format!(
                        "experiment '{}' reads observable {}, and the plant exposes {}",
                        self.name,
                        self.observable,
                        observables.len()
                    ))
                })?;
                observable.read_out(&plant.evolve(own)?.evolve(&configured)?)
            }
            (
                Response::Intervention {
                    factors,
                    instrument,
                },
                None,
            ) => factors
                .into_iter()
                .try_fold(candidate.clone(), |h, (node, factor)| {
                    h.intervene_mechanism(node, factor)
                })?
                .evaluate(&instrument),
            (Response::Channel(_), None) => Err(QuantumError::CalculationError(format!(
                "structural candidate '{}' is predicted through interventions, and the model \
                 answered experiment '{}' with a channel",
                candidate.name(),
                self.name
            ))),
            (Response::Intervention { .. }, Some(_)) => {
                Err(QuantumError::CalculationError(format!(
                    "mechanism candidate '{}' is predicted through a channel, and the model \
                     answered experiment '{}' with interventions",
                    candidate.name(),
                    self.name
                )))
            }
        }?;
        if !prediction.is_finite() || prediction < R::zero() || prediction > R::one() {
            return Err(QuantumError::NormalizationError(format!(
                "candidate '{}' predicts {prediction:?} for experiment '{}', not a probability",
                candidate.name(),
                self.name
            )));
        }
        Ok(prediction)
    }
}
