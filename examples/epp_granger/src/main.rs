/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::Arc;

// Contextoid IDs
const OIL_PRICE_ID: IdentificationValue = 0;
const SHIPPING_ACTIVITY_ID: IdentificationValue = 1;
const TIME_ID: IdentificationValue = 2;

// Causaloid IDs
const PREDICTOR_CAUSALOID_ID: IdentificationValue = 1;

fn main() {
    println!("\n--- Granger Causality Example: Oil Prices and Shipping Activity ---");

    // 1. Setup the Contexts (Factual and Counterfactual)
    let factual_context = get_context_with_data();
    let control_context = get_counterfactual_context(&factual_context);

    // 2. Define the Predictive Causaloid
    let shipping_predictor_causaloid = get_shipping_predictor_causaloid();

    // Create the CausaloidGraph
    let mut causaloid_graph = CausaloidGraph::new(0);
    let predictor_idx = causaloid_graph
        .add_causaloid(shipping_predictor_causaloid)
        .unwrap();
    causaloid_graph.freeze();
    let causaloid_graph_arc = Arc::new(causaloid_graph);

    // Simulate prediction for a future time step (e.g., Q5)
    let prediction_time_step = 4.0; // Q5 (after Q1, Q2, Q3, Q4)

    // 3. Execute the Granger Test

    // Factual Evaluation
    println!("\n--- Factual Evaluation (with Oil Prices) ---");
    let mut factual_input_map = PropagatingEffect::new_map();
    factual_input_map.insert(TIME_ID, PropagatingEffect::Numerical(prediction_time_step));
    // Pass the factual context to the causaloid graph for evaluation
    // The causaloid's internal logic will query the context it's associated with.
    // For this example, we'll pass the context directly to the causaloid's evaluate function
    // by associating the causaloid with the context before evaluation.

    // Temporarily associate the causaloid with the factual context for evaluation
    let mut temp_predictor_causaloid_factual = causaloid_graph_arc
        .get_causaloid(predictor_idx)
        .unwrap()
        .clone();
    let factual_context_arc = Arc::new(factual_context);
    temp_predictor_causaloid_factual.set_context(Some(Arc::clone(&factual_context_arc)));

    let factual_prediction_res = temp_predictor_causaloid_factual.evaluate(&factual_input_map);
    let factual_prediction = factual_prediction_res.unwrap().as_numerical().unwrap();
    println!(
        "Factual Prediction for Q{:.0} Shipping Activity: {:.2}",
        prediction_time_step + 1.0,
        factual_prediction
    );

    // Assuming a known actual value for Q5 for error calculation
    let actual_q5_shipping = 105.0; // Example actual value
    let error_factual = (factual_prediction - actual_q5_shipping).abs();
    println!("Factual Prediction Error: {:.2}", error_factual);

    // Counterfactual Evaluation
    println!("\n--- Counterfactual Evaluation (without Oil Prices) ---");
    let mut counterfactual_input_map = PropagatingEffect::new_map();
    counterfactual_input_map.insert(TIME_ID, PropagatingEffect::Numerical(prediction_time_step));

    // Temporarily associate the causaloid with the counterfactual context for evaluation
    let mut temp_predictor_causaloid_control = causaloid_graph_arc
        .get_causaloid(predictor_idx)
        .unwrap()
        .clone();
    let control_context_arc = Arc::new(control_context);
    temp_predictor_causaloid_control.set_context(Some(Arc::clone(&control_context_arc)));

    let counterfactual_prediction_res =
        temp_predictor_causaloid_control.evaluate(&counterfactual_input_map);
    let counterfactual_prediction = counterfactual_prediction_res
        .unwrap()
        .as_numerical()
        .unwrap();
    println!(
        "Counterfactual Prediction for Q{:.0} Shipping Activity: {:.2}",
        prediction_time_step + 1.0,
        counterfactual_prediction
    );

    let error_counterfactual = (counterfactual_prediction - actual_q5_shipping).abs();
    println!(
        "Counterfactual Prediction Error: {:.2}",
        error_counterfactual
    );

    // 4. Compare and Conclude
    println!("\n--- Granger Causality Conclusion ---");
    if error_factual < error_counterfactual {
        println!("Conclusion: Past oil prices DO Granger-cause future shipping activity.");
        println!(
            "Factual error ({:.2}) < Counterfactual error ({:.2})",
            error_factual, error_counterfactual
        );
    } else {
        println!("Conclusion: Past oil prices DO NOT Granger-cause future shipping activity.");
        println!(
            "Factual error ({:.2}) >= Counterfactual error ({:.2})",
            error_factual, error_counterfactual
        );
    }
}

// Helper functions

fn get_context_with_data() -> BaseContext {
    let mut context = BaseContext::with_capacity(1, "Factual Context", 20);

    // Sample Data (Quarterly)
    // Oil Prices: Q1=50, Q2=52, Q3=55, Q4=58
    // Shipping Activity: Q1=100, Q2=102, Q3=105, Q4=108
    let data_points = vec![
        (0.0, 50.0, 100.0), // Q1: time, oil_price, shipping_activity
        (1.0, 52.0, 102.0), // Q2
        (2.0, 55.0, 105.0), // Q3
        (3.0, 58.0, 108.0), // Q4
    ];

    for (time, oil_price, shipping_activity) in data_points {
        let time_datoid =
            Contextoid::new(TIME_ID, ContextoidType::Datoid(Data::new(TIME_ID, time)));
        let oil_price_datoid = Contextoid::new(
            OIL_PRICE_ID,
            ContextoidType::Datoid(Data::new(OIL_PRICE_ID, oil_price)),
        );
        let shipping_activity_datoid = Contextoid::new(
            SHIPPING_ACTIVITY_ID,
            ContextoidType::Datoid(Data::new(SHIPPING_ACTIVITY_ID, shipping_activity)),
        );

        context.add_node(time_datoid).unwrap();
        context.add_node(oil_price_datoid).unwrap();
        context.add_node(shipping_activity_datoid).unwrap();
    }
    context
}

fn get_counterfactual_context(factual_context: &BaseContext) -> BaseContext {
    let mut control_context = factual_context.clone();

    // Remove or zero out oil_price dataoids in the cloned context
    // Iterate through the nodes and update the oil_price datoids
    // Note: This is a simplified approach. In a real scenario, you might remove the nodes or set them to a specific baseline.
    for i in 0..control_context.number_of_nodes() {
        let node = control_context.get_node(i).unwrap();
        if let ContextoidType::Datoid(data_node) = node.vertex_type() {
            if data_node.id() == OIL_PRICE_ID {
                let mut updated_data = data_node.clone();
                updated_data.set_data(0.0); // Set oil price to 0.0 in counterfactual
                control_context
                    .update_node(
                        data_node.id(),
                        Contextoid::new(data_node.id(), ContextoidType::Datoid(updated_data)),
                    )
                    .unwrap();
            }
        }
    }
    control_context
}

fn get_shipping_predictor_causaloid() -> BaseCausaloid {
    let predictor_id = PREDICTOR_CAUSALOID_ID;
    let predictor_description = "Predicts shipping activity based on historical data";

    let causal_fn = |effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
        let current_time_step = match effect {
            PropagatingEffect::Map(map) => map
                .get(&TIME_ID)
                .and_then(|boxed_effect| boxed_effect.as_numerical())
                .ok_or_else(|| {
                    CausalityError("Current time step not found in effect map".into())
                })?,
            _ => {
                return Err(CausalityError(
                    "Expected Map effect for predictor causaloid".into(),
                ));
            }
        };

        // In a real scenario, this causaloid would query the context it's associated with
        // to get historical data. Since causal_fn cannot capture context directly, we simulate
        // context lookup by assuming the context is available via the Causaloid's own context field.
        // This requires the Causaloid to be initialized with a context.
        // For this example, we'll use a simplified model that assumes access to the context.

        // Simulate context access and prediction logic
        // This is a placeholder for a more complex predictive model (e.g., linear regression)
        // For simplicity, we'll assume a direct lookup or a very simple model.
        // In a real DBN, the causaloid would query the context for historical data.
        // Here, we'll hardcode some logic based on the time step and assumed context data.

        let predicted_shipping_activity = match current_time_step as u64 {
            4 => {
                // Predicting for Q5, based on Q1-Q4
                // This is where the causaloid would query the context for historical data
                // For demonstration, we'll use a simple rule based on assumed historical data
                // If oil price data was available (not 0.0 in the context), it would influence this.
                // Since we can't access the context directly here, we'll make a simplified assumption.
                // If oil price was present (simulated by non-zero value), predict higher.
                // This part is highly simplified and would be replaced by actual model inference.
                let assumed_oil_price_present = true; // This would come from context query
                if assumed_oil_price_present {
                    105.0 + 3.0
                } else {
                    105.0
                }
            }
            _ => 0.0, // Default for other time steps
        };

        Ok(PropagatingEffect::Numerical(predicted_shipping_activity))
    };

    BaseCausaloid::new(predictor_id, causal_fn, predictor_description)
}
