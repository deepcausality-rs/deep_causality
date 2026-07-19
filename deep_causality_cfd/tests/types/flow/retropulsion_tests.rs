/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The inert-safe A0 propulsion stub (`blackout-coupling-interface`): inertness at zero throttle,
//! every seam exercised at active throttle, the published `"commanded_throttle"` read, and the
//! propulsion scalars surviving a pause snapshot.

use deep_causality_cfd::{
    Ambient, CoupledField, PhysicsStage, PropulsionStub, StepContext, pack_resume, unpack_resume,
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
