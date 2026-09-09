/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The deterministic PRNG behind the shot samplers.
//!
//! `SplitMix64` is `pub(crate)`, so it is pinned here through `sample_projector`, its only
//! consumer in the default build.
//!
//! # Why the existing sampler tests cannot pin it
//!
//! `born_sampler_tests.rs` checks two things of a seeded run: that the same seed reproduces the
//! same histogram, and that the sampled frequency converges on `Tr(Pρ)` within four standard
//! errors. Both hold for ANY deterministic uniform generator. Change the splitmix64 increment
//! from `0x9E3779B97F4A7C15`, or a shift from 30 to 29, and the stream is still deterministic
//! and still uniform, so every one of those assertions still passes while the documented
//! algorithm has been replaced.
//!
//! # Oracle provenance
//!
//! splitmix64 is Steele, Lea & Flood, "Fast splittable pseudorandom number generators", OOPSLA
//! 2014, and its output for a given seed is a published sequence. The counts below were computed
//! from that published algorithm — increment `0x9E3779B97F4A7C15`, multipliers
//! `0xBF58476D1CE4E5B9` and `0x94D049BB133111EB`, shifts 30, 27, 31 — evaluated outside this
//! crate, then combined with the sampler's documented rule: draw `u = (next_u64() >> 11) · 2⁻⁵³`
//! and count outcome 1 exactly when `u < p`.
//!
//! Each count is checked to be insensitive to the last unit in the last place of `p`, so the
//! oracle does not depend on how the Born trace happens to be summed.
//!
//! # What this oracle reaches, and what it cannot
//!
//! Measured by substituting each constant and re-running: the increment, both multipliers, the
//! first two shifts and the `>> 11` scaling all change these counts, and all are killed here
//! while the existing reproducibility and convergence tests accept them.
//!
//! The FINAL `z ^ (z >> 31)` is not reachable this way. Changing it to `>> 32` moves each draw
//! by about 1e-10, which is far below the threshold comparison, so `u < p` flips only for a draw
//! landing within 1e-10 of `p` — roughly one in ten billion. Every count here is identical under
//! that mutation, and no Bernoulli count through this API can see it, whatever the seed or shot
//! budget.
//!
//! Pinning it needs the raw `u64` stream against the published vectors
//! (seed 0 yields 0xE220A8397B1DCDAF, 0x6E789E6AA1B965F4, 0x06C45D188009454F), which requires
//! `SplitMix64` to be reachable from a test. It is `pub(crate)` with no re-export, and this
//! workspace keeps all tests in `tests/`, so that is an API decision rather than a test one and
//! is left open deliberately.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    DensityMatrix, Projection, ShotHistogram, born_projective_probability, sample_projector,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ket(a: f64, b: f64) -> CausalTensor<C> {
    CausalTensor::from_slice(&[Complex::new(a, 0.), Complex::new(b, 0.)], &[2])
}

/// `Tr(Pρ) = 0.64` for this pair, the same fixture the sampler tests use.
fn fixture() -> (DensityMatrix<f64>, Projection<f64, 2>) {
    (
        DensityMatrix::from_ket(&ket(0.6, 0.8)).unwrap(),
        Projection::from_ket(&ket(0., 1.)).unwrap(),
    )
}

#[test]
fn test_the_stream_matches_the_published_splitmix64_sequence() {
    let (rho, p) = fixture();
    assert!(
        (born_projective_probability(&rho, &p).unwrap() - 0.64).abs() < 1e-12,
        "the oracle below is computed against p = 0.64"
    );

    // (seed, shots, ones) from the published sequence. None equals the statistical
    // expectation — 41, 164, 41 and 82 respectively — so these pin the stream itself
    // rather than its mean.
    for (seed, shots, expected_ones) in [
        (7u64, 64u64, 44u64),
        (7, 256, 171),
        (20_260_821, 64, 42),
        (1, 128, 77),
    ] {
        let hist = sample_projector(&rho, &p, shots, seed).unwrap();
        assert_eq!(
            hist.count(1),
            expected_ones,
            "seed {seed}, {shots} shots: the draw sequence has moved"
        );
        assert_eq!(hist.count(0), shots - expected_ones);
        assert_eq!(hist.total(), shots);
    }
}

#[test]
fn test_the_uniform_never_reaches_one_and_never_falls_below_zero() {
    // The two ends of `(next_u64() >> 11) · 2⁻⁵³`. The shift leaves 53 bits, so the largest
    // draw is 1 − 2⁻⁵³ and the smallest is 0.
    //
    // A certain projector accepts every shot only because every draw is strictly below 1.0;
    // an impossible one accepts none only because no draw is below 0.0. A generator on the
    // closed interval, or one shifted by 10 bits so it can reach 2.0, breaks one of these
    // while leaving the mean untouched.
    let certain = (
        DensityMatrix::from_ket(&ket(0., 1.)).unwrap(),
        Projection::<f64, 2>::from_ket(&ket(0., 1.)).unwrap(),
    );
    assert!((born_projective_probability(&certain.0, &certain.1).unwrap() - 1.0).abs() < 1e-15);
    let hist = sample_projector(&certain.0, &certain.1, 4096, 11).unwrap();
    assert_eq!(hist.count(1), 4096, "some draw reached or exceeded 1.0");
    assert_eq!(hist.count(0), 0);

    let impossible = (
        DensityMatrix::from_ket(&ket(1., 0.)).unwrap(),
        Projection::<f64, 2>::from_ket(&ket(0., 1.)).unwrap(),
    );
    assert!(born_projective_probability(&impossible.0, &impossible.1).unwrap() < 1e-15);
    let hist = sample_projector(&impossible.0, &impossible.1, 4096, 11).unwrap();
    assert_eq!(hist.count(1), 0, "some draw fell below 0.0");
    assert_eq!(hist.count(0), 4096);
}

#[test]
fn test_the_stream_advances_rather_than_repeating() {
    // Each shot must consume a new draw. A generator that returned its first value forever
    // would still be deterministic and would still pass the reproducibility check, but the
    // histogram would be all-ones or all-zeros at every intermediate probability.
    let (rho, p) = fixture();
    let hist = sample_projector(&rho, &p, 512, 5).unwrap();
    assert!(
        hist.count(0) > 0 && hist.count(1) > 0,
        "a stuck stream gives {} ones and {} zeros",
        hist.count(1),
        hist.count(0)
    );
}
