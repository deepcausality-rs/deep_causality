// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Causaloid, Dataoid, Spaceoid, SpaceTempoid, Tempoid};

use crate::benchmarks::utils_shared;

const SMALL: usize = 10;
const MEDIUM: usize = 1_000;
const LARGE: usize = 10_000;

type CausalVector = Vec<Causaloid<'static, Dataoid, Spaceoid, Tempoid, SpaceTempoid>>;

pub fn get_small_collection_and_data()
    -> (CausalVector, [f64; SMALL + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_causaloid_collection(SMALL), utils_shared::generate_sample_data())
}

pub fn get_medium_collection_and_data()
    -> (CausalVector, [f64; MEDIUM + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_causaloid_collection(MEDIUM), utils_shared::generate_sample_data())
}

pub fn get_large_collection_and_data()
    -> (CausalVector, [f64; LARGE + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_causaloid_collection(LARGE), utils_shared::generate_sample_data())
}

fn build_causaloid_collection(
    k: usize
)
    -> CausalVector
{
    let mut v = Vec::with_capacity(k);

    for _ in 0..k {
        v.push(utils_shared::get_test_causaloid());
    }

    v
}
