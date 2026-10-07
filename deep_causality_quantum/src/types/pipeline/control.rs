/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::{Observable, QuantumPlant};
use crate::types::decision::{Check, CheckItem, CheckReport};
use crate::types::design::{
    Adjudication, CampaignWorld, DesignPlan, Experiment, MinCostCover, adjudicate_campaign, design,
};
use crate::types::pipeline::config::{Config, Mechanisms, PlantSubject, QclBuilder, Structural};
use crate::types::pipeline::configured_experiment::ConfiguredExperiment;
use crate::types::pipeline::evidence_source::{EvidenceSource, Observation, ObservedContext};
use crate::types::pipeline::ledger::Ledger;
use crate::types::pipeline::response::ResponseModel;
use crate::types::pipeline::spec::Spec;
use crate::types::pipeline::validate::Screened;
use crate::types::qcm::hypothesis::Hypothesis;
use crate::types::qpu::born_sampler::sample_probability;
use crate::types::qpu::histogram::{CountHistogram, ShotHistogram};
use crate::types::qpu::prng::SplitMix64;
use crate::types::qpu::shot_estimate::ShotEstimate;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_haft::Either;
use deep_causality_num::{FromPrimitive, NaturalNumber, ToPrimitive};

/// One live world after `fork`: a candidate, the plant under it, and its own ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlWorld<R: RealField, N, const D: usize> {
    name: String,
    hypothesis: Hypothesis<R>,
    plant: QuantumPlant<R>,
    ledger: Ledger<R, N>,
    readings: Vec<Reading<R>>,
    prediction: Option<R>,
    _d: core::marker::PhantomData<[(); D]>,
}

impl<R: RealField, N: NaturalNumber, const D: usize> ControlWorld<R, N, D> {
    /// The candidate this world runs under.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The plant under this candidate.
    pub fn plant(&self) -> &QuantumPlant<R> {
        &self.plant
    }

    /// This world's ledger, to be read beside the others and never joined.
    pub fn ledger(&self) -> &Ledger<R, N> {
        &self.ledger
    }

    /// The readings, one per observed experiment, in campaign order.
    pub fn readings(&self) -> &[Reading<R>] {
        &self.readings
    }

    /// The last read-out, if observed.
    pub fn read_out(&self) -> Option<&ShotEstimate<R>> {
        self.readings.last().map(|r| &r.read_out)
    }

    /// The last reading's verdict, if judged.
    pub fn verdict(&self) -> Option<&CheckReport<R>> {
        self.readings.last().and_then(|r| r.verdict.as_ref())
    }

    /// The model evaluation `compare` has yet to judge, or else the one the last reading was
    /// judged on.
    pub fn prediction(&self) -> Option<R> {
        self.prediction
            .or_else(|| self.readings.last().and_then(|r| r.prediction))
    }
}

/// One reading of one observed experiment: its read-out, its verdict once `gate` or `compare`
/// judged it, and the prediction `compare` judged.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading<R: RealField> {
    sequence: usize,
    experiment: String,
    read_out: ShotEstimate<R>,
    verdict: Option<CheckReport<R>>,
    prediction: Option<R>,
}

impl<R: RealField> Reading<R> {
    fn new(sequence: usize, experiment: String, read_out: ShotEstimate<R>) -> Self {
        Self {
            sequence,
            experiment,
            read_out,
            verdict: None,
            prediction: None,
        }
    }

    /// The experiment read: a configured experiment's name, the baseline's, or the observable's
    /// for a plain observation.
    pub fn experiment(&self) -> &str {
        &self.experiment
    }

    /// The read-out.
    pub fn read_out(&self) -> &ShotEstimate<R> {
        &self.read_out
    }

    /// The verdict, once judged.
    pub fn verdict(&self) -> Option<&CheckReport<R>> {
        self.verdict.as_ref()
    }

    /// The prediction `compare` judged, on a compared reading.
    pub fn prediction(&self) -> Option<R> {
        self.prediction
    }
}

/// A candidate the baseline refused: its prediction, the observation that contradicts it, and the
/// check that decided it, whose item is the candidate's index among the config's candidates.
#[derive(Debug, Clone, PartialEq)]
pub struct Refusal<R> {
    name: String,
    prediction: R,
    observed: ShotEstimate<R>,
    check: Check<R>,
}

impl<R: RealField> Refusal<R> {
    /// The refused candidate.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The read-out it predicted for the baseline experiment.
    pub fn prediction(&self) -> R {
        self.prediction
    }

    /// The baseline read-out.
    pub fn observed(&self) -> &ShotEstimate<R> {
        &self.observed
    }

    /// `|prediction − observed|` against the allowance of `sigmas` standard errors; rejected.
    pub fn check(&self) -> &Check<R> {
        &self.check
    }
}

/// The control stage: observe, gate, fork, predict, compare, design, adjudicate. Failure is
/// sticky and is carried out by `finalize` as the structured error.
///
/// Two paths lead to `adjudicate`, one per kind of candidate.
///
/// Mechanism candidates run `fork → observe → gate → adjudicate`. Each world carries the plant
/// evolved by its channel, `observe` measures every world on its own plant, `gate` judges each
/// read-out against the spec, and `adjudicate` folds the verdicts.
///
/// Structural candidates run `observe → fork → predict → compare → adjudicate`. Every structural
/// world carries the plant unchanged, so a measurement after the fork reads the same in all of
/// them. The root is observed once before the fork as the baseline, `predict` evaluates each
/// candidate's model, and `compare` turns each prediction into that world's read-out and its
/// verdict against the baseline. `adjudicate` then separates the predictions and keeps the world
/// whose prediction agrees with what was observed.
///
/// Configured experiments run `design_with → observe_experiment → fork → predict_with → compare →
/// adjudicate`. A [`ResponseModel`] computes every prediction, `design_with` plans over them,
/// `observe_experiment` runs a planned experiment against its evidence as the root observation,
/// and `predict_with` gives each world its prediction for it.
///
/// A campaign runs `observe_experiment → predict_with → compare` once per experiment after the
/// fork. Every world keeps one [`Reading`] per observed experiment, and `adjudicate` holds a
/// world only when every reading agrees, separating a pair by the sum of its separations over
/// the campaign, as [`adjudicate_campaign`] folds it.
///
/// A config that names a baseline experiment starts either path with [`baseline`](Self::baseline),
/// which observes it first and refuses the candidates it contradicts; `observe`, `fork` and
/// `design` fail until it ran. A config that names evidence funds the ledger with its shots, and
/// every observation draws from them.
pub struct Control<R: RealField, N, const D: usize> {
    plant: QuantumPlant<R>,
    candidates: Vec<(usize, Hypothesis<R>)>,
    declared: usize,
    observables: Vec<Observable<R, D>>,
    probes: Vec<Experiment<R>>,
    pending_baseline: Option<Experiment<R>>,
    seed: u64,
    shot_time: R,
    ledger: Ledger<R, N>,
    sequence: usize,
    root: Option<Reading<R>>,
    worlds: Vec<ControlWorld<R, N, D>>,
    refused: Vec<Refusal<R>>,
    observations: Vec<Observation<R>>,
    plan: Option<DesignPlan<R>>,
    adjudication: Option<Adjudication<R, D>>,
    failure: Option<QuantumError>,
}

/// What may enter `control`: a plant config whose candidates are mechanisms, or a screen. A plant
/// config with structural candidates has no implementation, which is the compiler saying that
/// structural candidates enter `control` only through `validate`.
pub trait ControlSource<R: RealField, N: NaturalNumber, const D: usize> {
    /// The control stage over this source.
    fn into_control(self) -> Control<R, N, D>;
}

impl<R, N, const D: usize> ControlSource<R, N, D> for &Config<R, N, PlantSubject<R, D, Mechanisms>>
where
    R: RealField,
    N: NaturalNumber,
{
    fn into_control(self) -> Control<R, N, D> {
        let candidates = self.subject().candidates().iter().cloned().enumerate();
        Control::new(self, candidates.collect())
    }
}

impl<R, N, const D: usize> ControlSource<R, N, D>
    for &Screened<R, N, PlantSubject<R, D, Structural>>
where
    R: RealField,
    N: NaturalNumber,
{
    fn into_control(self) -> Control<R, N, D> {
        let slots = self.admitted_slots().iter().copied();
        Control::new(
            self.config(),
            slots.zip(self.admitted().iter().cloned()).collect(),
        )
    }
}

impl QclBuilder {
    /// The control stage, over a mechanism config or a screen.
    pub fn control<R, N, const D: usize, S>(source: S) -> Control<R, N, D>
    where
        R: RealField,
        N: NaturalNumber,
        S: ControlSource<R, N, D>,
    {
        source.into_control()
    }
}

/// What one observed experiment yields before it is recorded: the read-out, the counts when the
/// evidence came as counts, and the ledgers with the shots charged, the root's before the fork
/// and every world's after it.
struct Gathered<R, N> {
    read_out: ShotEstimate<R>,
    counts: Option<CountHistogram>,
    ledgers: Vec<Ledger<R, N>>,
}

/// The report `control` finalizes into: the root ledger, every world's ledger side by side, the
/// candidates the baseline refused, the configured experiments observed, the plan and the
/// adjudication.
#[derive(Debug, Clone)]
pub struct ControlReport<R: RealField, N, const D: usize> {
    /// The ledger before the fork.
    pub ledger: Ledger<R, N>,
    /// The worlds, each with its own ledger.
    pub worlds: Vec<ControlWorld<R, N, D>>,
    /// The candidates the baseline refused, in the config's order.
    pub refused: Vec<Refusal<R>>,
    /// The configured experiments observed on the root, in order.
    pub observations: Vec<Observation<R>>,
    /// The design plan, if `design` ran.
    pub plan: Option<DesignPlan<R>>,
    /// The adjudication, if `adjudicate` ran.
    pub adjudication: Option<Adjudication<R, D>>,
}

impl<R: RealField, N: NaturalNumber, const D: usize> Control<R, N, D> {
    /// The control stage over `cfg`'s plant with `candidates`, each paired with its index among
    /// the config's candidates. Named evidence funds the ledger, and its seed is the run's draw
    /// seed.
    fn new<K>(
        cfg: &Config<R, N, PlantSubject<R, D, K>>,
        candidates: Vec<(usize, Hypothesis<R>)>,
    ) -> Self {
        let s = cfg.subject();
        let (ledger, seed) = (Ledger::new(), cfg.seed());
        #[cfg(feature = "qpu")]
        let (ledger, seed) = match cfg.evidence() {
            Some(e) => (ledger.budgeted(e.shot_count()), e.seed_value()),
            None => (ledger, seed),
        };
        Self {
            plant: s.plant().clone(),
            candidates,
            declared: s.candidates().len(),
            observables: s.observables().to_vec(),
            probes: cfg.probes().to_vec(),
            pending_baseline: cfg.baseline().cloned(),
            seed,
            shot_time: cfg.instrument_time().map_or(R::one(), |t| t.shot_time()),
            ledger,
            sequence: 0,
            root: None,
            worlds: Vec::new(),
            refused: Vec::new(),
            observations: Vec::new(),
            plan: None,
            adjudication: None,
            failure: None,
        }
    }
}

impl<R, N, const D: usize> Control<R, N, D>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber + ToPrimitive + FromPrimitive + core::fmt::Debug,
{
    fn fail(&mut self, e: QuantumError) {
        if self.failure.is_none() {
            self.failure = Some(e);
        }
    }

    /// The root's reading of the next observed experiment.
    fn record_root(&mut self, experiment: String, read_out: ShotEstimate<R>) {
        self.sequence += 1;
        self.root = Some(Reading::new(self.sequence, experiment, read_out));
    }

    /// The experiments observed so far, which select the next draw's seed: the root's before the
    /// fork, and after it the worlds', which every observation charges alike.
    fn observed_so_far(&self) -> N {
        self.worlds
            .first()
            .map_or(self.ledger.experiments(), |w| w.ledger.experiments())
    }

    /// Whether the config's baseline is still unobserved, failing `stage` if so: the baseline is
    /// the first observation and precedes the fork and the plan.
    fn awaits_baseline(&mut self, stage: &str) -> bool {
        let Some(b) = &self.pending_baseline else {
            return false;
        };
        let msg = format!(
            "the config names baseline '{}', which is observed first; call baseline before {stage}",
            b.name()
        );
        self.fail(QuantumError::CalculationError(msg));
        true
    }

    /// The observable at `index`, failing the run when the plant does not expose it.
    fn exposed(&mut self, index: usize) -> Option<Observable<R, D>> {
        let obs = self.observables.get(index).cloned();
        if obs.is_none() {
            self.fail(QuantumError::DimensionMismatch(format!(
                "observable {index} is not one the plant exposes ({} declared)",
                self.observables.len()
            )));
        }
        obs
    }

    /// `experiment` read at the live candidates' slots, in their order, after checking that it
    /// predicts for every candidate the config declares.
    fn at_slots(&self, experiment: &Experiment<R>) -> Result<Experiment<R>, QuantumError> {
        let slots: Vec<usize> = self.candidates.iter().map(|(slot, _)| *slot).collect();
        (experiment.predictions().len() == self.declared)
            .then(|| experiment.at_slots(&slots))
            .flatten()
            .ok_or_else(|| {
                QuantumError::DimensionMismatch(format!(
                    "experiment '{}' predicts for {} candidates, the config declares {}",
                    experiment.name(),
                    experiment.predictions().len(),
                    self.declared
                ))
            })
    }

    /// The device time `shots` take at `shot_time` each: seconds when the config names an
    /// instrument time, and one unit per shot otherwise, where `shot_time` is one.
    fn device_time(shots: u64, shot_time: R) -> Result<R, QuantumError> {
        R::from_u64(shots)
            .map(|count| count * shot_time)
            .ok_or_else(|| QuantumError::CalculationError("shot count is not representable".into()))
    }

    /// One observation of `obs` on `plant`: `shots` charged to `ledger` first, at `shot_time`
    /// each, so an overdrawn budget draws nothing, then sampled at the seed the ledger's
    /// experiment count selects.
    fn measure(
        obs: &Observable<R, D>,
        plant: &QuantumPlant<R>,
        ledger: Ledger<R, N>,
        shots: N,
        seed: u64,
        shot_time: R,
    ) -> Result<(ShotEstimate<R>, Ledger<R, N>), QuantumError> {
        let count = shots.to_u64().ok_or_else(|| {
            QuantumError::CalculationError("shot count does not fit a u64".into())
        })?;
        let time = Self::device_time(count, shot_time)?;
        let charged = ledger.observed(shots, time)?;
        let hist = obs.sample(plant, count, Self::draw_seed(seed, ledger.experiments())?)?;
        Ok((ShotEstimate::of_outcome(&hist, 1)?, charged))
    }

    /// The seed of one draw, mixed from the run seed and the full width of the experiments run
    /// so far: the low and the high 64-bit halves of the count enter under distinct odd
    /// constants, so two ledgers that differ only above `2^64` experiments draw at different
    /// seeds.
    fn draw_seed(seed: u64, experiments: N) -> Result<u64, QuantumError> {
        let n = experiments.to_u128().ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "experiment count {experiments:?} does not fit a u128"
            ))
        })?;
        // The two halves of the count; the cast keeps the low 64 bits by design.
        let low = n as u64;
        let high = (n >> 64) as u64;
        Ok(SplitMix64::new(
            seed.wrapping_add(low.wrapping_mul(0x9E37_79B9_7F4A_7C15))
                .wrapping_add(high.wrapping_mul(0xBF58_476D_1CE4_E5B9)),
        )
        .next_u64())
    }

    /// The measurement boundary: `shots` of `observable` on the plant, or on every world's plant
    /// after a fork. This stage, [`baseline`](Self::baseline) and
    /// [`observe_experiment`](Self::observe_experiment) are the only ones that touch `shots`,
    /// `experiments`, `device_time` and the budget; device time is charged at one unit per shot,
    /// and a draw beyond the remaining budget fails the run.
    ///
    /// Before the fork the read-out is the root's, the baseline `compare` judges predictions
    /// against. After the fork every world is measured on its own plant and keeps the read-out as
    /// a new reading, which is how a mechanism world earns its evidence: a world inherits no
    /// read-out from the root.
    pub fn observe(mut self, observable: usize, shots: N) -> Self {
        if self.failure.is_some() || self.awaits_baseline("observe") {
            return self;
        }
        let Some(obs) = self.exposed(observable) else {
            return self;
        };
        let seed = self.seed;
        if self.worlds.is_empty() {
            match Self::measure(&obs, &self.plant, self.ledger, shots, seed, self.shot_time) {
                Ok((e, l)) => {
                    self.record_root(obs.name().into(), e);
                    self.ledger = l;
                }
                Err(e) => self.fail(e),
            }
        } else {
            self.sequence += 1;
            for w in &mut self.worlds {
                match Self::measure(&obs, &w.plant, w.ledger, shots, seed, self.shot_time) {
                    Ok((e, l)) => {
                        w.readings
                            .push(Reading::new(self.sequence, obs.name().into(), e));
                        w.ledger = l;
                    }
                    Err(e) => {
                        self.failure.get_or_insert(e);
                        return self;
                    }
                }
            }
        }
        self
    }

    /// The config's baseline experiment observed on the root as the run's first observation, at
    /// the experiment's shots of `observable`, and every candidate whose prediction for it the
    /// observation contradicts refused before the fork and the plan.
    ///
    /// A candidate is refused when `|prediction − observed|` exceeds `sigmas` standard errors of
    /// the observed estimate, the test `compare` applies to a world. Its prediction is read at its
    /// index among the config's candidates, so a candidate the screen dropped does not shift the
    /// others. The refusals go to the report with what decided them, and the observation stays
    /// the root's read-out, the baseline `compare` judges predictions against.
    ///
    /// Fails with `CalculationError` when there is no baseline to observe (the config names
    /// none, or the stage already ran), on a `sigmas` that is not finite or is negative, and when
    /// the experiment's shots do not fit the count width; with `DimensionMismatch` when the
    /// experiment predicts for a different number of candidates than the config declares; and as
    /// `observe` fails.
    pub fn baseline(mut self, observable: usize, sigmas: R) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let Some(experiment) = self.pending_baseline.take() else {
            self.fail(QuantumError::CalculationError(
                "baseline has no experiment to observe: the config names none, or the stage \
                 already ran"
                    .into(),
            ));
            return self;
        };
        if !sigmas.is_finite() || sigmas < R::zero() {
            self.fail(QuantumError::CalculationError(format!(
                "baseline needs a finite, non-negative sigmas, got {sigmas:?}"
            )));
            return self;
        }
        let predicted = match self.at_slots(&experiment) {
            Ok(predicted) => predicted,
            Err(e) => {
                self.fail(e);
                return self;
            }
        };
        let Some(obs) = self.exposed(observable) else {
            return self;
        };
        let Some(shots) = N::from_u64(experiment.shots()) else {
            self.fail(QuantumError::CalculationError(format!(
                "baseline '{}' takes {} shots, which do not fit the count width",
                experiment.name(),
                experiment.shots()
            )));
            return self;
        };
        let observed = match Self::measure(
            &obs,
            &self.plant,
            self.ledger,
            shots,
            self.seed,
            self.shot_time,
        ) {
            Ok((observed, ledger)) => {
                self.ledger = ledger;
                self.record_root(experiment.name().into(), observed);
                observed
            }
            Err(e) => {
                self.fail(e);
                return self;
            }
        };
        let allowance = sigmas * observed.standard_error();
        let candidates = core::mem::take(&mut self.candidates);
        for ((slot, h), &prediction) in candidates.into_iter().zip(predicted.predictions()) {
            let gap = (prediction - observed.estimate()).abs();
            let check = Check::new(CheckItem::Index(slot), gap, allowance);
            if check.accepted {
                self.candidates.push((slot, h));
            } else {
                self.refused.push(Refusal {
                    name: h.name().into(),
                    prediction,
                    observed,
                    check,
                });
            }
        }
        self
    }

    /// The root's read-out judged against `spec` before the fork; after it, every world's
    /// readings that no stage has judged yet. A world has an unjudged reading only after
    /// `observe` ran on the forked worlds; a fork alone leaves every world without one, and
    /// `gate` then fails naming `observe`.
    pub fn gate(mut self, spec: Spec<R>) -> Self {
        if self.failure.is_some() {
            return self;
        }
        if self.worlds.is_empty() {
            match &mut self.root {
                Some(root) => root.verdict = Some(spec.judge(&root.read_out)),
                None => self.fail(QuantumError::CalculationError(
                    "gate needs a read-out; call observe first".into(),
                )),
            }
        } else {
            if self
                .worlds
                .iter()
                .any(|w| w.readings.iter().all(|r| r.verdict.is_some()))
            {
                self.fail(QuantumError::CalculationError(
                    "gate needs an unjudged read-out in every world; call observe first".into(),
                ));
                return self;
            }
            for w in &mut self.worlds {
                for r in w.readings.iter_mut().filter(|r| r.verdict.is_none()) {
                    r.verdict = Some(spec.judge(&r.read_out));
                }
            }
        }
        self
    }

    /// One live world per candidate, built above core by cloning: each world holds its own copy
    /// of the ledger, and none of them was moved into an arm. A mechanism candidate's world
    /// carries the plant evolved by its channel; a structural one's carries the plant as it is.
    ///
    /// A world starts with no reading. The root keeps its own read-out and verdict as the
    /// baseline; a world's evidence comes from `observe` and `gate` after the fork, or from
    /// `compare`, so a world evolved by a mechanism is never adjudicated on a measurement of the
    /// unevolved plant.
    ///
    /// Fails with `CalculationError` before the config's baseline was observed, and when there
    /// is no candidate to fork: the screen admitted none, the baseline refused every one, or the
    /// config declared none.
    pub fn fork(mut self) -> Self {
        if self.failure.is_some() || self.awaits_baseline("fork") {
            return self;
        }
        if self.candidates.is_empty() {
            self.fail(QuantumError::CalculationError(
                "fork has no candidate to fork: the screen admitted none, the baseline refused \
                 every one, or the config declared none"
                    .into(),
            ));
            return self;
        }
        let mut worlds = Vec::with_capacity(self.candidates.len());
        for (_, h) in &self.candidates {
            let plant = match h.channel() {
                Some(ch) => match self.plant.evolve(ch) {
                    Ok(p) => p,
                    Err(e) => {
                        self.fail(e);
                        return self;
                    }
                },
                None => self.plant.clone(),
            };
            worlds.push(ControlWorld {
                name: h.name().into(),
                hypothesis: h.clone(),
                plant,
                ledger: self.ledger,
                readings: Vec::new(),
                prediction: None,
                _d: core::marker::PhantomData,
            });
        }
        self.worlds = worlds;
        self
    }

    /// A model evaluation in every world: the Born read-out of `observable` on the world's plant
    /// for a mechanism candidate, or the joint operator against the observable's projector for a
    /// structural one. Counted on `predictions`, never billed.
    pub fn predict(mut self, observable: usize) -> Self {
        if self.failure.is_some() {
            return self;
        }
        if self.worlds.is_empty() {
            self.fail(QuantumError::CalculationError(
                "predict evaluates forked worlds; call fork first".into(),
            ));
            return self;
        }
        let Some(obs) = self.observables.get(observable).cloned() else {
            self.fail(QuantumError::DimensionMismatch(format!(
                "observable {observable} is not one the plant exposes"
            )));
            return self;
        };
        for w in &mut self.worlds {
            let value = if w.hypothesis.is_mechanism() {
                obs.read_out(&w.plant)
            } else {
                w.hypothesis.evaluate(obs.projection().matrix())
            };
            match value.and_then(|v| w.ledger.predicted().map(|l| (v, l))) {
                Ok((v, l)) => {
                    w.prediction = Some(v);
                    w.ledger = l;
                }
                Err(e) => {
                    self.failure.get_or_insert(e);
                    return self;
                }
            }
        }
        self
    }

    /// Each world's prediction turned into evidence against the root's read-out.
    ///
    /// The root's latest read-out, taken by `baseline`, by `observe` before the fork or by
    /// `observe_experiment`, is the baseline. For every world the prediction becomes the world's
    /// reading of that observation through `ShotEstimate::from_probability` at the baseline's
    /// shots, so it carries the shot noise it would have there, and the reading's verdict is one
    /// `Check` of `|prediction − baseline|` against `sigmas` standard errors of the baseline,
    /// examined over the baseline's shots. The prediction is consumed: the next `compare` needs a
    /// new one, and a second `compare` of the same observation replaces the reading rather than
    /// adding one. `adjudicate` then separates worlds by their predictions, and a world holds
    /// when its predictions agree with every observation.
    ///
    /// A mechanism world with a prediction goes through here as well; nothing forbids it.
    ///
    /// Fails with `CalculationError` on a `sigmas` that is not finite or is negative, when the
    /// root has no read-out (`observe` before `fork`), when there are no worlds (`fork`), or when
    /// a world has no prediction (`predict`); the message names the missing step. A prediction
    /// outside `[0, 1]` is refused by `ShotEstimate::from_probability` with its own error.
    pub fn compare(mut self, sigmas: R) -> Self {
        if self.failure.is_some() {
            return self;
        }
        if !sigmas.is_finite() || sigmas < R::zero() {
            self.fail(QuantumError::CalculationError(format!(
                "compare needs a finite, non-negative sigmas, got {sigmas:?}"
            )));
            return self;
        }
        let Some(root) = self.root.clone() else {
            self.fail(QuantumError::CalculationError(
                "compare needs the root read-out as the baseline; call observe before fork".into(),
            ));
            return self;
        };
        let baseline = root.read_out;
        if self.worlds.is_empty() {
            self.fail(QuantumError::CalculationError(
                "compare judges forked worlds; call fork first".into(),
            ));
            return self;
        }
        let allowance = sigmas * baseline.standard_error();
        let examined = match usize::try_from(baseline.shots()) {
            Ok(examined) => examined,
            Err(_) => {
                self.fail(QuantumError::CalculationError(format!(
                    "baseline shot count {} does not fit usize",
                    baseline.shots()
                )));
                return self;
            }
        };
        for w in &mut self.worlds {
            let Some(prediction) = w.prediction.take() else {
                self.failure
                    .get_or_insert(QuantumError::CalculationError(format!(
                        "world '{}' has no prediction; call predict before compare",
                        w.name
                    )));
                return self;
            };
            match ShotEstimate::from_probability(prediction, baseline.shots()) {
                Ok(e) => {
                    let gap = (prediction - baseline.estimate()).abs();
                    let reading = Reading {
                        verdict: Some(CheckReport::new(
                            vec![Check::new(CheckItem::Whole, gap, allowance)],
                            examined,
                        )),
                        prediction: Some(prediction),
                        ..Reading::new(root.sequence, root.experiment.clone(), e)
                    };
                    match w.readings.iter_mut().find(|r| r.sequence == root.sequence) {
                        Some(existing) => *existing = reading,
                        None => w.readings.push(reading),
                    }
                }
                Err(e) => {
                    self.failure.get_or_insert(e);
                    return self;
                }
            }
        }
        self
    }

    /// The design plan over the live candidates and the probe family, its cost committed to the
    /// root ledger. Each probe's predictions are read at the live candidates' indices among the
    /// config's candidates, so a candidate the screen dropped or the baseline refused takes its
    /// prediction with it.
    ///
    /// Fails with `CalculationError` before the config's baseline was observed; with
    /// `DimensionMismatch` when a probe predicts for a different number of candidates than the
    /// config declares; and as [`design`] fails.
    pub fn design(mut self, objective: MinCostCover<R>) -> Self {
        if self.failure.is_some() || self.awaits_baseline("design") {
            return self;
        }
        let probes = match self
            .probes
            .iter()
            .map(|probe| self.at_slots(probe))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(probes) => probes,
            Err(e) => {
                self.fail(e);
                return self;
            }
        };
        self.plan_over(&probes, objective);
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
    /// [`design`] fails.
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

    /// The cover of the live candidates' pairs by `probes`, committed to the root ledger.
    fn plan_over(&mut self, probes: &[Experiment<R>], objective: MinCostCover<R>) {
        match design(self.candidates.len(), probes, objective) {
            Ok(plan) => {
                self.ledger = self.ledger.costed(plan.total_cost());
                self.plan = Some(plan);
            }
            Err(e) => self.fail(e),
        }
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
        for w in &mut self.worlds {
            let predicted = experiment
                .predict(model, &w.hypothesis, &self.plant, &self.observables)
                .and_then(|v| w.ledger.predicted().map(|l| (v, l)));
            match predicted {
                Ok((v, l)) => {
                    w.prediction = Some(v);
                    w.ledger = l;
                }
                Err(e) => {
                    self.failure.get_or_insert(e);
                    return self;
                }
            }
        }
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
                if !p.is_finite() || p < R::zero() || p > R::one() {
                    return Err(QuantumError::NormalizationError(format!(
                        "truth '{}' predicts {p:?} for experiment '{}', not a probability",
                        truth.name(),
                        experiment.name()
                    )));
                }
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

    /// The worlds' readings folded as a campaign by [`adjudicate_campaign`]: a world holds when
    /// every reading's verdict holds, and a pair separates by the sum over the experiments both
    /// read. The survivor's separation is credited to the root ledger's `bits`. A world with one
    /// reading adjudicates as `adjudicate` folds a single read-out.
    ///
    /// Fails with `CalculationError` when a world has no reading or a reading no stage judged.
    pub fn adjudicate(mut self, floor_bits: R) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let mut worlds = Vec::with_capacity(self.worlds.len());
        for w in &self.worlds {
            let readings: Option<Vec<_>> = w
                .readings
                .iter()
                .map(|r| r.verdict.clone().map(|v| (v, r.read_out)))
                .collect();
            match readings {
                Some(readings) if !readings.is_empty() => {
                    worlds.push(CampaignWorld::new(w.name.clone(), readings))
                }
                _ => {
                    self.fail(QuantumError::CalculationError(format!(
                        "world '{}' reaches adjudicate without an observed and gated read-out",
                        w.name
                    )));
                    return self;
                }
            }
        }
        match adjudicate_campaign::<R, D>(&worlds, floor_bits) {
            Ok(a) => {
                if let Either::Left(s) = &a.outcome {
                    self.ledger = self.ledger.separated(s.separation_bits);
                }
                self.adjudication = Some(a);
            }
            Err(e) => self.fail(e),
        }
        self
    }

    /// The report, or the first structured error a stage raised.
    ///
    /// # Errors
    ///
    /// The first stage failure.
    pub fn finalize(self) -> Result<ControlReport<R, N, D>, QuantumError> {
        if let Some(e) = self.failure {
            return Err(e);
        }
        Ok(ControlReport {
            ledger: self.ledger,
            worlds: self.worlds,
            refused: self.refused,
            observations: self.observations,
            plan: self.plan,
            adjudication: self.adjudication,
        })
    }
}
