/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Granger via the Causal Monad
//!
//! Granger's predictive-causality test on
//! `PropagatingProcess<f64, (), SeriesContext>`. The Granger question is:
//! does including past oil prices improve our prediction of next-period
//! shipping activity? Two predictions are compared against the actual
//! next-period value:
//!
//! 1. **Factual** — predict from the full series (shipping history +
//!    oil-price history).
//! 2. **Counterfactual** — predict from the same chain run against a
//!    Context whose oil-price history has been removed. The chain is
//!    identical; only the Context differs.
//!
//! Whichever prediction has the lower error wins. The same single-stage
//! `predict_shipping` bind runs in both worlds; the only operator that
//! differs between runs is `.alternate_context(no_oil_ctx)` before the
//! bind.

use deep_causality_core::{
    AlternatableContext, CausalEffect, PropagatingEffect, PropagatingProcess,
};

fn main() {
    println!("\n=== Granger via the Causal Monad: do past oil prices predict shipping? ===\n");

    let factual = factual_series();
    let counterfactual = without_oil(&factual);

    let factual_pred = run(factual.clone()).value_cloned().unwrap();
    let counter_pred = start(factual)
        .alternate_context(counterfactual)
        .bind(predict_shipping)
        .value_cloned()
        .unwrap();

    let actual_q5 = 105.0;
    let err_factual = (factual_pred - actual_q5).abs();
    let err_counter = (counter_pred - actual_q5).abs();

    println!(
        "Factual prediction (with oil)      = {factual_pred:.2}  (error vs actual {actual_q5:.2}: {err_factual:.2})"
    );
    println!(
        "Counterfactual prediction (no oil) = {counter_pred:.2}  (error vs actual {actual_q5:.2}: {err_counter:.2})"
    );

    println!("\n--- Granger conclusion ---");
    if err_factual < err_counter {
        println!(
            "Past oil prices DO Granger-cause future shipping activity:\n\
             including oil history reduced the prediction error by {:.2}.",
            err_counter - err_factual
        );
    } else {
        println!("Past oil prices do NOT Granger-cause future shipping activity.");
    }
}

/// Run the seed-plus-bind chain on a fresh factual context.
fn run(series: SeriesContext) -> PropagatingProcess<f64, (), SeriesContext> {
    start(series).bind(predict_shipping)
}

// --- Model: series context, chain seed, predictor bind, fixtures ---

/// Time-series data carried in the Context channel. The counterfactual
/// world is the same data with `oil_prices` emptied; the chain reads from
/// the Context and adapts naturally.
#[derive(Clone, Debug, PartialEq)]
struct SeriesContext {
    oil_prices: Vec<f64>,
    shipping_activities: Vec<f64>,
}

const OIL_BASELINE: f64 = 50.0;
const SHIPPING_TREND: f64 = 3.0;
const OIL_COEFFICIENT: f64 = 0.5;

/// Build the seed carrier.
fn start(series: SeriesContext) -> PropagatingProcess<f64, (), SeriesContext> {
    let seed = PropagatingEffect::pure(0.0_f64);
    PropagatingProcess::with_state(seed, (), Some(series))
}

/// One-stage predictor: average past shipping, add a small upward trend,
/// adjust by (avg_oil - baseline) when oil history is available.
fn predict_shipping(
    _value: CausalEffect<f64>,
    state: (),
    context: Option<SeriesContext>,
) -> PropagatingProcess<f64, (), SeriesContext> {
    let series = context.expect("SeriesContext must be set");

    let prediction = if series.shipping_activities.is_empty() {
        100.0
    } else {
        let avg_shipping: f64 = mean(&series.shipping_activities);
        let oil_adjustment = if series.oil_prices.is_empty() {
            0.0
        } else {
            (mean(&series.oil_prices) - OIL_BASELINE) * OIL_COEFFICIENT
        };
        avg_shipping + SHIPPING_TREND - oil_adjustment
    };

    let next = PropagatingEffect::pure(prediction);
    PropagatingProcess::with_state(next, state, Some(series))
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

/// Factual time-series: four quarters of (oil_price, shipping_activity).
fn factual_series() -> SeriesContext {
    SeriesContext {
        oil_prices: vec![50.0, 52.0, 55.0, 58.0],
        shipping_activities: vec![100.0, 102.0, 105.0, 108.0],
    }
}

/// Counterfactual: same shipping history; oil-price history removed.
fn without_oil(factual: &SeriesContext) -> SeriesContext {
    SeriesContext {
        oil_prices: Vec::new(),
        shipping_activities: factual.shipping_activities.clone(),
    }
}
