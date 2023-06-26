// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;

use crate::prelude::Causaloid;
use crate::utils::{bench_utils_shared, test_utils};

const SMALL: usize = 100;
const MEDIUM: usize = 10_000;
const LARGE: usize = 100_000;


pub fn get_small_map_and_data()
    -> (HashMap<usize, Causaloid>, [f64; SMALL + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = SMALL;
    (build_causality_map(k), bench_utils_shared::generate_sample_data(k))
}

pub fn get_medium_map_and_data()
    -> (HashMap<usize, Causaloid>, [f64; MEDIUM + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = MEDIUM;
    (build_causality_map(k), bench_utils_shared::generate_sample_data(k))
}

pub fn get_large_map_and_data()
    -> (HashMap<usize, Causaloid>, [f64; LARGE + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = LARGE;
    (build_causality_map(k), bench_utils_shared::generate_sample_data(k))
}

fn build_causality_map(
    k: usize
)
    -> HashMap<usize, Causaloid>
{
    let mut v = HashMap::with_capacity(k);
    for k in 0..k {
        v.insert(k, test_utils::get_test_causaloid());
    }

    v
}
