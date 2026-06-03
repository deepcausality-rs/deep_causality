/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_augment::get_configurations_multi;
use deep_causality_algorithms::brcd::{BossConfig, BrcdErrorEnum, boss_learn};
use deep_causality_rand::{Rng, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, MixedGraph};
use std::f64::consts::TAU;

fn next_normal(rng: &mut Xoshiro256) -> f64 {
    let u1: f64 = (rng.random::<f64>()).max(1e-12);
    let u2: f64 = rng.random::<f64>();
    (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
}

/// Genuine linear-Gaussian chain X(0) → Y(1) → Z(2).
fn chain(n: usize, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let mut flat = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = next_normal(&mut rng);
        let y = x + next_normal(&mut rng);
        let z = y + next_normal(&mut rng);
        flat.extend([x, y, z]);
    }
    CausalTensor::from_slice(&flat, &[n, 3])
}

/// A genuine collider X(0) → Z(2) ← Y(1): X, Y independent, Z = X + Y + noise.
///
/// The noise is deliberately large (std 3): conditioning on a collider couples
/// its parents, so a small-noise `Z = X + Y` would tempt a score-based search
/// into a spurious X–Y edge. Large noise keeps that induced dependence weak, so
/// the unshielded collider is the unambiguous optimum.
fn collider(n: usize, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let mut flat = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = next_normal(&mut rng);
        let y = next_normal(&mut rng);
        let z = x + y + 3.0 * next_normal(&mut rng);
        flat.extend([x, y, z]);
    }
    CausalTensor::from_slice(&flat, &[n, 3])
}

/// Sorted arcs and undirected edges identifying a CPDAG.
type CpdagSignature = (Vec<(usize, usize)>, Vec<(usize, usize)>);

/// Sorted (arcs, undirected) signature for comparing two CPDAGs.
fn signature(g: &MixedGraph<()>) -> CpdagSignature {
    let mut arcs = Vec::new();
    for v in 0..g.num_vertices() {
        for p in g.parents(v) {
            arcs.push((p, v));
        }
    }
    arcs.sort_unstable();
    let mut und = g.undirected_edges();
    und.sort_unstable();
    (arcs, und)
}

#[test]
fn learns_the_chain_cpdag() {
    let data = chain(600, 11);
    let cfg = BossConfig::<f64>::default();
    let cpdag = boss_learn(&data, &cfg).unwrap();

    // The chain's CPDAG is the undirected path X—Y—Z (no v-structure).
    assert_eq!(cpdag.edge_kind(0, 1), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(1, 2), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(0, 2), None);
}

#[test]
fn learns_the_collider_v_structure() {
    let data = collider(600, 7);
    let cfg = BossConfig::<f64>::default();
    let cpdag = boss_learn(&data, &cfg).unwrap();

    // X ⫫ Y but both cause Z: the unshielded collider is oriented.
    assert_eq!(cpdag.edge_kind(0, 2), Some(EdgeKind::Directed));
    assert_eq!(cpdag.edge_kind(1, 2), Some(EdgeKind::Directed));
    assert!(cpdag.parents(2).contains(&0) && cpdag.parents(2).contains(&1));
    assert_eq!(cpdag.edge_kind(0, 1), None);
}

#[test]
fn learned_cpdag_is_accepted_by_configuration_enumeration() {
    let data = chain(400, 3);
    let cfg = BossConfig::<f64>::default();
    let cpdag = boss_learn(&data, &cfg).unwrap();

    // A valid CPDAG: brcd_run's configuration enumeration accepts it.
    let configs = get_configurations_multi(&cpdag, &[0]).unwrap();
    assert!(!configs.is_empty());
}

#[test]
fn is_deterministic_for_a_fixed_seed() {
    let data = chain(400, 9);
    let cfg = BossConfig::<f64>::default();
    let a = boss_learn(&data, &cfg).unwrap();
    let b = boss_learn(&data, &cfg).unwrap();
    assert_eq!(signature(&a), signature(&b));
}

#[test]
fn one_dimensional_data_is_a_dimension_mismatch() {
    let data = CausalTensor::from_slice(&[1.0_f64, 2.0, 3.0], &[3]);
    let err = boss_learn(&data, &BossConfig::<f64>::default()).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::DimensionMismatch);
}

#[test]
fn fewer_than_two_rows_is_empty_data() {
    // One observation: a sample covariance is undefined.
    let data = CausalTensor::from_slice(&[1.0_f64, 2.0, 3.0], &[1, 3]);
    let err = boss_learn(&data, &BossConfig::<f64>::default()).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::EmptyData);
}

#[test]
fn zero_columns_is_a_dimension_mismatch() {
    let data: CausalTensor<f64> = CausalTensor::from_slice(&[], &[4, 0]);
    let err = boss_learn(&data, &BossConfig::<f64>::default()).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::DimensionMismatch);
}
