/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the open-boundary Leray projection (`leray_project_open_opts`): the constrained
//! projection generalized to admit nonzero net boundary flux under mixed boundary conditions.
//!
//! Pinned here:
//! - **closed-domain reduction**: with both sets empty it equals the plain projection, and with
//!   only zeroed edges it is bit-identical to the constrained projection (the prerequisite's
//!   non-breaking guarantee);
//! - **open-boundary correctness**: a uniform prescribed inflow against an outflow pressure
//!   reference yields uniform, divergence-free flow whose free outflow flux balances the inflow
//!   (mass conservation);
//! - **well-posedness**: a prescribed inflow with no reference is rejected (the net flux has
//!   nowhere to leave).

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
};

fn manifold_2d(
    shape: [usize; 2],
    periodic: [bool; 2],
    metric: CubicalReggeGeometry<2, f64>,
) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new(shape, periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn wall_tangential_edges(complex: &LatticeComplex<2, f64>) -> Vec<usize> {
    let periodic = complex.periodic();
    let shape = complex.shape();
    complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            let axis = c.orientation().trailing_zeros() as usize;
            let pos = c.position();
            (0..2)
                .any(|w| w != axis && !periodic[w] && (pos[w] == 0 || pos[w] + 1 == shape[w]))
                .then_some(i)
        })
        .collect()
}

fn random_field(len: usize, seed: u64) -> CausalTensor<f64> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let data: Vec<f64> = (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            2.0 * ((state >> 11) as f64 / (1u64 << 53) as f64) - 1.0
        })
        .collect();
    CausalTensor::new(data, vec![len]).unwrap()
}

fn sup(v: impl IntoIterator<Item = f64>) -> f64 {
    v.into_iter().fold(0.0, |m, x| m.max(x.abs()))
}

/// All roles empty ⇒ the plain projection, bit-identically.
#[test]
fn open_with_no_boundary_equals_the_plain_projection() {
    let m = manifold_2d([6, 6], [true, true], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 7);
    let opts = HodgeDecomposeOptions::default();

    let plain = m.leray_project_opts(&field, &opts).unwrap();
    let open = m
        .leray_project_open_opts(&field, &[], &[], &[], &opts)
        .unwrap();
    assert_eq!(plain.projected().as_slice(), open.projected().as_slice());
}

/// Only zeroed edges ⇒ the constrained projection, bit-identically (the closed-domain reduction).
#[test]
fn open_with_only_zeroed_edges_equals_the_constrained_projection() {
    let m = manifold_2d([8, 6], [true, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let edges = wall_tangential_edges(m.complex());
    let field = random_field(n1, 11);
    let opts = HodgeDecomposeOptions::default();

    let constrained = m
        .leray_project_constrained_opts(&field, &edges, &opts)
        .unwrap();
    let open = m
        .leray_project_open_opts(&field, &edges, &[], &[], &opts)
        .unwrap();
    assert_eq!(
        constrained.projected().as_slice(),
        open.projected().as_slice()
    );
}

/// A uniform prescribed inflow on the west face of a periodic-y channel against an east outflow
/// pressure reference: the projection produces uniform `u_x = U`, zero `u_y`, divergence-free, and
/// the free outflow flux at the east balances the inflow (mass conservation).
#[test]
fn uniform_inflow_outflow_channel_is_uniform_and_mass_conservative() {
    // x: open (inflow on the west, reference outflow on the east), y: periodic — so the exact
    // divergence-free answer is uniform.
    let shape = [5usize, 4usize];
    let m = manifold_2d(shape, [false, true], CubicalReggeGeometry::unit());
    let complex = m.complex();
    let n1 = complex.num_cells(1);
    let inflow = 0.7_f64; // edge integral on a unit-length edge.

    // West inflow: x-edges at x = 0. East outflow reference: vertices at x = shape[0]-1.
    let west_x_edges: Vec<usize> = complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            (c.orientation().trailing_zeros() == 0 && c.position()[0] == 0).then_some(i)
        })
        .collect();
    let east_vertices: Vec<usize> = complex
        .iter_cells(0)
        .enumerate()
        .filter_map(|(i, c)| (c.position()[0] == shape[0] - 1).then_some(i))
        .collect();
    let last_x_col = shape[0] - 2; // index of the easternmost x-edge column.
    let east_x_edges: Vec<usize> = complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            (c.orientation().trailing_zeros() == 0 && c.position()[0] == last_x_col).then_some(i)
        })
        .collect();
    assert!(!west_x_edges.is_empty() && !east_vertices.is_empty());

    // Seed: inflow on the west x-edges, zero elsewhere.
    let mut data = vec![0.0; n1];
    for &e in &west_x_edges {
        data[e] = inflow;
    }
    let field = CausalTensor::new(data, vec![n1]).unwrap();

    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(10_000),
    };
    let p = m
        .leray_project_open_opts(&field, &[], &west_x_edges, &east_vertices, &opts)
        .unwrap();
    let u = p.projected().as_slice();

    // Every x-edge carries the uniform inflow; every y-edge is zero.
    for (i, c) in complex.iter_cells(1).enumerate() {
        if c.orientation().trailing_zeros() == 0 {
            assert!(
                (u[i] - inflow).abs() < 1e-9,
                "x-edge {i} not uniform: {} vs {inflow}",
                u[i]
            );
        } else {
            assert!(u[i].abs() < 1e-9, "y-edge {i} nonzero: {}", u[i]);
        }
    }

    // Divergence-free at interior vertices (the open inlet/outlet columns carry boundary flux, so
    // their divergence is expectedly nonzero — that is the open boundary condition).
    let codiff = m.codifferential_of(u, 1).into_vec();
    let interior_div = sup(complex.iter_cells(0).enumerate().filter_map(|(i, c)| {
        let x = c.position()[0];
        (x > 0 && x + 1 < shape[0]).then_some(codiff[i])
    }));
    assert!(
        interior_div < 1e-9,
        "interior divergence {interior_div:e} above solve exactness"
    );

    // Mass conservation: total flux in at the west equals total flux out at the east.
    let flux_in: f64 = west_x_edges.iter().map(|&e| u[e]).sum();
    let flux_out: f64 = east_x_edges.iter().map(|&e| u[e]).sum();
    assert!(
        (flux_in - flux_out).abs() < 1e-9,
        "mass not conserved: in {flux_in} vs out {flux_out}"
    );
}

/// A prescribed inflow with no outflow reference is an unbalanced open domain — rejected.
#[test]
fn inflow_without_reference_is_rejected() {
    let m = manifold_2d([5, 4], [false, true], CubicalReggeGeometry::unit());
    let complex = m.complex();
    let n1 = complex.num_cells(1);
    let west_x_edges: Vec<usize> = complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            (c.orientation().trailing_zeros() == 0 && c.position()[0] == 0).then_some(i)
        })
        .collect();
    let field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let err = m.leray_project_open_opts(
        &field,
        &[],
        &west_x_edges,
        &[],
        &HodgeDecomposeOptions::default(),
    );
    assert!(err.is_err(), "inflow without reference must be rejected");
}

/// Warm start (constrained path, no reference): seeding CG with the cold solve's potential yields
/// the same projection to tolerance. `None` and a mismatched-length guess both fall back to the
/// cold start and give the same answer.
#[test]
fn constrained_warm_start_matches_the_cold_projection() {
    let m = manifold_2d([6, 6], [false, false], CubicalReggeGeometry::unit());
    let edges = wall_tangential_edges(m.complex());
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 11);
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(10_000),
    };

    let cold = m
        .leray_project_constrained_opts(&field, &edges, &opts)
        .unwrap();
    let phi = cold.potential().as_slice().to_vec();

    let warm = m
        .leray_project_constrained_warm_opts(&field, &edges, &opts, Some(&phi))
        .unwrap();
    assert!(
        sup(cold
            .projected()
            .as_slice()
            .iter()
            .zip(warm.projected().as_slice())
            .map(|(a, b)| a - b))
            < 1e-9,
        "warm projection disagrees with cold"
    );

    let none = m
        .leray_project_constrained_warm_opts(&field, &edges, &opts, None)
        .unwrap();
    assert_eq!(none.projected().as_slice(), cold.projected().as_slice());

    let bad = vec![0.0_f64; 3]; // wrong length ⇒ ignored, falls back to a cold start.
    let warm_bad = m
        .leray_project_constrained_warm_opts(&field, &edges, &opts, Some(&bad))
        .unwrap();
    assert_eq!(warm_bad.projected().as_slice(), cold.projected().as_slice());
}

/// Warm start (open path with an outflow reference): seeding CG with the cold potential yields the
/// same uniform inflow/outflow solution, exercising the pinned/non-live masking of the guess.
#[test]
fn open_reference_warm_start_matches_the_cold_projection() {
    let shape = [5usize, 4usize];
    let m = manifold_2d(shape, [false, true], CubicalReggeGeometry::unit());
    let complex = m.complex();
    let n1 = complex.num_cells(1);
    let west_x_edges: Vec<usize> = complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            (c.orientation().trailing_zeros() == 0 && c.position()[0] == 0).then_some(i)
        })
        .collect();
    let east_vertices: Vec<usize> = complex
        .iter_cells(0)
        .enumerate()
        .filter_map(|(i, c)| (c.position()[0] == shape[0] - 1).then_some(i))
        .collect();
    let mut data = vec![0.0; n1];
    for &e in &west_x_edges {
        data[e] = 0.7;
    }
    let field = CausalTensor::new(data, vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(10_000),
    };

    let cold = m
        .leray_project_open_opts(&field, &[], &west_x_edges, &east_vertices, &opts)
        .unwrap();
    let phi = cold.potential().as_slice().to_vec();
    let warm = m
        .leray_project_open_warm_opts(
            &field,
            &[],
            &west_x_edges,
            &east_vertices,
            &opts,
            Some(&phi),
        )
        .unwrap();
    assert!(
        sup(cold
            .projected()
            .as_slice()
            .iter()
            .zip(warm.projected().as_slice())
            .map(|(a, b)| a - b))
            < 1e-9,
        "open warm projection disagrees with cold"
    );
}
