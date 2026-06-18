/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Holistic benchmark for the **DEC Navier–Stokes marching** solver (`CfdFlow::march`).
//!
//! Two axes are swept so the report shows how cost scales with the work, not just one point:
//!
//! - **grid** — fixed step budget, square grids `16²…48²` (cost grows with cell count and the
//!   per-step constrained projection);
//! - **steps** — fixed `24²` grid, step budgets `10…40` (cost grows linearly with the march length).
//!
//! The caller-owned geometry is materialized once per case and lent to each timed run (design D2), so
//! the measurement is the march itself, not geometry construction. Sizes are modest to keep the suite
//! fast; raise the constants locally to profile at reporting resolutions. The `MarchConfig` type is
//! left unnamed (it carries the builder's zone/physics witnesses), so each case is built inline.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, Mesh, Observe, Seed};
use std::hint::black_box;

/// Kinematic viscosity for the marching solver.
const NU: f64 = 0.05;
/// Time step (CFL-safe across the benched grids).
const DT: f64 = 0.005;
/// Step budget for the grid sweep.
const GRID_STEPS: usize = 20;
/// Grid edge for the steps sweep.
const STEPS_GRID: usize = 24;

fn bench_march_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("dec_ns_march/grid");
    for &n in &[16usize, 24, 32, 48] {
        let config = CfdConfigBuilder::march::<2, f64>("bench-march")
            .mesh(Mesh::box_domain([n, n]))
            .solver(
                CfdConfigBuilder::dec_ns()
                    .viscosity(NU)
                    .time_step(DT)
                    .build()
                    .expect("solver config"),
            )
            .seed(Seed::Rest)
            .march_for(GRID_STEPS)
            .observe(Observe::default().kinetic_energy())
            .build()
            .expect("march config");
        let manifold = config.materialize().expect("materialize geometry");
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let report = CfdFlow::march(black_box(&config))
                    .on(black_box(&manifold))
                    .run()
                    .expect("march runs");
                black_box(report);
            })
        });
    }
    group.finish();
}

fn bench_march_steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("dec_ns_march/steps");
    for &steps in &[10usize, 20, 40] {
        let config = CfdConfigBuilder::march::<2, f64>("bench-march")
            .mesh(Mesh::box_domain([STEPS_GRID, STEPS_GRID]))
            .solver(
                CfdConfigBuilder::dec_ns()
                    .viscosity(NU)
                    .time_step(DT)
                    .build()
                    .expect("solver config"),
            )
            .seed(Seed::Rest)
            .march_for(steps)
            .observe(Observe::default().kinetic_energy())
            .build()
            .expect("march config");
        let manifold = config.materialize().expect("materialize geometry");
        group.bench_with_input(BenchmarkId::from_parameter(steps), &steps, |b, _| {
            b.iter(|| {
                let report = CfdFlow::march(black_box(&config))
                    .on(black_box(&manifold))
                    .run()
                    .expect("march runs");
                black_box(report);
            })
        });
    }
    group.finish();
}

criterion_group!(dec_ns_march_benches, bench_march_grid, bench_march_steps);
criterion_main!(dec_ns_march_benches);
