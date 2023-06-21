/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use crate::prelude::Causaloid;
use crate::utils::{bench_utils_shared, test_utils};

const SMALL: usize = 1000;
const MEDIUM: usize = 100_000;
const LARGE: usize = 1_000_000;

pub fn get_small_collection_and_data()
    -> (Vec<Causaloid>, [f64; SMALL + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = SMALL;
    (build_causaloid_collection(k), bench_utils_shared::generate_sample_data(k))
}

pub fn get_medium_collection_and_data()
    -> (Vec<Causaloid>, [f64; MEDIUM + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = MEDIUM;
    (build_causaloid_collection(k), bench_utils_shared::generate_sample_data(k))
}

pub fn get_large_collection_and_data()
    -> (Vec<Causaloid>, [f64; LARGE + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = LARGE;
    (build_causaloid_collection(k), bench_utils_shared::generate_sample_data(k))
}

fn build_causaloid_collection(
    k: usize
)
    -> Vec<Causaloid>
{
    let mut v = Vec::with_capacity(k);

    for _ in 0..k {
        v.push(test_utils::get_test_causaloid());
    }

    v
}
