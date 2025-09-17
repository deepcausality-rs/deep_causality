/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr;
use deep_causality_data_structures::CausalTensor;

#[test]
fn test_mrmr_select_features() {
    let data = vec![
        // F0,  F1,  F2,  Target
        1.0, 2.0, 3.0, 1.6, 2.0, 4.1, 6.0, 3.5, 3.0, 6.2, 9.0, 5.5, 4.0, 8.1, 12.0, 7.5,
    ];
    let mut tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    let selected_features = mrmr::select_features(&mut tensor, 2, 3).unwrap();

    // Based on calculation, F2 is most relevant, then F0 is chosen due to lower redundancy.
    assert_eq!(selected_features, vec![2, 0]);
}
