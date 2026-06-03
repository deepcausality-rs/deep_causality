/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_config::BrcdConfig;
use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_algorithms::brcd::brcd_run;
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

/// Linear-Gaussian chain X → Y → Z: `X=εx`, `Y=y_intercept+1.5X+εy`, `Z=2Y+εz`.
/// Columns are [X, Y, Z]; `n` rows.
fn chain_data(n: usize, y_intercept: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = dist.sample(&mut rng);
        let y = y_intercept + 1.5 * x + dist.sample(&mut rng);
        let z = 2.0 * y + dist.sample(&mut rng);
        data.push(x);
        data.push(y);
        data.push(z);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

/// A discrete chain X → Y → Z with integer states in `{0, 1, 2}`. `shift`
/// perturbs Y's mechanism between the two regimes.
fn discrete_chain(n: usize, shift: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let bucket = |v: f64| -> f64 {
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
        data.push(x);
        data.push(y);
        data.push(z);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

/// The undirected chain CPDAG X — Y — Z (arcs = [], edges = {(X,Y),(Y,Z)}).
fn chain_cpdag() -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
    let mut g = MixedGraph::new(3, data, 0).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g
}

#[test]
fn recovers_the_perturbed_mechanism() {
    // The anomaly perturbs p(Y | X) (Y's intercept jumps), so BRCD should rank
    // Y (index 1) as the top root cause.
    let normal = chain_data(120, 0.0, 1);
    let anomalous = chain_data(120, 4.0, 2);
    let cpdag = chain_cpdag();
    let config = BrcdConfig::continuous(7);

    let result = brcd_run(&normal, &anomalous, &cpdag, &config).unwrap();
    assert_eq!(result.top(), Some(&[1][..]), "ranks: {:?}", result.ranks());
}

#[test]
fn produces_a_normalized_ranking_over_all_candidates() {
    let normal = chain_data(80, 0.0, 11);
    let anomalous = chain_data(80, 3.0, 12);
    let cpdag = chain_cpdag();
    let result = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::continuous(3)).unwrap();

    // k = 1 over 3 variables → 3 single-element candidate sets.
    assert_eq!(result.ranks().len(), 3);
    assert!(result.ranks().iter().all(|c| c.len() == 1));
    // The three variables each appear exactly once.
    let mut seen: Vec<usize> = result.ranks().iter().map(|c| c[0]).collect();
    seen.sort_unstable();
    assert_eq!(seen, vec![0, 1, 2]);
    // Posterior is in descending order and finite.
    let post = result.posterior();
    assert!(post.iter().all(|p| p.is_finite()));
    assert!(post.windows(2).all(|w| w[0] >= w[1]));
}

#[test]
fn is_deterministic_under_a_fixed_seed() {
    let normal = chain_data(60, 0.0, 21);
    let anomalous = chain_data(60, 3.0, 22);
    let cpdag = chain_cpdag();
    let a = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::continuous(9)).unwrap();
    let b = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::continuous(9)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn display_renders_the_ranking() {
    let normal = chain_data(40, 0.0, 31);
    let anomalous = chain_data(40, 3.0, 32);
    let cpdag = chain_cpdag();
    let result = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::continuous(1)).unwrap();
    let text = format!("{result}");
    assert!(text.contains("BRCD Root-Cause Ranking"));
    assert!(text.contains("posterior="));
}

#[test]
fn misaligned_datasets_are_rejected() {
    let normal = chain_data(20, 0.0, 41);
    // Anomalous with only 2 columns → mismatch.
    let anomalous = CausalTensor::new(vec![0.0_f64; 20 * 2], vec![20, 2]).unwrap();
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::continuous(0)).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn too_many_root_causes_is_rejected() {
    let normal = chain_data(20, 0.0, 51);
    let anomalous = chain_data(20, 3.0, 52);
    let cpdag = chain_cpdag();
    let mut config = BrcdConfig::continuous(0);
    config.num_root_causes = 4; // > 3 variables
    assert_eq!(
        brcd_run(&normal, &anomalous, &cpdag, &config).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn zero_root_causes_is_rejected() {
    let normal = chain_data(20, 0.0, 61);
    let anomalous = chain_data(20, 3.0, 62);
    let cpdag = chain_cpdag();
    let mut config = BrcdConfig::continuous(0);
    config.num_root_causes = 0; // k must be >= 1
    assert_eq!(
        brcd_run(&normal, &anomalous, &cpdag, &config).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn empty_datasets_are_rejected() {
    let normal = CausalTensor::new(Vec::<f64>::new(), vec![0, 3]).unwrap();
    let anomalous = CausalTensor::new(Vec::<f64>::new(), vec![0, 3]).unwrap();
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::continuous(0)).err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
}

#[test]
fn scores_the_discrete_family() {
    // Exercises the FamilyKind::Discrete branch: state binning, the Dirichlet
    // likelihood, and the full structural + posterior pipeline on discrete data.
    let normal = discrete_chain(120, 0.0, 71);
    let anomalous = discrete_chain(120, 1.5, 72);
    let cpdag = chain_cpdag();

    let result = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::discrete(7)).unwrap();

    // k = 1 over 3 variables → 3 single-element candidate sets, each variable once.
    assert_eq!(result.ranks().len(), 3);
    assert!(result.ranks().iter().all(|c| c.len() == 1));
    let mut seen: Vec<usize> = result.ranks().iter().map(|c| c[0]).collect();
    seen.sort_unstable();
    assert_eq!(seen, vec![0, 1, 2]);

    // Posterior is finite and ranked descending.
    let post = result.posterior();
    assert!(post.iter().all(|p| p.is_finite()));
    assert!(post.windows(2).all(|w| w[0] >= w[1]));
}

#[test]
fn discrete_family_is_deterministic_under_a_fixed_seed() {
    let normal = discrete_chain(60, 0.0, 81);
    let anomalous = discrete_chain(60, 1.5, 82);
    let cpdag = chain_cpdag();
    let a = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::discrete(9)).unwrap();
    let b = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::discrete(9)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn discrete_family_rejects_negative_states() {
    // The discrete family rounds each value to a non-negative integer state; a
    // negative value (here -1.0) is outside the 0..K range and must be rejected.
    let normal = CausalTensor::new(vec![-1.0_f64, 0.0, 1.0, 0.0, 1.0, 2.0], vec![2, 3]).unwrap();
    let anomalous = CausalTensor::new(vec![0.0_f64, 1.0, 2.0, 1.0, 2.0, 0.0], vec![2, 3]).unwrap();
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::discrete(0)).err(),
        Some(BrcdError(BrcdErrorEnum::StateOutOfRange))
    );
}
