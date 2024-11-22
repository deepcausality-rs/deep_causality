// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::{criterion_group, Criterion};
use rand::Rng;

use dcl_data_structures::prelude::ArrayType::{Array1D, Array2D, Array3D, Array4D};
use dcl_data_structures::prelude::{ArrayGrid, Grid, PointIndex};

const WIDTH: usize = 10;
const HEIGHT: usize = 10;
const DEPTH: usize = 10;
const TIME: usize = 10;

fn set_array_grid_1d_safe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array1D);
    let g: &Grid<[usize; HEIGHT], usize> = ag.array_grid_1d().expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_1d_safe", |b| b.iter(|| g.set(point, value)));
}

fn set_array_grid_2d_safe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array2D);
    let g: &Grid<[[usize; WIDTH]; HEIGHT], usize> =
        ag.array_grid_2d().expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_2d_safe", |b| b.iter(|| g.set(point, value)));
}

fn set_array_grid_3d_safe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array3D);
    let g: &Grid<[[[usize; WIDTH]; HEIGHT]; DEPTH], usize> =
        ag.array_grid_3d().expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_3d_safe", |b| b.iter(|| g.set(point, value)));
}

fn set_array_grid_4d_safe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array4D);
    let g: &Grid<[[[[usize; WIDTH]; HEIGHT]; DEPTH]; TIME], usize> =
        ag.array_grid_4d().expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_4d_safe", |b| b.iter(|| g.set(point, value)));
}

fn get_point_index() -> PointIndex {
    let mut rng = rand::thread_rng();
    PointIndex {
        x: rng.gen_range(0..WIDTH),
        y: rng.gen_range(0..HEIGHT),
        z: rng.gen_range(0..DEPTH),
        t: rng.gen_range(0..TIME),
    }
}

fn get_value() -> usize {
    rand::thread_rng().gen()
}

criterion_group! {
    name = array_grid;
    config = Criterion::default().sample_size(100);
    targets =
        set_array_grid_1d_safe_benchmark,
        set_array_grid_2d_safe_benchmark,
        set_array_grid_3d_safe_benchmark,
        set_array_grid_4d_safe_benchmark
}
