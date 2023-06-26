// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use criterion::{Criterion, criterion_group};
use deep_causality::prelude::{ArrayStorage, SlidingWindow};
use deep_causality::prelude::sliding_window::new_with_array_storage;
use crate::benchmarks::fields::{MULT, SIZE};

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

fn get_sliding_window() -> SlidingWindow<ArrayStorage<Data, SIZE, MULT>, Data> {
    new_with_array_storage()
}

fn array_backed_benchmark(criterion: &mut Criterion)
{
    let mut w = get_sliding_window();
    criterion.bench_function("array_push", |bencher| {
        bencher.iter(||
            w.push( Data{ dats: 0 })
        )
    });
}


criterion_group! {
    name = window_array_backed;
    config = Criterion::default().sample_size(100);
    targets =
    array_backed_benchmark,
}