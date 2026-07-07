/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Additional coverage for lattice gauge field operations:
//! - SU(2) Metropolis proposals (off-diagonal + diagonal perturbation loops)
//! - the "action decreases" Metropolis accept branch
//! - smearing dimension guard (D <= 1)
//! - empty-plane average-plaquette short circuit (count == 0)

use deep_causality_num_complex::Complex;
use deep_causality_rand::types::Xoshiro256;
use deep_causality_topology::{
    LatticeComplex, LatticeGaugeField, SU2, SmearingParams, TopologyErrorEnum, U1,
};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// SU(2) Metropolis: exercises the non-trivial perturbation generation loops
// (off-diagonal Hermitian fill and diagonal fill), which only run for n >= 2,
// plus the "delta_s < 0 => always accept" branch on a hot configuration.
// ============================================================================

#[test]
fn test_su2_metropolis_update_runs_perturbation_loops() {
    let lattice = Arc::new(LatticeComplex::new([2, 2], [true, true]));
    let mut rng = Xoshiro256::new();

    // A random (hot) SU(2) field: matrix_dim() == 2, so the off-diagonal loop
    // (j in i+1..n) and the diagonal loop (i in 0..n-1) both execute.
    let mut field =
        LatticeGaugeField::<SU2, 2, Complex<f64>, f64>::try_random(lattice, 4.0, &mut rng)
            .expect("random SU(2) field");

    let edges: Vec<_> = field.links().keys().cloned().collect();
    assert!(!edges.is_empty());

    let mut saw_accept = false;
    // Many updates on a hot field: at least some proposals lower the local action
    // (delta_s < 0), driving the unconditional-accept branch.
    for _ in 0..200 {
        for edge in &edges {
            if field
                .try_metropolis_update(edge, 0.3, &mut rng)
                .expect("metropolis update")
            {
                saw_accept = true;
            }
        }
    }
    assert!(saw_accept, "expected at least one accepted SU(2) update");
}

#[test]
fn test_su2_metropolis_sweep_returns_valid_rate() {
    let lattice = Arc::new(LatticeComplex::new([2, 2], [true, true]));
    let mut rng = Xoshiro256::new();
    let mut field =
        LatticeGaugeField::<SU2, 2, Complex<f64>, f64>::try_random(lattice, 2.0, &mut rng)
            .expect("random SU(2) field");

    let rate = field.try_metropolis_sweep(0.3, &mut rng).expect("sweep");
    assert!((0.0..=1.0).contains(&rate));
}

// ============================================================================
// Smearing: D <= 1 must be rejected.
// ============================================================================

#[test]
fn test_smear_rejects_one_dimensional_lattice() {
    let lattice = Arc::new(LatticeComplex::new([4], [true]));
    let field = LatticeGaugeField::<U1, 1, Complex<f64>, f64>::identity(lattice, 1.0);

    let params = SmearingParams::ape_default();
    let err = field
        .try_smear(&params)
        .expect_err("smearing requires D >= 2");
    match err.0 {
        TopologyErrorEnum::LatticeGaugeError(ref msg) => {
            assert!(msg.contains("D >= 2"), "unexpected message: {msg}");
        }
        ref other => panic!("expected LatticeGaugeError, got {:?}", other),
    }
}

// ============================================================================
// try_average_plaquette: with D == 1 there are no mu < nu planes, so the
// plaquette counter stays zero and the function returns 1.0.
// ============================================================================

#[test]
fn test_average_plaquette_no_planes_returns_one() {
    let lattice = Arc::new(LatticeComplex::new([4], [true]));
    let field = LatticeGaugeField::<U1, 1, Complex<f64>, f64>::identity(lattice, 1.0);

    let avg = field.try_average_plaquette().expect("average plaquette");
    assert!((avg - 1.0).abs() < 1e-12, "expected 1.0, got {avg}");
}

#[test]
fn test_average_plaquette_empty_lattice_returns_one() {
    // No sites at all -> count stays zero -> 1.0.
    let lattice = Arc::new(LatticeComplex::<2, f64>::new([0, 0], [false, false]));
    let links: HashMap<_, deep_causality_topology::LinkVariable<U1, Complex<f64>, f64>> =
        HashMap::new();
    let field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
        LatticeGaugeField::from_links_unchecked(lattice, links, 1.0, ());

    let avg = field.try_average_plaquette().expect("average plaquette");
    assert!((avg - 1.0).abs() < 1e-12, "expected 1.0, got {avg}");
}
