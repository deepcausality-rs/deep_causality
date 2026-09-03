/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Channel`: CPTP-checked once, applied without re-checking, composed without re-deriving.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, QuantumErrorEnum, QubitOperator, apply_kraus, check_completely_positive,
    check_trace_preserving, frobenius_norm,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>) -> CausalTensor<C> {
    CausalTensor::new(data, vec![2, 2]).unwrap()
}

fn depolarizing_kraus(p: f64) -> Vec<CausalTensor<C>> {
    let s0 = (1.0 - 3.0 * p / 4.0).sqrt();
    let s = (p / 4.0_f64).sqrt();
    vec![
        mat(vec![c(s0, 0.), c(0., 0.), c(0., 0.), c(s0, 0.)]),
        mat(vec![c(0., 0.), c(s, 0.), c(s, 0.), c(0., 0.)]),
        mat(vec![c(0., 0.), c(0., -s), c(0., s), c(0., 0.)]),
        mat(vec![c(s, 0.), c(0., 0.), c(0., 0.), c(-s, 0.)]),
    ]
}

fn ground() -> CausalTensor<C> {
    mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 0.)])
}

fn max_abs_diff(a: &CausalTensor<C>, b: &CausalTensor<C>) -> f64 {
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| ((x.re - y.re).powi(2) + (x.im - y.im).powi(2)).sqrt())
        .fold(0.0, f64::max)
}

#[test]
fn test_a_cptp_family_constructs_and_holds_its_choi_and_kraus() {
    let ch = Channel::from_kraus(&depolarizing_kraus(0.3)).unwrap();
    assert_eq!((ch.d_in(), ch.d_out()), (2, 2));
    assert_eq!(ch.kraus().unwrap().len(), 4);
    assert_eq!(ch.choi().shape(), &[4, 4]);
    // The Choi passes the shipped checks at the tolerance the carrier used.
    check_completely_positive(ch.choi(), 1e-9).unwrap();
    check_trace_preserving(ch.choi(), 2, 2, 1e-9).unwrap();
}

#[test]
fn test_a_non_trace_preserving_family_fails_at_construction() {
    // 0.5·I is CP but not TP: Tr_out(J) = 0.25·I.
    let k = mat(vec![c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)]);
    let err = Channel::from_kraus(&[k]).unwrap_err();
    match err.0 {
        QuantumErrorEnum::NonCptpChannel(msg) => {
            assert!(msg.contains("trace-preserving"), "{msg}");
            assert!(msg.contains("0.75"), "carries the measured defect: {msg}");
        }
        other => panic!("expected NonCptpChannel, got {other:?}"),
    }
}

#[test]
fn test_an_empty_or_malformed_family_is_refused() {
    assert!(matches!(
        Channel::<f64>::from_kraus(&[]).unwrap_err().0,
        QuantumErrorEnum::NonCptpChannel(_)
    ));
    let bad = CausalTensor::new(vec![c(1., 0.); 3], vec![3]).unwrap();
    assert!(matches!(
        Channel::from_kraus(&[bad]).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_unitary_builds_the_family_itself_and_acts_as_conjugation() {
    let x = QubitOperator::<f64>::pauli_x();
    let ch = Channel::unitary(&x).unwrap();
    assert_eq!(ch.kraus().unwrap().len(), 1);
    // X|0⟩⟨0|X = |1⟩⟨1|.
    let out = ch.apply(&ground()).unwrap();
    let excited = mat(vec![c(0., 0.), c(0., 0.), c(0., 0.), c(1., 0.)]);
    assert!(max_abs_diff(&out, &excited) < 1e-12);
}

#[test]
fn test_application_routes_to_the_shipped_kraus_apply_and_does_not_revalidate() {
    // A thousand applications of one constructed channel, each equal to `apply_kraus` directly,
    // with no construction in the loop and therefore no check in it.
    let kraus = depolarizing_kraus(0.2);
    let ch = Channel::from_kraus(&kraus).unwrap();
    let mut state = ground();
    for _ in 0..1000 {
        let via_channel = ch.apply(&state).unwrap();
        let via_kraus = apply_kraus(&kraus, &state).unwrap();
        assert!(max_abs_diff(&via_channel, &via_kraus) < 1e-15);
        state = via_channel;
    }
    // Repeated depolarising contracts toward the maximally mixed state.
    assert!((state.as_slice()[0].re - 0.5).abs() < 1e-6);
}

#[test]
fn test_composition_inherits_cptp_and_applies_through_the_choi() {
    let a = Channel::from_kraus(&depolarizing_kraus(0.3)).unwrap();
    let b = Channel::from_kraus(&depolarizing_kraus(0.5)).unwrap();
    let ab = a.compose(&b).unwrap();
    // No family: the composite applies through its Choi.
    assert!(ab.kraus().is_none());
    assert_eq!((ab.d_in(), ab.d_out()), (2, 2));
    // And agrees with applying the two in sequence.
    let sequential = b.apply(&a.apply(&ground()).unwrap()).unwrap();
    let composed = ab.apply(&ground()).unwrap();
    assert!(max_abs_diff(&sequential, &composed) < 1e-12);
    // The composite is still CPTP, which is inherited rather than re-derived.
    check_completely_positive(ab.choi(), 1e-9).unwrap();
    check_trace_preserving(ab.choi(), 2, 2, 1e-9).unwrap();
}

#[test]
fn test_a_dimension_disagreement_is_caught_at_apply_and_at_compose() {
    let ch = Channel::from_kraus(&depolarizing_kraus(0.1)).unwrap();
    let three = CausalTensor::new(vec![c(1., 0.); 9], vec![3, 3]).unwrap();
    assert!(matches!(
        ch.apply(&three).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // A qutrit-input channel cannot follow a qubit-output one.
    let iso: Vec<CausalTensor<C>> = {
        // A 2×3 isometry-like Kraus family is not needed; a 3×3 identity family suffices for
        // the dimension check to fire before any arithmetic.
        let mut id = vec![c(0., 0.); 9];
        for i in 0..3 {
            id[i * 3 + i] = c(1., 0.);
        }
        vec![CausalTensor::new(id, vec![3, 3]).unwrap()]
    };
    let qutrit = Channel::from_kraus(&iso).unwrap();
    assert!(matches!(
        ch.compose(&qutrit).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_a_changed_channel_is_a_second_construction() {
    let first = Channel::from_kraus(&depolarizing_kraus(0.1)).unwrap();
    let before = first.clone();
    let second = Channel::from_kraus(&depolarizing_kraus(0.9)).unwrap();
    assert_eq!(first, before, "the first channel is unaffected");
    assert!(frobenius_norm(first.choi()) != frobenius_norm(second.choi()));
}

#[test]
fn test_the_default_is_the_identity_channel() {
    let id = Channel::<f64>::default();
    let out = id.apply(&ground()).unwrap();
    assert!(max_abs_diff(&out, &ground()) < 1e-15);
}
