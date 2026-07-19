/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The powered-descent flight-sensor producers (`flight-sensor-scalars`): dynamic pressure from the
//! carrier's freestream, descent rate positive downward from the truth state, the partial-state
//! refusal, and the configurable field names that keep producer and gate in step.

use deep_causality_cfd::{Ambient, CoupledField, FlightSensors, PhysicsStage, StepContext};
use deep_causality_physics::{SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const M_BAR: f64 = 4.81e-26; // air mean molecular mass, kg
const N_INF: f64 = 1.0e22; // freestream number density, m^-3
const SPEED: f64 = 2_000.0; // flight speed, m/s
const R_EARTH: f64 = 6_371_000.0;
const DT: f64 = 0.1;

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

/// A field carrying the carrier's published freestream and a truth state descending along +x
/// (radially outward position, inward velocity).
fn sensed_field(radial_velocity: f64) -> CoupledField<f64> {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("freestream_n", vec![N_INF]);
    field.set_scalar("flight_speed", vec![SPEED]);
    field.set_scalar(
        "truth_state",
        vec![R_EARTH + 40_000.0, 0.0, 0.0, radial_velocity, 0.0, 0.0],
    );
    field
}

#[test]
fn dynamic_pressure_is_half_rho_v_squared() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = sensed_field(-100.0);

    PhysicsStage::<2, f64>::apply(&FlightSensors::new(M_BAR), &ctx, &mut field).unwrap();

    let expected = 0.5 * (N_INF * M_BAR) * SPEED * SPEED;
    let q = field.scalar("q_inf").unwrap()[0];
    assert!(
        (q - expected).abs() < 1e-9 * expected.abs(),
        "q_inf {q} vs expected {expected}"
    );
}

#[test]
fn descending_vehicle_reports_a_positive_rate() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    // Velocity radially inward (negative x with position on +x) is a descent.
    let mut field = sensed_field(-120.0);

    PhysicsStage::<2, f64>::apply(&FlightSensors::new(M_BAR), &ctx, &mut field).unwrap();

    let rate = field.scalar("descent_rate").unwrap()[0];
    assert!(
        rate > 0.0,
        "a descending vehicle must report a positive rate, got {rate}"
    );
    assert!((rate - 120.0).abs() < 1e-9, "rate {rate} vs expected 120");
}

#[test]
fn ascending_vehicle_reports_a_negative_rate() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = sensed_field(80.0);

    PhysicsStage::<2, f64>::apply(&FlightSensors::new(M_BAR), &ctx, &mut field).unwrap();

    let rate = field.scalar("descent_rate").unwrap()[0];
    assert!(
        rate < 0.0,
        "an ascending vehicle reports negative, got {rate}"
    );
}

#[test]
fn the_rate_grows_with_the_inward_velocity_component() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let sensors = FlightSensors::new(M_BAR);

    let mut slow = sensed_field(-50.0);
    let mut fast = sensed_field(-150.0);
    PhysicsStage::<2, f64>::apply(&sensors, &ctx, &mut slow).unwrap();
    PhysicsStage::<2, f64>::apply(&sensors, &ctx, &mut fast).unwrap();

    assert!(fast.scalar("descent_rate").unwrap()[0] > slow.scalar("descent_rate").unwrap()[0]);
}

#[test]
fn a_partial_truth_state_publishes_no_rate() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("freestream_n", vec![N_INF]);
    field.set_scalar("flight_speed", vec![SPEED]);
    // Position only — the velocity half of the truth state is missing.
    field.set_scalar("truth_state", vec![R_EARTH, 0.0, 0.0]);

    PhysicsStage::<2, f64>::apply(&FlightSensors::new(M_BAR), &ctx, &mut field).unwrap();

    assert!(
        field.scalar("descent_rate").is_none(),
        "a partial truth state must publish nothing rather than a half-derived rate"
    );
    // The independent q∞ path still publishes.
    assert!(field.scalar("q_inf").is_some());
}

#[test]
fn an_absent_freestream_publishes_no_dynamic_pressure() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("truth_state", vec![R_EARTH, 0.0, 0.0, -10.0, 0.0, 0.0]);

    PhysicsStage::<2, f64>::apply(&FlightSensors::new(M_BAR), &ctx, &mut field).unwrap();

    assert!(field.scalar("q_inf").is_none());
    assert!(field.scalar("descent_rate").is_some());
}

#[test]
fn the_published_names_follow_the_configuration() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = sensed_field(-60.0);

    let sensors = FlightSensors::new(M_BAR).with_field_names("q_dyn", "sink_rate");
    PhysicsStage::<2, f64>::apply(&sensors, &ctx, &mut field).unwrap();

    assert!(field.scalar("q_dyn").is_some());
    assert!(field.scalar("sink_rate").is_some());
    assert!(field.scalar("q_inf").is_none());
    assert!(field.scalar("descent_rate").is_none());
}

#[test]
fn the_input_names_follow_the_configuration() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("n_free", vec![N_INF]);
    field.set_scalar("v_flight", vec![SPEED]);
    field.set_scalar("truth", vec![R_EARTH, 0.0, 0.0, -30.0, 0.0, 0.0]);

    let sensors = FlightSensors::new(M_BAR).with_input_names("n_free", "v_flight", "truth");
    PhysicsStage::<2, f64>::apply(&sensors, &ctx, &mut field).unwrap();

    assert!(field.scalar("q_inf").is_some());
    assert!((field.scalar("descent_rate").unwrap()[0] - 30.0).abs() < 1e-9);
}
