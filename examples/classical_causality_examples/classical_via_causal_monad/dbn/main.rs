/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # DBN via the Causal Monad
//!
//! Umbrella World as a Dynamic Bayesian Network, implemented on
//! `PropagatingProcess<FloatType, WeatherState, BaseContext>` with all three
//! channels exercised:
//!
//! * **State channel** (`WeatherState`): the Markov state. Carries
//!   `rained_yesterday`, the running day index, and the umbrella counter
//!   that evolves day by day through the bind chain.
//! * **Context channel** (`WeatherContext`): the conditional probability
//!   tables (CPTs) for the current climate regime. Constant within a
//!   regime, alternated via `alternate_context` when the regime changes.
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
    BaseContext, Context, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
};
use deep_causality_core::{
    AlternatableContext, CausalEffect, PropagatingEffect, PropagatingProcess,
};

/// Node indices of the two `Data` contextoids a climate regime carries.
const P_RAIN_GIVEN_RAIN: usize = 0;
const P_RAIN_GIVEN_DRY: usize = 1;

type FloatType = f64;

fn main() {
    println!("\n=== DBN via the Causal Monad: Umbrella World with a Regime Change ===\n");
    run_baseline_only();
    run_regime_change();
}

const DAYS: u32 = 10;
const REGIME_SWITCH_DAY: u32 = 6;
const UMBRELLA_THRESHOLD: f64 = 0.5;

fn run_baseline_only() {
    println!("--- Run 1: baseline climate for all {DAYS} days ---");
    let baseline = baseline_climate();
    let process = simulate_n_days(start_in(baseline.clone()), DAYS);
    print_summary("baseline-only", process.state());
    println!();
}

fn run_regime_change() {
    println!(
        "--- Run 2: baseline for days 1..{REGIME_SWITCH_DAY}, alternate_context(monsoon) on day {REGIME_SWITCH_DAY} ---"
    );
    let baseline = baseline_climate();
    let monsoon = monsoon_climate();

    // Phase 1: baseline for the first 5 days.
    let mid = simulate_n_days(start_in(baseline), REGIME_SWITCH_DAY - 1);

    // Regime change: alternate the Context. State and value continue
    // through the chain untouched; the audit log records the switch.
    let after_switch = mid.alternate_context(monsoon);

    // Phase 2: remaining days under the alternated context.
    let final_process = simulate_n_days(after_switch, DAYS - (REGIME_SWITCH_DAY - 1));

    print_summary("regime-change", final_process.state());

    println!("\nAudit log (regime-change run):");
    println!("{}", final_process.logs());
    println!();
}

/// Iterate the daily bind step `n` times.
fn simulate_n_days(
    mut process: PropagatingProcess<FloatType, WeatherState, BaseContext>,
    n: u32,
) -> PropagatingProcess<FloatType, WeatherState, BaseContext> {
    for _ in 0..n {
        process = process.bind(step_day);
    }
    process
}

fn print_summary(label: &str, state: &WeatherState) {
    println!(
        "Summary [{label}]: days={}, rainy_days={}, umbrellas_carried={}",
        state.day, state.rainy_days, state.umbrellas_carried
    );
}

// --- Model: world state, climate context, and the daily bind step ---

/// Build a climate regime: its conditional probability table as two `Data` contextoids in one
/// typed [`BaseContext`]. The regime's name is the context's own name, so the label the summary
/// prints is read back off the context rather than carried beside it.
fn climate(label: &str, p_rain_given_rain: FloatType, p_rain_given_dry: FloatType) -> BaseContext {
    let mut context = Context::with_capacity(1, label, 2);
    for (id, value) in [(1, p_rain_given_rain), (2, p_rain_given_dry)] {
        context
            .add_node(Contextoid::new(
                id,
                ContextoidType::Datoid(Data::new(id, value)),
            ))
            .expect("climate contextoid is accepted");
    }
    context
}

/// Read one `Data` contextoid's payload out of a climate regime.
fn read(context: &BaseContext, index: usize) -> FloatType {
    context
        .get_node(index)
        .expect("contextoid is present")
        .vertex_type()
        .dataoid()
        .expect("contextoid is a Datoid")
        .get_data()
}

/// Evolving Markov state: yesterday's rain outcome, plus running counters.
#[derive(Clone, Debug, Default, PartialEq)]
struct WeatherState {
    day: u32,
    rained_yesterday: bool,
    rainy_days: u32,
    umbrellas_carried: u32,
}

fn baseline_climate() -> BaseContext {
    climate("baseline", 0.40, 0.20)
}

fn monsoon_climate() -> BaseContext {
    climate("monsoon", 0.95, 0.60)
}

/// Build the seed carrier. Initial Markov state: yesterday it rained.
fn start_in(regime: BaseContext) -> PropagatingProcess<FloatType, WeatherState, BaseContext> {
    let seed = PropagatingEffect::pure(0.0 as FloatType);
    let initial = WeatherState {
        day: 0,
        rained_yesterday: true,
        rainy_days: 0,
        umbrellas_carried: 0,
    };
    PropagatingProcess::with_state(seed, initial, Some(regime))
}

/// One bind = one day. Reads the climate from the Context, the previous
/// day's outcome from the State, then computes today's rain probability,
/// the deterministic rain outcome, and the umbrella decision; updates
/// the State and emits the probability as the next value.
fn step_day(
    _value: CausalEffect<FloatType>,
    state: WeatherState,
    context: Option<BaseContext>,
) -> PropagatingProcess<FloatType, WeatherState, BaseContext> {
    let ctx = context.expect("the climate regime must be set");

    let p_rain = if state.rained_yesterday {
        read(&ctx, P_RAIN_GIVEN_RAIN)
    } else {
        read(&ctx, P_RAIN_GIVEN_DRY)
    };

    // Deterministic rain rule: rains iff p > 0.5. Reproducible across runs.
    let rains_today = p_rain > 0.5;
    let take_umbrella = p_rain > UMBRELLA_THRESHOLD;

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
