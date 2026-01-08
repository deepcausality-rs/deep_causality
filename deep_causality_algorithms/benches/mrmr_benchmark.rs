/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group, criterion_main};
use deep_causality_algorithms::mrmr::mrmr_features_selector;
use deep_causality_tensor::CausalTensor;

fn generate_test_tensor(rows: usize, cols: usize) -> CausalTensor<f64> {
    let mut data = Vec::with_capacity(rows * cols);
    for i in 0..(rows * cols) {
        data.push(i as f64);
    }
    CausalTensor::new(data, vec![rows, cols]).unwrap()
}

fn generate_test_tensor_cdl(rows: usize, cols: usize) -> CausalTensor<Option<f64>> {
    let mut data = Vec::with_capacity(rows * cols);
    for i in 0..(rows * cols) {
        if (i * 3 + i / 5) % 13 < 2 {
            // A more complex pattern
            data.push(None);
        } else {
            data.push(Some(i as f64));
        }
    }
    CausalTensor::new(data, vec![rows, cols]).unwrap()
}

fn mrmr_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mRMR Feature Selector");

    let rows = 1000;
    let cols = 100;
    let num_features_to_select = 10;
    let target_col = cols - 1;

    // Benchmark the standard implementation
    group.bench_function("mrmr_features_selector", |b| {
        let tensor = generate_test_tensor(rows, cols);
        b.iter(|| {
            mrmr_features_selector(&tensor, num_features_to_select, target_col).unwrap();
        });
    });

    // Benchmark the cdl implementation
    group.bench_function("mrmr_features_selector_cdl", |b| {
        let tensor = generate_test_tensor_cdl(rows, cols);
        b.iter(|| {
            mrmr_features_selector(&tensor, num_features_to_select, target_col).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, mrmr_benchmark);
criterion_main!(benches);
