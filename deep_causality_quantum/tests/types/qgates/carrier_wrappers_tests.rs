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
    let plant = plant_from_ket(&ket(1., 0.)).value_cloned().expect("a plant");
    let evolved = plant_evolve(&plant, &x).value_cloned().expect("an evolved plant");
    let obs: Observable<f64, 2> = observable_from_ket("excited", &ket(0., 1.))
        .value_cloned()
        .expect("an observable");
    let p = observable_read_out(&obs, &evolved)
        .value_cloned()
        .expect("a read-out");
    assert!((p - 1.0).abs() < 1e-12);

    let rot = qubit_rotation(Axis::Z, 0.3).value_cloned().expect("a rotation");
    let ph = qubit_phase(0.3).value_cloned().expect("a phase");
    assert!(rot.unitarity_defect() < 1e-12 && ph.unitarity_defect() < 1e-12);

    let twice = channel_compose(&x, &x).value_cloned().expect("a composite");
    assert!(twice.kraus().is_none());
}

#[test]
fn test_a_failure_short_circuits_with_its_structured_cause() {
    // A non-CPTP family offered to the lifted constructor.
    let half = CausalTensor::from_slice(&[c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)], &[2, 2]);
    let effect = channel_from_kraus(&[half]);
    assert!(effect.value().is_none(), "no channel value exists");
    assert!(effect.is_err(), "the error channel carries the cause");
    let shown = format!("{:?}", effect);
    assert!(shown.contains("Non-CPTP"), "the typed cause survives: {shown}");

    // A non-finite angle.
    assert!(qubit_rotation::<f64>(Axis::X, f64::NAN).value().is_none());
    // A mismatched read-out.
    let obs: Observable<f64, 2> = observable_from_ket("z", &ket(1., 0.)).value_cloned().unwrap();
    let three = CausalTensor::from_slice(&[c(1., 0.), c(0., 0.), c(0., 0.)], &[3]);
    let plant = plant_from_ket(&three).value_cloned().unwrap();
    assert!(observable_read_out(&obs, &plant).value().is_none());
}
