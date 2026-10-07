/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # DBN via the Causal Monad
//!
//! Umbrella World as a Dynamic Bayesian Network, implemented on
//! `PropagatingProcess<FloatType, WeatherState, ClimateContext>` with all three
//! channels exercised:
//!
//! * **State channel** (`WeatherState`): the Markov state. Carries
//!   `rained_yesterday`, the running day index, and the umbrella counter
//!   that evolves day by day through the bind chain.
//! * **Context channel** (`ClimateContext`): the conditional probability
//!   tables (CPTs) for the current climate regime and the rain probability above which the
//!   person carries an umbrella, held as three Datoid contextoids. Constant within a regime,
//!   alternated via `alternate_context` when the regime changes.
//! * **Value channel**: today's rain probability emitted by each step.
//!
//! The example runs a 10-day simulation twice:
//!
//! 1. **Baseline all the way.** Dry-leaning climate for all 10 days.
//! 2. **Regime change mid-stream.** Days 1-5 baseline; on day 6 the
//!    operator calls `alternate_context(monsoon_ctx)` and the remaining
//!    days run under the monsoon CPTs.
//!
//! The umbrella counter at the end of each run shows what the operator
//! avoided (or caused) by switching the regime, and the audit log
//! contains exactly one `!!ContextAlternation!!` entry pinpointing the
//! switch.
//!
//! Sampling is deterministic (`p > 0.5` decides whether it rains) so the
//! example is reproducible without an RNG.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{
    AlternatableContext, CausalEffect, CausalityError, PropagatingEffect, PropagatingProcess,
};
use deep_causality_num::lift;
use std::error::Error;

/// Contextoid id: CPT entry P(rain | rained yesterday), probability.
const P_RAIN_GIVEN_RAIN: ContextoidId = 1;
/// Contextoid id: CPT entry P(rain | dry yesterday), probability.
const P_RAIN_GIVEN_DRY: ContextoidId = 2;
/// Contextoid id: the rain probability above which the person carries an umbrella, probability.
const UMBRELLA_THRESHOLD: ContextoidId = 3;

type FloatType = f64;

/// A climate regime: numeric data only, no space and no time. The day count lives in the state.
type ClimateContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n=== DBN via the Causal Monad: Umbrella World with a Regime Change ===\n");
    run_baseline_only()?;
    run_regime_change()
}

const DAYS: u32 = 10;
const REGIME_SWITCH_DAY: u32 = 6;

fn run_baseline_only() -> Result<(), Box<dyn Error>> {
    println!("--- Run 1: baseline climate for all {DAYS} days ---");
    let baseline = baseline_climate()?;
    let process = simulate_n_days(start_in(baseline.clone()), DAYS);
    print_summary("baseline-only", final_state(&process)?);
    println!();
    Ok(())
}

fn run_regime_change() -> Result<(), Box<dyn Error>> {
    println!(
        "--- Run 2: baseline for days 1..{REGIME_SWITCH_DAY}, alternate_context(monsoon) on day {REGIME_SWITCH_DAY} ---"
    );
    let baseline = baseline_climate()?;
    let monsoon = monsoon_climate()?;

    // Phase 1: baseline for the first 5 days.
    let mid = simulate_n_days(start_in(baseline), REGIME_SWITCH_DAY - 1);

    // Regime change: alternate the Context. State and value continue
    // through the chain untouched; the audit log records the switch.
    let after_switch = mid.alternate_context(monsoon);

    // Phase 2: remaining days under the alternated context.
    let final_process = simulate_n_days(after_switch, DAYS - (REGIME_SWITCH_DAY - 1));

    print_summary("regime-change", final_state(&final_process)?);

    println!("\nAudit log (regime-change run):");
    println!("{}", final_process.logs());
    println!();
    Ok(())
}

/// Iterate the daily bind step `n` times.
fn simulate_n_days(
    mut process: PropagatingProcess<FloatType, WeatherState, ClimateContext>,
    n: u32,
) -> PropagatingProcess<FloatType, WeatherState, ClimateContext> {
    for _ in 0..n {
        process = process.bind(step_day);
    }
    process
}

/// The state a finished simulation reached, or the error that ended it.
fn final_state(
    process: &PropagatingProcess<FloatType, WeatherState, ClimateContext>,
) -> Result<&WeatherState, CausalityError> {
    match process.error() {
        Some(error) => Err(error.clone()),
        None => Ok(process.state()),
    }
}

fn print_summary(label: &str, state: &WeatherState) {
    println!(
        "Summary [{label}]: days={}, rainy_days={}, umbrellas_carried={}",
        state.day, state.rainy_days, state.umbrellas_carried
    );
}

// --- Model: world state, climate context, and the daily bind step ---

/// Build a climate regime: its conditional probability table and the person's umbrella
/// threshold as three `Data` contextoids in one [`ClimateContext`]. The regime's name is the
/// context's own name, so the label the summary prints is read back off the context rather than
/// carried beside it.
fn climate(
    label: &str,
    p_rain_given_rain: FloatType,
    p_rain_given_dry: FloatType,
    umbrella_threshold: FloatType,
) -> Result<ClimateContext, ContextIndexError> {
    let facts = [
        (P_RAIN_GIVEN_RAIN, p_rain_given_rain),
        (P_RAIN_GIVEN_DRY, p_rain_given_dry),
        (UMBRELLA_THRESHOLD, umbrella_threshold),
    ];
    let mut context = Context::with_capacity(1, label, facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read the `Data` contextoid with contextoid id `id` out of a climate regime.
fn read(context: &ClimateContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context
        .get_data_by_id(id)
        .ok_or_else(|| CausalityError::ModelError(format!("no Datoid with contextoid id {id}")))
}

/// Evolving Markov state: yesterday's rain outcome, plus running counters.
#[derive(Clone, Debug, Default, PartialEq)]
struct WeatherState {
    day: u32,
    rained_yesterday: bool,
    rainy_days: u32,
    umbrellas_carried: u32,
}

fn baseline_climate() -> Result<ClimateContext, ContextIndexError> {
    climate("baseline", 0.40, 0.20, 0.5)
}

fn monsoon_climate() -> Result<ClimateContext, ContextIndexError> {
    climate("monsoon", 0.95, 0.60, 0.5)
}

/// Build the seed carrier. Initial Markov state: yesterday it rained.
fn start_in(regime: ClimateContext) -> PropagatingProcess<FloatType, WeatherState, ClimateContext> {
    let seed = PropagatingEffect::pure(lift::<FloatType>(0.0));
    let initial = WeatherState {
        day: 0,
        rained_yesterday: true,
        rainy_days: 0,
        umbrellas_carried: 0,
    };
    PropagatingProcess::with_state(seed, initial, Some(regime))
}

/// One bind = one day. Reads the climate and the umbrella threshold from the Context, the
/// previous day's outcome from the State, then computes today's rain probability, the
/// deterministic rain outcome, and the umbrella decision; updates the State and emits the
/// probability as the next value.
fn step_day(
    _value: CausalEffect<FloatType>,
    state: WeatherState,
    context: Option<ClimateContext>,
) -> PropagatingProcess<FloatType, WeatherState, ClimateContext> {
    let Some(ctx) = context else {
        return PropagatingProcess::from_error(CausalityError::MissingContext());
    };

    let cpt_entry = if state.rained_yesterday {
        P_RAIN_GIVEN_RAIN
    } else {
        P_RAIN_GIVEN_DRY
    };
    let (p_rain, umbrella_threshold) = match (read(&ctx, cpt_entry), read(&ctx, UMBRELLA_THRESHOLD))
    {
        (Ok(p_rain), Ok(umbrella_threshold)) => (p_rain, umbrella_threshold),
        (Err(error), _) | (_, Err(error)) => return PropagatingProcess::from_error(error),
    };

    // Deterministic rain rule: rains iff p > 0.5. Reproducible across runs.
    let rains_today = p_rain > 0.5;
    let take_umbrella = p_rain > umbrella_threshold;

    let next_state = WeatherState {
        day: state.day + 1,
        rained_yesterday: rains_today,
        rainy_days: state.rainy_days + u32::from(rains_today),
        umbrellas_carried: state.umbrellas_carried + u32::from(take_umbrella),
    };

    println!(
        "  day {:>2} [{}] p(rain)={:.2} rains={} umbrella={}",
        next_state.day,
        ctx.name(),
        p_rain,
        rains_today,
        take_umbrella
    );

    let next = PropagatingEffect::pure(p_rain);
    PropagatingProcess::with_state(next, next_state, Some(ctx))
}
