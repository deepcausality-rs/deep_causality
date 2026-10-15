/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Benchmarks for the biconnectivity decomposition trio
//! (`articulation_points`, `bridges`, `biconnected_components`).
//!
//! Fixture: a chain of `k` triangles sharing consecutive vertices
//! (triangles `{0,1,2}`, `{2,3,4}`, `{4,5,6}`, ...). For `k` triangles:
//!   V = 2k + 1, E = 3k
//!   articulation_points = k - 1 (the shared vertices)
//!   bridges             = 0
//!   biconnected_components = k
//!
//! This shape exercises every branch of each algorithm (tree edges,
//! back edges, articulation-condition firing, edge-stack popping).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group};
use std::hint::black_box;
use ultragraph::*;

use crate::benchmarks::data::Data;
use crate::benchmarks::fields::SIZES;

/// 10K vertices, ~15K edges — matches the ServiceRadar-scale target
/// called out in the spec's complexity-guarantee scenario.
const XLARGE_TRIANGLES: usize = 5_000;

/// The triangle counts each algorithm runs at: the three shared sizes and `XLARGE_TRIANGLES`.
const TRIANGLE_COUNTS: [usize; 4] = [SIZES[0], SIZES[1], SIZES[2], XLARGE_TRIANGLES];

fn build_triangle_chain(num_triangles: usize) -> UltraGraph<Data> {
    let v = 2 * num_triangles + 1;
    let mut g: UltraGraphContainer<Data, _> = UltraGraph::with_capacity(v, None);
    for _ in 0..v {
        g.add_node(Data::default()).unwrap();
    }
    for k in 0..num_triangles {
        let a = 2 * k;
        let b = 2 * k + 1;
        let c = 2 * k + 2;
        g.add_edge(a, b, ()).unwrap();
        g.add_edge(b, c, ()).unwrap();
        g.add_edge(c, a, ()).unwrap();
    }
    g.freeze();
    g
}

/// Runs `algorithm` on a frozen chain of each triangle count in `TRIANGLE_COUNTS`, in a group
/// named `group_name` whose parameter is the triangle count and whose throughput is the edge
/// count, `3k`.
fn bench_on_triangle_chains<R>(
    c: &mut Criterion,
    group_name: &str,
    algorithm: impl Fn(&UltraGraph<Data>) -> R,
) {
    let mut group = c.benchmark_group(group_name);
    for k in TRIANGLE_COUNTS {
        let g = build_triangle_chain(k);
        group.throughput(Throughput::Elements((3 * k) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, _| {
            b.iter(|| algorithm(black_box(&g)))
        });
    }
    group.finish();
}

fn biconnectivity_benchmarks(c: &mut Criterion) {
    bench_on_triangle_chains(c, "articulation_points", |g| {
        g.articulation_points().unwrap()
    });
    bench_on_triangle_chains(c, "bridges", |g| g.bridges().unwrap());
    bench_on_triangle_chains(c, "biconnected_components", |g| {
        g.biconnected_components().unwrap()
    });
}

criterion_group! {
    name = biconnectivity_bench_collection;
    config = Criterion::default().sample_size(20);
    targets = biconnectivity_benchmarks
}
