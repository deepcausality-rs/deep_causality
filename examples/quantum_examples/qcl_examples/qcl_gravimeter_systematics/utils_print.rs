/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every line the gravimeter-systematics example prints.

use crate::FloatType;
use crate::constants::{FLOOR_BITS, MICRO_GAL, OFFSET_UGAL, SEED};
use crate::model_config::{CANDIDATES, candidate_name};
use crate::model_types::{Physics, Systematic, Verdict, WorldRun};
use deep_causality_algebra::Real;
use deep_causality_num::{One, Zero, lift_count, lower};
use deep_causality_quantum::{InterferometerModel, QuantumError};

/// The offset, in µGal, behind read-out `p`: the inverse of `½(1 + C sin(k_eff T² Δg))`.
fn offset_ugal(
    p: FloatType,
    interferometer: &InterferometerModel<FloatType>,
    physics: &Physics,
) -> FloatType {
    let t = interferometer.interrogation_time();
    let two = FloatType::one() + FloatType::one();
    ((two * p - FloatType::one()) / physics.contrast).asin()
        / (interferometer.k_eff() * t * t)
        / MICRO_GAL
}

/// The title, the instrument, and the run's assumptions.
pub fn print_header(
    interferometer: &InterferometerModel<FloatType>,
    physics: &Physics,
) -> Result<(), QuantumError> {
    let time = interferometer.instrument_time()?;
    println!("=== Gravimeter systematics: which effect explains the offset? ===");
    println!("precision: {}", core::any::type_name::<FloatType>());
    println!(
        "instrument (Louchet-Chauvet et al. 2011): T = {} s, {:.3} s cycle, {:.1} µGal/√Hz in one configuration, white noise to {} s",
        lower(interferometer.interrogation_time()),
        lower(interferometer.cycle_time()),
        lower(interferometer.sensitivity() / MICRO_GAL),
        lower(interferometer.white_noise_range())
    );
    println!(
        "  one effective draw takes {:.3e} s; each planned experiment takes the fewest draws that reach {} bits",
        lower(time.shot_time()),
        lower(FLOOR_BITS)
    );
    println!(
        "  the offset to explain: {} µGal after the tide; the tide: {} µGal semidiurnal",
        lower(OFFSET_UGAL),
        lower(physics.tide.0 / MICRO_GAL)
    );
    println!("  placeholders for the lab's own measurements are marked in constants.rs");
    println!("seed: {SEED}\n");
    Ok(())
}

/// One run: the baseline's refusals, the plan, and the campaign.
pub fn print_run(
    run: &WorldRun,
    interferometer: &InterferometerModel<FloatType>,
    physics: &Physics,
) {
    let s = &run.scenario;
    let without = s.without.map_or(String::new(), |w| {
        format!(", {} left off the list", candidate_name(w))
    });
    println!(
        "[{}] the observations come from {} at {} µGal{}{}",
        s.label,
        candidate_name(s.truth),
        lower(s.offset_ugal),
        if s.records_tide {
            ""
        } else {
            ", the contexts record no tide"
        },
        without
    );
    if run.refused.is_empty() {
        println!("    baseline: every candidate can produce the passive reading");
    }
    for (name, predicted, observed) in &run.refused {
        println!(
            "    baseline refuses {name}: it predicts {:.2} µGal, the instrument reads {:.2} µGal (tide included)",
            lower(offset_ugal(*predicted, interferometer, physics)),
            lower(offset_ugal(*observed, interferometer, physics))
        );
    }
    if run.plan.is_empty() {
        println!("    no plan: fewer than two candidates remain\n");
        return;
    }
    println!(
        "    plan: {} at {:.0} s, covers every pair: {}",
        run.plan.join(", "),
        lower(run.plan_cost),
        yes_no(run.plan_complete)
    );
    if !run.eliminated.is_empty() {
        println!(
            "      no planned experiment singles out {}: the plan identifies it by elimination",
            run.eliminated.join(", ")
        );
    }
    println!(
        "    campaign ran {} for {:.0} s, with {} re-plans as the tide moved: {}\n",
        run.campaign.join(", "),
        lower(run.campaign_cost),
        run.replans,
        describe(&run.campaign_verdict)
    );
}

/// The checks the example makes about its own decisions, printed; whether all hold.
pub fn print_gates(runs: &[WorldRun]) -> bool {
    println!("[checks]");
    let mut all = true;
    for run in runs {
        let s = &run.scenario;
        let truth = candidate_name(s.truth);
        let named_truth = run.campaign_verdict == Verdict::Survivor(truth.to_string());
        let held = if !s.records_tide {
            let every = run.refused.len() == CANDIDATES.len();
            println!(
                "    without the tide the baseline refuses every candidate: {}",
                yes_no(every)
            );
            every
        } else if let Some(left_off) = s.without {
            let outcome = match &run.campaign_verdict {
                Verdict::Survivor(name) => format!("misattributes it to {name}"),
                Verdict::OutsideTheModel => "returns no survivor".to_string(),
                Verdict::Unresolved => "ends unresolved".to_string(),
            };
            println!(
                "    with {} left off, the campaign {outcome}",
                candidate_name(left_off)
            );
            run.campaign_verdict != Verdict::Unresolved
        } else if s.offset_ugal > FloatType::zero() {
            let tilt =
                run.refused.len() == 1 && run.refused[0].0 == candidate_name(Systematic::Tilt);
            println!(
                "    a positive offset: the baseline refuses only {}: {}   the campaign names {truth}: {}",
                candidate_name(Systematic::Tilt),
                yes_no(tilt),
                yes_no(named_truth)
            );
            tilt && named_truth
        } else {
            // The campaign and the plan add the same costs in different orders, so the two sums
            // may differ by the rounding of a sum of that many terms.
            let terms = lift_count::<FloatType>(run.plan.len() as u64);
            let rounding = run.plan_cost * FloatType::epsilon() * terms;
            let cheaper = run.campaign_cost <= run.plan_cost + rounding;
            let eliminated = run.eliminated == [candidate_name(Systematic::Wavefront)];
            println!(
                "    {truth:<20} named: {}   spends no more than the plan: {}   plan leaves {} to elimination: {}",
                yes_no(named_truth),
                yes_no(cheaper),
                candidate_name(Systematic::Wavefront),
                yes_no(eliminated && run.plan_complete)
            );
            named_truth && cheaper && eliminated && run.plan_complete
        };
        all &= held;
    }
    println!("    every check holds: {}", yes_no(all));
    all
}

/// A verdict, in words.
fn describe(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Survivor(name) => format!("{name} survives"),
        Verdict::OutsideTheModel => "no candidate holds: outside the model".to_string(),
        Verdict::Unresolved => "unresolved".to_string(),
    }
}

/// A check's verdict, as a word.
fn yes_no(ok: bool) -> &'static str {
    if ok { "yes" } else { "NO" }
}
