/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The inert-safe A0 propulsion stub (`blackout-coupling-interface`): inertness at zero throttle,
//! every seam exercised at active throttle, the published `"commanded_throttle"` read, and the
//! propulsion scalars surviving a pause snapshot.

use deep_causality_cfd::{
    Ambient, CoupledField, PhysicsStage, PlumeNozzle, PlumeObstruction, PropulsionStub,
    RetroThrust, StepContext, pack_resume, unpack_resume,
};
use deep_causality_physics::{
    Area, Force, Pressure, SolenoidalField, VelocityOneForm, propellant_mass_flow_kernel,
    srp_preserved_drag_fraction_kernel, srp_thrust_coefficient_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

// Stub parameters shared across the active-path tests.
const THRUST: f64 = 2_000.0; // full-throttle thrust, N
const ISP: f64 = 250.0; // specific impulse, s
const Q_INF: f64 = 2_800.0; // freestream dynamic pressure, Pa
const S_REF: f64 = 0.785; // reference area, m²
const DT: f64 = 0.1;

/// A tiny periodic manifold + zero fluid state, so the stub runs through a real `StepContext`
/// (the stub reads only `dt` and the field, but the context is genuine).
fn empty_context() -> (Manifold<LatticeComplex<2, f64>, f64>, SolenoidalField<f64>) {
    let n = 4;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    let n1 = manifold.complex().num_cells(1);
    let zero = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let velocity = VelocityOneForm::from_raw(zero);
    let (state, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();
    (manifold, state)
}

/// A corridor-class field carrying mass/propellant and a lift force already on the channel.
fn powered_field() -> CoupledField<f64> {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("mass", vec![1_000.0]);
    field.set_scalar("propellant", vec![200.0]);
    field.set_aero_force([-5.0, 1.0, 0.0]); // axial drag −5, lateral lift +1
    field
}

#[test]
fn inert_at_absent_throttle_touches_nothing() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stub = PropulsionStub::new(THRUST, ISP, Q_INF, S_REF);

    let before = powered_field();
    let mut field = powered_field();
    stub.apply(&ctx, &mut field).expect("inert apply");

    // No force write, no scalar mutation, no ignition, no log entry.
    assert_eq!(field.aero_force(), before.aero_force());
    assert_eq!(field.scalar("mass"), before.scalar("mass"));
    assert_eq!(field.scalar("propellant"), before.scalar("propellant"));
    assert_eq!(field.scalar("ignited"), None);
    assert_eq!(field.log().messages().count(), 0);
}

#[test]
fn inert_at_zero_and_negative_throttle() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stub = PropulsionStub::new(THRUST, ISP, Q_INF, S_REF);

    for throttle in [0.0_f64, -0.3] {
        let mut field = powered_field();
        field.set_throttle_action(throttle);
        stub.apply(&ctx, &mut field).expect("inert apply");
        assert_eq!(field.aero_force(), Some([-5.0, 1.0, 0.0]));
        assert_eq!(field.scalar("ignited"), None);
        assert_eq!(field.scalar("mass"), Some([1_000.0].as_slice()));
    }
}

#[test]
fn active_throttle_exercises_every_seam() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stub = PropulsionStub::new(THRUST, ISP, Q_INF, S_REF);

    let throttle = 0.5;
    let mut field = powered_field();
    field.set_throttle_action(throttle);
    stub.apply(&ctx, &mut field).expect("active apply");

    // Expected values recomputed from the same kernels the stub uses.
    let thrust = throttle * THRUST;
    let mdot = propellant_mass_flow_kernel(Force::new(thrust).unwrap(), ISP)
        .unwrap()
        .value();
    let dm = mdot * DT;
    let c_t = srp_thrust_coefficient_kernel(
        Force::new(thrust).unwrap(),
        Pressure::new(Q_INF).unwrap(),
        Area::new(S_REF).unwrap(),
    )
    .unwrap();
    let fraction = srp_preserved_drag_fraction_kernel(c_t).unwrap();
    let a_thrust = thrust / 1_000.0; // pre-depletion mass
    let expected_axial = -5.0 + (fraction - 1.0) * (-5.0) - a_thrust;

    let approx = |a: f64, b: f64| (a - b).abs() < 1e-9;
    // Propellant and mass both fell by ṁ·dt.
    assert!(approx(field.scalar("propellant").unwrap()[0], 200.0 - dm));
    assert!(approx(field.scalar("mass").unwrap()[0], 1_000.0 - dm));
    // Ignition flag set.
    assert_eq!(field.scalar("ignited"), Some([1.0].as_slice()));
    // Force channel holds lift (y) plus the A0-scaled drag and thrust (x).
    let f = field.aero_force().expect("force written");
    assert!(
        approx(f[0], expected_axial),
        "axial {} vs {expected_axial}",
        f[0]
    );
    assert!(approx(f[1], 1.0), "lateral lift preserved");
    assert!(approx(f[2], 0.0));
    // The decrement is a genuine drag reduction: powered axial drag is smaller in magnitude than
    // the −5 unpowered drag once thrust is removed (fraction < 1 in this C_T band).
    assert!(fraction < 1.0);
}

#[test]
fn reads_the_published_commanded_throttle() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stub = PropulsionStub::new(THRUST, ISP, Q_INF, S_REF);

    // No throttle channel write; the world publishes the throttle as a constant scalar.
    let mut field = powered_field();
    field.set_scalar("commanded_throttle", vec![0.5]);
    stub.apply(&ctx, &mut field)
        .expect("active via published throttle");
    assert_eq!(field.scalar("ignited"), Some([1.0].as_slice()));
}

#[test]
fn the_throttle_channel_takes_precedence_over_the_published_constant() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stub = PropulsionStub::new(THRUST, ISP, Q_INF, S_REF);

    // A guidance stage's zero-throttle write must win over a nonzero published constant, so the
    // stub stays inert (the channel is the authority, the published constant the default).
    let mut field = powered_field();
    field.set_scalar("commanded_throttle", vec![0.8]);
    field.set_throttle_action(0.0);
    stub.apply(&ctx, &mut field)
        .expect("inert via channel override");
    assert_eq!(field.scalar("ignited"), None);
    assert_eq!(field.aero_force(), Some([-5.0, 1.0, 0.0]));
}

#[test]
fn active_throttle_without_carried_mass_is_an_error() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stub = PropulsionStub::new(THRUST, ISP, Q_INF, S_REF);

    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_throttle_action(0.5); // active, but no "mass" scalar rides the field
    assert!(stub.apply(&ctx, &mut field).is_err());
}

// ── RetroThrust: the production thrust stage (retro-thrust-stage) ──

#[test]
fn retro_thrust_composes_onto_the_lift_vector() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = RetroThrust::new(THRUST, ISP);

    let throttle = 0.5;
    let mut field = powered_field(); // lift [-5, 1, 0], mass 1000, propellant 200
    field.set_throttle_action(throttle);
    stage.apply(&ctx, &mut field).expect("active apply");

    // a_thrust = T/m = (0.5·2000)/1000 = 1.0, along −x; lateral lift untouched.
    let f = field.aero_force().expect("force written");
    let approx = |a: f64, b: f64| (a - b).abs() < 1e-9;
    assert!(
        approx(f[0], -6.0),
        "axial = lift −5 plus thrust −1, got {}",
        f[0]
    );
    assert!(approx(f[1], 1.0), "lateral lift preserved");
    assert!(approx(f[2], 0.0));
}

#[test]
fn retro_thrust_depletes_propellant_and_sets_ignition() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = RetroThrust::new(THRUST, ISP);

    let throttle = 0.5;
    let mut field = powered_field();
    field.set_throttle_action(throttle);
    stage.apply(&ctx, &mut field).expect("active apply");

    let thrust = throttle * THRUST;
    let mdot = propellant_mass_flow_kernel(Force::new(thrust).unwrap(), ISP)
        .unwrap()
        .value();
    let dm = mdot * DT;
    let approx = |a: f64, b: f64| (a - b).abs() < 1e-9;
    assert!(approx(field.scalar("propellant").unwrap()[0], 200.0 - dm));
    assert!(approx(field.scalar("mass").unwrap()[0], 1_000.0 - dm));
    assert_eq!(field.scalar("ignited"), Some([1.0].as_slice()));
}

#[test]
fn retro_thrust_is_strictly_inert_without_an_active_throttle() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = RetroThrust::new(THRUST, ISP);

    for throttle in [None, Some(0.0), Some(-0.4)] {
        let before = powered_field();
        let mut field = powered_field();
        if let Some(t) = throttle {
            field.set_throttle_action(t);
        }
        stage.apply(&ctx, &mut field).expect("inert apply");
        assert_eq!(field.aero_force(), before.aero_force());
        assert_eq!(field.scalar("mass"), before.scalar("mass"));
        assert_eq!(field.scalar("propellant"), before.scalar("propellant"));
        assert_eq!(field.scalar("ignited"), None);
        assert_eq!(field.log().messages().count(), 0);
    }
}

#[test]
fn retro_thrust_without_carried_mass_is_an_error() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = RetroThrust::new(THRUST, ISP);

    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_throttle_action(0.5);
    assert!(stage.apply(&ctx, &mut field).is_err());
}

#[test]
fn the_summed_force_a_navigation_stage_reads_includes_thrust() {
    // Composition order: the lift stage writes the ④ vector, RetroThrust adds onto it, and every
    // downstream force consumer (loads, truth, nav) reads the one summed vector — no navigation
    // change is needed for the IMU to feel the burn.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let mut field = powered_field();
    field.set_throttle_action(0.5);
    let lift_before = field.aero_force().expect("lift on the channel");

    RetroThrust::new(THRUST, ISP)
        .apply(&ctx, &mut field)
        .expect("thrust applies");

    let summed = field.aero_force().expect("summed force");
    assert!(
        summed[0] < lift_before[0],
        "thrust made the axial force more negative"
    );
    assert_eq!(summed[1], lift_before[1], "lift is not clobbered");
}

// ── PlumeObstruction: the production plume stage (plume-obstruction-stage) ──

/// A nozzle inside the Cordell validity envelope (M∞ = 2, γ_jet = 1.3, and a chamber pressure high
/// enough that p_exit/p∞ clears the model's ≥ 7 floor at the swept throttles).
fn nozzle() -> PlumeNozzle<f64> {
    PlumeNozzle {
        chamber_pressure_max: 2.0e6,
        chamber_temperature: 1_500.0,
        r_specific: 300.0,
        gamma_jet: 1.3,
        exit_mach: 3.0,
        nozzle_half_angle_rad: 15.0 * std::f64::consts::PI / 180.0,
        throat_diameter: 0.03,
        exit_radius: 0.03407,
        cone_length: 0.0712,
        p_inf: 1_000.0,
        mach_inf: 2.0,
        gamma_inf: 1.4,
    }
}

#[test]
fn plume_applies_the_a0_decrement_from_the_correlation() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = PlumeObstruction::new(THRUST, Q_INF, S_REF);

    let throttle = 0.5;
    let mut field = powered_field(); // lift [-5, 1, 0]
    field.set_throttle_action(throttle);
    stage.apply(&ctx, &mut field).expect("active apply");

    let thrust = throttle * THRUST;
    let c_t = srp_thrust_coefficient_kernel(
        Force::new(thrust).unwrap(),
        Pressure::new(Q_INF).unwrap(),
        Area::new(S_REF).unwrap(),
    )
    .unwrap();
    let fraction = srp_preserved_drag_fraction_kernel(c_t).unwrap();

    let approx = |a: f64, b: f64| (a - b).abs() < 1e-9;
    let f = field.aero_force().expect("force");
    // The axial drag is scaled by the preserved fraction; lateral lift untouched.
    assert!(
        approx(f[0], fraction * -5.0),
        "axial scaled by the A0 fraction"
    );
    assert!(approx(f[1], 1.0), "lateral lift preserved");
    // The applied fraction is published for the M1 band cross-check.
    assert!(approx(
        field.scalar("preserved_drag_fraction").unwrap()[0],
        fraction
    ));
}

#[test]
fn plume_is_strictly_inert_without_an_active_throttle() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = PlumeObstruction::new(THRUST, Q_INF, S_REF).with_plume_geometry(nozzle());

    let before = powered_field();
    let mut field = powered_field();
    stage.apply(&ctx, &mut field).expect("inert apply");
    assert_eq!(field.aero_force(), before.aero_force());
    assert_eq!(field.scalar("preserved_drag_fraction"), None);
    assert_eq!(field.scalar("plume_max_radius"), None);
    assert_eq!(field.log().messages().count(), 0);
}

#[test]
fn the_published_geometry_does_not_change_the_force_channel_decrement() {
    // The AMBER contract: the imprint (and the geometry that drives it) is state realism only —
    // the correlation is the drag authority with or without it.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let throttle = 0.5;

    let mut plain_field = powered_field();
    plain_field.set_throttle_action(throttle);
    PlumeObstruction::new(THRUST, Q_INF, S_REF)
        .apply(&ctx, &mut plain_field)
        .expect("applies");

    let mut geom_field = powered_field();
    geom_field.set_throttle_action(throttle);
    PlumeObstruction::new(THRUST, Q_INF, S_REF)
        .with_plume_geometry(nozzle())
        .apply(&ctx, &mut geom_field)
        .expect("applies");

    assert_eq!(
        plain_field.aero_force(),
        geom_field.aero_force(),
        "the decrement is identical with and without the published geometry"
    );
}

#[test]
fn the_stage_publishes_plume_geometry_when_opted_in() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let mut field = powered_field();
    field.set_throttle_action(0.5);

    PlumeObstruction::new(THRUST, Q_INF, S_REF)
        .with_plume_geometry(nozzle())
        .apply(&ctx, &mut field)
        .expect("geometry published");

    let r_max = field.scalar("plume_max_radius").expect("max radius")[0];
    let pen = field.scalar("plume_penetration").expect("penetration")[0];
    assert!(r_max > 0.0, "a real plume has positive radius");
    assert!(pen > 0.0, "a real plume has positive penetration");
}

#[test]
fn the_applied_fraction_matches_the_correlation_across_a_sweep() {
    // The M1 band cross-check: the flight-time authority is the same cited curve M1 gated against.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = PlumeObstruction::new(THRUST, Q_INF, S_REF);

    for throttle in [0.25_f64, 0.5, 1.0] {
        let mut field = powered_field();
        field.set_throttle_action(throttle);
        stage.apply(&ctx, &mut field).expect("applies");

        let c_t = srp_thrust_coefficient_kernel(
            Force::new(throttle * THRUST).unwrap(),
            Pressure::new(Q_INF).unwrap(),
            Area::new(S_REF).unwrap(),
        )
        .unwrap();
        let expected = srp_preserved_drag_fraction_kernel(c_t).unwrap();
        let applied = field.scalar("preserved_drag_fraction").unwrap()[0];
        assert!(
            (applied - expected).abs() < 1e-12,
            "throttle {throttle}: applied {applied} vs correlation {expected}"
        );
    }
}

#[test]
fn propulsion_scalars_survive_a_pause_snapshot() {
    const WORLD: &[u8] = b"retropulsion-snapshot-world-v1";
    let mut field = CoupledField::<f64>::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("mass", vec![950.0]);
    field.set_scalar("propellant", vec![180.0]);
    field.set_scalar("ignited", vec![1.0]);
    field.set_throttle_action(0.4);

    let package = pack_resume(&field, 7, WORLD).expect("packs");
    let (restored, step) = unpack_resume::<f64>(&package).expect("unpacks");

    assert_eq!(step, 7);
    assert_eq!(restored.scalar("mass"), Some([950.0].as_slice()));
    assert_eq!(restored.scalar("propellant"), Some([180.0].as_slice()));
    assert_eq!(restored.scalar("ignited"), Some([1.0].as_slice()));
    assert_eq!(restored.throttle_action(), Some(0.4));
}
