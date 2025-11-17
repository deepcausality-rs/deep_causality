/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::GrangerCausaloid;
use crate::{OIL_PRICE_ID, SHIPPING_ACTIVITY_ID, TIME_ID};
use deep_causality::{
    BaseContext, CausalEffectLog, CausalFnOutput, CausalityError, Causaloid, Contextoid,
    ContextoidType, ContextuableGraph, Data, Datable, EffectValue, Identifiable,
    IdentificationValue,
};
use std::sync::{Arc, RwLock};

pub(crate) fn get_factual_causaloid(predictor_id: IdentificationValue) -> GrangerCausaloid {
    let predictor_description = "Predicts shipping activity based on factual historical data";
    let factual_context = Arc::new(RwLock::new(get_context_with_data()));

    // `new_with_context` is used to create a causaloid that has access to its context.
    Causaloid::new_with_context(
        predictor_id,
        shipping_predictor_logic,
        Arc::clone(&factual_context),
        predictor_description,
    )
}

pub(crate) fn get_counterfactual_causaloid(predictor_id: IdentificationValue) -> GrangerCausaloid {
    let factual_context = Arc::new(RwLock::new(get_context_with_data()));
    let counterfactual_context = Arc::new(RwLock::new(get_counterfactual_context(
        &factual_context.read().unwrap(),
    )));
    let predictor_description =
        "Predicts shipping activity based on counterfactual historical data";

    // `new_with_context` is used to create a causaloid that has access to its context.
    Causaloid::new_with_context(
        predictor_id,
        shipping_predictor_logic,
        Arc::clone(&counterfactual_context),
        predictor_description,
    )
}

/// The main logic for the predictive causaloid.
/// This function has access to the context and performs a prediction based on its contents.
//  fn(effect: I, context: &Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>) -> Result<CausalFnOutput<O>, CausalityError>;
fn shipping_predictor_logic(
    _effect: EffectValue, // Not used as data are read from teh context.
    context: &Arc<RwLock<BaseContext>>,
) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
    let mut log = CausalEffectLog::new();

    let mut oil_prices: Vec<f64> = Vec::new();
    let mut shipping_activities: Vec<f64> = Vec::new();

    // Iterate through all nodes in the context graph to gather historical data.
    // A real implementation would likely use more sophisticated queries, but this
    // demonstrates accessing the full context.
    let context_guard = context.read().unwrap();
    for i in 0..context_guard.number_of_nodes() {
        if let Some(node) = context_guard.get_node(i)
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
    // plus an adjustment based on the average oil price.
    if shipping_activities.is_empty() {
        return Ok(CausalFnOutput::new(EffectValue::Numerical(100.0), log));
    }

    let avg_shipping: f64 =
        shipping_activities.iter().sum::<f64>() / shipping_activities.len() as f64;
    log.add_entry(format!("Average Shipment activity: {}", avg_shipping).as_str());

    let mut oil_price_effect = 0.0;
    if !oil_prices.is_empty() {
        let avg_oil = oil_prices.iter().sum::<f64>() / oil_prices.len() as f64;
        // Simple model: higher avg oil price slightly decreases the next shipping activity value.
        // The numbers are chosen to make the factual error smaller.
        oil_price_effect = (avg_oil - 50.0) * 0.5; // 50 is a baseline oil price
    }
    log.add_entry(format!("Oil Price effect: {}", oil_price_effect).as_str());

    // Predict the next value by taking the average and adding a trend factor,
    // adjusted by the oil price effect.
    let prediction = avg_shipping + 3.0 - oil_price_effect;
    log.add_entry(format!("Predicted shipment: {}", prediction).as_str());

    Ok(CausalFnOutput::new(EffectValue::Numerical(prediction), log))
}

/// Creates the factual context containing all historical data.
pub(crate) fn get_context_with_data() -> BaseContext {
    let mut context = BaseContext::with_capacity(1, "Factual Context", 20);
    let mut id_counter = 0;

    // Sample Data (Quarterly)
    let data_points = vec![
        (0.0, 50.0, 100.0), // Q1: time, oil_price, shipping_activity
        (1.0, 52.0, 102.0), // Q2
        (2.0, 55.0, 105.0), // Q3
        (3.0, 58.0, 108.0), // Q4
    ];

    for (time, oil_price, shipping_activity) in data_points {
        // Each contextoid needs a unique ID for the context graph.
        // The ID within the Data payload is used to identify the data type.

        // Time data
        let time_datoid =
            Contextoid::new(id_counter, ContextoidType::Datoid(Data::new(TIME_ID, time)));
        context.add_node(time_datoid).unwrap();
        id_counter += 1;

        // Oil price data
        let oil_price_datoid = Contextoid::new(
            id_counter,
            ContextoidType::Datoid(Data::new(OIL_PRICE_ID, oil_price)),
        );
        context.add_node(oil_price_datoid).unwrap();
        id_counter += 1;

        // Shipping activity data
        let shipping_activity_datoid = Contextoid::new(
            id_counter,
            ContextoidType::Datoid(Data::new(SHIPPING_ACTIVITY_ID, shipping_activity)),
        );
        context.add_node(shipping_activity_datoid).unwrap();
        id_counter += 1;
    }
    context
}

/// Creates the counterfactual context by cloning the factual one and removing oil price data.
pub(crate) fn get_counterfactual_context(factual_context: &BaseContext) -> BaseContext {
    let mut control_context = BaseContext::with_capacity(2, "Counterfactual Context", 20);

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
                control_context.add_node(node.clone()).unwrap();
            }
        }
    }
    control_context
}
