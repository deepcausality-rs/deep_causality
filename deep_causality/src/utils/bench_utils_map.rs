// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;

use crate::prelude::{Causaloid, Dataoid, Spaceoid, SpaceTempoid, Tempoid};
use crate::utils::{bench_utils_shared, test_utils};

const SMALL: usize = 10;
const MEDIUM: usize = 1_000;
const LARGE: usize = 10_000;

type CausalMap = HashMap<usize, Causaloid<'static, Dataoid, Spaceoid, Tempoid, SpaceTempoid>>;

pub fn get_small_map_and_data()
    -> (CausalMap, [f64; SMALL + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = SMALL;
    (build_causality_map(k), bench_utils_shared::generate_sample_data())
}

pub fn get_medium_map_and_data()
    -> (CausalMap, [f64; MEDIUM + 1])
{ // Builds a linear graph: root -> a -> b -> c
    let k = MEDIUM;
    (build_causality_map(k), bench_utils_shared::generate_sample_data())
}

pub fn get_large_map_and_data()
    -> (CausalMap, [f64; LARGE + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_causality_map(LARGE), bench_utils_shared::generate_sample_data())
}

fn build_causality_map(
    k: usize
)
    -> CausalMap
{
    let mut v = HashMap::with_capacity(k);
    for k in 0..k {
        v.insert(k, test_utils::get_test_causaloid());
    }

    v
}
