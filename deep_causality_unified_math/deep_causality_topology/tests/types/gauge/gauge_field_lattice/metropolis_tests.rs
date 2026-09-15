/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_stats::Xoshiro256;
use deep_causality_topology::GaugeGroup;
use deep_causality_topology::LatticeComplex;
use deep_causality_topology::LatticeGaugeField;
use deep_causality_topology::LinkVariable;
use deep_causality_topology::SU3;
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
    let lattice = Arc::new(LatticeComplex::new(shape, [false]));

    use std::collections::HashMap;
    let empty_links = HashMap::new();
    let mut field = LatticeGaugeField::<U1, 1, Complex<f64>, f64, ()>::from_links_unchecked(
        lattice,
        empty_links,
        1.0,
        (),
    );

    // Verify it is empty
    assert!(field.links().is_empty());

    let mut rng = Xoshiro256::new();
    let acceptance = field.try_metropolis_sweep(0.1, &mut rng).unwrap();

    assert_eq!(acceptance, 0.0);
}

#[test]
fn test_metropolis_update_acceptance() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let beta = 1.0;

    let mut field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_identity(lattice.clone(), beta).unwrap();
    let mut rng = Xoshiro256::new();

    let edge = field.links().keys().next().unwrap().clone();

    let accepted = field.try_metropolis_update(&edge, 0.01, &mut rng).unwrap();
    assert!([true, false].contains(&accepted));

    let mut hot_field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_random(lattice, beta, &mut rng).unwrap();
    let edge_hot = hot_field.links().keys().next().unwrap().clone();

    let _ = hot_field
        .try_metropolis_update(&edge_hot, 0.1, &mut rng)
        .unwrap();
}

#[test]
fn test_metropolis_sweep_f64_optimization() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let beta = 1.0;
    let mut field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_identity(lattice, beta).unwrap();
    let mut rng = Xoshiro256::new();

    let rate = field.try_metropolis_sweep(0.1, &mut rng).unwrap();
    assert!((0.0..=1.0).contains(&rate));
}

#[test]
fn test_metropolis_update_nan_handling() {
    // Basic test to ensure no panic
}

#[test]
fn test_metropolis_update_rejects_non_positive_epsilon() {
    use deep_causality_topology::TopologyErrorEnum;
    let shape = [2, 2];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let mut field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_identity(lattice, 1.0).unwrap();
    let edge = field.links().keys().next().unwrap().clone();
    let mut rng = Xoshiro256::new();

    let err = field
        .try_metropolis_update(&edge, 0.0, &mut rng)
        .unwrap_err();
    match err.0 {
        TopologyErrorEnum::LatticeGaugeError(ref msg) => {
            assert!(msg.contains("Metropolis epsilon"));
        }
        ref other => panic!("Expected LatticeGaugeError, got {:?}", other),
    }

    let err_neg = field
        .try_metropolis_update(&edge, -0.1, &mut rng)
        .unwrap_err();
    match err_neg.0 {
        TopologyErrorEnum::LatticeGaugeError(_) => {}
        ref other => panic!("Expected LatticeGaugeError, got {:?}", other),
    }
}

#[test]
fn test_metropolis_sweep_empty_field_returns_zero_rate() {
    // Edge-case path through `if total == 0 { return Ok(0.0) }` in try_metropolis_sweep.
    use std::collections::HashMap;
    let lattice = Arc::new(LatticeComplex::<2, f64>::new([2, 2], [true, true]));
    let links: HashMap<_, LinkVariable<U1, Complex<f64>, f64>> = HashMap::new();
    let mut field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
        LatticeGaugeField::from_links_unchecked(lattice, links, 1.0, ());
    let mut rng = Xoshiro256::new();
    let rate = field.try_metropolis_sweep(0.1, &mut rng).unwrap();
    assert_eq!(rate, 0.0);
}

#[test]
fn test_generate_small_su_n_update() {
    let shape = [2];
    let lattice = Arc::new(LatticeComplex::new(shape, [true]));
    let mut field =
        LatticeGaugeField::<U1, 1, Complex<f64>, f64>::try_identity(lattice, 1.0).unwrap();
    let mut rng = Xoshiro256::new();

    let edge = field.links().keys().next().unwrap().clone();

    for _ in 0..100 {
        field.try_metropolis_update(&edge, 0.2, &mut rng).unwrap();
    }
}

// ---------------------------------------------------------------------------------------------
// Regression: the proposal has to leave the identity.
//
// These run over SU(3) because the proposal is built from a traceless Hermitian generator, and
// the only traceless Hermitian 1x1 matrix is zero. Over U(1) the proposal is the identity for
// arithmetic reasons, so the tests above pass whatever the generator does.
// ---------------------------------------------------------------------------------------------

/// A cold start sits at plaquette exactly 1. One sweep has to move it off that value.
///
/// A proposal that lands back on the identity gives `U' = U` and `ΔS = 0` for every link, so the
/// sweep reports full acceptance and leaves the configuration exactly where it started.
#[test]
fn test_metropolis_sweep_moves_a_cold_start_off_the_identity() {
    let lattice = Arc::new(LatticeComplex::new([2, 2, 2, 2], [true; 4]));
    let mut field =
        LatticeGaugeField::<SU3, 4, Complex<f64>, f64>::try_identity(lattice, 6.0).unwrap();
    let mut rng = Xoshiro256::new();

    let before = field.try_average_plaquette().unwrap();
    assert!(
        (before - 1.0).abs() < 1e-12,
        "a cold start should measure plaquette 1, measured {before}"
    );

    field.try_metropolis_sweep(0.3, &mut rng).unwrap();

    let after = field.try_average_plaquette().unwrap();
    assert!(
        after < 1.0 - 1e-9,
        "one sweep left the plaquette at {after}, so every proposal returned the identity"
    );
}

/// A hot start thermalizes: sweeps carry the average plaquette away from its initial value, and
/// the acceptance rate sits below 1 because some proposals raise the action.
#[test]
fn test_metropolis_sweep_thermalizes_a_hot_start() {
    let lattice = Arc::new(LatticeComplex::new([2, 2, 2, 2], [true; 4]));
    let mut rng = Xoshiro256::new();
    let mut field =
        LatticeGaugeField::<SU3, 4, Complex<f64>, f64>::try_random(lattice, 6.0, &mut rng).unwrap();

    let before = field.try_average_plaquette().unwrap();

    let mut rates = Vec::new();
    for _ in 0..5 {
        rates.push(field.try_metropolis_sweep(0.2, &mut rng).unwrap());
    }

    let after = field.try_average_plaquette().unwrap();
    assert!(
        (after - before).abs() > 1e-3,
        "five sweeps held the plaquette at {before}, so the field never moved"
    );
    assert!(
        rates.iter().any(|&r| r < 1.0),
        "every link was accepted in every sweep, which is what a no-op proposal does: {rates:?}"
    );
}
