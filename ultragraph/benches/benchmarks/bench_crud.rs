/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Benchmarks for adding, reading, updating and removing edges and nodes, for traversal, and for
//! the `freeze` and `unfreeze` transitions, at each size in `fields::SIZES`.
//!
//! Fixtures:
//! - chain (`utils::build_linear_graph(n)`): a root and `n` nodes linked in a line, every node of
//!   degree at most 1. Used for node operations, traversal and the state transitions.
//! - star (`utils::build_star_graph(n)`): a hub at index 0 with an edge to each of `n` leaves, so
//!   the hub's degree is `n`. Used for edge operations, whose cost depends on the source's degree.
//! - shuffled chain (`utils::build_shuffled_chain(n)`): the chain laid out in a seeded random order
//!   of positions. Used for the `*_walk_shuffled` benchmarks, whose steps land at unpredictable
//!   addresses; the ordered walks are the best case, where every step moves by one position.
//!
//! A benchmark that changes the graph builds a fresh fixture in its setup and receives it by
//! reference (`iter_batched_ref`), so criterion excludes both the build and the drop from the
//! measurement. Every random index comes from the generator seeded with `fields::SEED`: read
//! benchmarks cycle through a precomputed sequence, so the generator's cost stays out of the
//! measurement, and two runs measure the same indices.

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group};
use deep_causality_rand::Rng;
use std::hint::black_box;
use ultragraph::*;

use crate::benchmarks::data::Data;
use crate::benchmarks::fields::{INDEX_COUNT, SIZES};
use crate::benchmarks::utils::{
    build_linear_graph, build_shuffled_chain, build_star_graph, random_indices, seeded_rng,
};

const HUB: usize = 0;

// --- Create ---------------------------------------------------------------------------------

/// Adds an edge from the hub of a star with `n` leaves to one of its leaves.
fn bench_add_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_edge");
    for n in SIZES {
        let mut rng = seeded_rng();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || (build_star_graph(n), rng.random_range(1..n + 1)),
                |(g, leaf)| g.add_edge(black_box(HUB), *leaf, ()),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

// --- Read -----------------------------------------------------------------------------------

/// Tests for the edge from the hub of a star with `n` leaves to a random leaf.
fn bench_contains_edge(c: &mut Criterion, group_name: &str, frozen: bool) {
    let mut group = c.benchmark_group(group_name);
    for n in SIZES {
        let mut g = build_star_graph(n);
        if frozen {
            g.freeze();
        }
        let leaves = random_indices(1..n + 1);
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                cursor = (cursor + 1) & (INDEX_COUNT - 1);
                black_box(g.contains_edge(black_box(HUB), leaves[cursor]))
            })
        });
    }
    group.finish();
}

/// Lists the `n` outgoing edges of the hub of a star, in the dynamic state.
fn bench_get_edges(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_edges");
    for n in SIZES {
        let g = build_star_graph(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(g.get_edges(black_box(HUB))))
        });
    }
    group.finish();
}

/// Walks a frozen chain of `n + 1` nodes from the root to its end through `outbound_edges`.
fn bench_outbound_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("outbound_walk");
    for n in SIZES {
        let mut g = build_linear_graph(n);
        g.freeze();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut node = black_box(0);
                let mut steps = 0_usize;
                while let Some(next) = g
                    .outbound_edges(node)
                    .expect("Failed to list outbound edges")
                    .next()
                {
                    node = next;
                    steps += 1;
                }
                steps
            })
        });
    }
    group.finish();
}

/// Walks a frozen chain of `n + 1` nodes from its end back to the root through `inbound_edges`.
fn bench_inbound_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("inbound_walk");
    for n in SIZES {
        let mut g = build_linear_graph(n);
        g.freeze();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut node = black_box(n);
                let mut steps = 0_usize;
                while let Some(previous) = g
                    .inbound_edges(node)
                    .expect("Failed to list inbound edges")
                    .next()
                {
                    node = previous;
                    steps += 1;
                }
                steps
            })
        });
    }
    group.finish();
}

/// Walks a frozen chain of `n + 1` nodes, laid out in a seeded random order, from its first node to
/// its last through `outbound_edges`. Unlike `outbound_walk`, consecutive steps land at
/// unpredictable addresses, so the CPU cannot run ahead by predicting the next one.
fn bench_outbound_walk_shuffled(c: &mut Criterion) {
    let mut group = c.benchmark_group("outbound_walk_shuffled");
    for n in SIZES {
        let (mut g, first, _) = build_shuffled_chain(n);
        g.freeze();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut node = black_box(first);
                let mut steps = 0_usize;
                while let Some(next) = g
                    .outbound_edges(node)
                    .expect("Failed to list outbound edges")
                    .next()
                {
                    node = next;
                    steps += 1;
                }
                steps
            })
        });
    }
    group.finish();
}

/// Walks the shuffled chain of `bench_outbound_walk_shuffled` from its last node back to its first
/// through `inbound_edges`.
fn bench_inbound_walk_shuffled(c: &mut Criterion) {
    let mut group = c.benchmark_group("inbound_walk_shuffled");
    for n in SIZES {
        let (mut g, _, last) = build_shuffled_chain(n);
        g.freeze();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut node = black_box(last);
                let mut steps = 0_usize;
                while let Some(previous) = g
                    .inbound_edges(node)
                    .expect("Failed to list inbound edges")
                    .next()
                {
                    node = previous;
                    steps += 1;
                }
                steps
            })
        });
    }
    group.finish();
}

// --- Update ---------------------------------------------------------------------------------

/// Replaces the payload of a random node of a chain of `n + 1` nodes.
fn bench_update_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_node");
    for n in SIZES {
        let mut g = build_linear_graph(n);
        let nodes = random_indices(0..n + 1);
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                cursor = (cursor + 1) & (INDEX_COUNT - 1);
                g.update_node(nodes[cursor], black_box(Data::default()))
            })
        });
    }
    group.finish();
}

// --- Delete ---------------------------------------------------------------------------------

/// Removes a random node, other than the root, from a chain of `n + 1` nodes.
fn bench_remove_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_node");
    for n in SIZES {
        let mut rng = seeded_rng();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || (build_linear_graph(n), rng.random_range(1..n + 1)),
                |(g, node)| g.remove_node(*node),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

/// Removes the edge from the hub of a star with `n` leaves to a random leaf.
fn bench_remove_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_edge");
    for n in SIZES {
        let mut rng = seeded_rng();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || (build_star_graph(n), rng.random_range(1..n + 1)),
                |(g, leaf)| g.remove_edge(black_box(HUB), *leaf),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

// --- State transitions ----------------------------------------------------------------------

/// Freezes a dynamic chain of `n + 1` nodes.
fn bench_freeze(c: &mut Criterion) {
    let mut group = c.benchmark_group("freeze");
    for n in SIZES {
        group.throughput(Throughput::Elements((n + 1) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || build_linear_graph(n),
                |g| g.freeze(),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

/// Unfreezes a frozen chain of `n + 1` nodes.
fn bench_unfreeze(c: &mut Criterion) {
    let mut group = c.benchmark_group("unfreeze");
    for n in SIZES {
        group.throughput(Throughput::Elements((n + 1) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || {
                    let mut g = build_linear_graph(n);
                    g.freeze();
                    g
                },
                |g| g.unfreeze(),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

fn crud_benchmarks(c: &mut Criterion) {
    bench_add_edge(c);

    bench_contains_edge(c, "contains_edge_dynamic", false);
    bench_contains_edge(c, "contains_edge_frozen", true);
    bench_get_edges(c);
    bench_outbound_walk(c);
    bench_inbound_walk(c);
    bench_outbound_walk_shuffled(c);
    bench_inbound_walk_shuffled(c);

    bench_update_node(c);

    bench_remove_node(c);
    bench_remove_edge(c);

    bench_freeze(c);
    bench_unfreeze(c);
}

criterion_group! {
    name = crud_bench_collection;
    config = Criterion::default().sample_size(100);
    targets = crud_benchmarks
}
