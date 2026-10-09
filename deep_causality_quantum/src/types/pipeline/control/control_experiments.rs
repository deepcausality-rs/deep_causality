/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The configured-experiment stages of [`Control`], whose predictions a [`ResponseModel`]
//! computes and whose evidence an [`EvidenceSource`] supplies, and the stage's state as a
//! campaign reads and charges it.

use crate::QuantumError;
use crate::types::carriers::{Observable, QuantumPlant};
use crate::types::design::{Adjudication, Experiment, MinCostCover};
use crate::types::pipeline::configured_experiment::ConfiguredExperiment;
use crate::types::pipeline::control::{Control, ControlWorld};
use crate::types::pipeline::evidence_source::{EvidenceSource, Observation, ObservedContext};
use crate::types::pipeline::ledger::Ledger;
use crate::types::pipeline::response::ResponseModel;
use crate::types::qpu::born_sampler::sample_probability;
use crate::types::qpu::histogram::{CountHistogram, ShotHistogram};
use crate::types::qpu::shot_estimate::ShotEstimate;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, NaturalNumber, ToPrimitive};

/// What one observed experiment yields before it is recorded: the read-out, the counts when the
/// evidence came as counts, and the ledgers with the shots charged, the root's before the fork
/// and every world's after it.
struct Gathered<R, N> {
    read_out: ShotEstimate<R>,
    counts: Option<CountHistogram>,
    ledgers: Vec<Ledger<R, N>>,
}

impl<R, N, const D: usize> Control<R, N, D>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber + ToPrimitive + FromPrimitive + core::fmt::Debug,
{
    /// A configured experiment observed from `source` as the run's first observation, and every
    /// candidate whose prediction for it the observation contradicts refused before the fork and
    /// the plan: [`baseline`](Self::baseline) for evidence a source supplies rather than the
    /// plant.
    ///
    /// Each candidate's prediction is computed by `model` as [`ConfiguredExperiment::predict`]
    /// computes it, and a candidate is refused when `|prediction − observed|` exceeds `sigmas`
    /// standard errors of the observed estimate. The observation is recorded as
    /// [`observe_experiment`](Self::observe_experiment) records one before the fork: the root's
    /// read-out, charged to the root ledger, and an [`Observation`] carrying the configuration's
    /// context.
    ///
    /// Fails with `CalculationError` while the config names a baseline still to observe, after an
    /// observation or the fork, and on a `sigmas` that is not finite or is negative; and with the
    /// errors of the predictions, the evidence or the snapshot.
    pub fn baseline_with<C, M>(
        mut self,
        model: &M,
        experiment: &ConfiguredExperiment<R, C>,
        source: &EvidenceSource<R>,
        sigmas: R,
    ) -> Self
    where
        M: ResponseModel<R, C>,
        C: ObservedContext,
    {
        if self.failure.is_some() || self.awaits_baseline("baseline_with") {
            return self;
        }
        if self.root.is_some() || !self.worlds.is_empty() {
            self.fail(QuantumError::CalculationError(format!(
                "baseline_with observes '{}' as the run's first observation, before the fork; \
                 an observation or the fork came first",
                experiment.name()
            )));
            return self;
        }
        if !sigmas.is_finite() || sigmas < R::zero() {
            self.fail(QuantumError::CalculationError(format!(
                "baseline_with needs a finite, non-negative sigmas, got {sigmas:?}"
            )));
            return self;
        }
        let gathered = self
            .candidates
            .iter()
            .map(|(_, h)| experiment.predict(model, h, &self.plant, &self.observables))
            .collect::<Result<Vec<R>, _>>()
            .and_then(|predicted| {
                let evidence = self.evidence(model, experiment, source)?;
                let context = experiment.configuration().context_snapshot()?;
                Ok((predicted, evidence, context))
            });
        match gathered {
            Ok((predicted, evidence, context)) => {
                self.ledger = evidence.ledgers[0];
                self.record_root(experiment.name().into(), evidence.read_out);
                self.observations.push(Observation::new(
                    experiment.name().into(),
                    evidence.read_out,
                    evidence.counts,
                    context,
                ));
                self.refuse_contradicted(&predicted, evidence.read_out, sigmas);
            }
            Err(e) => self.fail(e),
        }
        self
    }

    /// The design plan over configured experiments: each one's predictions for the live
    /// candidates computed by `model` as [`ConfiguredExperiment::predict`] computes them, then the
    /// cover [`design`](Self::design) solves, its cost committed to the root ledger. The plan's
    /// entries index `experiments`. Planning evaluates the model without counting the evaluations
    /// on a ledger; [`predict_with`](Self::predict_with) counts them.
    ///
    /// Fails with `CalculationError` before the config's baseline was observed; with the error of
    /// a prediction; with [`Experiment::new`]'s error on a prediction outside `[0, 1]`; and as
    /// [`design`](crate::design) fails.
    pub fn design_with<C, M>(
        mut self,
        model: &M,
        experiments: &[ConfiguredExperiment<R, C>],
        objective: MinCostCover<R>,
    ) -> Self
    where
        M: ResponseModel<R, C>,
    {
        if self.failure.is_some() || self.awaits_baseline("design") {
            return self;
        }
        let probes = experiments
            .iter()
            .map(|e| {
                let predictions = self
                    .candidates
                    .iter()
                    .map(|(_, h)| e.predict(model, h, &self.plant, &self.observables))
                    .collect::<Result<Vec<R>, _>>()?;
                Experiment::new(e.name(), e.cost(), e.shots(), predictions)
            })
            .collect::<Result<Vec<_>, _>>();
        match probes {
            Ok(probes) => self.plan_over(&probes, objective),
            Err(e) => self.fail(e),
        }
        self
    }

    /// Each world's prediction for a configured experiment, computed by `model` as
    /// [`ConfiguredExperiment::predict`] computes it and counted on the world's `predictions`.
    /// `compare` then judges it against the root read-out.
    ///
    /// Fails with `CalculationError` before the fork, and with the prediction's error.
    pub fn predict_with<C, M>(mut self, model: &M, experiment: &ConfiguredExperiment<R, C>) -> Self
    where
        M: ResponseModel<R, C>,
    {
        if self.failure.is_some() {
            return self;
        }
        if self.worlds.is_empty() {
            self.fail(QuantumError::CalculationError(
                "predict_with evaluates forked worlds; call fork first".into(),
            ));
            return self;
        }
        self.predict_worlds(|w, plant, observables| {
            experiment.predict(model, &w.hypothesis, plant, observables)
        });
        self
    }

    /// A configured experiment observed on the device, from `source`: the Born sampler at the
    /// read-out the source's truth predicts, counts a lab recorded, or a published value as
    /// effective draws. The read-out becomes the root's latest, the baseline `compare` judges
    /// predictions against. The shots are charged as `observe` charges them: to the root ledger
    /// before the fork, and after it to every world's ledger, since the device's evidence belongs
    /// to every world's history. The [`Observation`] goes to the report with the experiment's
    /// name, the read-out, the counts and the configuration's context at observation time.
    ///
    /// Simulated evidence draws at the seed `observe` would use for the next observation, so a
    /// mechanism truth draws the histogram `observe` draws on the plant it predicts.
    ///
    /// Fails with `CalculationError` before the config's baseline was observed;
    /// `NormalizationError` when a simulated truth predicts a value outside `[0, 1]`; and with
    /// the errors of the prediction, the read-out, the charge or the snapshot.
    pub fn observe_experiment<C, M>(
        mut self,
        model: &M,
        experiment: &ConfiguredExperiment<R, C>,
        source: &EvidenceSource<R>,
    ) -> Self
    where
        M: ResponseModel<R, C>,
        C: ObservedContext,
    {
        if self.failure.is_some() || self.awaits_baseline("observe_experiment") {
            return self;
        }
        let observed = self
            .evidence(model, experiment, source)
            .and_then(|evidence| Ok((evidence, experiment.configuration().context_snapshot()?)));
        match observed {
            Ok((evidence, context)) => {
                if self.worlds.is_empty() {
                    self.ledger = evidence.ledgers[0];
                } else {
                    for (w, ledger) in self.worlds.iter_mut().zip(evidence.ledgers) {
                        w.ledger = ledger;
                    }
                }
                self.record_root(experiment.name().into(), evidence.read_out);
                self.observations.push(Observation::new(
                    experiment.name().into(),
                    evidence.read_out,
                    evidence.counts,
                    context,
                ));
            }
            Err(e) => self.fail(e),
        }
        self
    }

    /// The evidence `source` yields for `experiment`, with the ledgers after the shots are
    /// charged, which happens before anything is drawn.
    fn evidence<C, M>(
        &self,
        model: &M,
        experiment: &ConfiguredExperiment<R, C>,
        source: &EvidenceSource<R>,
    ) -> Result<Gathered<R, N>, QuantumError>
    where
        M: ResponseModel<R, C>,
    {
        let shots = match source {
            EvidenceSource::Simulated(_) => experiment.shots(),
            EvidenceSource::Recorded(hist) => hist.total(),
            EvidenceSource::Published(draws) => draws.read_out()?.shots(),
        };
        let count = N::from_u64(shots).ok_or_else(|| {
            QuantumError::CalculationError(format!("{shots} shots do not fit the count width"))
        })?;
        let time = Self::device_time(shots, self.shot_time)?;
        let charged = if self.worlds.is_empty() {
            vec![self.ledger.observed(count, time)?]
        } else {
            self.worlds
                .iter()
                .map(|w| w.ledger.observed(count, time))
                .collect::<Result<Vec<_>, _>>()?
        };
        let (read_out, counts) = match source {
            EvidenceSource::Simulated(truth) => {
                let p = experiment.predict(model, truth, &self.plant, &self.observables)?;
                let seed = Self::draw_seed(self.seed, self.observed_so_far())?;
                let hist = sample_probability(p, shots, seed)?;
                (ShotEstimate::of_outcome(&hist, 1)?, Some(hist))
            }
            EvidenceSource::Recorded(hist) => {
                (ShotEstimate::of_outcome(hist, 1)?, Some(hist.clone()))
            }
            EvidenceSource::Published(draws) => (draws.read_out()?, None),
        };
        Ok(Gathered {
            read_out,
            counts,
            ledgers: charged,
        })
    }

    /// The forked worlds.
    pub(crate) fn worlds(&self) -> &[ControlWorld<R, N, D>] {
        &self.worlds
    }

    /// The plant.
    pub(crate) fn plant(&self) -> &QuantumPlant<R> {
        &self.plant
    }

    /// The observables the plant exposes.
    pub(crate) fn observables(&self) -> &[Observable<R, D>] {
        &self.observables
    }

    /// The first failure a stage raised.
    pub(crate) fn failure(&self) -> Option<&QuantumError> {
        self.failure.as_ref()
    }

    /// The last adjudication.
    pub(crate) fn adjudication(&self) -> Option<&Adjudication<R, D>> {
        self.adjudication.as_ref()
    }

    /// The control stage with `cost` committed to the root ledger, the cost of an experiment a
    /// campaign ran.
    pub(crate) fn charged(mut self, cost: R) -> Self {
        self.ledger = self.ledger.costed(cost);
        self
    }
}
