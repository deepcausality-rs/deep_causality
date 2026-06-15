/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! End-to-end tests of the Flow facade's marching solver, via the fluent
//! `Flow::march` builder: a periodic Taylor–Green vortex (energy decays) and a
//! lid-driven cavity from rest (the moving lid injects energy) — reproducing the
//! `dec_taylor_green_re1600` and `dec_lid_cavity_re1000` physics through the DSL.

use deep_causality_cfd::{DecNs, Flow, Mesh, Observe, Seed};

#[test]
fn taylor_green_periodic_energy_decays() {
    let n = 8usize;
    let k = 2.0 * std::f64::consts::PI / n as f64;
    let nu = 1.0 / (k * 100.0);
    let dt = 0.2 * k;

    let report = Flow::march::<3, f64>("tgv-flow")
        .mesh(Mesh::periodic_cube(n))
        .solver(
            DecNs::config()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("valid config"),
        )
        .seed(Seed::TaylorGreenVortex)
        .march_for(10)
        .observe(Observe::default().kinetic_energy())
        .run()
        .expect("march runs");

    let energy = report
        .series("kinetic_energy")
        .expect("kinetic_energy series");
    assert_eq!(energy.len(), 11, "seed + 10 steps");
    assert!(energy[0] > 0.0, "seed carries energy: {energy:?}");
    assert!(
        energy.last().unwrap() < &energy[0],
        "viscous dissipation decays the energy: {energy:?}"
    );
    for w in energy.windows(2) {
        assert!(w[1] <= w[0] + 1e-12, "non-increasing energy: {energy:?}");
    }
}

#[test]
fn lid_cavity_from_rest_gains_energy() {
    let n = 17usize;
    let h = 1.0 / (n as f64 - 1.0);
    let nu = 1.0 / 1000.0; // lid speed 1, Re = 1000
    let dt = 0.45 * h;

    let report = Flow::march::<2, f64>("cavity-re1000")
        .mesh(Mesh::box_domain([n, n]).spacing(h))
        .solver(
            DecNs::config()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("valid config"),
        )
        .lid([1.0, 0.0])
        .seed(Seed::Rest)
        .march_for(20)
        .observe(Observe::default().kinetic_energy())
        .run()
        .expect("cavity march runs");

    let energy = report
        .series("kinetic_energy")
        .expect("kinetic_energy series");
    assert_eq!(energy.len(), 21, "rest + 20 steps");
    // The seeded state already carries the prescribed lid velocity on the lid edges.
    assert!(
        energy[0] > 0.0,
        "the configured lid carries boundary velocity at t=0: {}",
        energy[0]
    );
    // As the cavity flow develops from the lid, total kinetic energy grows.
    assert!(
        *energy.last().unwrap() > energy[0],
        "the moving lid drives the interior flow, growing the energy: {energy:?}"
    );
}
