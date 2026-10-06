/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{OIL_BASELINE, OIL_COEFFICIENT, OIL_PRICE_ID, SHIPPING_ACTIVITY_ID, SHIPPING_TREND};
use deep_causality::{
    CausalEffect, CausalityError, CausalityErrorEnum, Causaloid, Identifiable, IdentificationValue,
    PropagatingEffect, PropagatingProcess,
};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    DiscreteTime, NoSpace, NoSpaceTime, TimeScale,
};
use std::sync::{Arc, RwLock};

/// The world the predictor reads: numeric data, quarters as discrete time ticks, no space.
pub type GrangerContext = Context<Data<f64>, NoSpace<f64>, DiscreteTime, NoSpaceTime<f64>>;

pub type GrangerCausaloid = Causaloid<f64, f64, (), Arc<RwLock<GrangerContext>>>;

pub(crate) fn get_factual_causaloid(
    predictor_id: IdentificationValue,
) -> Result<GrangerCausaloid, ContextIndexError> {
    let predictor_description = "Predicts shipping activity based on factual historical data";
    let factual_context = Arc::new(RwLock::new(get_context_with_data()?));

    Ok(Causaloid::new_with_context(
        predictor_id,
        shipping_predictor_logic,
        Arc::clone(&factual_context),
        predictor_description,
    ))
}

pub(crate) fn get_counterfactual_causaloid(
    predictor_id: IdentificationValue,
) -> Result<GrangerCausaloid, ContextIndexError> {
    let factual_context = get_context_with_data()?;
    let counterfactual_context =
        Arc::new(RwLock::new(get_counterfactual_context(&factual_context)?));
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
    let mut oil_prices: Vec<f64> = Vec::new();
    let mut shipping_activities: Vec<f64> = Vec::new();

    // Iterate through all nodes in the context graph to gather historical data.
    for i in 0..context.number_of_nodes() {
        if let Some(node) = context.get_node(i)
            && let ContextoidType::Datoid(data_node) = node.vertex_type()
        {
            match data_node.id() {
                OIL_PRICE_ID => oil_prices.push(data_node.get_data()),
                SHIPPING_ACTIVITY_ID => shipping_activities.push(data_node.get_data()),
                _ => (),
            }
        }
    }

    // --- Simple Prediction Model ---
    // Predicts next shipping activity based on the average of past activity,
    // plus an adjustment based on the average oil price. The coefficients are read from the context.
    if shipping_activities.is_empty() {
        return Ok(100.0);
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

/// Creates the factual context: the shipping-model coefficients at indices `OIL_BASELINE`,
/// `SHIPPING_TREND` and `OIL_COEFFICIENT`, then each quarter's time tick, oil price and shipping
/// activity.
pub(crate) fn get_context_with_data() -> Result<GrangerContext, ContextIndexError> {
    let mut context = GrangerContext::with_capacity(1, "Factual Context", 20);

    // Shipping-model coefficients: oil baseline 50.0, shipping trend 3.0, oil coefficient 0.5.
    // Their data ids differ from OIL_PRICE_ID and SHIPPING_ACTIVITY_ID, so the series scan in
    // the predictor skips them.
    for (id, value) in [(2, 50.0), (3, 3.0), (4, 0.5)] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    let mut id_counter = 5;

    // Sample Data (Quarterly)
    let data_points = vec![
        (0, 50.0, 100.0), // Q1: quarter, oil_price, shipping_activity
        (1, 52.0, 102.0), // Q2
        (2, 55.0, 105.0), // Q3
        (3, 58.0, 108.0), // Q4
    ];

    for (quarter, oil_price, shipping_activity) in data_points {
        // Time of the observation, in quarters
        let time_tempoid = Contextoid::new(
            id_counter,
            ContextoidType::Tempoid(DiscreteTime::new(id_counter, TimeScale::Quarter, quarter)),
        );
        context.add_node(time_tempoid)?;
        id_counter += 1;

        // Oil price data
        let oil_price_datoid = Contextoid::new(
            id_counter,
            ContextoidType::Datoid(Data::new(OIL_PRICE_ID, oil_price)),
        );
        context.add_node(oil_price_datoid)?;
        id_counter += 1;

        // Shipping activity data
        let shipping_activity_datoid = Contextoid::new(
            id_counter,
            ContextoidType::Datoid(Data::new(SHIPPING_ACTIVITY_ID, shipping_activity)),
        );
        context.add_node(shipping_activity_datoid)?;
        id_counter += 1;
    }
    Ok(context)
}

/// Creates the counterfactual context by cloning the factual one and removing oil price data.
/// The coefficients and the quarters stay.
pub(crate) fn get_counterfactual_context(
    factual_context: &GrangerContext,
) -> Result<GrangerContext, ContextIndexError> {
    let mut control_context = GrangerContext::with_capacity(2, "Counterfactual Context", 20);

    // Iterate through the factual context and add all nodes EXCEPT oil price nodes.
    for i in 0..factual_context.number_of_nodes() {
        if let Some(node) = factual_context.get_node(i) {
            let mut should_add = true;
            if let ContextoidType::Datoid(data_node) = node.vertex_type()
                && data_node.id() == OIL_PRICE_ID
            {
                should_add = false;
            }
            if should_add {
                control_context.add_node(node.clone())?;
            }
        }
    }
    Ok(control_context)
}

/// Read one shipping-model coefficient out of the context.
fn read(context: &GrangerContext, index: usize) -> Result<f64, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| CausalityError::ModelError(format!("context node {index} is not a Datoid")))
}
