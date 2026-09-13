/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The carrier lifts on the causal monad: a stage reads a value, and a failure short-circuits
//! with its structured cause.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, Observable, QubitOperator, channel_compose, channel_from_kraus, channel_unitary,
    observable_from_ket, observable_read_out, plant_evolve, plant_from_ket, qubit_phase,
    qubit_rotation,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn ket(a: f64, b: f64) -> CausalTensor<C> {
    CausalTensor::from_slice(&[c(a, 0.), c(b, 0.)], &[2])
}

#[test]
fn test_a_stage_reads_a_value_rather_than_a_result() {
    // No match arm, no turbofish, no closure at the call sites below.
    let x = channel_unitary(&QubitOperator::<f64>::pauli_x())
        .value_cloned()
        .expect("a unitary channel");
    let plant = plant_from_ket(&ket(1., 0.))
        .value_cloned()
        .expect("a plant");
    let evolved = plant_evolve(&plant, &x)
        .value_cloned()
        .expect("an evolved plant");
    let obs: Observable<f64, 2> = observable_from_ket("excited", &ket(0., 1.))
        .value_cloned()
        .expect("an observable");
    let p = observable_read_out(&obs, &evolved)
        .value_cloned()
        .expect("a read-out");
    assert!((p - 1.0).abs() < 1e-12);

    let rot = qubit_rotation(Axis::Z, 0.3)
        .value_cloned()
        .expect("a rotation");
    let ph = qubit_phase(0.3).value_cloned().expect("a phase");
    assert!(rot.unitarity_defect() < 1e-12 && ph.unitarity_defect() < 1e-12);

    let twice = channel_compose(&x, &x).value_cloned().expect("a composite");
    assert!(twice.kraus().is_none());
}

#[test]
fn test_the_phase_lift_carries_the_documented_matrix_including_its_global_phase() {
    // `QubitOperator::phase` documents `P(θ) = diag(1, e^{iθ})`. The Z rotation
    // `diag(e^{-iθ/2}, e^{iθ/2})` is the same operator up to a global phase, so it induces the
    // SAME channel and no read-out through `channel_unitary` can separate them — measured, a
    // `qubit_phase` that lifted `rotation(Axis::Z, θ)` passes every channel-mediated test here.
    //
    // The global phase is not spare, though: the crate keeps it for the Haruna Hadamard exactly
    // because a controlled use of the gate observes it. So the matrix is pinned directly, at an
    // angle where the two forms differ in both entries.
    let theta = core::f64::consts::FRAC_PI_2;
    let op = qubit_phase(theta).value_cloned().expect("a phase");
    let m = op.matrix();

    // diag(1, e^{iπ/2}) = diag(1, i).
    assert!(
        (m.as_slice()[0].re - 1.0).abs() < 1e-12,
        "{:?}",
        m.as_slice()[0]
    );
    assert!(m.as_slice()[0].im.abs() < 1e-12, "{:?}", m.as_slice()[0]);
    assert!(m.as_slice()[1].re.abs() < 1e-12 && m.as_slice()[1].im.abs() < 1e-12);
    assert!(m.as_slice()[2].re.abs() < 1e-12 && m.as_slice()[2].im.abs() < 1e-12);
    assert!(m.as_slice()[3].re.abs() < 1e-12, "{:?}", m.as_slice()[3]);
    assert!(
        (m.as_slice()[3].im - 1.0).abs() < 1e-12,
        "{:?}",
        m.as_slice()[3]
    );

    // The Z rotation at the same angle is diag(e^{-iπ/4}, e^{iπ/4}), whose first entry is not 1.
    let rot = qubit_rotation(Axis::Z, theta)
        .value_cloned()
        .expect("a rotation");
    assert!(
        (rot.matrix().as_slice()[0].re - 1.0).abs() > 1e-6,
        "the two lifts must not produce the same matrix"
    );
}

#[test]
fn test_the_rotation_lift_is_not_the_phase_lift() {
    // Unitarity is satisfied by both, so `unitarity_defect` cannot tell them apart: measured,
    // a `qubit_rotation` that lifted `QubitOperator::phase` passes the check above. Their
    // ACTION separates them. A π rotation about X is the bit flip up to phase, so it moves
    // |0> to the excited state; the π phase is diag(1, -1), which leaves |0> alone.
    let pi = core::f64::consts::PI;
    let excited: Observable<f64, 2> = observable_from_ket("excited", &ket(0., 1.))
        .value_cloned()
        .expect("an observable");
    let ground = plant_from_ket(&ket(1., 0.))
        .value_cloned()
        .expect("a plant");

    let flipped = plant_evolve(
        &ground,
        &channel_unitary(
            &qubit_rotation(Axis::X, pi)
                .value_cloned()
                .expect("a rotation"),
        )
        .value_cloned()
        .expect("a channel"),
    )
    .value_cloned()
    .expect("an evolved plant");
    let p = observable_read_out(&excited, &flipped)
        .value_cloned()
        .expect("a read-out");
    assert!(
        (p - 1.0).abs() < 1e-12,
        "an X(π) rotation flips the qubit, got {p}"
    );

    let phased = plant_evolve(
        &ground,
        &channel_unitary(&qubit_phase(pi).value_cloned().expect("a phase"))
            .value_cloned()
            .expect("a channel"),
    )
    .value_cloned()
    .expect("an evolved plant");
    let p = observable_read_out(&excited, &phased)
        .value_cloned()
        .expect("a read-out");
    assert!(
        (p - 0.0).abs() < 1e-12,
        "a π phase leaves |0> alone, got {p}"
    );
}

#[test]
fn test_composition_is_ordered() {
    // The existing composite is X with X — identical operands, so no order can matter and a
    // wrapper that swapped them passes. Two different rotations separate them, but only in the
    // right observable: measured, the excited and ground read-outs are both exactly 1/2 for
    // both orders, because a pi/2 rotation from |0> lands on the equator either way. The |+>
    // axis is where the orders differ, at 1.0 against 0.5.
    let half_pi = core::f64::consts::FRAC_PI_2;
    let rx = channel_unitary(&qubit_rotation(Axis::X, half_pi).value_cloned().unwrap())
        .value_cloned()
        .expect("a channel");
    let rz = channel_unitary(&qubit_rotation(Axis::Z, half_pi).value_cloned().unwrap())
        .value_cloned()
        .expect("a channel");

    let sqrt_half = core::f64::consts::FRAC_1_SQRT_2;
    let plus: Observable<f64, 2> = observable_from_ket("plus", &ket(sqrt_half, sqrt_half))
        .value_cloned()
        .expect("an observable");
    let ground = plant_from_ket(&ket(1., 0.))
        .value_cloned()
        .expect("a plant");

    let read = |ch: &deep_causality_quantum::Channel<f64>| {
        let evolved = plant_evolve(&ground, ch)
            .value_cloned()
            .expect("an evolved plant");
        observable_read_out(&plus, &evolved)
            .value_cloned()
            .expect("a read-out")
    };

    let forward = read(
        &channel_compose(&rx, &rz)
            .value_cloned()
            .expect("a composite"),
    );
    let reverse = read(
        &channel_compose(&rz, &rx)
            .value_cloned()
            .expect("a composite"),
    );

    assert!(
        (forward - 1.0).abs() < 1e-12,
        "X then Z reaches |+>, got {forward}"
    );
    assert!(
        (reverse - 0.5).abs() < 1e-12,
        "Z then X does not, got {reverse}"
    );
}

#[test]
fn test_a_failure_short_circuits_with_its_structured_cause() {
    // A non-CPTP family offered to the lifted constructor.
    let half = CausalTensor::from_slice(&[c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)], &[2, 2]);
    let effect = channel_from_kraus(&[half]);
    assert!(effect.value().is_none(), "no channel value exists");
    assert!(effect.is_err(), "the error channel carries the cause");
    let shown = format!("{:?}", effect);
    assert!(
        shown.contains("Non-CPTP"),
        "the typed cause survives: {shown}"
    );

    // A non-finite angle.
    assert!(qubit_rotation::<f64>(Axis::X, f64::NAN).value().is_none());
    // A mismatched read-out.
    let obs: Observable<f64, 2> = observable_from_ket("z", &ket(1., 0.))
        .value_cloned()
        .unwrap();
    let three = CausalTensor::from_slice(&[c(1., 0.), c(0., 0.), c(0., 0.)], &[3]);
    let plant = plant_from_ket(&three).value_cloned().unwrap();
    assert!(observable_read_out(&obs, &plant).value().is_none());
}

#[test]
fn test_qubit_phase_lifts_a_value_and_short_circuits_on_a_non_finite_angle() {
    // Zero, a negative angle and a positive one all lift to a value: the wrapper
    // does not quietly restrict the domain of the kernel it adapts.
    for theta in [0.0f64, -core::f64::consts::FRAC_PI_2, core::f64::consts::PI] {
        assert!(
            qubit_phase(theta).value().is_some(),
            "phase {theta} should lift"
        );
    }
    // Only a non-representable angle reaches the error channel.
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let effect = qubit_phase(bad);
        assert!(effect.value().is_none(), "phase {bad} must not lift");
        assert!(effect.is_err());
    }
}

#[test]
fn test_from_matrix_refuses_a_non_unitary_before_it_can_reach_channel_unitary() {
    // `QubitOperator` validates at construction, so `channel_unitary` has no
    // non-unitary input to refuse: the guard sits one layer down. Pinning it here
    // records where the refusal actually happens.
    let bad = QubitOperator::<f64>::from_matrix(CausalTensor::from_slice(
        &[c(2., 0.), c(0., 0.), c(0., 0.), c(1., 0.)],
        &[2, 2],
    ));
    assert!(bad.is_err(), "a non-unitary matrix is no qubit operator");

    // Every shipped operator is unitary and lifts to a channel.
    for op in [
        QubitOperator::<f64>::identity(),
        QubitOperator::<f64>::pauli_x(),
        QubitOperator::<f64>::pauli_y(),
        QubitOperator::<f64>::pauli_z(),
        QubitOperator::<f64>::hadamard(),
    ] {
        assert!(channel_unitary(&op).value().is_some());
    }
}

#[test]
fn test_channel_compose_short_circuits_when_the_dimensions_disagree() {
    // A qubit channel and a qutrit channel do not compose.
    let qubit = channel_unitary(&QubitOperator::<f64>::pauli_x())
        .value_cloned()
        .expect("a qubit channel");
    let id3 = CausalTensor::from_slice(
        &[
            c(1., 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(1., 0.),
            c(0., 0.),
            c(0., 0.),
            c(0., 0.),
            c(1., 0.),
        ],
        &[3, 3],
    );
    let qutrit = channel_from_kraus(&[id3])
        .value_cloned()
        .expect("a qutrit channel");
    let effect = channel_compose(&qubit, &qutrit);
    assert!(effect.value().is_none(), "2 and 3 do not compose");
    assert!(effect.is_err());

    // The same channel composed with itself does, so the refusal above is the
    // dimension check and not a wrapper that never succeeds.
    assert!(channel_compose(&qubit, &qubit).value().is_some());
}

#[test]
fn test_plant_evolve_short_circuits_when_the_channel_does_not_fit_the_plant() {
    let three = CausalTensor::from_slice(&[c(1., 0.), c(0., 0.), c(0., 0.)], &[3]);
    let qutrit_plant = plant_from_ket(&three).value_cloned().expect("a plant");
    let qubit_channel = channel_unitary(&QubitOperator::<f64>::pauli_x())
        .value_cloned()
        .expect("a channel");
    let effect = plant_evolve(&qutrit_plant, &qubit_channel);
    assert!(effect.value().is_none());
    assert!(effect.is_err());
}

#[test]
fn test_plant_from_ket_short_circuits_on_a_zero_ket() {
    // The zero vector has no ray to normalize.
    let effect = plant_from_ket(&ket(0., 0.));
    assert!(effect.value().is_none());
    assert!(effect.is_err());
}

#[test]
fn test_observable_from_ket_short_circuits_when_the_ket_does_not_match_the_dimension() {
    // `D` is 2 but the ket has three amplitudes.
    let three = CausalTensor::from_slice(&[c(1., 0.), c(0., 0.), c(0., 0.)], &[3]);
    let effect: deep_causality_core::PropagatingEffect<Observable<f64, 2>> =
        observable_from_ket("mismatched", &three);
    assert!(effect.value().is_none());
    assert!(effect.is_err());
}
