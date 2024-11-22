// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::{criterion_group, Criterion};

use dcl_data_structures::prelude::{window_type, ArrayStorage, SlidingWindow};

use crate::benchmarks::fields::{MULT, SIZE};

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

fn get_sliding_window() -> SlidingWindow<ArrayStorage<Data, SIZE, MULT>, Data> {
    window_type::new_with_array_storage()
}

fn array_backed_benchmark(criterion: &mut Criterion) {
    let mut w = get_sliding_window();
    criterion.bench_function("array_push", |bencher| {
        bencher.iter(|| w.push(Data { dats: 0 }))
    });
}

criterion_group! {
    name = window_array_backed;
    config = Criterion::default().sample_size(100);
    targets =
    array_backed_benchmark,
}
