/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::{Observable, QuantumPlant};
use crate::types::decision::CheckReport;
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

/// The control stage: observe, gate, fork, predict, design, adjudicate. Failure is sticky and is
/// carried out by `finalize` as the structured error.
pub struct Control<R: RealField, N, const D: usize> {
    plant: QuantumPlant<R>,
    candidates: Vec<Hypothesis<R>>,
    observables: Vec<Observable<R, D>>,
    probes: Vec<Experiment<R>>,
    seed: u64,
    ledger: Ledger<R, N>,
    read_out: Option<ShotEstimate<R>>,
    verdict: Option<CheckReport<R>>,
    worlds: Vec<ControlWorld<R, N, D>>,
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
        let s = self.subject();
        Control::new(
            s.plant().clone(),
            s.candidates().to_vec(),
            s.observables().to_vec(),
            self.probes().to_vec(),
            self.seed(),
        )
    }
}

impl<R, N, const D: usize> ControlSource<R, N, D>
    for &Screened<R, N, PlantSubject<R, D, Structural>>
where
    R: RealField,
    N: NaturalNumber,
{
    fn into_control(self) -> Control<R, N, D> {
        let s = self.config().subject();
        Control::new(
            s.plant().clone(),
            self.admitted().to_vec(),
            s.observables().to_vec(),
            self.config().probes().to_vec(),
            self.config().seed(),
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
/// plan and the adjudication.
#[derive(Debug, Clone)]
pub struct ControlReport<R: RealField, N, const D: usize> {
    /// The ledger before the fork.
    pub ledger: Ledger<R, N>,
    /// The worlds, each with its own ledger.
    pub worlds: Vec<ControlWorld<R, N, D>>,
    /// The design plan, if `design` ran.
    pub plan: Option<DesignPlan<R>>,
    /// The adjudication, if `adjudicate` ran.
    pub adjudication: Option<Adjudication<R, D>>,
}

impl<R: RealField, N: NaturalNumber, const D: usize> Control<R, N, D> {
    fn new(
        plant: QuantumPlant<R>,
        candidates: Vec<Hypothesis<R>>,
        observables: Vec<Observable<R, D>>,
        probes: Vec<Experiment<R>>,
        seed: u64,
    ) -> Self {
        Self {
            plant,
            candidates,
            observables,
            probes,
            seed,
            ledger: Ledger::new(),
            read_out: None,
            verdict: None,
            worlds: Vec::new(),
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

    fn draw_seed(seed: u64, experiments: N) -> u64 {
        let n = experiments.to_u64().unwrap_or(u64::MAX);
        SplitMix64::new(seed.wrapping_add(n.wrapping_mul(0x9E37_79B9_7F4A_7C15))).next_u64()
    }

    /// The measurement boundary: `shots` of `observable` on the plant, or on every world's plant
    /// after a fork. The only stage that touches `shots`, `experiments` and `device_time`; device
    /// time is charged at one unit per shot.
    pub fn observe(mut self, observable: usize, shots: N) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let Some(obs) = self.observables.get(observable).cloned() else {
            self.fail(QuantumError::DimensionMismatch(format!(
                "observable {observable} is not one the plant exposes ({} declared)",
                self.observables.len()
            )));
            return self;
        };
        let Some(count) = shots.to_u64() else {
            self.fail(QuantumError::CalculationError(
                "shot count does not fit a u64".into(),
            ));
            return self;
        };
        let Some(time) = R::from_u64(count) else {
            self.fail(QuantumError::CalculationError(
                "shot count is not representable".into(),
            ));
            return self;
        };
        let seed = self.seed;
        let run = |plant: &QuantumPlant<R>,
                   ledger: Ledger<R, N>|
         -> Result<(ShotEstimate<R>, Ledger<R, N>), QuantumError> {
            let hist = obs.sample(plant, count, Self::draw_seed(seed, ledger.experiments()))?;
            let estimate = ShotEstimate::of_outcome(&hist, 1)?;
            Ok((estimate, ledger.observed(shots, time)?))
        };
        if self.worlds.is_empty() {
            match run(&self.plant, self.ledger) {
                Ok((e, l)) => {
                    self.read_out = Some(e);
                    self.ledger = l;
                }
                Err(e) => self.fail(e),
            }
        } else {
            for w in &mut self.worlds {
                match run(&w.plant, w.ledger) {
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

    /// The last read-out judged against `spec`, on the root or on every world.
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
    /// of the ledger, the read-out and the verdict, and none of them was moved into an arm. A
    /// mechanism candidate's world carries the plant evolved by its channel; a structural one's
    /// carries the plant as it is.
    pub fn fork(mut self) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let mut worlds = Vec::with_capacity(self.candidates.len());
        for h in &self.candidates {
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
                read_out: self.read_out,
                verdict: self.verdict.clone(),
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

    /// The design plan over the candidates and the probe family, its cost committed to the
    /// root ledger.
    pub fn design(mut self, objective: MinCostCover<R>) -> Self {
        if self.failure.is_some() {
            return self;
        }
        match design(self.candidates.len(), &self.probes, objective) {
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
            plan: self.plan,
            adjudication: self.adjudication,
        })
    }
}
