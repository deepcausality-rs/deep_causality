/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::BaseCausaloidVec;

use crate::benchmarks::utils_shared;

const SMALL: usize = 10;
const MEDIUM: usize = 1_000;
const LARGE: usize = 10_000;

pub fn get_small_collection_and_data() -> (BaseCausaloidVec<f64, bool>, [f64; SMALL]) {
    (
        build_causaloid_collection(SMALL),
        utils_shared::generate_sample_data(),
    )
}

pub fn get_medium_collection_and_data() -> (BaseCausaloidVec<f64, bool>, [f64; MEDIUM]) {
    (
        build_causaloid_collection(MEDIUM),
        utils_shared::generate_sample_data(),
    )
}

pub fn get_large_collection_and_data() -> (BaseCausaloidVec<f64, bool>, [f64; LARGE]) {
    (
        build_causaloid_collection(LARGE),
        utils_shared::generate_sample_data(),
    )
}

fn build_causaloid_collection(k: usize) -> BaseCausaloidVec<f64, bool> {
    let mut v = Vec::with_capacity(k);

    for _ in 0..k {
        v.push(utils_shared::get_test_causaloid());
    }

    v
}
