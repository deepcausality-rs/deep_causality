/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Observable`: a named projector carrying its own read-out, the one site a verdict enters.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Observable, Projection, QuantumErrorEnum, QuantumPlant, ShotHistogram,
    born_projective_probability,
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
fn test_a_ket_becomes_a_named_read_out_in_one_step() {
    let excited: Observable<f64, 2> =
        Observable::from_ket("excited_population", &ket(0., 1.)).unwrap();
    assert_eq!(excited.name(), "excited_population");
    assert_eq!(excited.dim(), 2);
    assert_eq!(excited.projection().rank(), 1);
    let plant = QuantumPlant::from_ket(&ket(0.6, 0.8)).unwrap();
    let p = excited.read_out(&plant).unwrap();
    assert!((0.0..=1.0).contains(&p));
    assert!((p - 0.64).abs() < 1e-12, "|⟨1|ψ⟩|² = 0.64");
    // The read-out is the shipped Born call, not a restatement of it.
    assert_eq!(
        p,
        born_projective_probability(plant.state(), excited.projection()).unwrap()
    );
    // And the Prob verdict is the same number at the boundary.
    assert!((excited.read_out_prob(&plant).unwrap().0 - 0.64).abs() < 1e-12);
}

#[test]
fn test_a_dimension_mismatch_is_reported_by_the_shipped_boundary() {
    let obs: Observable<f64, 2> = Observable::from_ket("z", &ket(1., 0.)).unwrap();
    let three = CausalTensor::from_slice(&[c(1., 0.), c(0., 0.), c(0., 0.)], &[3]);
    let plant = QuantumPlant::from_ket(&three).unwrap();
    match obs.read_out(&plant).unwrap_err().0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(
                msg.contains('3') && msg.contains('2'),
                "names both dimensions: {msg}"
            );
        }
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
}

#[test]
fn test_non_commuting_projectors_stay_visible_to_the_fold() {
    // |0⟩⟨0| and |+⟩⟨+| are in general position; |0⟩⟨0| and |1⟩⟨1| commute.
    let z0: Observable<f64, 2> = Observable::from_ket("z0", &ket(1., 0.)).unwrap();
    let z1: Observable<f64, 2> = Observable::from_ket("z1", &ket(0., 1.)).unwrap();
    let r = std::f64::consts::FRAC_1_SQRT_2;
    let plus: Observable<f64, 2> = Observable::from_ket("plus", &ket(r, r)).unwrap();
    assert!(!z0.projection().commutes_with(plus.projection()));
    assert!(z0.projection().commutes_with(z1.projection()));
    // Each carries its own read-out; the observable folds nothing.
    let plant = QuantumPlant::from_ket(&ket(1., 0.)).unwrap();
    assert!((z0.read_out(&plant).unwrap() - 1.0).abs() < 1e-12);
    assert!((plus.read_out(&plant).unwrap() - 0.5).abs() < 1e-12);
}

#[test]
fn test_a_malformed_ket_is_refused() {
    assert!(matches!(
        Observable::<f64, 2>::from_ket("zero", &ket(0., 0.))
            .unwrap_err()
            .0,
        QuantumErrorEnum::NormalizationError(_)
    ));
    let three = CausalTensor::from_slice(&[c(1., 0.), c(0., 0.), c(0., 0.)], &[3]);
    assert!(matches!(
        Observable::<f64, 2>::from_ket("wrong_d", &three)
            .unwrap_err()
            .0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_an_observable_over_a_given_projection_keeps_it() {
    let one = Projection::<f64, 2>::one();
    let obs = Observable::new("always", one.clone());
    assert!(obs.projection().leq(&one) && one.leq(obs.projection()));
    let plant = QuantumPlant::<f64>::default();
    assert!((obs.read_out(&plant).unwrap() - 1.0).abs() < 1e-12);
}

#[test]
fn test_sampling_the_read_out_converges_on_the_born_value() {
    let excited: Observable<f64, 2> = Observable::from_ket("excited", &ket(0., 1.)).unwrap();
    let plant = QuantumPlant::from_ket(&ket(0.6, 0.8)).unwrap();
    let hist = excited.sample(&plant, 4096, 20260821).unwrap();
    let freq = hist.count(1) as f64 / hist.total() as f64;
    let se = (0.64 * 0.36 / 4096.0_f64).sqrt();
    assert!((freq - 0.64).abs() < 4.0 * se, "freq {freq}, se {se}");
}
