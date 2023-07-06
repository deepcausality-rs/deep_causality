// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use criterion::{Criterion, criterion_group};
use dcl_data_structures::prelude::{SlidingWindow, VectorStorage};
use dcl_data_structures::prelude::sliding_window::new_with_vector_storage;
use crate::benchmarks::fields::{MULT, SIZE};

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}
fn get_sliding_window() -> SlidingWindow<VectorStorage<Data>, Data> {
    new_with_vector_storage(SIZE, MULT)
}

fn vector_backed_benchmark(criterion: &mut Criterion)
{
    let mut w = get_sliding_window();
    criterion.bench_function("vector_push", |bencher| {
        bencher.iter(||
            w.push(Data{dats:0})
        )
    });
}

criterion_group! {
    name = window_vector_backed;
    config = Criterion::default().sample_size(100);
    targets =
    vector_backed_benchmark,
}