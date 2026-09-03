/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The default-build Born sampler and the histogram it fills.
//!
//! The one physical anchor is that the sampled frequency of a projector converges on `Tr(Pρ)`;
//! everything else is the bookkeeping the spec names: reproducible at a seed, nothing drawn on a
//! malformed read-out, counts exposed through the shipped histogram trait.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CountHistogram, DensityMatrix, Projection, QuantumErrorEnum, ShotHistogram,
    born_projective_probability, sample_projector,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ket(a: f64, b: f64) -> CausalTensor<C> {
    CausalTensor::from_slice(&[Complex::new(a, 0.), Complex::new(b, 0.)], &[2])
}

fn state_and_projector() -> (DensityMatrix<f64>, Projection<f64, 2>) {
    (
        DensityMatrix::from_ket(&ket(0.6, 0.8)).unwrap(),
        Projection::from_ket(&ket(0., 1.)).unwrap(),
    )
}

#[test]
fn test_a_projector_read_out_converges_on_the_born_value() {
    let (rho, p) = state_and_projector();
    let born = born_projective_probability(&rho, &p).unwrap();
    assert!((born - 0.64).abs() < 1e-12);
    let shots = 8192u64;
    let hist = sample_projector(&rho, &p, shots, 7).unwrap();
    assert_eq!(hist.total(), shots);
    assert_eq!(hist.num_bits(), 1);
    assert_eq!(hist.count(1) + hist.count(0), shots);
    let freq = hist.count(1) as f64 / shots as f64;
    let se = (born * (1.0 - born) / shots as f64).sqrt();
    assert!(
        (freq - born).abs() < 4.0 * se,
        "freq {freq} vs born {born}, se {se}"
    );
    // The entries are exposed ascending by outcome.
    let entries = hist.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0, 0);
    assert_eq!(entries[1].0, 1);
}

#[test]
fn test_a_fixed_seed_reproduces_the_histogram_exactly_and_seeds_differ() {
    let (rho, p) = state_and_projector();
    let a = sample_projector(&rho, &p, 2048, 20260821).unwrap();
    let b = sample_projector(&rho, &p, 2048, 20260821).unwrap();
    assert_eq!(a, b);
    let c = sample_projector(&rho, &p, 2048, 20260822).unwrap();
    assert_ne!(a, c);
}

#[test]
fn test_a_malformed_read_out_draws_nothing() {
    let three = CausalTensor::from_slice(
        &[
            Complex::new(1., 0.),
            Complex::new(0., 0.),
            Complex::new(0., 0.),
        ],
        &[3],
    );
    let rho = DensityMatrix::from_ket(&three).unwrap();
    let p = Projection::<f64, 2>::from_ket(&ket(0., 1.)).unwrap();
    let err = sample_projector(&rho, &p, 1000, 1).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_zero_shots_is_an_empty_histogram() {
    let (rho, p) = state_and_projector();
    let hist = sample_projector(&rho, &p, 0, 1).unwrap();
    assert_eq!(hist.total(), 0);
    assert!(hist.entries().is_empty());
}

#[test]
fn test_the_histogram_is_plain_data_in_the_default_build() {
    let mut h = CountHistogram::new(2).unwrap();
    h.record(3).unwrap();
    h.record_n(1, 4).unwrap();
    h.record_n(0, 0).unwrap();
    assert_eq!(h.total(), 5);
    assert_eq!(h.count(1), 4);
    assert_eq!(h.count(2), 0);
    assert_eq!(h.entries(), vec![(1, 4), (3, 1)]);
}

#[test]
fn test_the_draw_happens_at_the_pipeline_scalar() {
    // The same state and projector at f32: the Born value is computed at f32 and compared at f32,
    // and the frequency still converges on it.
    let rho = DensityMatrix::<f32>::from_ket(&CausalTensor::from_slice(
        &[Complex::new(0.6f32, 0.), Complex::new(0.8f32, 0.)],
        &[2],
    ))
    .unwrap();
    let p = Projection::<f32, 2>::from_ket(&CausalTensor::from_slice(
        &[Complex::new(0.0f32, 0.), Complex::new(1.0f32, 0.)],
        &[2],
    ))
    .unwrap();
    let hist = sample_projector(&rho, &p, 4096, 3).unwrap();
    let freq = hist.count(1) as f32 / 4096.0;
    assert!((freq - 0.64).abs() < 0.04);
}
