/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod model;
mod types;

use deep_causality::*;

// Define IDs for different data types within the context
const OIL_PRICE_ID: IdentificationValue = 0;
const SHIPPING_ACTIVITY_ID: IdentificationValue = 1;
const TIME_ID: IdentificationValue = 2;

// Define ID for the causaloid
const PREDICTOR_CAUSALOID_ID: IdentificationValue = 1;

fn main() {
    println!("Granger Causality Example: Oil Prices and Shipping Activity ");
    // Create two instances of the causaloid, one for each context.
    let factual_causaloid = model::get_factual_causaloid(PREDICTOR_CAUSALOID_ID);
    let counterfactual_causaloid = model::get_counterfactual_causaloid(PREDICTOR_CAUSALOID_ID);

    // 2. Execute the Granger Test
    // Factual Evaluation (with oil price history). The oil price history is stored in the context,
    // therefore the initial PropagatingEffect is set to None as its not used in this example.
    let res_factual = factual_causaloid.evaluate(&PropagatingEffect::none());
    assert!(res_factual.is_ok());
    // For detailed result inspection, use the dbg! macro.
    //dbg!(&res_factual);

    let factual_prediction = res_factual.value.as_numerical().unwrap();
    println!(
        "Factual Prediction for Q5 Shipping Activity: {:.2}",
        factual_prediction
    );

    println!(
        "Explain Factual Prediction for Q5 Shipping Activity: {}",
        res_factual.explain()
    );

    // Counterfactual Evaluation (without oil price history). The shipment history is also stored in the context,
    // therefore the initial PropagatingEffect is set to None as its not used in this example.
    let res_counter_factual = counterfactual_causaloid.evaluate(&PropagatingEffect::none());
    assert!(res_counter_factual.is_ok());

    let counterfactual_prediction = res_counter_factual.value.as_numerical().unwrap();
    println!(
        "Counterfactual Prediction for Q5 Shipping Activity: {:.2}",
        counterfactual_prediction
    );

    println!(
        "Explain Counterfactual Prediction for Q5 Shipping Activity: {}",
        res_counter_factual.explain()
    );

    // 3. Compare and Conclude
    // This is the hypothetical "true" value for Q5, used to measure prediction error.
    let actual_q5_shipping = 105.0;
    let error_factual = (factual_prediction - actual_q5_shipping).abs();
    let error_counterfactual = (counterfactual_prediction - actual_q5_shipping).abs();

    println!("Granger Causality Conclusion");
    println!("Actual Q5 Shipping Activity: {:.2}", actual_q5_shipping);
    println!(
        "Factual Prediction Error (with oil data): {:.2}",
        error_factual
    );
    println!(
        "Counterfactual Prediction Error (no oil data): {:.2}",
        error_counterfactual
    );

    println!();
    if error_factual < error_counterfactual {
        println!("Conclusion: Past oil prices DO Granger-cause future shipping activity.");
        println!("Because the error is lower when oil price history is included.");
    } else {
        println!("Conclusion: Past oil prices DO NOT Granger-cause future shipping activity.");
        println!("Because including oil price history did not improve the prediction.");
    }
}
