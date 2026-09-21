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

use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, Datable, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime,
};
use deep_causality_core::{
    AlternatableContext, CausalEffect, PropagatingEffect, PropagatingProcess,
};

/// The scalar this example works in. Declared here, per example, so changing the shared alias in
/// `deep_causality_core` cannot silently reconfigure every example that names one.
type FloatType = f64;

/// The world this chain reasons about is two time series, so its data node carries a sequence.
///
/// `Data<T>` requires `Clone` of its payload, not `Copy` — `Copy` is asked for only by the
/// `Adjustable` impl, where `ArrayGrid`'s fixed-size array backing needs it. That is what makes
/// `Data<Vec<FloatType>>` a valid context node and lets this example carry a real `Context`
/// instead of a struct of its own.
type SeriesContext = Context<
    Data<Vec<FloatType>>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    FloatType,
    FloatType,
>;

/// Node indices of the two series contextoids.
const OIL_PRICES: usize = 0;
const SHIPPING_ACTIVITIES: usize = 1;

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
fn run(series: SeriesContext) -> PropagatingProcess<FloatType, (), SeriesContext> {
    start(series).bind(predict_shipping)
}

// --- Model: series context, chain seed, predictor bind, fixtures ---

/// Build the world: the two histories as `Data<Vec<FloatType>>` contextoids. The counterfactual
/// world is the same call with `oil_prices` empty; the chain reads from the Context and adapts.
fn series_world(
    label: &str,
    oil_prices: Vec<FloatType>,
    shipping_activities: Vec<FloatType>,
) -> SeriesContext {
    let mut context = Context::with_capacity(1, label, 2);
    for (id, series) in [(1, oil_prices), (2, shipping_activities)] {
        context
            .add_node(Contextoid::new(
                id,
                ContextoidType::Datoid(Data::new(id, series)),
            ))
            .expect("series contextoid is accepted");
    }
    context
}

/// Read one series out of the world.
fn read(context: &SeriesContext, index: usize) -> Vec<FloatType> {
    context
        .get_node(index)
        .expect("contextoid is present")
        .vertex_type()
        .dataoid()
        .expect("contextoid is a Datoid")
        .get_data()
}

const OIL_BASELINE: FloatType = 50.0;
const SHIPPING_TREND: FloatType = 3.0;
const OIL_COEFFICIENT: FloatType = 0.5;

/// Build the seed carrier.
fn start(series: SeriesContext) -> PropagatingProcess<FloatType, (), SeriesContext> {
    let seed = PropagatingEffect::pure(0.0 as FloatType);
    PropagatingProcess::with_state(seed, (), Some(series))
}

/// One-stage predictor: average past shipping, add a small upward trend,
/// adjust by (avg_oil - baseline) when oil history is available.
fn predict_shipping(
    _value: CausalEffect<FloatType>,
    state: (),
    context: Option<SeriesContext>,
) -> PropagatingProcess<FloatType, (), SeriesContext> {
    let series = context.expect("the series world must be set");
    let shipping_activities = read(&series, SHIPPING_ACTIVITIES);
    let oil_prices = read(&series, OIL_PRICES);

    let prediction = if shipping_activities.is_empty() {
        100.0
    } else {
        let avg_shipping: FloatType = mean(&shipping_activities);
        let oil_adjustment = if oil_prices.is_empty() {
            0.0
        } else {
            (mean(&oil_prices) - OIL_BASELINE) * OIL_COEFFICIENT
        };
        avg_shipping + SHIPPING_TREND - oil_adjustment
    };

    let next = PropagatingEffect::pure(prediction);
    PropagatingProcess::with_state(next, state, Some(series))
}

/// The mean, dispatched to `deep_causality_stats`. The fixtures below are never empty, so the
/// crate's refusal on an empty slice cannot fire; `0.0` keeps this a total function anyway.
fn mean(xs: &[FloatType]) -> FloatType {
    deep_causality_stats::mean(xs).unwrap_or(0.0)
}

/// Factual time-series: four quarters of (oil_price, shipping_activity).
fn factual_series() -> SeriesContext {
    series_world(
        "factual",
        vec![50.0, 52.0, 55.0, 58.0],
        vec![100.0, 102.0, 105.0, 108.0],
    )
}

/// Counterfactual: same shipping history; oil-price history removed.
fn without_oil(factual: &SeriesContext) -> SeriesContext {
    series_world(
        "counterfactual",
        Vec::new(),
        read(factual, SHIPPING_ACTIVITIES),
    )
}
