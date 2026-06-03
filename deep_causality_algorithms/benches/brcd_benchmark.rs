/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Benchmarks for Bayesian Root Cause Discovery (`brcd_run`).
//!
//! Built on the base verification example: a linear-Gaussian chain `X → Y → Z`
//! where the anomalous regime perturbs `p(Y | X)`. The continuous (ridge-Gaussian)
//! family is swept across per-regime row counts to show how the per-family fits
//! scale with sample size; the discrete (Dirichlet) family is measured at one
//! representative size for comparison. The CPDAG is the fixed three-node chain, so
//! the timing isolates the estimator rather than the structural enumeration.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use deep_causality_algorithms::brcd::{BrcdConfig, brcd_run};
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::hint::black_box;

/// Linear-Gaussian chain `X = εx`, `Y = y_intercept + 1.5·X + εy`, `Z = 2·Y + εz`
/// (the base-verification generator). Columns are `[X, Y, Z]`; `n` rows.
fn chain_data(n: usize, y_intercept: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = dist.sample(&mut rng);
        let y = y_intercept + 1.5 * x + dist.sample(&mut rng);
        let z = 2.0 * y + dist.sample(&mut rng);
        data.extend_from_slice(&[x, y, z]);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

/// A discrete chain `X → Y → Z` with integer states in `{0, 1, 2}`; `shift`
/// perturbs Y's mechanism between regimes.
fn discrete_chain(n: usize, shift: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let bucket = |v: f64| -> f64 {
        if v < -0.5 {
            0.0
        } else if v < 0.5 {
            1.0
        } else {
            2.0
        }
    };
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = bucket(dist.sample(&mut rng));
        let y = bucket(0.8 * x - 0.8 + shift + dist.sample(&mut rng));
        let z = bucket(0.8 * y - 0.8 + dist.sample(&mut rng));
        data.extend_from_slice(&[x, y, z]);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

/// The undirected chain CPDAG `X — Y — Z`.
fn chain_cpdag() -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
    let mut g = MixedGraph::new(3, data, 0).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g
}

fn brcd_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BRCD Root-Cause Discovery");
    let graph = chain_cpdag();

    // Continuous (ridge-Gaussian) family across a sweep of per-regime row counts.
    let config = BrcdConfig::continuous(7);
    for &n in &[100_usize, 500, 1000] {
        let normal = chain_data(n, 0.0, 1);
        let anomalous = chain_data(n, 4.0, 2);
        group.bench_with_input(BenchmarkId::new("continuous", n), &n, |b, _| {
            b.iter(|| black_box(brcd_run(&normal, &anomalous, &graph, &config).unwrap()));
        });
    }

    // Discrete (Dirichlet) family at a representative size.
    let discrete_config = BrcdConfig::discrete(7);
    let n = 500_usize;
    let normal = discrete_chain(n, 0.0, 3);
    let anomalous = discrete_chain(n, 1.5, 4);
    group.bench_with_input(BenchmarkId::new("discrete", n), &n, |b, _| {
        b.iter(|| black_box(brcd_run(&normal, &anomalous, &graph, &discrete_config).unwrap()));
    });

    group.finish();
}

criterion_group!(benches, brcd_benchmark);
criterion_main!(benches);
