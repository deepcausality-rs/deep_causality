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
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{
    AlternatableContext, CausalEffect, CausalityError, PropagatingEffect, PropagatingProcess,
};
use std::error::Error;

/// The scalar this example works in. Declared here, per example, so changing the shared alias in
/// `deep_causality_core` cannot silently reconfigure every example that names one.
type FloatType = f64;

/// One quantity of the world: an observed quarterly series, or a scalar coefficient of the
/// shipping model.
#[derive(Clone, Debug, PartialEq)]
enum Quantity {
    Series(Vec<FloatType>),
    Scalar(FloatType),
}

impl Default for Quantity {
    fn default() -> Self {
        Quantity::Series(Vec::new())
    }
}

/// The world this chain reasons about is two time series and the coefficients of the shipping
/// model, so its data node carries a [`Quantity`]. The quarters are the positions in each series;
/// the context holds no space and no time node.
///
/// `Data<T>` requires `Clone` of its payload, not `Copy` — `Copy` is asked for only by the
/// `Adjustable` impl, where `ArrayGrid`'s fixed-size array backing needs it. That is what makes
/// `Data<Quantity>` a valid context node and lets this example carry a real `Context`
/// instead of a struct of its own.
type SeriesContext = Context<Data<Quantity>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node indices of the five contextoids: the two series, then the oil price the adjustment is
/// measured from, the quarterly shipping trend, and the shipping change per unit of oil price
/// above that baseline.
const OIL_PRICES: usize = 0;
const SHIPPING_ACTIVITIES: usize = 1;
const OIL_BASELINE: usize = 2;
const SHIPPING_TREND: usize = 3;
const OIL_COEFFICIENT: usize = 4;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n=== Granger via the Causal Monad: do past oil prices predict shipping? ===\n");

    let factual = factual_series()?;
    let counterfactual = without_oil(&factual)?;

    let factual_pred = value_of(&run(factual.clone()))?;
    let counter_pred = value_of(
        &start(factual)
            .alternate_context(counterfactual)
            .bind(predict_shipping),
    )?;

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
    Ok(())
}

/// Run the seed-plus-bind chain on a fresh factual context.
fn run(series: SeriesContext) -> PropagatingProcess<FloatType, (), SeriesContext> {
    start(series).bind(predict_shipping)
}

// --- Model: series context, chain seed, predictor bind, fixtures ---

/// Build the world: the two histories and the three model coefficients as `Data<Quantity>`
/// contextoids. The counterfactual world is the same call with `oil_prices` empty; the chain
/// reads from the Context and adapts.
fn series_world(
    label: &str,
    oil_prices: Vec<FloatType>,
    shipping_activities: Vec<FloatType>,
    oil_baseline: FloatType,
    shipping_trend: FloatType,
    oil_coefficient: FloatType,
) -> Result<SeriesContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, label, 5);
    let quantities = [
        Quantity::Series(oil_prices),
        Quantity::Series(shipping_activities),
        Quantity::Scalar(oil_baseline),
        Quantity::Scalar(shipping_trend),
        Quantity::Scalar(oil_coefficient),
    ];
    for (id, quantity) in (1..).zip(quantities) {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, quantity)),
        ))?;
    }
    Ok(context)
}

/// Read one quantity out of the world.
fn read(context: &SeriesContext, index: usize) -> Result<Quantity, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| CausalityError::ModelError(format!("context node {index} is not a Datoid")))
}

/// Read one series out of the world.
fn read_series(context: &SeriesContext, index: usize) -> Result<Vec<FloatType>, CausalityError> {
    match read(context, index)? {
        Quantity::Series(values) => Ok(values),
        Quantity::Scalar(_) => Err(CausalityError::TypeConversionError(format!(
            "context node {index} holds a scalar, not a series"
        ))),
    }
}

/// Read one model coefficient out of the world.
fn read_scalar(context: &SeriesContext, index: usize) -> Result<FloatType, CausalityError> {
    match read(context, index)? {
        Quantity::Scalar(value) => Ok(value),
        Quantity::Series(_) => Err(CausalityError::TypeConversionError(format!(
            "context node {index} holds a series, not a scalar"
        ))),
    }
}

/// The value a finished chain carries, or the error that ended it.
fn value_of(
    process: &PropagatingProcess<FloatType, (), SeriesContext>,
) -> Result<FloatType, CausalityError> {
    match process.error() {
        Some(error) => Err(error.clone()),
        None => process
            .value_cloned()
            .ok_or(CausalityError::ValueNotAvailable()),
    }
}

/// Build the seed carrier.
fn start(series: SeriesContext) -> PropagatingProcess<FloatType, (), SeriesContext> {
    let seed = PropagatingEffect::pure(0.0 as FloatType);
    PropagatingProcess::with_state(seed, (), Some(series))
}

/// One-stage predictor: emits the prediction [`predict`] computes from the Context.
fn predict_shipping(
    _value: CausalEffect<FloatType>,
    state: (),
    context: Option<SeriesContext>,
) -> PropagatingProcess<FloatType, (), SeriesContext> {
    let Some(series) = context else {
        return PropagatingProcess::from_error(CausalityError::MissingContext());
    };
    match predict(&series) {
        Ok(prediction) => {
            PropagatingProcess::with_state(PropagatingEffect::pure(prediction), state, Some(series))
        }
        Err(error) => PropagatingProcess::from_error(error),
    }
}

/// Average past shipping, add the shipping trend, adjust by
/// (avg_oil - oil baseline) * oil coefficient when oil history is available. Every input is
/// read from the world.
fn predict(series: &SeriesContext) -> Result<FloatType, CausalityError> {
    let shipping_activities = read_series(series, SHIPPING_ACTIVITIES)?;
    let oil_prices = read_series(series, OIL_PRICES)?;

    if shipping_activities.is_empty() {
        return Ok(100.0);
    }
    let avg_shipping: FloatType = mean(&shipping_activities)?;
    let oil_adjustment = if oil_prices.is_empty() {
        0.0
    } else {
        (mean(&oil_prices)? - read_scalar(series, OIL_BASELINE)?)
            * read_scalar(series, OIL_COEFFICIENT)?
    };
    Ok(avg_shipping + read_scalar(series, SHIPPING_TREND)? - oil_adjustment)
}

/// The mean, dispatched to `deep_causality_stats`. An empty slice is an error.
fn mean(xs: &[FloatType]) -> Result<FloatType, CausalityError> {
    deep_causality_stats::mean(xs).map_err(|error| CausalityError::ModelError(error.to_string()))
}

/// Factual world: four quarters of (oil_price, shipping_activity), an oil baseline of 50.0, a
/// shipping trend of 3.0 per quarter, and an oil coefficient of 0.5.
fn factual_series() -> Result<SeriesContext, ContextIndexError> {
    series_world(
        "factual",
        vec![50.0, 52.0, 55.0, 58.0],
        vec![100.0, 102.0, 105.0, 108.0],
        50.0,
        3.0,
        0.5,
    )
}

/// Counterfactual: same shipping history and coefficients; oil-price history removed.
fn without_oil(factual: &SeriesContext) -> Result<SeriesContext, Box<dyn Error>> {
    Ok(series_world(
        "counterfactual",
        Vec::new(),
        read_series(factual, SHIPPING_ACTIVITIES)?,
        read_scalar(factual, OIL_BASELINE)?,
        read_scalar(factual, SHIPPING_TREND)?,
        read_scalar(factual, OIL_COEFFICIENT)?,
    )?)
}
