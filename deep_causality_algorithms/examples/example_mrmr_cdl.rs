/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::mrmr::mrmr_features_selector;
use deep_causality_tensor::CausalTensor;

fn main() {
    // 1. Prepare your data with some missing values (None)
    let data = vec![
        // F0,        F1,        F2,        Target
        Some(10.0),
        Some(12.0),
        Some(1.0),
        Some(11.0),
        Some(20.0),
        Some(21.0),
        None,
        Some(22.0), // F2 missing
        Some(30.0),
        None,
        Some(2.0),
        Some(31.0), // F1 missing
        Some(40.0),
        Some(40.0),
        Some(8.0),
        None, // Target missing
        Some(50.0),
        Some(55.0),
        Some(3.0),
        Some(52.0),
    ];
    let tensor = CausalTensor::new(data, vec![5, 4]).unwrap();

    // 2. Run the feature selector
    // Select 2 features, with the target variable in column 3.
    let selected_features_with_scores = mrmr_features_selector(&tensor, 3, 3).unwrap();

    // 3. Interpret the results
    println!("Selected features and their normalized scores (Generic MRMR):");
    for (index, score) in selected_features_with_scores {
        println!("- Feature Index: {}, Importance Score: {:.4}", index, score);
    }
}
