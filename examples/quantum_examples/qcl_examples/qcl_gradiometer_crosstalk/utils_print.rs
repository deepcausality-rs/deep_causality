/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every line the gradiometer-crosstalk example prints.

use crate::constants::{
    DARK_BACKGROUND, FLOOR_BITS, LEAK, PLATFORM_SETUP_TIME, SEED, SETUP_TIME, STEP_RATE,
};
use crate::model::GradiometerModel;
use crate::model_config::plant;
use crate::model_types::{Cause, Cloud, Consequence, Instrument, Setting, Verdict, WorldRun};
use crate::{FloatType, Screen};
use deep_causality_algebra::Real;
use deep_causality_num::{lift, lift_count, lower};
use deep_causality_quantum::{ConfiguredExperiment, DesignPlan, InterferometerModel, QuantumError};

/// One Eötvös, 10⁻⁹ s⁻², the unit gravity gradients are quoted in.
fn eotvos() -> FloatType {
    lift(1.0e-9)
}

/// The title, the instrument, and the placeholders.
pub fn print_header(
    instrument: &Instrument,
    interferometer: &InterferometerModel<FloatType>,
) -> Result<(), QuantumError> {
    let time = interferometer.instrument_time()?;
    println!("=== Gradiometer crosstalk: is the correlation between the clouds benign? ===");
    println!("precision: {}", core::any::type_name::<FloatType>());
    println!(
        "instrument (Sorrentino et al. 2014): T = {} s, contrast {}, cycle {} s, white noise to {} s",
        lower(interferometer.interrogation_time()),
        lower(interferometer.contrast()),
        lower(interferometer.cycle_time()),
        lower(interferometer.white_noise_range())
    );
    println!(
        "  one effective draw takes {:.3e} s; a planned experiment sizes its draws to the {}-bit floor",
        lower(time.shot_time()),
        lower(FLOOR_BITS)
    );
    println!(
        "  rotation: {:.2e} rad of centrifugal phase at Earth's horizontal rate, {:.3} rad at the step's {} rad/s",
        lower(instrument.operating_rotation_phase),
        lower(instrument.step_rotation_phase),
        lower(STEP_RATE)
    );
    println!(
        "placeholders for the lab's own measurements: leak {}, dark background {}, setup {} s, platform setup {} s",
        lower(LEAK),
        lower(DARK_BACKGROUND),
        lower(SETUP_TIME),
        lower(PLATFORM_SETUP_TIME)
    );
    println!("seed: {SEED}\n");
    Ok(())
}

/// A world: each experiment's prediction under every candidate fitted to the world's passive
/// read-out, and the plan.
pub fn print_world(
    world: &Cause,
    screen: &Screen,
    model: &GradiometerModel,
    experiments: &[ConfiguredExperiment<FloatType, Setting>],
    plan: &DesignPlan<FloatType>,
) -> Result<(), QuantumError> {
    let the_plant = plant()?;
    let candidates = screen.admitted();
    println!(
        "[world: {}] observations drawn from it; candidates fitted to its passive read-out",
        world.name
    );
    println!(
        "    {:<24} {}",
        "predicted read-out",
        candidates
            .iter()
            .map(|h| format!("{:>16}", h.name()))
            .collect::<String>()
    );
    for (i, e) in experiments.iter().enumerate() {
        let predicted = candidates
            .iter()
            .map(|h| {
                e.predict::<_, 4>(model, h, &the_plant, &[])
                    .map(|p| format!("{:>16.5}", lower(p)))
            })
            .collect::<Result<String, _>>()?;
        let chosen = plan
            .entries()
            .iter()
            .find(|p| p.experiment == i)
            .map_or(String::new(), |p| {
                format!("   planned: {} draws, {:.1} s", p.shots, lower(p.cost))
            });
        println!("    {:<24} {predicted}{chosen}", e.name());
    }
    println!(
        "    plan: {:.1} s, covers every pair: {}",
        lower(plan.total_cost()),
        yes_no(plan.is_complete())
    );
    Ok(())
}

/// What the campaign and the static plan concluded.
pub fn print_run(run: &WorldRun) {
    println!(
        "    campaign ran {} and spent {:.1} s: {}",
        run.campaign_experiments.join(", "),
        lower(run.campaign_cost),
        describe(&run.campaign)
    );
    println!(
        "    static plan, every planned experiment run, {:.1} s: {}",
        lower(run.plan_cost),
        describe(&run.plan)
    );
}

/// The survivor's consequence.
pub fn print_consequence(consequence: &Consequence, instrument: &Instrument) {
    let bias = consequence.bias();
    println!(
        "    {}: the instrument reads {:.5} rad; without the mechanism it would read {:.5} rad",
        consequence.cause,
        lower(consequence.reading),
        lower(consequence.unbiased)
    );
    println!(
        "    gradient bias {:+.4} rad = {:+.1} E; action: {}\n",
        lower(bias),
        lower(bias / instrument.phase_per_gradient / eotvos()),
        consequence.action
    );
}

/// The checks the example makes about its own decisions, printed; whether all hold.
pub fn print_gates(runs: &[WorldRun]) -> bool {
    println!("[checks]");
    let mut all = true;
    for run in runs {
        let named = match run.expected {
            Some(name) => {
                let campaign = run.campaign == Verdict::Survivor(name.to_string());
                let plan = run.plan == Verdict::Survivor(name.to_string());
                println!(
                    "    world {:<18} campaign names {name}: {}   static plan names it: {}",
                    run.world,
                    yes_no(campaign),
                    yes_no(plan)
                );
                campaign && plan
            }
            None => {
                let outside = run.plan == Verdict::OutsideTheModel;
                println!(
                    "    world {:<18} static plan reports it outside the model: {}",
                    run.world,
                    yes_no(outside)
                );
                if let Verdict::Survivor(name) = &run.campaign {
                    println!(
                        "        the campaign stopped at {name} after {}: a survivor ends it before the \
                         experiment that would contradict it runs",
                        run.campaign_experiments.join(", ")
                    );
                }
                outside
            }
        };
        // The campaign and the plan add the same costs in different orders, so the two sums may
        // differ by the rounding of a sum of that many terms.
        let terms = lift_count::<FloatType>(run.plan_settings.len() as u64);
        let rounding = run.plan_cost * FloatType::epsilon() * terms;
        let cheaper = run.campaign_cost <= run.plan_cost + rounding;
        let plan = run.plan_complete
            && run.plan_settings
                == [
                    Setting::Brighten(Cloud::A),
                    Setting::Brighten(Cloud::B),
                    Setting::RotationStep,
                ];
        println!(
            "        campaign spends no more than the plan: {}   plan is E1, E2, E3 and complete: {}",
            yes_no(cheaper),
            yes_no(plan)
        );
        all &= named && cheaper && plan;
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
