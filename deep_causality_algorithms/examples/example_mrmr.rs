/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::mrmr::mrmr_features_selector;
use deep_causality_tensor::CausalTensor;

fn main() {
    // 1. Prepare your data
    let data = vec![
        // F0,  F1,  F2,  Target
        10.0, 12.0, 1.0,
        11.0, // F0 is close to Target, F1 is also somewhat close, F2 is random
        20.0, 21.0, 5.0, 22.0, 30.0, 33.0, 2.0, 31.0, 40.0, 40.0, 8.0, 43.0, 50.0, 55.0, 3.0, 52.0,
    ];
    let tensor = CausalTensor::new(data, vec![5, 4]).unwrap();

    // 2. Run the feature selector
    // Select 2 features, with the target variable in column 3.
    let selected_features_with_scores = mrmr_features_selector(&tensor, 3, 3).unwrap();

    // 3. Interpret the results
    println!("Selected features and their scores:");
    for (index, score) in selected_features_with_scores {
        println!("- Feature Index: {}, Importance Score: {:.4}", index, score);
    }
}
