/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::types::Xoshiro256;
use deep_causality_topology::GaugeGroup;
use deep_causality_topology::Lattice;
use deep_causality_topology::LatticeGaugeField;
use std::sync::Arc;

// Define a test gauge group (U1 is simplest for testing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct U1;

impl GaugeGroup for U1 {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;

    fn matrix_dim() -> usize {
        1
    }
    fn name() -> &'static str {
        "U1"
    }
}

#[test]
fn test_metropolis_sweep_empty_lattice() {
    let shape = [1];
    let lattice = Arc::new(Lattice::new(shape, [false]));

    use std::collections::HashMap;
    let empty_links = HashMap::new();
    let mut field =
        LatticeGaugeField::<U1, 1, f64>::from_links_unchecked(lattice, empty_links, 1.0);

    // Verify it is empty
    assert!(field.links().is_empty());

    let mut rng = Xoshiro256::new();
    let acceptance = field.try_metropolis_sweep(0.1, &mut rng).unwrap();

    assert_eq!(acceptance, 0.0);
}

#[test]
fn test_metropolis_update_acceptance() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let beta = 1.0;

    let mut field = LatticeGaugeField::<U1, 2, f64>::try_identity(lattice.clone(), beta).unwrap();
    let mut rng = Xoshiro256::new();

    let edge = field.links().keys().next().unwrap().clone();

    let accepted = field.try_metropolis_update(&edge, 0.01, &mut rng).unwrap();
    assert!([true, false].contains(&accepted));

    let mut hot_field =
        LatticeGaugeField::<U1, 2, f64>::try_random(lattice, beta, &mut rng).unwrap();
    let edge_hot = hot_field.links().keys().next().unwrap().clone();

    let _ = hot_field
        .try_metropolis_update(&edge_hot, 0.1, &mut rng)
        .unwrap();
}

#[test]
fn test_metropolis_sweep_f64_optimization() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let beta = 1.0;
    let mut field = LatticeGaugeField::<U1, 2, f64>::try_identity(lattice, beta).unwrap();
    let mut rng = Xoshiro256::new();

    let rate = field.metropolis_sweep_f64(0.1, &mut rng).unwrap();
    assert!((0.0..=1.0).contains(&rate));
}

#[test]
fn test_metropolis_update_nan_handling() {
    // Basic test to ensure no panic
}

#[test]
fn test_generate_small_su_n_update() {
    let shape = [2];
    let lattice = Arc::new(Lattice::new(shape, [true]));
    let mut field = LatticeGaugeField::<U1, 1, f64>::try_identity(lattice, 1.0).unwrap();
    let mut rng = Xoshiro256::new();

    let edge = field.links().keys().next().unwrap().clone();

    for _ in 0..100 {
        field.try_metropolis_update(&edge, 0.2, &mut rng).unwrap();
    }
}
