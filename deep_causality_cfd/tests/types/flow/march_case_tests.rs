/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! End-to-end tests of the Flow facade's marching solver, via the fluent
//! `Flow::march` builder: a periodic Taylor–Green vortex (energy decays) and a
//! lid-driven cavity from rest (the moving lid injects energy) — reproducing the
//! `dec_taylor_green_re1600` and `dec_lid_cavity_re1000` physics through the DSL.

use deep_causality_cfd::{Body, CfdConfigBuilder, Mesh, Observe, Seed};

#[test]
fn taylor_green_periodic_energy_decays() {
    let n = 8usize;
    let k = 2.0 * std::f64::consts::PI / n as f64;
    let nu = 1.0 / (k * 100.0);
    let dt = 0.2 * k;

    let report = super::run_march(CfdConfigBuilder::march::<3, f64>("tgv-flow")
        .mesh(Mesh::periodic_cube(n))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("valid config"),
        )
        .seed(Seed::TaylorGreenVortex)
        .march_for(10)
        .observe(Observe::default().kinetic_energy())
        .build().expect("config build"))
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

    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("cavity-re1000")
        .mesh(Mesh::box_domain([n, n]).spacing(h))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("valid config"),
        )
        .lid([1.0, 0.0])
        .seed(Seed::Rest)
        .march_for(20)
        .observe(Observe::default().kinetic_energy())
        .build().expect("config build"))
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

#[test]
fn cut_cell_channel_stays_divergence_free() {
    // A small periodic-x channel with an immersed disk, driven by a moving top
    // wall (the dec_cylinder_wake configuration, miniaturized). The solver should
    // hold the field divergence-free and the wall should drive a nonzero flow.
    let (nx, ny) = (24usize, 12usize);
    let center = [6.0_f64, 6.0];
    let radius = 2.0_f64;
    let nu = 0.02;
    let dt = 0.05;

    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("cyl-channel")
        .mesh(Mesh::channel([nx, ny]).immersed(Body::disk(center, radius).merge_floor(0.25)))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("valid config"),
        )
        .moving_wall(1, true, [1.0, 0.0])
        .seed(Seed::Rest)
        .march_for(5)
        .observe(Observe::default().divergence().max_speed())
        .build().expect("config build"))
        .expect("cut-cell channel runs");

    let div = report.series("divergence").expect("divergence series");
    let speed = report.series("max_speed").expect("max_speed series");
    assert_eq!(div.len(), 6, "seed + 5 steps");
    for d in div {
        assert!(
            *d < 1e-6,
            "divergence held near zero by the projector: {div:?}"
        );
    }
    assert!(
        *speed.last().unwrap() > 0.0,
        "the moving wall drives flow past the cylinder: {speed:?}"
    );
}

#[test]
fn march_until_steady_stops_a_rest_field_early() {
    // A rest field with no forcing stays at rest, so the step-to-step energy change
    // is zero from the first step — `march_until_steady` must stop early, well
    // before max_steps.
    let report = super::run_march(CfdConfigBuilder::march::<3, f64>("steady-rest")
        .mesh(Mesh::periodic_cube(6))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.05)
                .build()
                .expect("valid config"),
        )
        .seed(Seed::Rest)
        .march_until_steady(1e-9, 50)
        .observe(Observe::default().kinetic_energy())
        .build().expect("config build"))
        .expect("steady march runs");

    let energy = report.series("kinetic_energy").expect("energy series");
    // Seed + at most a couple of steps before the steady predicate trips.
    assert!(
        energy.len() < 5,
        "steady detection should stop the rest field early, got {} samples",
        energy.len()
    );
    for e in energy {
        assert!(
            e.abs() < 1e-20,
            "a rest field carries no energy: {energy:?}"
        );
    }
}

#[test]
fn open_zone_cylinder_inflow_drives_flow() {
    // The dec_cylinder_validation configuration, miniaturized: an all-wall box whose
    // boundaries are reconfigured by an Inflow/Outflow/SlipWall zone tuple, with an
    // immersed disk. The west inflow prescribes a uniform stream, so the flow is
    // nonzero from the seed onward.
    use deep_causality_cfd::{Inflow, Outflow, SlipWall};

    let (nx, ny) = (24usize, 12usize);
    let center = [6.0_f64, 6.0];
    let radius = 2.0_f64;

    let zones = (
        Inflow::<2, f64>::new(0, false, 1.0).expect("inflow west"),
        (
            Outflow::<2>::new(0, true).expect("outflow east"),
            (
                SlipWall::<2>::new(1, false).expect("slip bottom"),
                SlipWall::<2>::new(1, true).expect("slip top"),
            ),
        ),
    );

    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("cyl-validation")
        .mesh(Mesh::box_domain([nx, ny]).immersed(Body::disk(center, radius).merge_floor(0.25)))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.02)
                .time_step(0.05)
                .build()
                .expect("valid config"),
        )
        .zones(zones)
        .seed(Seed::Rest)
        .march_for(3)
        .observe(Observe::default().max_speed())
        .build().expect("config build"))
        .expect("open-zone cylinder runs");

    let speed = report.series("max_speed").expect("max_speed series");
    assert_eq!(speed.len(), 4, "seed + 3 steps");
    assert!(
        *speed.last().unwrap() > 0.0,
        "the west inflow drives a nonzero flow: {speed:?}"
    );
}

#[test]
fn drag_lift_on_the_cylinder_are_finite_and_streamwise() {
    // The open-zone cylinder, observed for drag/lift. The west inflow pushes the disk,
    // so the streamwise drag coefficient must be finite and positive (higher pressure
    // upstream), while the symmetric configuration keeps the lift small.
    use deep_causality_cfd::{Inflow, Outflow, SlipWall};

    let (nx, ny) = (24usize, 12usize);
    let center = [6.0_f64, 6.0];
    let radius = 2.0_f64;
    let u_ref = 1.0_f64;

    let zones = (
        Inflow::<2, f64>::new(0, false, u_ref).expect("inflow west"),
        (
            Outflow::<2>::new(0, true).expect("outflow east"),
            (
                SlipWall::<2>::new(1, false).expect("slip bottom"),
                SlipWall::<2>::new(1, true).expect("slip top"),
            ),
        ),
    );

    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("cyl-drag")
        .mesh(Mesh::box_domain([nx, ny]).immersed(Body::disk(center, radius).merge_floor(0.25)))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.02)
                .time_step(0.05)
                .build()
                .expect("valid config"),
        )
        .zones(zones)
        .seed(Seed::Rest)
        .march_for(8)
        .observe(Observe::default().drag(u_ref))
        .build().expect("config build"))
        .expect("drag-observed cylinder runs");

    let drag = report.series("drag").expect("drag series");
    let lift = report.series("lift").expect("lift series");
    assert_eq!(drag.len(), 9, "seed + 8 steps");
    assert_eq!(lift.len(), 9, "seed + 8 steps");
    assert!(
        drag.iter().chain(lift.iter()).all(|c| c.is_finite()),
        "drag/lift coefficients stay finite: drag={drag:?} lift={lift:?}"
    );
    assert!(
        *drag.last().unwrap() > 0.0,
        "the inflow exerts a positive streamwise drag on the disk: {drag:?}"
    );
    assert!(
        drag.last().unwrap().abs() > lift.last().unwrap().abs(),
        "a streamwise-symmetric stream gives drag ≫ lift: drag={drag:?} lift={lift:?}"
    );
}

#[test]
fn uniform_x_seed_carries_the_free_stream() {
    // A uniform streamwise seed on a periodic box is divergence-free and curl-free, so
    // the projection preserves it: the max speed at the seed equals the prescribed speed.
    let speed = 2.0_f64;
    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("uniform-x")
        .mesh(Mesh::periodic_cube(8))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.02)
                .build()
                .expect("valid config"),
        )
        .seed(Seed::UniformX { speed })
        .march_for(2)
        .observe(Observe::default().kinetic_energy().max_speed())
        .build().expect("config build"))
        .expect("uniform-x march runs");

    let energy = report.series("kinetic_energy").expect("energy series");
    let max_speed = report.series("max_speed").expect("max_speed series");
    assert!(
        energy[0] > 0.0,
        "the uniform stream carries energy: {energy:?}"
    );
    assert!(
        (max_speed[0] - speed).abs() < 0.1,
        "the seed carries the prescribed free-stream speed: {max_speed:?}"
    );
}

#[test]
fn wake_probe_records_a_finite_transverse_signal() {
    // The open-zone cylinder seeded with a uniform stream, with a wake probe one body
    // downstream on the centerline. The probe must record a finite transverse-velocity
    // sample per step (the raw signal a Strouhal number is read from).
    use deep_causality_cfd::{Inflow, Outflow, SlipWall};

    let (nx, ny) = (24usize, 12usize);
    let center = [6.0_f64, 6.0];
    let radius = 2.0_f64;
    let u_ref = 1.0_f64;

    let zones = (
        Inflow::<2, f64>::new(0, false, u_ref).expect("inflow west"),
        (
            Outflow::<2>::new(0, true).expect("outflow east"),
            (
                SlipWall::<2>::new(1, false).expect("slip bottom"),
                SlipWall::<2>::new(1, true).expect("slip top"),
            ),
        ),
    );

    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("cyl-probe")
        .mesh(Mesh::box_domain([nx, ny]).immersed(Body::disk(center, radius).merge_floor(0.25)))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.02)
                .time_step(0.05)
                .build()
                .expect("valid config"),
        )
        .zones(zones)
        .seed(Seed::UniformX { speed: u_ref })
        .march_for(6)
        // The probe sits one diameter downstream of the body, on the wake centerline.
        .observe(Observe::default().probe([10.0, 6.0]))
        .build().expect("config build"))
        .expect("probe cylinder runs");

    let probe = report.series("probe").expect("probe series");
    assert_eq!(probe.len(), 7, "seed + 6 steps");
    assert!(
        probe.iter().all(|v| v.is_finite()),
        "the wake probe records a finite transverse signal: {probe:?}"
    );
}

#[test]
fn centerline_profile_follows_the_lid() {
    // The lid-driven cavity: the vertical centerline velocity profile u_x(y) should rise
    // toward the moving lid at the top and vanish at the no-slip floor.
    let n = 17usize;
    let h = 1.0 / (n as f64 - 1.0);

    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("cavity-centerline")
        .mesh(Mesh::box_domain([n, n]).spacing(h))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(1.0 / 1000.0)
                .time_step(0.45 * h)
                .build()
                .expect("valid config"),
        )
        .lid([1.0, 0.0])
        .seed(Seed::Rest)
        .march_for(40)
        .observe(Observe::default().centerline(1))
        .build().expect("config build"))
        .expect("centerline cavity runs");

    let profile = report.series("centerline").expect("centerline series");
    assert_eq!(profile.len(), n, "one sample per lattice row along y");
    assert!(
        profile.iter().all(|v| v.is_finite()),
        "the centerline profile is finite: {profile:?}"
    );
    // The lid (top, last sample) drives a positive streamwise velocity; the floor rests.
    assert!(
        *profile.last().unwrap() > 0.1,
        "the lid drives a positive u_x at the top of the centerline: {profile:?}"
    );
    assert!(
        *profile.last().unwrap() > *profile.first().unwrap(),
        "u_x grows from the resting floor up to the moving lid: {profile:?}"
    );
}

#[test]
fn drag_without_an_immersed_body_is_rejected() {
    let report = super::run_march(CfdConfigBuilder::march::<2, f64>("no-body-drag")
        .mesh(Mesh::box_domain([8, 8]))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.05)
                .build()
                .expect("valid config"),
        )
        .seed(Seed::Rest)
        .march_for(1)
        .observe(Observe::default().drag(1.0))
        .build().expect("config build"));
    assert!(
        report.is_err(),
        "drag without an immersed body must be rejected"
    );
}
