/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Crosstalk attribution: direct cause or common cause.
//!
//! Two qubits' errors are correlated beyond independence. Three structures fit the passive
//! observation exactly, because they are Markov equivalent, and a fourth is a cycle. This is the
//! keystone example: the only one that runs `validate` and then `control` on one problem, so it
//! is where the hand-off between the halves is exercised.
//!
//!   * `build()` refuses the cyclic candidate as `CyclicStructureUnsupported`, by decision and
//!     before any check runs. Under Definition 3.1 the C₃ criterion does not reject it, so if it is
//!     to be kept out, the builder has to do it and say why.
//!   * `validate` screens the other three: each is a normalised process, Markov, and free of a
//!     C₃ in the structure its own supports encode; all three are admitted.
//!   * `control` takes the screen, and `design_with` computes every experiment's predicted
//!     read-out under every candidate from the response model and returns the cheapest plan: the
//!     two targeted interventions at cost 2.
//!   * A campaign runs the plan against observations drawn from H₁, cheapest experiment first,
//!     and stops as soon as one candidate holds and separates from the others. Each world's
//!     prediction is judged against the observation; no commutation test runs, because a
//!     threshold on a real quantity is a classical proposition.
//!
//! Predictions and the plan are computed; the observations are sampled. The physics behind each
//! factor and each response is a modelling assumption stated in `model.rs`.

mod constants;
mod model;

use deep_causality_core::CausalFlow;
use deep_causality_haft::Either;
use deep_causality_num::{Float106, lower};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Campaign, CampaignRules, CampaignStop, CommutatorTolerance, EvidenceSource, MinCostCover,
    QclBuilder, QuantumErrorEnum,
};

use crate::constants::{
    AGREEMENT_SIGMAS, CANDIDATE_COUNT, DRIFT, EXPECTED_PLAN_COST, FLOOR_BITS, SEED, SHOTS, ZERO,
};
use crate::model::{
    CrosstalkModel, e1_projector, e2_projector, experiments, h1_direct_q1_to_q2,
    h2_direct_q2_to_q1, h3_common_bath, h4_cyclic, plant, systems,
};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the factors,
/// the separations, the plan's costs and every read-out recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

/// The count working type.
pub type NumberType = u64;

/// The complex scalar.
pub type C = Complex<FloatType>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let floor_bits = FLOOR_BITS;
    println!("=== Crosstalk attribution: direct cause or common cause ===");
    println!("precision: {}", core::any::type_name::<FloatType>());
    println!(
        "shots: {SHOTS}   floor: {} bits   seed: {SEED}\n",
        lower(FLOOR_BITS)
    );

    // -- build(): the cyclic candidate is refused, by decision --------------------------
    println!("[build] four declared structures");
    let refused = QclBuilder::config::<FloatType, NumberType>()
        .over_plant(plant()?, &[e2_projector()?, e1_projector()?])
        .candidates(&[
            h1_direct_q1_to_q2()?,
            h2_direct_q2_to_q1()?,
            h3_common_bath()?,
            h4_cyclic()?,
        ])
        .seed(SEED)
        .build();
    match refused {
        Ok(_) => return Err("a cyclic candidate built; build() should refuse it".into()),
        Err(e) => match e.0 {
            QuantumErrorEnum::CyclicStructureUnsupported(msg) => {
                println!("    ✓ refused at build(): {msg}");
            }
            other => return Err(format!("refused for the wrong reason: {other:?}").into()),
        },
    }

    let candidates = [
        h1_direct_q1_to_q2()?,
        h2_direct_q2_to_q1()?,
        h3_common_bath()?,
    ];
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_plant(plant()?, &[e2_projector()?, e1_projector()?])
        .candidates(&candidates)
        .seed(SEED)
        .build()?;

    // -- validate: normalisation, Markov, then C₃ on each candidate's own structure -----
    println!("\n[validate] three acyclic structures");
    let sys = systems();
    let screened = QclBuilder::validate(&cfg)
        .check_normalization()
        .check_markov(&CommutatorTolerance::<FloatType>::default())
        .check_decomposable(&sys, &sys)
        .finalize()?;
    for (name, report) in screened.stages() {
        println!(
            "    {name:<20} {:?}  examined {}",
            report.verdict(),
            report.examined()
        );
    }
    for h in screened.admitted() {
        println!("    admitted: {}", h.name());
    }
    let all_three_admitted = screened.admitted().len() == CANDIDATE_COUNT;
    println!(
        "    all {CANDIDATE_COUNT} acyclic candidates admitted: {}",
        yes_no(all_three_admitted)
    );

    // -- design: every prediction computed, then the minimum-cost cover ------------------
    let exps = experiments()?;
    let planned = QclBuilder::control::<FloatType, NumberType, 4, _>(&screened)
        .design_with(&CrosstalkModel, &exps, MinCostCover::new(floor_bits))
        .finalize()?;
    let plan = planned
        .plan
        .as_ref()
        .ok_or("design should have produced a plan")?;
    println!(
        "\n[design] predicted read-out under H1 / H2 / H3, and the cover at {} bits",
        lower(floor_bits)
    );
    let the_plant = plant()?;
    for (i, e) in exps.iter().enumerate() {
        let predicted = candidates
            .iter()
            .map(|h| e.predict::<_, 4>(&CrosstalkModel, h, &the_plant, &[]))
            .collect::<Result<Vec<_>, _>>()?;
        let chosen = plan.entries().iter().find(|p| p.experiment == i);
        println!(
            "    {:<24} cost {}   {:.2} / {:.2} / {:.2}   {}",
            e.name(),
            lower(e.cost()),
            lower(predicted[0]),
            lower(predicted[1]),
            lower(predicted[2]),
            chosen.map_or("—".to_string(), |p| format!(
                "chosen, resolves {:?}",
                p.resolves
            ))
        );
    }
    let every_experiment = exps.iter().fold(ZERO, |sum, e| sum + e.cost());
    println!(
        "    plan: {:?}  total cost {}, of the {} all {} experiments cost",
        plan.entries()
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        lower(plan.total_cost()),
        lower(every_experiment),
        exps.len()
    );
    println!(
        "    tightest pair separates at {:.1} bits against the floor",
        lower(
            plan.report()
                .worst()
                .ok_or("the plan should separate every pair")?
                .measured
        )
    );
    let plan_is_complete = plan.is_complete();
    let plan_costs_two = plan.total_cost() == EXPECTED_PLAN_COST;
    println!(
        "    the plan covers every pair: {}   and costs {}: {}",
        yes_no(plan_is_complete),
        lower(EXPECTED_PLAN_COST),
        yes_no(plan_costs_two)
    );

    // -- the campaign: the plan run against observations drawn from H₁ -------------------
    println!(
        "\n[campaign] H1 generates the observations; cheapest planned experiment first, stop when the evidence decides"
    );
    let truth = EvidenceSource::Simulated(h1_direct_q1_to_q2()?);
    let rules = CampaignRules::new(MinCostCover::new(floor_bits), AGREEMENT_SIGMAS, DRIFT)?;
    let steps = exps.len() + 1;
    let campaign = CausalFlow::value(Campaign::new(
        QclBuilder::control::<FloatType, NumberType, 4, _>(&screened).fork(),
    ))
    .context(exps.clone())
    .iterate_until(Campaign::is_stopped, steps, |flow| {
        Campaign::step(flow, &CrosstalkModel, &truth, &rules)
    })
    .finish()?;
    let stop = campaign.stop().cloned();
    let (run, spent) = (campaign.experiments_run().len(), campaign.spent());
    let report = campaign.finalize()?;
    for (k, observation) in report.observations.iter().enumerate() {
        let read_out = observation.read_out();
        println!(
            "    {}: observed {:.3} ± {:.3} over {} shots",
            observation.experiment(),
            lower(read_out.estimate()),
            lower(read_out.standard_error()),
            observation.shots()
        );
        for w in &report.worlds {
            let reading = &w.readings()[k];
            let prediction = reading
                .prediction()
                .ok_or("a compared reading has a prediction")?;
            let holds = reading.verdict().is_some_and(|v| v.accepted());
            println!(
                "        {:<14} predicts {:.2}   {}",
                w.name(),
                lower(prediction),
                if holds { "consistent" } else { "rejected" }
            );
        }
    }
    let named_the_truth = match &stop {
        Some(CampaignStop::Survivor(s)) => {
            println!(
                "    {} survives, {:.1} bits from its nearest rival, after {run} of the plan's {} experiments (spent {} of {}).",
                s.name,
                lower(s.separation_bits),
                plan.entries().len(),
                lower(spent),
                lower(plan.total_cost())
            );
            println!("    Direct cause, not shared bath.");
            println!(
                "    -> a scheduling or echo fix applies; frequency reallocation is not required."
            );
            s.name.starts_with("H1")
        }
        other => {
            println!("    the campaign stopped without a survivor: {other:?}");
            false
        }
    };
    let survivor_matches = matches!(
        &report.adjudication.map(|a| a.outcome),
        Some(Either::Left(s)) if s.name.starts_with("H1")
    );

    // The survivor is the structure the observations were drawn from. Naming it up front and
    // checking at the end is what separates a demonstration from a coincidence.
    println!(
        "\n    the campaign named the structure the observations came from: {}",
        yes_no(named_the_truth && survivor_matches)
    );

    if !(all_three_admitted
        && plan_is_complete
        && plan_costs_two
        && named_the_truth
        && survivor_matches
        && spent <= plan.total_cost())
    {
        return Err("a check the example makes about its own decision failed".into());
    }

    Ok(())
}

/// A check's verdict, as a word.
fn yes_no(ok: bool) -> &'static str {
    if ok { "yes" } else { "NO" }
}
