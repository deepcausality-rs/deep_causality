// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

#[cfg(feature = "unsafe")]
use criterion::criterion_group;
#[cfg(feature = "unsafe")]
use criterion::Criterion;
#[cfg(feature = "unsafe")]
use dcl_data_structures::prelude::{window_type, SlidingWindow, UnsafeArrayStorage};

#[cfg(feature = "unsafe")]
use crate::benchmarks::fields::SIZE;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

#[cfg(feature = "unsafe")]
fn get_sliding_window() -> SlidingWindow<UnsafeArrayStorage<Data, SIZE, 2>, Data> {
    window_type::new_with_unsafe_array_storage()
}

#[cfg(feature = "unsafe")]
fn array_backed_benchmark(criterion: &mut Criterion) {
    let mut w = get_sliding_window();
    criterion.bench_function("unsafe_array_push", |bencher| {
        bencher.iter(|| w.push(Data { dats: 0 }))
    });
}

#[cfg(feature = "unsafe")]
criterion_group! {
    name = window_unsafe_array_backed;
    config = Criterion::default().sample_size(100);
    targets =
    array_backed_benchmark,
}
