/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group, criterion_main};
use deep_causality_uncertain::{Uncertain, UncertainBool};

// --- Sampling Performance Benchmarks ---

fn bench_sampling_point(c: &mut Criterion) {
    let uncertain = Uncertain::<f64>::point(10.0);
    c.bench_function("sampling_point", |b| {
        b.iter(|| uncertain.sample_from_entropy().unwrap());
    });
}

fn bench_sampling_normal(c: &mut Criterion) {
    let uncertain = Uncertain::<f64>::normal(10.0, 2.0);
    c.bench_function("sampling_normal", |b| {
        b.iter(|| uncertain.sample_from_entropy().unwrap());
    });
}

fn bench_sampling_bernoulli(c: &mut Criterion) {
    let uncertain = UncertainBool::<f64>::bernoulli(0.5);
    c.bench_function("sampling_bernoulli", |b| {
        b.iter(|| uncertain.sample_from_entropy().unwrap());
    });
}

// Benchmark for a simple arithmetic chain
fn bench_sampling_arithmetic_chain(c: &mut Criterion) {
    let a = Uncertain::<f64>::normal(10.0, 1.0);
    let b = Uncertain::<f64>::normal(5.0, 0.5);
    let c_val = Uncertain::<f64>::normal(2.0, 0.1);
    let d = (a + b) * c_val; // (Normal + Normal) * Normal

    c.bench_function("sampling_arithmetic_chain", |b| {
        b.iter(|| d.sample_from_entropy().unwrap());
    });
}

// Benchmark for a conditional operation
fn bench_sampling_conditional(c: &mut Criterion) {
    let condition = UncertainBool::<f64>::bernoulli(0.5);
    let if_true = Uncertain::<f64>::normal(100.0, 10.0);
    let if_false = Uncertain::<f64>::normal(200.0, 20.0);
    let result = Uncertain::conditional(condition, if_true, if_false);

    c.bench_function("sampling_conditional", |b| {
        b.iter(|| result.sample_from_entropy().unwrap());
    });
}

// --- Operator Overhead Benchmarks (Graph Construction) ---

// Benchmark for simple f64 addition (baseline)
fn bench_f64_add_baseline(c: &mut Criterion) {
    let x = 10.0;
    let y = 5.0;
    c.bench_function("f64_add_baseline", |b| {
        b.iter(|| x + y);
    });
}

// Benchmark for Uncertain<f64> addition (graph construction only)
fn bench_uncertain_f64_add_graph_construction(c: &mut Criterion) {
    let a = Uncertain::<f64>::point(10.0);
    let b = Uncertain::<f64>::point(5.0);
    c.bench_function("uncertain_f64_add_graph_construction", |bencher| {
        bencher.iter(|| {
            let cloned_a = a.clone();
            let cloned_b = b.clone();
            let _ = cloned_a + cloned_b;
        });
    });
}

// Benchmark for map operation (graph construction only)
fn bench_uncertain_f64_map_graph_construction(c: &mut Criterion) {
    let uncertain = Uncertain::<f64>::point(10.0);
    c.bench_function("uncertain_f64_map_graph_construction", |b| {
        b.iter(|| {
            let _ = uncertain.clone().map(|x| x * 2.0);
        });
    });
}

// --- Combined Benchmarks (Graph Construction + Sampling) ---

// Benchmark for a simple arithmetic operation including sampling
fn bench_uncertain_f64_add_and_sample(c: &mut Criterion) {
    let a = Uncertain::<f64>::normal(10.0, 1.0);
    let b = Uncertain::<f64>::normal(5.0, 0.5);
    c.bench_function("uncertain_f64_add_and_sample", |bencher| {
        bencher.iter(|| {
            let cloned_a = a.clone();
            let cloned_b = b.clone();
            let sum = cloned_a + cloned_b;
            sum.sample_from_entropy().unwrap();
        });
    });
}

// Benchmark for a complex chain including sampling
fn bench_complex_chain_and_sample(c: &mut Criterion) {
    let s1 = Uncertain::<f64>::normal(50.0, 2.0);
    let s2 = Uncertain::<f64>::normal(52.0, 1.5);
    let s3 = Uncertain::<f64>::normal(48.0, 1.0);

    let combined_reading = (s1 + s2 + s3) / Uncertain::<f64>::point(3.0);
    let is_high = combined_reading.greater_than(51.0);
    let final_decision = Uncertain::conditional(
        is_high,
        Uncertain::<f64>::point(1.0),
        Uncertain::<f64>::point(0.0),
    );

    c.bench_function("complex_chain_and_sample", |b| {
        b.iter(|| {
            final_decision.sample_from_entropy().unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_sampling_point,
    bench_sampling_normal,
    bench_sampling_bernoulli,
    bench_sampling_arithmetic_chain,
    bench_sampling_conditional,
    bench_f64_add_baseline,
    bench_uncertain_f64_add_graph_construction,
    bench_uncertain_f64_map_graph_construction,
    bench_uncertain_f64_add_and_sample,
    bench_complex_chain_and_sample,
);
criterion_main!(benches);
