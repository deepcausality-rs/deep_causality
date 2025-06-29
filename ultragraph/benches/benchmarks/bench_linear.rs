/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};
use rand::Rng;

use ultragraph::prelude::*;

use crate::benchmarks::data::Data;
use crate::benchmarks::fields::{LARGE, MEDIUM, SMALL};
use crate::benchmarks::utils;

// Graph size
// const SMALL: usize = 10;
// const MEDIUM: usize = 100;
// const LARGE: usize = 1_000;

fn get_empty_ultra_graph(capacity: usize) -> UltraGraph<Data> {
    ultragraph::new_with_matrix_storage::<Data>(capacity)
}

fn get_pre_filled_ultra_graph(capacity: usize) -> UltraGraph<Data> {
    match capacity {
        SMALL => utils::build_linear_graph(SMALL),
        MEDIUM => utils::build_linear_graph(MEDIUM),
        LARGE => utils::build_linear_graph(LARGE),
        _ => unreachable!(),
    }
}

fn small_add_node_benchmark(criterion: &mut Criterion) {
    let capacity = SMALL;
    let mut g = get_empty_ultra_graph(capacity);
    criterion.bench_function("small_add_node", |bencher| {
        bencher.iter(|| g.add_node(Data::default()))
    });
}

fn small_get_node_benchmark(criterion: &mut Criterion) {
    let capacity = SMALL;
    let g = get_pre_filled_ultra_graph(capacity);

    criterion.bench_function("small_get_node", |bencher| {
        bencher.iter(|| g.get_node(rand::rng().random_range(0..capacity)))
    });
}

fn medium_add_node_benchmark(criterion: &mut Criterion) {
    let capacity = MEDIUM;
    let mut g = get_empty_ultra_graph(capacity);
    criterion.bench_function("medium_add_node", |bencher| {
        bencher.iter(|| g.add_node(Data::default()))
    });
}

fn medium_get_node_benchmark(criterion: &mut Criterion) {
    let capacity = MEDIUM;
    let g = get_pre_filled_ultra_graph(capacity);

    criterion.bench_function("medium_get_node", |bencher| {
        bencher.iter(|| g.get_node(rand::rng().random_range(0..capacity)))
    });
}

fn large_add_node_benchmark(criterion: &mut Criterion) {
    let capacity = LARGE;
    let mut g = get_empty_ultra_graph(capacity);
    criterion.bench_function("large_add_node", |bencher| {
        bencher.iter(|| g.add_node(Data::default()))
    });
}

fn large_get_node_benchmark(criterion: &mut Criterion) {
    let capacity = LARGE;
    let g = get_pre_filled_ultra_graph(capacity);

    criterion.bench_function("array_push", |bencher| {
        bencher.iter(|| g.get_node(rand::rng().random_range(0..capacity)))
    });
}

criterion_group! {
    name = liner_graph_bench_collection;
    config = Criterion::default().sample_size(100);
    targets =
    small_add_node_benchmark,
    small_get_node_benchmark,
    medium_add_node_benchmark,
    medium_get_node_benchmark,
    large_add_node_benchmark,
    large_get_node_benchmark,
}
