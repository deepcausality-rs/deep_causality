/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    CausalEffect, CausalityError, CausalityErrorEnum, Causaloid, Identifiable, IdentificationValue,
    PropagatingEffect, PropagatingProcess,
};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    DiscreteTime, NoSpace, NoSpaceTime, TimeScale,
};
use std::ops::Range;
use std::sync::{Arc, RwLock};

/// The world the predictor reads: numeric data, quarters as discrete time ticks, no space.
pub type GrangerContext = Context<Data<f64>, NoSpace<f64>, DiscreteTime, NoSpaceTime<f64>>;

/// Contextoid id: the oil price the oil adjustment is measured from.
const OIL_BASELINE: ContextoidId = 1;
/// Contextoid id: the shipping trend, per quarter.
const SHIPPING_TREND: ContextoidId = 2;
/// Contextoid id: the shipping change per unit of oil price above the baseline.
const OIL_COEFFICIENT: ContextoidId = 3;
/// Contextoid ids of the quarters' time ticks. Quarter `q` (`0` for Q1) has its time tick at
/// `QUARTER_IDS.start + q`, its oil price at `OIL_PRICE_IDS.start + q` and its shipping activity
/// at `SHIPPING_ACTIVITY_IDS.start + q`.
const QUARTER_IDS: Range<ContextoidId> = 4..8;
/// Contextoid ids of the quarterly oil prices.
const OIL_PRICE_IDS: Range<ContextoidId> = 8..12;
/// Contextoid ids of the quarterly shipping activities.
const SHIPPING_ACTIVITY_IDS: Range<ContextoidId> = 12..16;

pub type GrangerCausaloid = Causaloid<f64, f64, (), Arc<RwLock<GrangerContext>>>;

/// The predictor over the factual context.
pub(crate) fn get_factual_causaloid(
    predictor_id: IdentificationValue,
    factual_context: GrangerContext,
) -> GrangerCausaloid {
    let predictor_description = "Predicts shipping activity based on factual historical data";

    Causaloid::new_with_context(
        predictor_id,
        shipping_predictor_logic,
        Arc::new(RwLock::new(factual_context)),
        predictor_description,
    )
}

/// The predictor over the counterfactual context derived from `factual_context`.
pub(crate) fn get_counterfactual_causaloid(
    predictor_id: IdentificationValue,
    factual_context: &GrangerContext,
) -> Result<GrangerCausaloid, ContextIndexError> {
    let counterfactual_context =
        Arc::new(RwLock::new(get_counterfactual_context(factual_context)?));
    let predictor_description =
        "Predicts shipping activity based on counterfactual historical data";

    Ok(Causaloid::new_with_context(
        predictor_id,
        shipping_predictor_logic,
        Arc::clone(&counterfactual_context),
        predictor_description,
    ))
}

/// The value an evaluation carries, or the error that ended it.
pub(crate) fn value_of(effect: &PropagatingEffect<f64>) -> Result<f64, CausalityError> {
    match effect.error() {
        Some(error) => Err(error.clone()),
        None => effect
            .value_cloned()
            .ok_or(CausalityError::ValueNotAvailable()),
    }
}

/// The main logic for the predictive causaloid.
/// This function has access to the context and performs a prediction based on its contents.
/// New API signature: fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
fn shipping_predictor_logic(
    _effect: CausalEffect<f64>,
    _state: (),
    context: Option<Arc<RwLock<GrangerContext>>>,
) -> PropagatingProcess<f64, (), Arc<RwLock<GrangerContext>>> {
    let ctx_arc = match context {
        Some(c) => c,
        None => {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Context is missing".into(),
            )));
        }
    };

    let context_guard = match ctx_arc.read() {
        Ok(guard) => guard,
        Err(_) => {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Context lock is poisoned".into(),
            )));
        }
    };

    match predict(&context_guard) {
        Ok(prediction) => PropagatingProcess::pure(prediction),
        Err(error) => PropagatingProcess::from_error(error),
    }
}

/// Predicts the next shipping activity from the historical data and the coefficients in the
/// context.
fn predict(context: &GrangerContext) -> Result<f64, CausalityError> {
    // Gather the historical series by contextoid id. A quarter the context holds no node for
    // contributes nothing, so the counterfactual context yields an empty oil-price series.
    let oil_prices: Vec<f64> = OIL_PRICE_IDS
        .filter_map(|id| context.get_data_by_id(id))
        .collect();
    let shipping_activities: Vec<f64> = SHIPPING_ACTIVITY_IDS
        .filter_map(|id| context.get_data_by_id(id))
        .collect();

    // --- Simple Prediction Model ---
    // Predicts next shipping activity based on the average of past activity,
    // plus an adjustment based on the average oil price. The coefficients are read from the context.
    if shipping_activities.is_empty() {
        return Err(CausalityError::ModelError(
            "the context holds no shipping activity",
        ));
    }

    let avg_shipping: f64 = mean(&shipping_activities)?;

    let mut oil_price_effect = 0.0;
    if !oil_prices.is_empty() {
        let avg_oil = mean(&oil_prices)?;
        // Simple model: higher avg oil price slightly decreases the next shipping activity value.
        oil_price_effect =
            (avg_oil - read(context, OIL_BASELINE)?) * read(context, OIL_COEFFICIENT)?;
    }

    // Predict the next value by taking the average and adding a trend factor,
    // adjusted by the oil price effect.
    Ok(avg_shipping + read(context, SHIPPING_TREND)? - oil_price_effect)
}

/// The mean, dispatched to `deep_causality_stats`. An empty slice is an error.
fn mean(xs: &[f64]) -> Result<f64, CausalityError> {
    deep_causality_stats::mean(xs).map_err(|error| CausalityError::ModelError(error.to_string()))
}

/// Creates the factual context: the shipping-model coefficients, then each quarter's time tick,
/// oil price and shipping activity, each node keyed by its contextoid id.
pub(crate) fn get_context_with_data() -> Result<GrangerContext, ContextIndexError> {
    let coefficients = [
        (OIL_BASELINE, 50.0),
        (SHIPPING_TREND, 3.0),
        (OIL_COEFFICIENT, 0.5),
    ];
    // Sample data, one tuple per quarter: (quarter, oil price, shipping activity).
    let data_points = [
        (0, 50.0, 100.0), // Q1
        (1, 52.0, 102.0), // Q2
        (2, 55.0, 105.0), // Q3
        (3, 58.0, 108.0), // Q4
    ];
    let mut context = GrangerContext::with_capacity(
        1,
        "Factual Context",
        coefficients.len() + 3 * data_points.len(),
    );

    for (id, value) in coefficients {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }

    for (quarter, oil_price, shipping_activity) in data_points {
        // Time of the observation, in quarters
        let tick_id = QUARTER_IDS.start + quarter;
        context.add_node(Contextoid::new(
            tick_id,
            ContextoidType::Tempoid(DiscreteTime::new(tick_id, TimeScale::Quarter, quarter)),
        ))?;

        for (id, value) in [
            (OIL_PRICE_IDS.start + quarter, oil_price),
            (SHIPPING_ACTIVITY_IDS.start + quarter, shipping_activity),
        ] {
            context.add_node(Contextoid::new(
                id,
                ContextoidType::Datoid(Data::new(id, value)),
            ))?;
        }
    }
    Ok(context)
}

/// Creates the counterfactual context: every node of the factual context except the oil prices,
/// the contextoids in `OIL_PRICE_IDS`. The coefficients, the quarters and the shipping activities
/// stay.
pub(crate) fn get_counterfactual_context(
    factual_context: &GrangerContext,
) -> Result<GrangerContext, ContextIndexError> {
    let mut control_context = GrangerContext::with_capacity(
        2,
        "Counterfactual Context",
        factual_context.number_of_nodes(),
    );

    for node in
        (0..factual_context.number_of_nodes()).filter_map(|index| factual_context.get_node(index))
    {
        if !OIL_PRICE_IDS.contains(&node.id()) {
            control_context.add_node(node.clone())?;
        }
    }
    Ok(control_context)
}

/// Read the shipping-model coefficient with contextoid id `id` out of the context.
fn read(context: &GrangerContext, id: ContextoidId) -> Result<f64, CausalityError> {
    context
        .get_data_by_id(id)
        .ok_or_else(|| CausalityError::ModelError(format!("no Datoid with contextoid id {id}")))
}
