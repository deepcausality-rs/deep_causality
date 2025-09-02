/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
#[cfg(feature = "unsafe")]
use crate::benchmarks::fields::MULT;
#[cfg(feature = "unsafe")]
use crate::benchmarks::fields::SIZE;
#[cfg(feature = "unsafe")]
use criterion::Criterion;
#[cfg(feature = "unsafe")]
use criterion::criterion_group;
#[cfg(feature = "unsafe")]
use deep_causality_data_structures::{SlidingWindow, UnsafeVectorStorage, window_type};

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

#[cfg(feature = "unsafe")]
fn get_sliding_window() -> SlidingWindow<UnsafeVectorStorage<Data>, Data> {
    window_type::new_with_unsafe_vector_storage(SIZE, MULT)
}

#[cfg(feature = "unsafe")]
fn vector_backed_benchmark(criterion: &mut Criterion) {
    let mut w = get_sliding_window();
    criterion.bench_function("vector_push", |bencher| {
        bencher.iter(|| w.push(Data { dats: 0 }))
    });
}

#[cfg(feature = "unsafe")]
criterion_group! {
    name = window_unsafe_vector_backed;
    config = Criterion::default().sample_size(100);
    targets =
    vector_backed_benchmark,
}
