/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Benchmarks for building a graph and for adding and reading nodes, on the chain fixture
//! (`utils::build_linear_graph`) at each size in `fields::SIZES`.

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group};
use std::hint::black_box;
use ultragraph::*;

use crate::benchmarks::data::Data;
use crate::benchmarks::fields::{INDEX_COUNT, SIZES};
use crate::benchmarks::utils::{
    build_linear_graph, build_linear_graph_with_capacity, random_indices,
};

/// Builds a chain of `n + 1` nodes and `n` edges from an empty graph, by inserts.
fn bench_build_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_graph");
    for n in SIZES {
        group.throughput(Throughput::Elements((n + 1) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_with_large_drop(|| build_linear_graph(black_box(n)))
        });
    }
    group.finish();
}

/// Adds one node to a chain that already holds `n + 1` nodes. The setup reserves room for one more
/// node, so the measured insert never reallocates the node tables; the amortised cost, growth
/// included, is the per-node rate of `build_graph`.
fn bench_add_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_node");
    for n in SIZES {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || build_linear_graph_with_capacity(n, n + 2),
                |g| g.add_node(black_box(Data::default())),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

/// Reads a random node of a chain of `n + 1` nodes.
fn bench_get_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_node");
    for n in SIZES {
        let g = build_linear_graph(n);
        let nodes = random_indices(0..n + 1);
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                cursor = (cursor + 1) & (INDEX_COUNT - 1);
                black_box(g.get_node(nodes[cursor]))
            })
        });
    }
    group.finish();
}

fn linear_graph_benchmarks(c: &mut Criterion) {
    bench_build_graph(c);
    bench_add_node(c);
    bench_get_node(c);
}

criterion_group! {
    name = linear_graph_bench_collection;
    config = Criterion::default().sample_size(100);
    targets = linear_graph_benchmarks
}
