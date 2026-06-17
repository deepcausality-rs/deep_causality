/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Holistic benchmark for the **MMS verification** solver (`CfdFlow::verify`).
//!
//! Two axes exercise both verification modes:
//!
//! - **pointwise** — the analytic Taylor–Green residual sampled at one point, swept over viscosities
//!   `0.01…1.0` (the pure manufactured-solution evaluation, no march);
//! - **amplitude_march** — the decaying-amplitude check `a(t) = exp(−2νt)`, swept over step budgets
//!   `10…100` (cost grows with the marched horizon).
//!
//! Configs are built once per case; only the `verify` run is timed.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, TaylorGreen};
use std::hint::black_box;

/// Density for the Taylor–Green manufactured field.
const RHO: f64 = 1.0;
/// Sample point shared by both axes.
const SAMPLE: [f64; 3] = [1.0, 0.5, 0.0];
/// Time step for the amplitude-march axis.
const AMP_DT: f64 = 0.01;

fn bench_verify_pointwise(c: &mut Criterion) {
    let mut group = c.benchmark_group("mms_verify/pointwise");
    for &nu in &[0.01f64, 0.1, 1.0] {
        let config = CfdConfigBuilder::verify::<f64, _>("bench-verify", TaylorGreen::new(nu, RHO))
            .sample_at(SAMPLE, 0.0)
            .build()
            .expect("verify config");
        group.bench_with_input(BenchmarkId::from_parameter(nu), &nu, |b, _| {
            b.iter(|| {
                let report = CfdFlow::verify(black_box(&config))
                    .run()
                    .expect("verify runs");
                black_box(report);
            })
        });
    }
    group.finish();
}

fn bench_verify_amplitude_march(c: &mut Criterion) {
    let mut group = c.benchmark_group("mms_verify/amplitude_march");
    for &steps in &[10usize, 50, 100] {
        let config = CfdConfigBuilder::verify::<f64, _>("bench-amp", TaylorGreen::new(0.1, RHO))
            .sample_at(SAMPLE, 0.0)
            .amplitude_march(AMP_DT, steps)
            .build()
            .expect("verify config");
        group.bench_with_input(BenchmarkId::from_parameter(steps), &steps, |b, _| {
            b.iter(|| {
                let report = CfdFlow::verify(black_box(&config))
                    .run()
                    .expect("verify runs");
                black_box(report);
            })
        });
    }
    group.finish();
}

criterion_group!(
    mms_verify_benches,
    bench_verify_pointwise,
    bench_verify_amplitude_march
);
criterion_main!(mms_verify_benches);
