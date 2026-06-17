/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Holistic benchmark for the **operator-accuracy** solver (`CfdFlow::operator_study`).
//!
//! The viscous DEC operator `δd` is swept over resolution **ladders** of growing length
//! (`[16,32]`, `[16,32,64]`, `[16,32,64,128]`). The study evaluates the operator on a periodic torus
//! at each rung and computes the observed convergence order between rungs, so the cost is dominated by
//! the largest rung; the ladder sweep shows how the study scales as finer rungs are added. `Operator`
//! currently has a single variant (`Viscous`); add variants here when more land.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use deep_causality_cfd::{CfdFlow, Operator};
use std::hint::black_box;

fn bench_operator_ladders(c: &mut Criterion) {
    let mut group = c.benchmark_group("operator_study/viscous");
    let ladders: [&[usize]; 3] = [&[16, 32], &[16, 32, 64], &[16, 32, 64, 128]];
    for ladder in ladders {
        let label = format!("{}rungs_to_{}", ladder.len(), ladder.last().unwrap());
        group.bench_with_input(BenchmarkId::from_parameter(label), &ladder, |b, &ladder| {
            b.iter(|| {
                let report = CfdFlow::operator_study::<f64>("bench-operator")
                    .operator(Operator::Viscous)
                    .resolutions(ladder.to_vec())
                    .run()
                    .expect("operator study runs");
                black_box(report);
            })
        });
    }
    group.finish();
}

criterion_group!(operator_study_benches, bench_operator_ladders);
criterion_main!(operator_study_benches);
