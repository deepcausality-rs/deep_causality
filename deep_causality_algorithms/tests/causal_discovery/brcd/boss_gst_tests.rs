/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::{
    BicScorer, BossConfig, BrcdErrorEnum, FamilyScorer, Gst, family_bic,
};
use deep_causality_tensor::{CausalTensor, CausalTensorStatsExt};
use std::cell::Cell;

const RIDGE: f64 = 1e-6;
const LAMBDA: f64 = 2.0;

fn data_from_columns(cols: &[&[f64]]) -> CausalTensor<f64> {
    let n = cols[0].len();
    let k = cols.len();
    let mut flat = Vec::with_capacity(n * k);
    for i in 0..n {
        for c in cols {
            flat.push(c[i]);
        }
    }
    CausalTensor::from_slice(&flat, &[n, k])
}

// Strong linear chain X(0) -> Y(1) -> Z(2): Y = 3X + ε, Z = 3Y + ε.
fn chain_cov() -> (CausalTensor<f64>, usize) {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];
    let y = [
        0.1, 2.9, 6.05, 8.95, 12.1, 14.9, 18.05, 20.95, 24.1, 26.9, 30.05, 32.95,
    ];
    let z = [
        0.25, 8.75, 18.25, 26.75, 36.35, 44.65, 54.05, 62.95, 72.35, 80.65, 90.25, 98.75,
    ];
    (data_from_columns(&[&x, &y, &z]), x.len())
}

/// A scorer that counts the calls it forwards to an inner [`BicScorer`], so the
/// cache test can assert a re-trace performs no fresh scoring.
struct CountingScorer<'a> {
    inner: BicScorer<'a, f64>,
    calls: Cell<usize>,
}

impl<'a> CountingScorer<'a> {
    fn new(inner: BicScorer<'a, f64>) -> Self {
        Self {
            inner,
            calls: Cell::new(0),
        }
    }
    fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl FamilyScorer<f64> for CountingScorer<'_> {
    fn score(
        &self,
        node: usize,
        parents: &[usize],
    ) -> Result<f64, deep_causality_algorithms::brcd::BrcdError> {
        self.calls.set(self.calls.get() + 1);
        self.inner.score(node, parents)
    }
    fn num_vars(&self) -> usize {
        self.inner.num_vars()
    }
}

#[test]
fn trace_recovers_the_direct_parent_in_a_chain() {
    let (data, n) = chain_cov();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    // Z given {X, Y}: Y screens X, so grow adds {X,Y} then shrink drops X → {Y}.
    let mut gst = Gst::new(2, &scorer).unwrap();
    let (parents, score) = gst.trace(&[0, 1], &scorer).unwrap();
    assert_eq!(parents, vec![1]);

    // The returned score is exactly the family score of the chosen set.
    let expected = family_bic(&cov, n, 2, &[1], RIDGE, LAMBDA).unwrap();
    assert!((score - expected).abs() < 1e-9, "{score} vs {expected}");
}

#[test]
fn trace_picks_the_only_available_parent() {
    let (data, n) = chain_cov();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    // Y given {X}: X is the direct cause, so it is selected.
    let mut gst = Gst::new(1, &scorer).unwrap();
    let (parents, _) = gst.trace(&[0], &scorer).unwrap();
    assert_eq!(parents, vec![0]);
}

#[test]
fn empty_prefix_yields_no_parents() {
    let (data, n) = chain_cov();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    let mut gst = Gst::new(2, &scorer).unwrap();
    let (parents, score) = gst.trace(&[], &scorer).unwrap();
    assert!(parents.is_empty());
    let expected = family_bic(&cov, n, 2, &[], RIDGE, LAMBDA).unwrap();
    assert!((score - expected).abs() < 1e-9);
}

#[test]
fn a_vertex_is_never_its_own_parent() {
    let (data, n) = chain_cov();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    // Even if the vertex appears in the prefix, it is forbidden as a parent.
    let mut gst = Gst::new(1, &scorer).unwrap();
    let (parents, _) = gst.trace(&[0, 1, 2], &scorer).unwrap();
    assert!(!parents.contains(&1), "vertex 1 must not parent itself");
}

#[test]
fn re_tracing_the_same_prefix_uses_the_cache_with_no_new_scoring() {
    let (data, n) = chain_cov();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = CountingScorer::new(BicScorer::new(&cov, n, &cfg).unwrap());

    let mut gst = Gst::new(2, &scorer).unwrap();
    let first = gst.trace(&[0, 1], &scorer).unwrap();
    let after_first = scorer.calls();
    assert!(after_first > 0, "first trace must score");

    let second = gst.trace(&[0, 1], &scorer).unwrap();
    // No additional score calls: the grow/shrink results are cached.
    assert_eq!(scorer.calls(), after_first, "re-trace must hit the cache");
    assert_eq!(first.0, second.0);
    assert!((first.1 - second.1).abs() < 1e-12);
}

#[test]
fn vertex_getter_reports_the_target() {
    let (data, n) = chain_cov();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();
    let gst = Gst::new(2, &scorer).unwrap();
    assert_eq!(gst.vertex(), 2);
}

#[test]
fn new_propagates_an_out_of_range_vertex_error() {
    let (data, n) = chain_cov();
    let cov = data.sample_covariance().unwrap(); // 3 × 3
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    match Gst::new(7, &scorer) {
        Err(e) => assert_eq!(*e.kind(), BrcdErrorEnum::NodeOutOfBounds),
        Ok(_) => panic!("expected an out-of-range vertex error"),
    }
}
