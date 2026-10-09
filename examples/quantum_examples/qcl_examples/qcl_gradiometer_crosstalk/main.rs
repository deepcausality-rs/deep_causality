/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gradiometer crosstalk: is the correlation between the two clouds benign?
//!
//! A gradiometer runs two atom clouds off one laser and one mirror, and their read-outs fluctuate
//! together. If the shared phase is all there is, differential extraction removes it and the
//! gradient is unbiased. If light from one cloud reaches the other's detection signal, or the
//! platform rotates, the correlation biases the gradient. Each of the four candidates reproduces
//! the passive read-out exactly, so passive data cannot tell them apart.
//!
//! For each world the observations are drawn from, the example fits the candidates to that
//! world's passive read-out, screens them, plans the cheapest experiments that separate every
//! pair, runs the plan, and runs a sequential campaign that stops as soon as the evidence decides.
//! The survivor's consequence is the gradient bias it implies: the instrument's reading of its
//! world, less the same reading with the mechanism removed. The two-way leak is no candidate, and
//! the static plan reports it as outside the model.
//!
//! Predictions and plans are computed; observations are sampled. The leak, the dark background
//! and the setup times are placeholders for a lab's own measurements; `constants.rs` names them.

mod constants;
mod model;
mod model_config;
mod model_types;
mod utils_print;

use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum};
use deep_causality_haft::Either;
use deep_causality_num::Float106;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Adjudication, Ambiguity, Campaign, CampaignRules, CampaignStop, CommutatorTolerance,
    ConfiguredExperiment, DesignPlan, EvidenceSource, MinCostCover, PlantSubject, QclBuilder,
    Screened, Structural,
};
use std::error::Error;

use crate::constants::{
    AGREEMENT_SIGMAS, ATOMS_A, ATOMS_B, DRIFT, FLOOR_BITS, ROTATION, SEED, SHARED_PHASE, SIGNAL_A,
    SIGNAL_B,
};
use crate::model::{GradiometerModel, benign_reading, hypothesis, passive_joint};
use crate::model_config::{
    CANDIDATES, both_excited, candidates, experiments, instrument, plant, worlds,
};
use crate::model_types::{Cause, Consequence, Instrument, Setting, Verdict, WorldRun};
use crate::utils_print::{print_consequence, print_gates, print_header, print_run, print_world};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the fits, the
/// predictions, the plan and every read-out recompute at that precision.
pub type FloatType = Float106;

/// The count working type.
pub type NumberType = u64;

/// The complex scalar.
pub type C = Complex<FloatType>;

/// The screen a structural configuration ends in.
pub type Screen = Screened<FloatType, NumberType, PlantSubject<FloatType, 4, Structural>>;

fn main() -> Result<(), Box<dyn Error>> {
    let (instrument, interferometer) = instrument()?;
    let model = GradiometerModel { instrument };
    let experiments = experiments()?;
    let objective = MinCostCover::new(FLOOR_BITS).timed(interferometer.instrument_time()?);
    let rules = CampaignRules::new(objective, AGREEMENT_SIGMAS, DRIFT)?;
    print_header(&instrument, &interferometer)?;

    let mut runs = Vec::new();
    for world in worlds(&instrument) {
        let truth = hypothesis(&world, &instrument)?;
        let causes = candidates(&passive_joint(&truth)?, &instrument)?;
        let screen = screen(&causes, &instrument)?;
        let plan = QclBuilder::control::<FloatType, NumberType, 4, _>(&screen)
            .design_with(&model, &experiments, objective)
            .finalize()?
            .plan
            .ok_or("design should produce a plan")?;
        print_world(&world, &screen, &model, &experiments, &plan)?;

        let source = EvidenceSource::Simulated(truth);
        let planned = run_plan(&screen, &plan, &experiments, &model, &source)?;
        let (campaign, ran, spent) = run_campaign(&screen, &experiments, &model, &source, &rules)?;
        let run = WorldRun {
            world: world.name,
            expected: CANDIDATES
                .iter()
                .find(|(_, m)| *m == world.mechanism)
                .map(|(name, _)| *name),
            campaign,
            campaign_experiments: ran,
            campaign_cost: spent,
            plan: planned,
            plan_cost: plan.total_cost(),
            plan_complete: plan.is_complete(),
            plan_settings: plan
                .entries()
                .iter()
                .map(|e| *experiments[e.experiment].configuration())
                .collect(),
        };
        print_run(&run);
        if let Verdict::Survivor(name) = &run.campaign
            && let Some(cause) = causes.iter().find(|c| c.name == name)
        {
            print_consequence(&consequence(cause, &instrument)?, &instrument);
        }
        runs.push(run);
    }

    if !print_gates(&runs) {
        return Err("a check the example makes about its own decision failed".into());
    }
    Ok(())
}

/// The candidates as factorizations, configured and screened: each a normalised process, Markov,
/// and free of a C₃ in the structure its own supports encode.
fn screen(causes: &[Cause], instrument: &Instrument) -> Result<Screen, Box<dyn Error>> {
    let hypotheses = causes
        .iter()
        .map(|c| hypothesis(c, instrument))
        .collect::<Result<Vec<_>, _>>()?;
    let config = QclBuilder::config::<FloatType, NumberType>()
        .over_plant(plant()?, &[both_excited()?])
        .candidates(&hypotheses)
        .seed(SEED)
        .build()?;
    let systems = [ATOMS_A, ATOMS_B, SIGNAL_A, SIGNAL_B, SHARED_PHASE, ROTATION];
    Ok(QclBuilder::validate(&config)
        .check_normalization()
        .check_markov(&CommutatorTolerance::<FloatType>::default())
        .check_decomposable(&systems, &systems)
        .finalize()?)
}

/// Every planned experiment run against `source`, then the campaign adjudication of all of them.
fn run_plan(
    screen: &Screen,
    plan: &DesignPlan<FloatType>,
    experiments: &[ConfiguredExperiment<FloatType, Setting>],
    model: &GradiometerModel,
    source: &EvidenceSource<FloatType>,
) -> Result<Verdict, Box<dyn Error>> {
    let mut control = QclBuilder::control::<FloatType, NumberType, 4, _>(screen).fork();
    for entry in plan.entries() {
        let experiment = experiments[entry.experiment]
            .clone()
            .with_shots(entry.shots)?;
        control = control
            .observe_experiment(model, &experiment, source)
            .predict_with(model, &experiment)
            .compare(AGREEMENT_SIGMAS);
    }
    let report = control.adjudicate(FLOOR_BITS).finalize()?;
    Ok(verdict(report.adjudication.as_ref()))
}

/// The sequential campaign: the cheapest planned experiment first, until one candidate holds and
/// separates, none holds, or nothing left separates the rest. Returns the verdict, the experiments
/// run and what they cost.
fn run_campaign(
    screen: &Screen,
    experiments: &[ConfiguredExperiment<FloatType, Setting>],
    model: &GradiometerModel,
    source: &EvidenceSource<FloatType>,
    rules: &CampaignRules<FloatType>,
) -> Result<(Verdict, Vec<String>, FloatType), Box<dyn Error>> {
    let campaign = CausalFlow::value(Campaign::new(
        QclBuilder::control::<FloatType, NumberType, 4, _>(screen).fork(),
    ))
    .context(experiments.to_vec())
    .iterate_until(Campaign::is_stopped, experiments.len() + 1, |flow| {
        Campaign::step(flow, model, source, rules)
    })
    .finish()?;
    let ran = campaign
        .experiments_run()
        .iter()
        .map(|&i| experiments[i].name().to_string())
        .collect();
    let verdict = match campaign.stop() {
        Some(CampaignStop::Survivor(s)) => Verdict::Survivor(s.name.clone()),
        Some(CampaignStop::OutsideTheModel) => Verdict::OutsideTheModel,
        Some(CampaignStop::Unresolvable(_)) | None => Verdict::Unresolved,
    };
    Ok((verdict, ran, campaign.spent()))
}

/// The verdict an adjudication reached.
fn verdict(adjudication: Option<&Adjudication<FloatType, 4>>) -> Verdict {
    match adjudication.map(|a| &a.outcome) {
        Some(Either::Left(s)) => Verdict::Survivor(s.name.clone()),
        Some(Either::Right(Ambiguity::NoSurvivor { .. })) => Verdict::OutsideTheModel,
        _ => Verdict::Unresolved,
    }
}

/// What the survivor implies. The flow reads the instrument's differential phase in the
/// survivor's world, alternates the context to the same world with the mechanism removed and reads
/// it again, then branches on whether the mechanism biases the gradient to its corrective action.
fn consequence(cause: &Cause, instrument: &Instrument) -> Result<Consequence, Box<dyn Error>> {
    let read = |world: Option<&Cause>| -> Result<FloatType, CausalityError> {
        let world = world.ok_or_else(|| CausalityError::new(CausalityErrorEnum::MissingContext))?;
        Ok(benign_reading(
            &passive_joint(&hypothesis(world, instrument)?)?,
            instrument,
        )?)
    };
    Ok(CausalFlow::value(())
        .context(cause.clone())
        .try_step_with(|(), _, world| read(world))
        .alternate_context(cause.without_mechanism())
        .try_step_with(|reading, _, world| Ok(Consequence::of(cause, reading, read(world)?)))
        .branch_with(
            |c, _, _| c.mechanism.biases_gradient(),
            |flow| flow.map(Consequence::corrected),
            |flow| flow,
        )
        .finish()?)
}
