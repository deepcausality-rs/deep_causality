/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gravimeter systematics: which effect explains the offset?
//!
//! After the Earth-tide correction an atom gravimeter reads −5 µGal against a trusted reference.
//! Seven systematics could produce that offset, and a passive run cannot tell them apart: each is
//! calibrated to produce it. Each candidate is a mechanism, a phase channel on the interferometer's
//! read-out, and a response model computes what every configuration of the instrument does to it.
//!
//! Each run observes the passive configuration first, through `baseline_with`, which refuses a
//! candidate that cannot produce the observation. It plans the cheapest experiments that separate
//! every remaining pair in instrument time, then runs a sequential campaign that stops as soon as
//! one candidate holds and separates. Every experiment is an instrument context at the time it
//! runs, holding the Earth tide recorded then; after each step the campaign's contexts move to the
//! new time, and a tide that moved the planned experiment's predictions forces a re-plan.
//!
//! Predictions and plans are computed; observations are sampled. The values `constants.rs` marks as
//! placeholders stand for a lab's own measurements.

mod constants;
mod model;
mod model_config;
mod model_types;
mod utils_print;

use deep_causality_algebra::Real;
use deep_causality_core::CausalFlow;
use deep_causality_num::{Float106, Zero, lift_count, to_count};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Campaign, CampaignRules, CampaignStop, EvidenceSource, Experiment, InstrumentTime,
    InterferometerModel, MinCostCover, QclBuilder, QuantumError, design,
};
use std::error::Error;

use crate::constants::{AGREEMENT_SIGMAS, DRIFT, FLOOR_BITS, MICRO_GAL, SEED};
use crate::model::{GravimeterModel, calibrated, hypothesis, output_port, plant};
use crate::model_config::{
    baseline_draws, candidate_name, candidates, experiments, interferometer, passive, physics,
    scenarios,
};
use crate::model_types::{Cause, Physics, Refusal, Scenario, Tide, Verdict, WorldRun};
use crate::utils_print::{print_gates, print_header, print_run};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the
/// calibrations, the predictions, the plans and every read-out recompute at that precision.
pub type FloatType = Float106;

/// The count working type.
pub type NumberType = u64;

/// The complex scalar.
pub type C = Complex<FloatType>;

fn main() -> Result<(), Box<dyn Error>> {
    let physics = physics()?;
    let interferometer = interferometer()?;
    let time = interferometer.instrument_time()?;
    let rules = CampaignRules::new(
        MinCostCover::new(FLOOR_BITS).timed(time),
        AGREEMENT_SIGMAS,
        DRIFT,
    )?;
    print_header(&interferometer, &physics)?;

    let mut runs = Vec::new();
    for scenario in scenarios() {
        let run = run(&scenario, &physics, &interferometer, time, &rules)?;
        print_run(&run, &interferometer, &physics);
        runs.push(run);
    }

    if !print_gates(&runs) {
        return Err("a check the example makes about its own decision failed".into());
    }
    Ok(())
}

/// One scenario: the baseline and its refusals, the static plan over the candidates it leaves,
/// and the sequential campaign.
fn run(
    scenario: &Scenario,
    physics: &Physics,
    interferometer: &InterferometerModel<FloatType>,
    time: InstrumentTime<FloatType>,
    rules: &CampaignRules<FloatType>,
) -> Result<WorldRun, Box<dyn Error>> {
    let offset = scenario.offset_ugal * MICRO_GAL;
    let causes = candidates(offset, scenario.without, physics);
    let truth = Cause {
        name: format!("world: {}", candidate_name(scenario.truth)),
        systematic: scenario.truth,
        size: calibrated(scenario.truth, offset, physics),
    };
    let model = GravimeterModel {
        physics: *physics,
        worlds: causes
            .iter()
            .map(|c| (c.clone(), Tide::Recorded))
            .chain([(truth.clone(), Tide::Actual)])
            .collect(),
    };
    let hypotheses = causes
        .iter()
        .map(|c| hypothesis(&c.name, physics))
        .collect::<Result<Vec<_>, _>>()?;
    let config = QclBuilder::config::<FloatType, NumberType>()
        .over_plant(plant()?, &[output_port()?])
        .mechanisms(&hypotheses)
        .instrument_time(time)
        .seed(SEED)
        .build()?;
    let source = EvidenceSource::Simulated(hypothesis(&truth.name, physics)?);
    let records = scenario.records_tide;

    // The baseline: the passive configuration observed first, refusing what it contradicts.
    let draws = baseline_draws(offset, interferometer, physics)?;
    let e0 = passive(0, records, draws, interferometer, physics)?;
    let baseline = || {
        QclBuilder::control::<FloatType, NumberType, 2, _>(&config).baseline_with(
            &model,
            &e0,
            &source,
            AGREEMENT_SIGMAS,
        )
    };
    let refused: Vec<Refusal> = baseline()
        .finalize()?
        .refused
        .iter()
        .map(|r| Refusal {
            name: r.name().to_string(),
            predicted: r.prediction(),
            observed: r.observed().estimate(),
        })
        .collect();
    let live: Vec<_> = hypotheses
        .iter()
        .filter(|h| !refused.iter().any(|r| r.name == h.name()))
        .cloned()
        .collect();
    let mut run = WorldRun {
        scenario: *scenario,
        refused,
        remaining: live.len(),
        plan: Vec::new(),
        plan_cost: FloatType::zero(),
        plan_complete: false,
        eliminated: Vec::new(),
        campaign: Vec::new(),
        campaign_cost: FloatType::zero(),
        replans: 0,
        campaign_verdict: if live.is_empty() {
            Verdict::OutsideTheModel
        } else {
            Verdict::Unresolved
        },
    };
    if live.len() < 2 {
        return Ok(run);
    }

    // The static plan over the candidates the baseline left, at the time the baseline ends.
    let start = seconds(lift_count::<FloatType>(draws) * time.shot_time())?;
    let family = experiments(start, records, interferometer, physics)?;
    let (the_plant, observables) = (plant()?, [output_port()?]);
    let predicted = |e: &deep_causality_quantum::ConfiguredExperiment<FloatType, _>| {
        live.iter()
            .map(|h| e.predict(&model, h, &the_plant, &observables))
            .collect::<Result<Vec<FloatType>, QuantumError>>()
    };
    let probes = family
        .iter()
        .map(|e| Experiment::new(e.name(), e.cost(), e.shots(), predicted(e)?))
        .collect::<Result<Vec<_>, QuantumError>>()?;
    let plan = design(live.len(), &probes, *rules.objective())?;
    run.plan = plan.entries().iter().map(|e| e.name.clone()).collect();
    run.plan_cost = plan.total_cost();
    run.plan_complete = plan.is_complete();
    // Only a plan that separates every pair identifies a candidate it never singles out.
    if plan.is_complete() && !plan.entries().is_empty() {
        run.eliminated = live
            .iter()
            .enumerate()
            .filter(|&(i, _)| {
                plan.entries().iter().all(|e| {
                    let p = probes[e.experiment].predictions();
                    shared(p) == Some(p[i])
                })
            })
            .map(|(_, h)| h.name().to_string())
            .collect();
    }

    // The campaign, its contexts moved to the time each step ends.
    let campaign = CausalFlow::value(Campaign::new(baseline().fork()))
        .context(family.clone())
        .iterate_until(Campaign::is_stopped, family.len() + 1, |flow| {
            Campaign::step(flow, &model, &source, rules).and_then(|campaign, _, _| {
                let at = seconds(campaign.spent())
                    .and_then(|spent| experiments(start + spent, records, interferometer, physics));
                match at {
                    Ok(at) => CausalFlow::value(campaign).context(at),
                    Err(e) => CausalFlow::fail(e.into()).context(Vec::new()),
                }
            })
        })
        .finish()?;
    run.campaign = campaign
        .experiments_run()
        .iter()
        .map(|&i| family[i].name().to_string())
        .collect();
    run.campaign_cost = campaign.spent();
    run.replans = campaign.replans();
    run.campaign_verdict = match campaign.stop() {
        Some(CampaignStop::Survivor(s)) => Verdict::Survivor(s.name.clone()),
        Some(CampaignStop::OutsideTheModel) => Verdict::OutsideTheModel,
        Some(CampaignStop::Unresolvable(_)) | None => Verdict::Unresolved,
    };
    Ok(run)
}

/// The prediction most candidates share in one experiment, when one prediction is shared by more
/// candidates than any other: what it predicts for every candidate it does not move. A candidate
/// whose prediction is this in every planned experiment is never singled out, so the plan
/// identifies it by elimination. With no such prediction, a tie or no predictions, it is `None`
/// and the experiment identifies no candidate by elimination.
fn shared(predictions: &[FloatType]) -> Option<FloatType> {
    let count = |p: &FloatType| predictions.iter().filter(|q| *q == p).count();
    let most = predictions.iter().map(count).max()?;
    let mut modes = predictions.iter().filter(|p| count(p) == most);
    let mode = *modes.next()?;
    modes.all(|p| *p == mode).then_some(mode)
}

/// `duration` in whole seconds, rounded up.
fn seconds(duration: FloatType) -> Result<u64, QuantumError> {
    to_count(duration.ceil())
        .ok_or_else(|| QuantumError::CalculationError("a duration does not fit a count".into()))
}
