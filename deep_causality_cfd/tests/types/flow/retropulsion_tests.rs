/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The inert-safe A0 propulsion stub (`blackout-coupling-interface`): inertness at zero throttle,
//! every seam exercised at active throttle, the published `"commanded_throttle"` read, and the
//! propulsion scalars surviving a pause snapshot.

use deep_causality_cfd::{
    Ambient, CoupledField, PRESERVED_DRAG_FRACTION_FIELD, PhysicsStage, PlumeNozzle,
    PlumeObstruction, PropulsionStub, RetroThrust, StepContext, pack_resume, unpack_resume,
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
    // Retro thrust is aimed against the carried flight velocity, so the fixture states one. Purely
    // +x here, which is what makes the axial assertions below meaningful.
    field.set_scalar("truth_state", vec![6.4e6, 0.0, 0.0, 500.0, 0.0, 0.0]);
    // The plume stage normalizes its thrust coefficient against the **sensed** freestream, so the
    // fixture states one. Q_INF is the value the stage used to take at construction.
    field.set_scalar("q_inf", vec![Q_INF]);
    field.set_scalar("flight_mach", vec![2.0]);
    field.set_scalar("p_inf", vec![1_000.0]);
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
        gamma_inf: 1.4,
    }
}

#[test]
fn plume_applies_the_a0_decrement_from_the_correlation() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 0);
    let stage = PlumeObstruction::new(THRUST, S_REF);

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
    let stage = PlumeObstruction::new(THRUST, S_REF).with_plume_geometry(nozzle());

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
    PlumeObstruction::new(THRUST, S_REF)
        .apply(&ctx, &mut plain_field)
        .expect("applies");

    let mut geom_field = powered_field();
    geom_field.set_throttle_action(throttle);
    PlumeObstruction::new(THRUST, S_REF)
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

    PlumeObstruction::new(THRUST, S_REF)
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
    let stage = PlumeObstruction::new(THRUST, S_REF);

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

#[test]
fn retro_thrust_opposes_the_flight_velocity_not_a_fixed_axis() {
    // The regression: a corridor-class trajectory is mostly tangential with a radial descent, so a
    // hardcoded axis points along the flight path almost nowhere — and on the radial axis it
    // accelerates the descent instead of arresting it.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = powered_field();
    // Descending on −x at 1300 m/s while flying tangentially on +y at 7860 m/s: the corridor's own
    // initial truth velocity.
    field.set_scalar("truth_state", vec![6.4e6, 0.0, 0.0, -1_300.0, 7_860.0, 0.0]);
    field.set_throttle_action(0.5);
    let before = field.aero_force().unwrap();

    PhysicsStage::<2, f64>::apply(&RetroThrust::new(THRUST, ISP), &ctx, &mut field).unwrap();

    let after = field.aero_force().unwrap();
    let delta = [
        after[0] - before[0],
        after[1] - before[1],
        after[2] - before[2],
    ];
    // The thrust must oppose the velocity: its dot product with v is negative, and it is very
    // nearly anti-parallel to v.
    let v = [-1_300.0_f64, 7_860.0, 0.0];
    let dot = delta[0] * v[0] + delta[1] * v[1] + delta[2] * v[2];
    assert!(
        dot < 0.0,
        "retro thrust must oppose the velocity, got {delta:?}"
    );
    let mag_d = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
    let mag_v = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let cos = dot / (mag_d * mag_v);
    assert!(
        (cos + 1.0).abs() < 1e-9,
        "thrust is anti-parallel to the velocity (cos {cos})"
    );
    // And specifically: the tangential component dominates, which a fixed −x axis would have missed
    // entirely while pushing the vehicle down.
    assert!(
        delta[1].abs() > delta[0].abs(),
        "the tangential term dominates for this trajectory: {delta:?}"
    );
    assert!(delta[1] < 0.0, "tangential thrust opposes +y motion");
    assert!(
        delta[0] > 0.0,
        "radial thrust arrests the descent rather than adding to it"
    );
}

#[test]
fn an_active_burn_without_a_truth_state_cannot_be_aimed() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = powered_field();
    field.take_scalar("truth_state");
    field.set_throttle_action(0.5);

    let err = PhysicsStage::<2, f64>::apply(&RetroThrust::new(THRUST, ISP), &ctx, &mut field)
        .unwrap_err();
    assert!(format!("{err:?}").contains("resolve the flight direction"));
}

// ── SRP closure validity (change `fix-retropulsion-measurement-integrity`) ───────────────────

#[test]
fn the_closure_normalizes_against_the_sensed_dynamic_pressure() {
    // The dynamic pressure used to be taken at construction, which let this closure and the safety
    // gate's dynamic C_T cap normalize the same coefficient against two different pressures in one
    // step. The stage senses it now, so changing the sensed value changes the applied fraction.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let stage = PlumeObstruction::new(THRUST, S_REF);

    let read = |q: f64| {
        let mut field = powered_field();
        field.set_scalar("commanded_throttle", vec![0.5]);
        field.set_scalar("q_inf", vec![q]);
        PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
        field
            .scalar(PRESERVED_DRAG_FRACTION_FIELD)
            .and_then(|s| s.first().copied())
            .unwrap()
    };

    // A higher q at the same thrust is a lower C_T, hence a larger preserved fraction.
    let low_q = read(Q_INF);
    let high_q = read(Q_INF * 4.0);
    assert!(
        high_q > low_q,
        "the sensed pressure must reach the correlation: {low_q} vs {high_q}"
    );
}

#[test]
fn an_absent_dynamic_pressure_fails_the_step() {
    // A fallback is how a second normalization survives unnoticed, so an absent sensor is an error.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = powered_field();
    field.set_scalar("commanded_throttle", vec![0.5]);
    let _ = field.take_scalar("q_inf");

    let err =
        PhysicsStage::<2, f64>::apply(&PlumeObstruction::new(THRUST, S_REF), &ctx, &mut field)
            .unwrap_err();
    assert!(format!("{err:?}").contains("q_inf"), "{err:?}");
}

#[test]
fn the_closure_stands_down_outside_its_mach_band() {
    // The Jarvinen-Adams mechanism is bow-shock displacement, and there is no bow shock to displace
    // below the dataset's Mach floor. Carrying the correlation down deletes most of a subsonic
    // vehicle's drag on the strength of a supersonic interaction.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let stage = PlumeObstruction::new(THRUST, S_REF).with_mach_band(0.4, 2.0);

    let mut field = powered_field();
    field.set_scalar("commanded_throttle", vec![0.5]);
    field.set_scalar("flight_mach", vec![0.01]); // deep subsonic, far below the floor
    let before = field.aero_force().unwrap();

    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();

    assert_eq!(
        field.aero_force().unwrap(),
        before,
        "no decrement outside the band"
    );
    assert!(
        field.scalar(PRESERVED_DRAG_FRACTION_FIELD).is_none(),
        "a stand-down must not leave a fraction that reads as a live measurement"
    );
}

#[test]
fn re_entering_the_band_resumes_the_closure() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let stage = PlumeObstruction::new(THRUST, S_REF).with_mach_band(0.4, 2.0);
    let mut field = powered_field();
    field.set_scalar("commanded_throttle", vec![0.5]);

    // Inside: a fraction is published.
    field.set_scalar("flight_mach", vec![1.5]);
    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
    assert!(field.scalar(PRESERVED_DRAG_FRACTION_FIELD).is_some());

    // Outside: it is cleared, and the crossing is recorded once.
    field.set_scalar("flight_mach", vec![0.1]);
    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
    assert!(field.scalar(PRESERVED_DRAG_FRACTION_FIELD).is_none());
    assert!(
        field
            .log()
            .messages()
            .any(|m| m.contains("SRP drag closure stood down"))
    );

    // Back inside: it resumes.
    field.set_scalar("flight_mach", vec![1.5]);
    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
    assert!(field.scalar(PRESERVED_DRAG_FRACTION_FIELD).is_some());
}

#[test]
fn an_unbounded_stage_applies_at_every_mach() {
    // The band is opt-in, so a world that never configures one behaves as before.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = powered_field();
    field.set_scalar("commanded_throttle", vec![0.5]);
    field.set_scalar("flight_mach", vec![0.01]);

    PhysicsStage::<2, f64>::apply(&PlumeObstruction::new(THRUST, S_REF), &ctx, &mut field).unwrap();
    assert!(field.scalar(PRESERVED_DRAG_FRACTION_FIELD).is_some());
}

#[test]
fn the_plume_geometry_tracks_the_sensed_freestream() {
    // Frozen freestream constants make the kernel's own validity envelope test the constant rather
    // than the flight, so a leg that leaves the envelope still receives geometry.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let stage = PlumeObstruction::new(THRUST, S_REF).with_plume_geometry(nozzle());

    let radius_at = |p_inf: f64| {
        let mut field = powered_field();
        field.set_scalar("commanded_throttle", vec![0.5]);
        field.set_scalar("p_inf", vec![p_inf]);
        PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
        field
            .scalar("plume_max_radius")
            .and_then(|s| s.first().copied())
            .unwrap()
    };

    // A jet expanding against thinner ambient air spreads further. Both pressures stay above the
    // model's own blunt-flow transition, so this measures the geometry rather than the refusal.
    assert!(
        radius_at(400.0) > radius_at(1_200.0),
        "the geometry must follow the ambient pressure"
    );
}

#[test]
fn the_geometry_kernel_rejects_a_freestream_outside_its_envelope() {
    // The Cordell envelope is a documented Mach range. Fed the flown Mach, the kernel's refusal
    // reaches the caller instead of being masked by a constant that sits inside the envelope.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let stage = PlumeObstruction::new(THRUST, S_REF).with_plume_geometry(nozzle());
    let mut field = powered_field();
    field.set_scalar("commanded_throttle", vec![0.5]);
    field.set_scalar("flight_mach", vec![9.0]); // far above the envelope

    let err = PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap_err();
    let text = format!("{err:?}");
    assert!(
        text.to_lowercase().contains("mach"),
        "the kernel's envelope refusal must reach the caller: {text}"
    );
}
