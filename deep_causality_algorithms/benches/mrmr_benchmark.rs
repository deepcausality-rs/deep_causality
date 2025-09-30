/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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

fn mrmr_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mRMR Feature Selector");

    let rows = 100;
    let cols = 20;
    let num_features_to_select = 5;
    let target_col = cols - 1;

    // Benchmark the new implementation
    group.bench_function("mrmr_features_selector_new_impl", |b| {
        let tensor = generate_test_tensor(rows, cols);
        b.iter(|| {
            // Clone the tensor for each iteration to ensure a fresh state
            let mut cloned_tensor = tensor.clone();
            mrmr_features_selector(&mut cloned_tensor, num_features_to_select, target_col).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, mrmr_benchmark);
criterion_main!(benches);
