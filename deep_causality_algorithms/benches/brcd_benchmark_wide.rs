/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Wide-graph benchmark for Bayesian Root Cause Discovery (`brcd_run`).
//!
//! Standalone because it is slower than the three-node benchmark. It approximates
//! the real-world shape: many variables at a realistic ~600-row regime size, over
//! a **mostly-directed** CPDAG (a directed backbone with a few isolated undirected
//! edges), matching the reversed-service-map CPDAGs the real cases use.
//!
//! The structural cost of BRCD is exponential in the size of the undirected
//! components (the cut-configuration enumeration; see the paper's Appendix E). A
//! fully-undirected chain is therefore a pathological worst case — ~5 ms at 10
//! variables but ~3.8 s at 20. Real CPDAGs are well-oriented, so this benchmark
//! keeps the undirected components small and isolated to stay representative.
//!
//! Run: `cargo bench -p deep_causality_algorithms --bench brcd_benchmark_wide`

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use deep_causality_algorithms::brcd::{BrcdConfig, brcd_run};
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::hint::black_box;

/// A wide linear-Gaussian chain `X0 → X1 → ... → X(d-1)` over `d` variables and
/// `n` rows. `anomaly_node` has its intercept shifted (the injected root cause).
fn wide_chain_data(
    d: usize,
    n: usize,
    anomaly_node: usize,
    shift: f64,
    seed: u64,
) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * d);
    for _ in 0..n {
        let mut prev = 0.0_f64;
        for j in 0..d {
            let intercept = if j == anomaly_node { shift } else { 0.0 };
            let v = intercept + 0.9 * prev + dist.sample(&mut rng);
            data.push(v);
            prev = v;
        }
    }
    CausalTensor::new(data, vec![n, d]).unwrap()
}

/// A mostly-directed CPDAG over `d` variables: a directed backbone
/// `X0 → X1 → ... → X(d-1)`, with a few isolated edges left undirected to exercise
/// the cut-configuration path without the exponential blow-up a large undirected
/// component would cause.
fn wide_cpdag(d: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); d], vec![d]).unwrap();
    let mut g = MixedGraph::new(d, data, 0).unwrap();
    for i in 0..d - 1 {
        // Sparse, isolated undirected edges (every fifth backbone edge); the rest
        // are directed, so each undirected component stays a single edge.
        if i % 5 == 2 {
            g.add_undirected(i, i + 1).unwrap();
        } else {
            g.add_arc(i, i + 1).unwrap();
        }
    }
    g
}

fn brcd_wide_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BRCD Wide Graph");
    // Real-world regime size (OB / Sock Shop windows are ~600 rows).
    let n_rows = 600_usize;
    let config = BrcdConfig::continuous(7);

    for &d in &[10_usize, 20, 40] {
        let anomaly = d / 2;
        let normal = wide_chain_data(d, n_rows, anomaly, 0.0, 1);
        let anomalous = wide_chain_data(d, n_rows, anomaly, 4.0, 2);
        let graph = wide_cpdag(d);
        group.bench_with_input(BenchmarkId::new("continuous_vars", d), &d, |b, _| {
            b.iter(|| black_box(brcd_run(&normal, &anomalous, &graph, &config).unwrap()));
        });
    }

    group.finish();
}

criterion_group!(benches, brcd_wide_benchmark);
criterion_main!(benches);
