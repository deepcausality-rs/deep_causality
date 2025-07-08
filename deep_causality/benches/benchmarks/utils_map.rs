/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use deep_causality::{BaseCausalMap, NumericalValue};

use crate::benchmarks::utils_shared;

const SMALL: usize = 10;
const MEDIUM: usize = 1_000;
const LARGE: usize = 10_000;

pub fn get_small_map_and_data() -> (BaseCausalMap, HashMap<usize, NumericalValue>) {
    (build_causality_map(SMALL), generate_data_map(SMALL))
}

pub fn get_medium_map_and_data() -> (BaseCausalMap, HashMap<usize, NumericalValue>) {
    (build_causality_map(MEDIUM), generate_data_map(MEDIUM))
}

pub fn get_large_map_and_data() -> (BaseCausalMap, HashMap<usize, NumericalValue>) {
    (build_causality_map(LARGE), generate_data_map(LARGE))
}

fn build_causality_map(k: usize) -> BaseCausalMap {
    let mut map = HashMap::with_capacity(k);
    for i in 0..k {
        // All causaloids are functionally identical, which is fine for this benchmark.
        // The differentiation comes from the data passed during evaluation via the map key.
        map.insert(i, utils_shared::get_test_causaloid());
    }
    map
}

fn generate_data_map(k: usize) -> HashMap<usize, NumericalValue> {
    let mut map = HashMap::with_capacity(k);
    for i in 0..k {
        // Use the same sample data value for each entry.
        map.insert(i, 0.99);
    }
    map
}
