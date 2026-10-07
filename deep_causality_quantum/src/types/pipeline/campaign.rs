/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The sequential campaign: plan over the candidates still open, run the cheapest planned
//! experiment, adjudicate, and stop as soon as the evidence decides or nothing left can.

use crate::QuantumError;
use crate::types::decision::Tolerance;
use crate::types::design::{Ambiguity, Experiment, MinCostCover, PlanEntry, Survivor, design};
use crate::types::pipeline::configured_experiment::ConfiguredExperiment;
use crate::types::pipeline::control::{Control, ControlReport, ControlWorld};
use crate::types::pipeline::evidence_source::{EvidenceSource, ObservedContext};
use crate::types::pipeline::response::ResponseModel;
use alloc::format;
use alloc::vec::Vec;
use core::cmp::Ordering;
use deep_causality_algebra::RealField;
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum};
use deep_causality_haft::Either;
use deep_causality_num::{FromPrimitive, NaturalNumber, ToPrimitive};

/// Why a campaign stopped.
#[derive(Debug, Clone, PartialEq)]
pub enum CampaignStop<R> {
    /// One candidate holds and separates from every rival by the floor.
    Survivor(Survivor<R>),
    /// No candidate holds: the observations fall outside the model.
    OutsideTheModel,
    /// Candidates remain unresolved, and no remaining experiment resolves them.
    Unresolvable(Ambiguity<R>),
}

/// What a campaign plans and judges by: the planning objective, the `sigmas` a prediction may
/// sit from an observation and still agree, and the `drift` in read-out probability that a
/// context change may move the planned experiment's predictions before the campaign re-plans.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CampaignRules<R> {
    objective: MinCostCover<R>,
    sigmas: R,
    drift: R,
}

impl<R: RealField + core::fmt::Debug> CampaignRules<R> {
    /// The rules of `objective`, `sigmas` and `drift`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] when `sigmas` or `drift` is not finite or is negative.
    pub fn new(objective: MinCostCover<R>, sigmas: R, drift: R) -> Result<Self, QuantumError> {
        for (name, value) in [("sigmas", sigmas), ("drift", drift)] {
            if !value.is_finite() || value < R::zero() {
                return Err(QuantumError::CalculationError(format!(
                    "a campaign needs a finite, non-negative {name}, got {value:?}"
                )));
            }
        }
        Ok(Self {
            objective,
            sigmas,
            drift,
        })
    }

    /// The planning objective.
    pub fn objective(&self) -> &MinCostCover<R> {
        &self.objective
    }

    /// The standard errors a prediction may sit from an observation.
    pub fn sigmas(&self) -> R {
        self.sigmas
    }

    /// The prediction change that forces a re-plan.
    pub fn drift(&self) -> R {
        self.drift
    }
}

/// The experiment a campaign planned to run next, with the open worlds and the predictions it
/// was planned on.
#[derive(Debug, Clone, PartialEq)]
struct Planned<R> {
    experiment: usize,
    shots: u64,
    cost: R,
    open: Vec<usize>,
    predictions: Vec<R>,
}

/// A sequential campaign over a forked control stage.
///
/// Each step keeps the experiment planned at the end of the last one unless the context has
/// moved its predictions by more than the rules' drift, in which case it re-plans; runs that
/// experiment (`observe_experiment → predict_with → compare → adjudicate`); and stops at a
/// separated survivor or when no candidate holds. Otherwise it plans the next experiment over the
/// open candidates, those that still hold and the rivals not yet separated from one of them, with
/// the experiments not yet run, and stops when no plan covers a pair that involves a candidate
/// that holds. The planned experiment is the cheapest entry of the plan that resolves such a
/// pair.
///
/// Every experiment runs at most once, so `k` experiments stop a campaign within `k + 1` steps.
/// Each plan covers the open pairs the last plan's remaining entries covered, so without a
/// context change a campaign spends at most the static plan's cost.
pub struct Campaign<R: RealField, N, const D: usize> {
    control: Control<R, N, D>,
    planned: Option<Planned<R>>,
    run: Vec<usize>,
    spent: R,
    replans: usize,
    stop: Option<CampaignStop<R>>,
}

impl<R, N, const D: usize> Campaign<R, N, D>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber + ToPrimitive + FromPrimitive + core::fmt::Debug,
{
    /// A campaign over `control`, which must have forked its worlds before the first step.
    pub fn new(control: Control<R, N, D>) -> Self {
        Self {
            control,
            planned: None,
            run: Vec::new(),
            spent: R::zero(),
            replans: 0,
            stop: None,
        }
    }

    /// Whether the campaign stopped.
    pub fn is_stopped(&self) -> bool {
        self.stop.is_some()
    }

    /// Why the campaign stopped, once it did.
    pub fn stop(&self) -> Option<&CampaignStop<R>> {
        self.stop.as_ref()
    }

    /// The experiments run, as indices into the experiments, in the order they ran.
    pub fn experiments_run(&self) -> &[usize] {
        &self.run
    }

    /// The cost of the experiments run.
    pub fn spent(&self) -> R {
        self.spent
    }

    /// How often a context change forced a re-plan.
    pub fn replans(&self) -> usize {
        self.replans
    }

    /// The control stage's report.
    ///
    /// # Errors
    ///
    /// The first failure a stage raised.
    pub fn finalize(self) -> Result<ControlReport<R, N, D>, QuantumError> {
        self.control.finalize()
    }

    /// One step of the campaign as a flow endomorphism for `CausalFlow::iterate_until`, with the
    /// experiments as they stand in the flow's context: changing the context between steps, as
    /// `alternate_context` does, is how a context change reaches the campaign.
    ///
    /// A missing context fails the flow with `MissingContext`; a step's `QuantumError` fails it
    /// with its conversion.
    pub fn step<C, M>(
        flow: CausalFlow<Self, (), Vec<ConfiguredExperiment<R, C>>>,
        model: &M,
        source: &EvidenceSource<R>,
        rules: &CampaignRules<R>,
    ) -> CausalFlow<Self, (), Vec<ConfiguredExperiment<R, C>>>
    where
        C: Clone + ObservedContext,
        M: ResponseModel<R, C>,
    {
        flow.try_step_with(|campaign, _, experiments| {
            let experiments = experiments
                .ok_or_else(|| CausalityError::new(CausalityErrorEnum::MissingContext))?;
            campaign
                .advance(experiments, model, source, rules)
                .map_err(Into::into)
        })
    }

    /// One step over `experiments`, the experiments as configured now. A stopped campaign is
    /// returned as it is.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] when the control stage has no forked worlds, and the
    /// first failure of a prediction, a plan or a stage.
    pub fn advance<C, M>(
        mut self,
        experiments: &[ConfiguredExperiment<R, C>],
        model: &M,
        source: &EvidenceSource<R>,
        rules: &CampaignRules<R>,
    ) -> Result<Self, QuantumError>
    where
        C: Clone + ObservedContext,
        M: ResponseModel<R, C>,
    {
        if self.stop.is_some() {
            return Ok(self);
        }
        if self.control.worlds().is_empty() {
            return Err(QuantumError::CalculationError(
                "a campaign runs over forked worlds; fork the control stage first".into(),
            ));
        }
        let planned = match self.planned.take() {
            Some(p) if !self.drifted(&p, experiments, model, rules)? => Some(p),
            Some(_) => {
                self.replans += 1;
                self.plan(experiments, model, rules)?
            }
            None => self.plan(experiments, model, rules)?,
        };
        let Some(next) = planned else {
            return Ok(self);
        };
        let experiment = experiments[next.experiment]
            .clone()
            .with_shots(next.shots)?;
        self.control = self
            .control
            .observe_experiment(model, &experiment, source)
            .predict_with(model, &experiment)
            .compare(rules.sigmas)
            .adjudicate(rules.objective.floor_bits)
            .charged(next.cost);
        if let Some(e) = self.control.failure() {
            return Err(e.clone());
        }
        self.run.push(next.experiment);
        self.spent += next.cost;
        match self.control.adjudication().map(|a| &a.outcome) {
            Some(Either::Left(survivor)) => {
                self.stop = Some(CampaignStop::Survivor(survivor.clone()));
            }
            Some(Either::Right(Ambiguity::NoSurvivor { .. })) => {
                self.stop = Some(CampaignStop::OutsideTheModel);
            }
            _ => self.planned = self.plan(experiments, model, rules)?,
        }
        Ok(self)
    }

    /// Whether the context moved the planned experiment's predictions by more than the drift, or
    /// no longer offers it.
    fn drifted<C, M>(
        &self,
        planned: &Planned<R>,
        experiments: &[ConfiguredExperiment<R, C>],
        model: &M,
        rules: &CampaignRules<R>,
    ) -> Result<bool, QuantumError>
    where
        M: ResponseModel<R, C>,
    {
        let Some(experiment) = experiments.get(planned.experiment) else {
            return Ok(true);
        };
        let now = self.predictions(experiment, &planned.open, model)?;
        Ok(now
            .iter()
            .zip(&planned.predictions)
            .any(|(&a, &b)| (a - b).abs() > rules.drift))
    }

    /// The predictions of `experiment` for the worlds at `open`.
    fn predictions<C, M>(
        &self,
        experiment: &ConfiguredExperiment<R, C>,
        open: &[usize],
        model: &M,
    ) -> Result<Vec<R>, QuantumError>
    where
        M: ResponseModel<R, C>,
    {
        let worlds = self.control.worlds();
        open.iter()
            .map(|&w| {
                experiment.predict(
                    model,
                    worlds[w].hypothesis(),
                    self.control.plant(),
                    self.control.observables(),
                )
            })
            .collect()
    }

    /// The next experiment over the open worlds and the experiments not yet run, or `None` with
    /// the campaign stopped as unresolvable when no plan covers a pair that involves a world
    /// that holds.
    fn plan<C, M>(
        &mut self,
        experiments: &[ConfiguredExperiment<R, C>],
        model: &M,
        rules: &CampaignRules<R>,
    ) -> Result<Option<Planned<R>>, QuantumError>
    where
        M: ResponseModel<R, C>,
    {
        let worlds = self.control.worlds();
        let floor = rules.objective.floor_bits;
        let slack = Tolerance::<R>::state()
            .threshold(1, floor)
            .expect("the state member answers the single-operator form");
        let holding: Vec<bool> = worlds.iter().map(holds).collect();
        let open: Vec<usize> = (0..worlds.len())
            .filter(|&i| {
                holding[i]
                    || (0..worlds.len())
                        .any(|h| holding[h] && separation(&worlds[h], &worlds[i]) + slack < floor)
            })
            .collect();
        let remaining: Vec<usize> = (0..experiments.len())
            .filter(|e| !self.run.contains(e))
            .collect();
        let entry = if open.len() < 2 || remaining.is_empty() {
            None
        } else {
            let probes = remaining
                .iter()
                .map(|&e| {
                    let x = &experiments[e];
                    let predictions = self.predictions(x, &open, model)?;
                    Experiment::new(x.name(), x.cost(), x.shots(), predictions)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let plan = design(open.len(), &probes, rules.objective)?;
            let involves_holder = |&(a, b): &(usize, usize)| holding[open[a]] || holding[open[b]];
            // Some pair that involves a world that holds must be coverable for a step to help.
            let decisive = (0..open.len())
                .flat_map(|a| ((a + 1)..open.len()).map(move |b| (a, b)))
                .any(|pair| involves_holder(&pair) && !plan.uncovered().contains(&pair));
            let by_cost = |x: &&PlanEntry<R>, y: &&PlanEntry<R>| {
                x.cost.partial_cmp(&y.cost).unwrap_or(Ordering::Equal)
            };
            decisive
                .then(|| {
                    plan.entries()
                        .iter()
                        .filter(|e| e.resolves.iter().any(involves_holder))
                        .min_by(by_cost)
                        .or_else(|| plan.entries().iter().min_by(by_cost))
                        .cloned()
                })
                .flatten()
                .map(|entry| (entry, probes))
        };
        match entry {
            Some((entry, probes)) => Ok(Some(Planned {
                experiment: remaining[entry.experiment],
                shots: entry.shots,
                cost: entry.cost,
                predictions: probes[entry.experiment].predictions().to_vec(),
                open,
            })),
            None => {
                self.stop = Some(CampaignStop::Unresolvable(self.ambiguity(&holding)));
                Ok(None)
            }
        }
    }

    /// The ambiguity the campaign ends in: the last adjudication's, or before any experiment ran,
    /// the candidates that all still hold.
    fn ambiguity(&self, holding: &[bool]) -> Ambiguity<R> {
        match self.control.adjudication().map(|a| &a.outcome) {
            Some(Either::Right(ambiguity)) => ambiguity.clone(),
            _ => {
                let worlds = self.control.worlds();
                if worlds.len() == 1 {
                    Ambiguity::Vacuous { worlds: 1 }
                } else {
                    Ambiguity::SeveralSurvive {
                        survivors: worlds
                            .iter()
                            .zip(holding)
                            .filter(|(_, h)| **h)
                            .map(|(w, _)| w.name().into())
                            .collect(),
                    }
                }
            }
        }
    }
}

/// Whether every reading of `world` agrees with its observation; a world with no reading holds.
fn holds<R: RealField, N, const D: usize>(world: &ControlWorld<R, N, D>) -> bool
where
    N: NaturalNumber,
{
    world
        .readings()
        .iter()
        .all(|r| r.verdict().is_some_and(|v| v.accepted() && !v.is_vacuous()))
}

/// The bits two worlds separate by over the experiments both read.
fn separation<R, N, const D: usize>(a: &ControlWorld<R, N, D>, b: &ControlWorld<R, N, D>) -> R
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber,
{
    a.readings()
        .iter()
        .zip(b.readings())
        .fold(R::zero(), |bits, (x, y)| {
            bits + x.read_out().separation_bits(y.read_out())
        })
}
