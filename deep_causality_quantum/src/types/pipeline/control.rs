/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::{Observable, QuantumPlant};
use crate::types::decision::{Check, CheckItem, CheckReport};
use crate::types::design::{
    Adjudication, DesignPlan, Experiment, MinCostCover, World, adjudicate, design,
};
use crate::types::pipeline::config::{Config, Mechanisms, PlantSubject, QclBuilder, Structural};
use crate::types::pipeline::ledger::Ledger;
use crate::types::pipeline::spec::Spec;
use crate::types::pipeline::validate::Screened;
use crate::types::qcm::hypothesis::Hypothesis;
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
    read_out: Option<ShotEstimate<R>>,
    verdict: Option<CheckReport<R>>,
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

    /// The last read-out, if observed.
    pub fn read_out(&self) -> Option<&ShotEstimate<R>> {
        self.read_out.as_ref()
    }

    /// The last verdict, if gated.
    pub fn verdict(&self) -> Option<&CheckReport<R>> {
        self.verdict.as_ref()
    }

    /// The last model evaluation, if predicted.
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
    ledger: Ledger<R, N>,
    read_out: Option<ShotEstimate<R>>,
    verdict: Option<CheckReport<R>>,
    worlds: Vec<ControlWorld<R, N, D>>,
    refused: Vec<Refusal<R>>,
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

/// The report `control` finalizes into: the root ledger, every world's ledger side by side, the
/// candidates the baseline refused, the plan and the adjudication.
#[derive(Debug, Clone)]
pub struct ControlReport<R: RealField, N, const D: usize> {
    /// The ledger before the fork.
    pub ledger: Ledger<R, N>,
    /// The worlds, each with its own ledger.
    pub worlds: Vec<ControlWorld<R, N, D>>,
    /// The candidates the baseline refused, in the config's order.
    pub refused: Vec<Refusal<R>>,
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
            ledger,
            read_out: None,
            verdict: None,
            worlds: Vec::new(),
            refused: Vec::new(),
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

    /// One observation of `obs` on `plant`: `shots` charged to `ledger` first, so an overdrawn
    /// budget draws nothing, then sampled at the seed the ledger's experiment count selects.
    fn measure(
        obs: &Observable<R, D>,
        plant: &QuantumPlant<R>,
        ledger: Ledger<R, N>,
        shots: N,
        seed: u64,
    ) -> Result<(ShotEstimate<R>, Ledger<R, N>), QuantumError> {
        let count = shots.to_u64().ok_or_else(|| {
            QuantumError::CalculationError("shot count does not fit a u64".into())
        })?;
        let time = R::from_u64(count).ok_or_else(|| {
            QuantumError::CalculationError("shot count is not representable".into())
        })?;
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
    /// after a fork. With [`baseline`](Self::baseline) the only stages that touch `shots`,
    /// `experiments`, `device_time` and the budget; device time is charged at one unit per shot,
    /// and a draw beyond the remaining budget fails the run.
    ///
    /// Before the fork the read-out is the root's, the baseline `compare` judges predictions
    /// against. After the fork every world is measured on its own plant, which is how a mechanism
    /// world earns its evidence: a world inherits no read-out from the root.
    pub fn observe(mut self, observable: usize, shots: N) -> Self {
        if self.failure.is_some() || self.awaits_baseline("observe") {
            return self;
        }
        let Some(obs) = self.exposed(observable) else {
            return self;
        };
        let seed = self.seed;
        if self.worlds.is_empty() {
            match Self::measure(&obs, &self.plant, self.ledger, shots, seed) {
                Ok((e, l)) => {
                    self.read_out = Some(e);
                    self.ledger = l;
                }
                Err(e) => self.fail(e),
            }
        } else {
            for w in &mut self.worlds {
                match Self::measure(&obs, &w.plant, w.ledger, shots, seed) {
                    Ok((e, l)) => {
                        w.read_out = Some(e);
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
        let observed = match Self::measure(&obs, &self.plant, self.ledger, shots, self.seed) {
            Ok((observed, ledger)) => {
                self.ledger = ledger;
                self.read_out = Some(observed);
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

    /// The last read-out judged against `spec`, on the root or on every world. A world has a
    /// read-out only after `observe` ran on the forked worlds, or after `compare`; a fork alone
    /// leaves every world without one, and `gate` then fails naming `observe`.
    pub fn gate(mut self, spec: Spec<R>) -> Self {
        if self.failure.is_some() {
            return self;
        }
        if self.worlds.is_empty() {
            match &self.read_out {
                Some(e) => self.verdict = Some(spec.judge(e)),
                None => self.fail(QuantumError::CalculationError(
                    "gate needs a read-out; call observe first".into(),
                )),
            }
        } else {
            for w in &mut self.worlds {
                match &w.read_out {
                    Some(e) => w.verdict = Some(spec.judge(e)),
                    None => {
                        self.failure.get_or_insert(QuantumError::CalculationError(
                            "gate needs a read-out in every world; call observe first".into(),
                        ));
                        return self;
                    }
                }
            }
        }
        self
    }

    /// One live world per candidate, built above core by cloning: each world holds its own copy
    /// of the ledger, and none of them was moved into an arm. A mechanism candidate's world
    /// carries the plant evolved by its channel; a structural one's carries the plant as it is.
    ///
    /// A world starts with no read-out and no verdict. The root keeps its own read-out and
    /// verdict as the baseline; a world's evidence comes from `observe` and `gate` after the
    /// fork, or from `compare`, so a world evolved by a mechanism is never adjudicated on a
    /// measurement of the unevolved plant.
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
                read_out: None,
                verdict: None,
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
    /// The root's read-out, taken by `baseline` or by `observe` before the fork, is the baseline. For every world
    /// the prediction becomes the world's read-out through `ShotEstimate::from_probability` at
    /// the baseline's shots, so it carries the shot noise it would have there, and the world's
    /// verdict is one `Check` of `|prediction − baseline|` against `sigmas` standard errors of
    /// the baseline, examined over the baseline's shots. `adjudicate` then separates worlds by
    /// their predictions, and a world's verdict holds when its prediction agrees with the
    /// observation.
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
        let Some(baseline) = self.read_out else {
            self.fail(QuantumError::CalculationError(
                "compare needs the root read-out as the baseline; call observe before fork".into(),
            ));
            return self;
        };
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
            let Some(prediction) = w.prediction else {
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
                    w.read_out = Some(e);
                    w.verdict = Some(CheckReport::new(
                        vec![Check::new(CheckItem::Whole, gap, allowance)],
                        examined,
                    ));
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
        match design(self.candidates.len(), &probes, objective) {
            Ok(plan) => {
                self.ledger = self.ledger.costed(plan.total_cost());
                self.plan = Some(plan);
            }
            Err(e) => self.fail(e),
        }
        self
    }

    /// The worlds' verdicts folded under the verdict law, with the survivor's separation credited
    /// to the root ledger's `bits`.
    pub fn adjudicate(mut self, floor_bits: R) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let mut worlds = Vec::with_capacity(self.worlds.len());
        for w in &self.worlds {
            match (&w.verdict, &w.read_out) {
                (Some(v), Some(e)) => {
                    worlds.push(World::<R, D>::read_out(w.name.clone(), v.clone(), *e))
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
        match adjudicate(&worlds, floor_bits) {
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
            plan: self.plan,
            adjudication: self.adjudication,
        })
    }
}
