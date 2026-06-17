/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `Seed` initial conditions, driven through the public march API (the `apply`
//! lowering is crate-private). Each variant seeds a different field; the seed sample is the first
//! entry of the kinetic-energy series.

use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, Mesh, Observe, Seed};

fn seed_energy_2d(seed: Seed) -> f64 {
    let config = CfdConfigBuilder::march::<2, f64>("seed")
        .mesh(Mesh::periodic_cube(6))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.005)
                .build()
                .unwrap(),
        )
        .seed(seed)
        .march_for(0)
        .observe(Observe::default().kinetic_energy())
        .build()
        .unwrap();
    let manifold = config.materialize().unwrap();
    let report = CfdFlow::march(&config).on(&manifold).run().unwrap();
    report.series("kinetic_energy").expect("energy")[0]
}

#[test]
fn rest_seed_has_zero_kinetic_energy() {
    assert_eq!(seed_energy_2d(Seed::Rest), 0.0);
}

#[test]
fn uniform_x_seed_carries_streamwise_momentum() {
    // A uniform free-stream projects onto a nonzero divergence-free field on the periodic torus.
    let energy = seed_energy_2d(Seed::UniformX { speed: 1.5 });
    assert!(
        energy > 0.0,
        "a uniform-x seed has kinetic energy, got {energy}"
    );
}

#[test]
fn taylor_green_vortex_seed_is_nonzero_in_3d() {
    let config = CfdConfigBuilder::march::<3, f64>("tgv-seed")
        .mesh(Mesh::periodic_cube(6))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.005)
                .build()
                .unwrap(),
        )
        .seed(Seed::TaylorGreenVortex)
        .march_for(0)
        .observe(Observe::default().kinetic_energy())
        .build()
        .unwrap();
    let manifold = config.materialize().unwrap();
    let report = CfdFlow::march(&config).on(&manifold).run().unwrap();
    assert!(report.series("kinetic_energy").expect("energy")[0] > 0.0);
}

#[test]
fn seed_is_debug_clone_copy() {
    let seed = Seed::UniformX { speed: 2.0 };
    let copied = seed;
    let _cloned = seed;
    assert!(format!("{copied:?}").contains("UniformX"));
    assert!(format!("{:?}", Seed::Rest).contains("Rest"));
    assert!(format!("{:?}", Seed::TaylorGreenVortex).contains("TaylorGreenVortex"));
}
