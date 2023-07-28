// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::Causaloid;
use crate::utils::{bench_utils_shared, test_utils};

const SMALL: usize = 100;
const MEDIUM: usize = 10_000;
const LARGE: usize = 100_000;

pub fn get_small_collection_and_data()
    -> (Vec<Causaloid<'static>>, [f64; SMALL + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_causaloid_collection(SMALL), bench_utils_shared::generate_sample_data())
}

pub fn get_medium_collection_and_data()
    -> (Vec<Causaloid<'static>>, [f64; MEDIUM + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_causaloid_collection(MEDIUM), bench_utils_shared::generate_sample_data())
}

pub fn get_large_collection_and_data()
    -> (Vec<Causaloid<'static>>, [f64; LARGE + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_causaloid_collection(LARGE), bench_utils_shared::generate_sample_data())
}

fn build_causaloid_collection(
    k: usize
)
    -> Vec<Causaloid<'static>>
{
    let mut v = Vec::with_capacity(k);

    for _ in 0..k {
        v.push(test_utils::get_test_causaloid());
    }

    v
}
