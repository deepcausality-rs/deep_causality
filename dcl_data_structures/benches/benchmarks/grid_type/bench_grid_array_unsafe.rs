// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::{criterion_group, Criterion};
use dcl_data_structures::prelude::{
    Array1D, Array2D, Array3D, Array4D, ArrayGrid, Grid, PointIndex,
};
use rand::Rng;

const WIDTH: usize = 10;
const HEIGHT: usize = 10;
const DEPTH: usize = 10;
const TIME: usize = 10;

fn set_array_grid_1d_unsafe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array1D);
    let g: &Grid<[usize; HEIGHT], usize> = ag.array_grid_1d().expect("failed to create array grid");

    let value = get_value();
    let point = PointIndex::new1d(get_point_index().x);

    criterion.bench_function("set_array_grid_1d_unsafe", |b| {
        b.iter(|| g.set(point, value))
    });
}

fn set_array_grid_2d_unsafe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array2D);
    let g: &Grid<[[usize; WIDTH]; HEIGHT], usize> =
        ag.array_grid_2d().expect("failed to create array grid");

    let value = get_value();
    let point = PointIndex::new2d(get_point_index().x, get_point_index().y);

    criterion.bench_function("set_array_grid_2d_unsafe", |b| {
        b.iter(|| g.set(point, value))
    });
}

fn set_array_grid_3d_unsafe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array3D);
    let g: &Grid<[[[usize; WIDTH]; HEIGHT]; DEPTH], usize> =
        ag.array_grid_3d().expect("failed to create array grid");

    let value = get_value();
    let point = PointIndex::new3d(
        get_point_index().x,
        get_point_index().y,
        get_point_index().z,
    );

    criterion.bench_function("set_array_grid_3d_unsafe", |b| {
        b.iter(|| g.set(point, value))
    });
}

fn set_array_grid_4d_unsafe_benchmark(criterion: &mut Criterion) {
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array4D);
    let g: &Grid<[[[[usize; WIDTH]; HEIGHT]; DEPTH]; TIME], usize> =
        ag.array_grid_4d().expect("failed to create array grid");

    let value = get_value();
    let point = PointIndex::new4d(
        get_point_index().x,
        get_point_index().y,
        get_point_index().z,
        get_point_index().t,
    );

    criterion.bench_function("set_array_grid_4d_unsafe", |b| {
        b.iter(|| g.set(point, value))
    });
}

fn get_point_index() -> PointIndex {
    let mut rng = rand::rng();
    PointIndex::new4d(
        rng.random_range(0..WIDTH),
        rng.random_range(0..HEIGHT),
        rng.random_range(0..DEPTH),
        rng.random_range(0..TIME),
    )
}

fn get_value() -> usize {
    let mut rng = rand::rng();
    rng.random_range(0..100)
}

#[cfg(feature = "unsafe")]
criterion_group! {
    name = array_grid_unsafe;
    config = Criterion::default().sample_size(100);
    targets =
        set_array_grid_1d_unsafe_benchmark,
        set_array_grid_2d_unsafe_benchmark,
        set_array_grid_3d_unsafe_benchmark,
        set_array_grid_4d_unsafe_benchmark
}
