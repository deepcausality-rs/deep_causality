// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::collections::HashMap;

use deep_causality::prelude::BaseCausalMap;

use crate::benchmarks::utils_shared;

const SMALL: usize = 10;
const MEDIUM: usize = 1_000;
const LARGE: usize = 10_000;

pub fn get_small_map_and_data<'l>() -> (BaseCausalMap, [f64; SMALL + 1]) {
    // Builds a linear graph: root -> a -> b -> c
    let k = SMALL;
    (build_causality_map(k), utils_shared::generate_sample_data())
}

pub fn get_medium_map_and_data<'l>() -> (BaseCausalMap, [f64; MEDIUM + 1]) {
    // Builds a linear graph: root -> a -> b -> c
    let k = MEDIUM;
    (build_causality_map(k), utils_shared::generate_sample_data())
}

pub fn get_large_map_and_data<'l>() -> (BaseCausalMap, [f64; LARGE + 1]) {
    // Builds a linear graph: root -> a -> b -> c
    (
        build_causality_map(LARGE),
        utils_shared::generate_sample_data(),
    )
}

fn build_causality_map<'l>(k: usize) -> BaseCausalMap {
    let mut v = HashMap::with_capacity(k);
    for k in 0..k {
        v.insert(k, utils_shared::get_test_causaloid());
    }

    v
}
