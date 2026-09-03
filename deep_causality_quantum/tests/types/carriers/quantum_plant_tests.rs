/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `QuantumPlant`: a sealed validated state that evolves by operation.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{Channel, QuantumErrorEnum, QuantumPlant, QubitOperator, Tolerance};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn ket(a: f64, b: f64) -> CausalTensor<C> {
    CausalTensor::from_slice(&[c(a, 0.), c(b, 0.)], &[2])
}

fn depolarizing(p: f64) -> Channel<f64> {
    let s0 = (1.0 - 3.0 * p / 4.0).sqrt();
    let s = (p / 4.0_f64).sqrt();
    let m = |d: [C; 4]| CausalTensor::from_slice(&d, &[2, 2]);
    Channel::from_kraus(&[
        m([c(s0, 0.), c(0., 0.), c(0., 0.), c(s0, 0.)]),
        m([c(0., 0.), c(s, 0.), c(s, 0.), c(0., 0.)]),
        m([c(0., 0.), c(0., -s), c(0., s), c(0., 0.)]),
        m([c(s, 0.), c(0., 0.), c(0., 0.), c(-s, 0.)]),
    ])
    .unwrap()
}

#[test]
fn test_evolution_yields_a_new_sealed_plant_and_leaves_the_receiver_alone() {
    let plant = QuantumPlant::from_ket(&ket(1.0, 0.0)).unwrap();
    let before = plant.clone();
    let evolved = plant.evolve(&depolarizing(0.4)).unwrap();
    assert_eq!(
        plant, before,
        "the receiver compares equal to its pre-evolution value"
    );
    assert_ne!(evolved, plant);
    // The evolved state passed the density-matrix checks: unit trace, no negative eigenvalue.
    let tol = Tolerance::<f64>::state().threshold(2, 1.0).unwrap();
    let m = evolved.matrix().as_slice();
    assert!((m[0].re + m[3].re - 1.0).abs() <= tol);
    assert!(m[0].re >= -tol && m[3].re >= -tol);
    // Depolarising |0⟩ at p = 0.4 leaves 1 − p/2 = 0.8 on |0⟩.
    assert!((m[0].re - 0.8).abs() < 1e-12);
}

#[test]
fn test_a_dimension_disagreement_is_caught_before_a_state_is_built() {
    let three = CausalTensor::from_slice(&[c(1., 0.), c(0., 0.), c(0., 0.)], &[3]);
    let plant = QuantumPlant::from_ket(&three).unwrap();
    assert_eq!(plant.dim(), 3);
    let err = plant.evolve(&depolarizing(0.1)).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_two_stages_reading_one_plant_see_one_state() {
    let plant = QuantumPlant::from_ket(&ket(0.6, 0.8)).unwrap();
    let before = plant.clone();
    let stage_a = |p: &QuantumPlant<f64>| p.state().purity();
    let stage_b = |p: &QuantumPlant<f64>| p.matrix().as_slice()[0].re;
    let a = stage_a(&plant);
    let b = stage_b(&plant);
    assert!((a - 1.0).abs() < 1e-12, "a pure state");
    assert!((b - 0.36).abs() < 1e-12);
    assert_eq!(plant, before);
}

#[test]
fn test_unitary_evolution_is_conjugation() {
    let plant = QuantumPlant::from_ket(&ket(1.0, 0.0)).unwrap();
    let flipped = plant
        .evolve(&Channel::unitary(&QubitOperator::pauli_x()).unwrap())
        .unwrap();
    assert!((flipped.matrix().as_slice()[3].re - 1.0).abs() < 1e-12);
    let back = flipped
        .evolve(&Channel::unitary(&QubitOperator::pauli_x()).unwrap())
        .unwrap();
    assert!((back.matrix().as_slice()[0].re - 1.0).abs() < 1e-12);
}

#[test]
fn test_a_malformed_ket_is_refused_with_a_typed_error() {
    let zero = ket(0.0, 0.0);
    assert!(matches!(
        QuantumPlant::from_ket(&zero).unwrap_err().0,
        QuantumErrorEnum::NormalizationError(_)
    ));
    let nan = CausalTensor::from_slice(&[c(f64::NAN, 0.), c(0., 0.)], &[2]);
    assert!(matches!(
        QuantumPlant::from_ket(&nan).unwrap_err().0,
        QuantumErrorEnum::NonFiniteValue(_)
    ));
}

#[test]
fn test_the_precision_lift_re_validates_at_the_target_scalar() {
    // Down, f64 → f32: the state is re-validated as a density matrix at f32's tolerance and the
    // entries agree to f32 precision. The up direction is the operator test's business, where the
    // refusal on rounded data is shown; a state's rounding can cancel in the trace by luck, so it
    // is not asserted either way here.
    let plant = QuantumPlant::from_ket(&ket(0.6, 0.8)).unwrap();
    let narrow: QuantumPlant<f32> = plant.lift(|x| x as f32).unwrap();
    assert_eq!(narrow.dim(), 2);
    assert!((narrow.state().purity() - 1.0).abs() <= f32::EPSILON.sqrt());
    for (a, b) in plant
        .matrix()
        .as_slice()
        .iter()
        .zip(narrow.matrix().as_slice())
    {
        assert!((a.re - b.re as f64).abs() < 1e-6);
    }
}

#[test]
fn test_the_default_is_the_ground_qubit() {
    let d = QuantumPlant::<f64>::default();
    assert_eq!(d.dim(), 2);
    assert!((d.matrix().as_slice()[0].re - 1.0).abs() < 1e-15);
}
