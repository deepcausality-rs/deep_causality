// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use criterion::{Criterion, criterion_group};
use rand::Rng;

use deep_causality::prelude::{ArrayGrid, Grid, PointIndex};
use deep_causality::prelude::ArrayType::{Array1D, Array2D, Array3D, Array4D};

const WIDTH: usize = 10;
const HEIGHT: usize = 10;
const DEPTH: usize = 10;
const TIME: usize = 10;

fn array_grid_1d_benchmark(criterion: &mut Criterion)
{
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array1D);
    let g: &Grid<[usize; HEIGHT], usize> = ag.array_grid_1d()
        .expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_1d_benchmark", |bencher| {
        bencher.iter(||
            g.set(point, value)
        )
    });
}

fn array_grid_2d_benchmark(criterion: &mut Criterion)
{
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array2D);
    let g = ag.array_grid_2d()
        .expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_2d_benchmark", |bencher| {
        bencher.iter(||
            g.set(point, value)
        )
    });
}

fn array_grid_3d_benchmark(criterion: &mut Criterion)
{
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array3D);
    let g = ag.array_grid_3d()
        .expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_3d_benchmark", |bencher| {
        bencher.iter(||
            g.set(point, value)
        )
    });
}

fn array_grid_4d_benchmark(criterion: &mut Criterion)
{
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(Array4D);
    let g = ag.array_grid_4d()
        .expect("failed to create array grid");

    let value = get_value();
    let point = get_point_index();

    criterion.bench_function("set_array_grid_4d_benchmark", |bencher| {
        bencher.iter(||
            g.set(point, value)
        )
    });
}


fn get_point_index() -> PointIndex {
    PointIndex {
        x: rand::thread_rng().gen_range(0..WIDTH),
        y: rand::thread_rng().gen_range(0..WIDTH),
        z: rand::thread_rng().gen_range(0..WIDTH),
        t: rand::thread_rng().gen_range(0..WIDTH),
    }
}

fn get_value() -> usize {
    rand::thread_rng().gen_range(0..WIDTH)
}

criterion_group! {
    name = array_grid;
    config = Criterion::default().sample_size(100);
    targets =
    array_grid_1d_benchmark,
    array_grid_2d_benchmark,
    array_grid_3d_benchmark,
    array_grid_4d_benchmark,
}