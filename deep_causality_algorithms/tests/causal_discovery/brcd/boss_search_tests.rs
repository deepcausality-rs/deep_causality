/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::{
    BicScorer, BossConfig, BrcdError, BrcdErrorEnum, FamilyScorer, best_order_search,
};
use deep_causality_rand::{Rng, Xoshiro256};
use deep_causality_tensor::{CausalTensor, CausalTensorStatsExt};
use std::collections::BTreeSet;
use std::f64::consts::TAU;

/// One standard-normal draw via Box-Muller from two uniforms.
fn next_normal(rng: &mut Xoshiro256) -> f64 {
    // Guard u1 away from 0 so ln is finite.
    let u1: f64 = (rng.random::<f64>()).max(1e-12);
    let u2: f64 = rng.random::<f64>();
    (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
}

/// `n × 3` samples from a genuine linear-Gaussian chain X(0) → Y(1) → Z(2) with
/// unit-scale independent noise, so the chain skeleton is identifiable (the
/// variables are not near-collinear). Seeded for reproducibility.
fn chain_data(n: usize, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let mut flat = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = next_normal(&mut rng);
        let y = x + next_normal(&mut rng);
        let z = y + next_normal(&mut rng);
        flat.push(x);
        flat.push(y);
        flat.push(z);
    }
    CausalTensor::from_slice(&flat, &[n, 3])
}

/// Undirected skeleton {min,max} edges implied by the per-variable parent sets.
fn skeleton(parents: &[Vec<usize>]) -> BTreeSet<(usize, usize)> {
    let mut s = BTreeSet::new();
    for (v, ps) in parents.iter().enumerate() {
        for &p in ps {
            s.insert((p.min(v), p.max(v)));
        }
    }
    s
}

#[test]
fn recovers_the_chain_skeleton() {
    let data = chain_data(600, 11);
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, 600, &cfg).unwrap();

    let res = best_order_search(&scorer, cfg.seed).unwrap();
    let sk = skeleton(&res.parents);
    // The chain's Markov equivalence class has skeleton X–Y–Z and no v-structure;
    // the learned DAG must share that skeleton (no spurious X–Z edge).
    assert!(sk.contains(&(0, 1)), "X–Y edge missing: {sk:?}");
    assert!(sk.contains(&(1, 2)), "Y–Z edge missing: {sk:?}");
    assert!(!sk.contains(&(0, 2)), "spurious X–Z edge: {sk:?}");

    let mut sorted = res.order.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, vec![0, 1, 2]);
}

#[test]
fn is_deterministic_for_a_fixed_seed() {
    let data = chain_data(400, 3);
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, 400, &cfg).unwrap();

    let a = best_order_search(&scorer, 42).unwrap();
    let b = best_order_search(&scorer, 42).unwrap();
    assert_eq!(a, b, "same seed + data must give the same result");
}

#[test]
fn single_variable_has_no_parents() {
    let col = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let n = col.len();
    let mut flat = Vec::new();
    for v in col {
        flat.push(v);
    }
    let data = CausalTensor::from_slice(&flat, &[n, 1]);
    let cov = data.sample_covariance().unwrap(); // 1 × 1
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    let res = best_order_search(&scorer, 0).unwrap();
    assert_eq!(res.order, vec![0]);
    assert_eq!(res.parents, vec![Vec::<usize>::new()]);
}

#[test]
fn two_correlated_variables_share_one_edge() {
    // Y = X + noise over a real Gaussian sample: a single, clear edge.
    let mut rng = Xoshiro256::from_seed(5);
    let n = 300;
    let mut flat = Vec::with_capacity(n * 2);
    for _ in 0..n {
        let x = next_normal(&mut rng);
        let y = x + 0.3 * next_normal(&mut rng);
        flat.push(x);
        flat.push(y);
    }
    let data = CausalTensor::from_slice(&flat, &[n, 2]);
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    let res = best_order_search(&scorer, 1).unwrap();
    assert_eq!(skeleton(&res.parents), BTreeSet::from([(0, 1)]));
}

/// A scorer that always errors, to exercise error propagation through the search.
struct FailingScorer {
    n: usize,
}
impl FamilyScorer<f64> for FailingScorer {
    fn score(&self, _node: usize, _parents: &[usize]) -> Result<f64, BrcdError> {
        Err(BrcdError(BrcdErrorEnum::NodeOutOfBounds))
    }
    fn num_vars(&self) -> usize {
        self.n
    }
}

#[test]
fn propagates_scorer_errors() {
    let scorer = FailingScorer { n: 3 };
    match best_order_search(&scorer, 0) {
        Err(e) => assert_eq!(*e.kind(), BrcdErrorEnum::NodeOutOfBounds),
        Ok(_) => panic!("expected the scorer error to propagate"),
    }
}
