/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # DBN via the Causal Monad
//!
//! Umbrella World as a Dynamic Bayesian Network, implemented on
//! `PropagatingProcess<f64, WeatherState, WeatherContext>` with all three
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

use deep_causality_core::{
    AlternatableContext, EffectValue, PropagatingEffect, PropagatingProcess,
};

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
    mut process: PropagatingProcess<f64, WeatherState, WeatherContext>,
    n: u32,
) -> PropagatingProcess<f64, WeatherState, WeatherContext> {
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

/// Conditional probability table for the current climate regime.
#[derive(Clone, Debug, PartialEq)]
struct WeatherContext {
    label: &'static str,
    /// P(rain today | rained yesterday).
    p_rain_given_rain: f64,
    /// P(rain today | dry yesterday).
    p_rain_given_dry: f64,
}

/// Evolving Markov state: yesterday's rain outcome, plus running counters.
#[derive(Clone, Debug, Default, PartialEq)]
struct WeatherState {
    day: u32,
    rained_yesterday: bool,
    rainy_days: u32,
    umbrellas_carried: u32,
}

fn baseline_climate() -> WeatherContext {
    WeatherContext {
        label: "baseline",
        p_rain_given_rain: 0.40,
        p_rain_given_dry: 0.20,
    }
}

fn monsoon_climate() -> WeatherContext {
    WeatherContext {
        label: "monsoon",
        p_rain_given_rain: 0.95,
        p_rain_given_dry: 0.60,
    }
}

/// Build the seed carrier. Initial Markov state: yesterday it rained.
fn start_in(climate: WeatherContext) -> PropagatingProcess<f64, WeatherState, WeatherContext> {
    let seed = PropagatingEffect::pure(0.0_f64);
    let initial = WeatherState {
        day: 0,
        rained_yesterday: true,
        rainy_days: 0,
        umbrellas_carried: 0,
    };
    PropagatingProcess::with_state(seed, initial, Some(climate))
}

/// One bind = one day. Reads the climate from the Context, the previous
/// day's outcome from the State, then computes today's rain probability,
/// the deterministic rain outcome, and the umbrella decision; updates
/// the State and emits the probability as the next value.
fn step_day(
    _value: EffectValue<f64>,
    state: WeatherState,
    context: Option<WeatherContext>,
) -> PropagatingProcess<f64, WeatherState, WeatherContext> {
    let ctx = context.expect("WeatherContext must be set");

    let p_rain = if state.rained_yesterday {
        ctx.p_rain_given_rain
    } else {
        ctx.p_rain_given_dry
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
        next_state.day, ctx.label, p_rain, rains_today, take_umbrella
    );

    let next = PropagatingEffect::pure(p_rain);
    PropagatingProcess::with_state(next, next_state, Some(ctx))
}
