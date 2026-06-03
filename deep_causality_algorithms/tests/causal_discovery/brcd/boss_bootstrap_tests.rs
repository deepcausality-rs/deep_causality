/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::{
    BootstrapConfig, BrcdConfig, BrcdErrorEnum, brcd_run_bootstrap,
};
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;

/// Linear-Gaussian chain X → Y → Z; `y_intercept` perturbs p(Y | X).
fn chain_data(n: usize, y_intercept: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = dist.sample(&mut rng);
        let y = y_intercept + 1.5 * x + dist.sample(&mut rng);
        let z = 2.0 * y + dist.sample(&mut rng);
        data.extend([x, y, z]);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

/// Discrete chain X → Y → Z with integer states in {0,1,2}; `shift` perturbs Y.
fn discrete_chain(n: usize, shift: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let bucket = |v: f64| {
        if v < -0.5 {
            0.0
        } else if v < 0.5 {
            1.0
        } else {
            2.0
        }
    };
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = bucket(dist.sample(&mut rng));
        let y = bucket(0.8 * x - 0.8 + shift + dist.sample(&mut rng));
        let z = bucket(0.8 * y - 0.8 + dist.sample(&mut rng));
        data.extend([x, y, z]);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

#[test]
fn returns_a_normalized_marginal_ranking() {
    let normal = chain_data(200, 0.0, 1);
    let anomalous = chain_data(200, 4.0, 2);
    let config = BrcdConfig::continuous(7);
    let boot = BootstrapConfig::new(10, 5);

    let result = brcd_run_bootstrap(&normal, &anomalous, &config, &boot).unwrap();

    // Every candidate appears once, and the marginal posterior is a normalized
    // distribution in descending order.
    assert_eq!(result.ranks().len(), 3);
    let mut seen: Vec<usize> = result.ranks().iter().map(|c| c[0]).collect();
    seen.sort_unstable();
    assert_eq!(seen, vec![0, 1, 2]);

    let post = result.posterior();
    assert!(post.iter().all(|p| p.is_finite() && *p >= 0.0));
    let sum: f64 = post.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-9,
        "marginal posterior must sum to 1: {sum}"
    );
    assert!(post.windows(2).all(|w| w[0] >= w[1]), "descending order");

    // The perturbed mechanism p(Y | X) makes Y the root cause.
    assert_eq!(result.top(), Some(&[1][..]), "ranks: {:?}", result.ranks());
}

#[test]
fn single_sample_single_cpdag_is_a_normalized_ranking() {
    // B = 1, k = 1: one CPDAG with weight 1 → the marginal is exactly that
    // CPDAG's normalized per-candidate posterior.
    let normal = chain_data(150, 0.0, 11);
    let anomalous = chain_data(150, 4.0, 12);
    let config = BrcdConfig::continuous(3);
    let boot = BootstrapConfig::new(1, 1);

    let result = brcd_run_bootstrap(&normal, &anomalous, &config, &boot).unwrap();
    let sum: f64 = result.posterior().iter().sum();
    assert!((sum - 1.0).abs() < 1e-9);
    assert_eq!(result.ranks().len(), 3);
}

#[test]
fn is_deterministic_for_a_fixed_seed() {
    let normal = chain_data(120, 0.0, 21);
    let anomalous = chain_data(120, 4.0, 22);
    let config = BrcdConfig::continuous(9);
    let boot = BootstrapConfig::new(8, 4);

    let a = brcd_run_bootstrap(&normal, &anomalous, &config, &boot).unwrap();
    let b = brcd_run_bootstrap(&normal, &anomalous, &config, &boot).unwrap();
    assert_eq!(a, b);
}

#[test]
fn marginalizes_the_discrete_family() {
    // Exercises the Dirichlet joint-likelihood branch of the CPDAG weight.
    let normal = discrete_chain(200, 0.0, 31);
    let anomalous = discrete_chain(200, 1.5, 32);
    let config = BrcdConfig::discrete(7);
    let boot = BootstrapConfig::new(6, 3);

    let result = brcd_run_bootstrap(&normal, &anomalous, &config, &boot).unwrap();
    assert_eq!(result.ranks().len(), 3);
    let sum: f64 = result.posterior().iter().sum();
    assert!((sum - 1.0).abs() < 1e-9);
    assert!(result.posterior().iter().all(|p| p.is_finite()));
}

#[test]
fn config_is_copy_and_constructible() {
    let c = BootstrapConfig::new(10, 5);
    let d = c; // Copy
    assert_eq!(c, d);
    assert_eq!(d.samples, 10);
    assert_eq!(d.top_k, 5);
}

#[test]
fn zero_samples_is_rejected() {
    let normal = chain_data(20, 0.0, 41);
    let anomalous = chain_data(20, 4.0, 42);
    let err = brcd_run_bootstrap(
        &normal,
        &anomalous,
        &BrcdConfig::continuous(0),
        &BootstrapConfig::new(0, 5),
    )
    .unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::DimensionMismatch);
}

#[test]
fn zero_top_k_is_rejected() {
    let normal = chain_data(20, 0.0, 51);
    let anomalous = chain_data(20, 4.0, 52);
    let err = brcd_run_bootstrap(
        &normal,
        &anomalous,
        &BrcdConfig::continuous(0),
        &BootstrapConfig::new(5, 0),
    )
    .unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::DimensionMismatch);
}

#[test]
fn fewer_than_two_rows_is_empty_data() {
    let normal = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![1, 3]).unwrap();
    let anomalous = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![1, 3]).unwrap();
    let err = brcd_run_bootstrap(
        &normal,
        &anomalous,
        &BrcdConfig::continuous(0),
        &BootstrapConfig::new(4, 2),
    )
    .unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::EmptyData);
}

#[test]
fn one_dimensional_data_is_a_dimension_mismatch() {
    let normal = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();
    let anomalous = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();
    let err = brcd_run_bootstrap(
        &normal,
        &anomalous,
        &BrcdConfig::continuous(0),
        &BootstrapConfig::new(4, 2),
    )
    .unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::DimensionMismatch);
}
