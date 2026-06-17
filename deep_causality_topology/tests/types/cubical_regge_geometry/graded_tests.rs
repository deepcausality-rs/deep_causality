/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the graded (variable-spacing) metric constructors — CFD rung R1.
//!
//! Covers: the geometric / tanh constructors build a correctly-sized `PerEdge` geometry
//! whose values match the analytic law; `cell_volume` dispatches the graded lengths
//! consistently; wall-normal-only grading leaves other axes uniform; and the headline
//! exactness gate — the Leray projection stays divergence-free under *strong* grading,
//! because the structure guarantees are combinatorial and metric-independent.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
};

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

// -- Group A: graded constructors -------------------------------------------------

#[test]
fn geometric_grading_produces_correct_per_edge_lengths() {
    let lattice = LatticeComplex::<3, f64>::cubic_torus(4);
    let ratio = [1.5, 1.0, 1.0];
    let geom = CubicalReggeGeometry::from_graded_geometric(&lattice, [1.0; 3], ratio);

    let lengths = geom
        .edge_lengths()
        .expect("a graded metric is a PerEdge geometry");
    assert_eq!(
        lengths.len(),
        lattice.iter_cells(1).count(),
        "PerEdge vector sized to the lattice's edge count"
    );

    // Every edge carries `base · ratio[axis]^pos`, laid out in iter_cells(1) order.
    for (i, cell) in lattice.iter_cells(1).enumerate() {
        let axis = cell.orientation().trailing_zeros() as usize;
        let pos = cell.position()[axis];
        let expected = ratio[axis].powi(pos as i32);
        assert!(
            (lengths[i] - expected).abs() < 1e-12,
            "edge {i} (axis {axis}, pos {pos}): {} vs expected {expected}",
            lengths[i]
        );
    }
}

#[test]
fn graded_cell_volume_equals_edge_length_product() {
    let lattice = LatticeComplex::<3, f64>::cubic_torus(4);
    let ratio = [1.5, 1.2, 1.0];
    let geom = CubicalReggeGeometry::from_graded_geometric(&lattice, [1.0; 3], ratio);

    // A top cube's volume is the product of its three spanning edge lengths.
    for cell in lattice.iter_cells(3) {
        let p = cell.position();
        let expected: f64 = (0..3).map(|a| ratio[a].powi(p[a] as i32)).product();
        let vol = geom.cell_volume(&lattice, &cell);
        assert!(
            (vol - expected).abs() < 1e-9 * expected.max(1.0),
            "cell at {p:?}: volume {vol} vs product {expected}"
        );
    }
}

#[test]
fn tanh_grading_clusters_the_graded_axis_and_leaves_others_uniform() {
    // Grade axis 1 (7 vertex layers, so 6 edges to resolve clustering) over a unit-length
    // domain; axes 0 and 2 are single-edge and stay uniform.
    let lattice = LatticeComplex::<3, f64>::open([2, 7, 2]);
    let geom = CubicalReggeGeometry::from_graded_tanh(&lattice, 1, 1.0, 3.0);
    let lengths = geom.edge_lengths().expect("PerEdge");

    let mut graded_axis_lengths = Vec::new();
    for (i, cell) in lattice.iter_cells(1).enumerate() {
        let axis = cell.orientation().trailing_zeros() as usize;
        assert!(lengths[i] > 0.0, "edge {i} has non-positive length");
        if axis == 1 {
            graded_axis_lengths.push(lengths[i]);
        } else {
            assert!(
                (lengths[i] - 1.0).abs() < 1e-12,
                "ungraded axis {axis} edge {i} not unit: {}",
                lengths[i]
            );
        }
    }

    // Two-sided clustering: the graded axis is not uniform.
    let min = graded_axis_lengths
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let max = graded_axis_lengths.iter().cloned().fold(0.0_f64, f64::max);
    assert!(
        max > min * 1.2,
        "tanh grading did not cluster (min {min}, max {max})"
    );
}

// -- Group A: headline exactness gate ---------------------------------------------

#[test]
fn leray_projection_stays_divergence_free_under_strong_grading() {
    // A strongly graded metric (growth ratio well outside the accuracy-good range):
    // structure must hold regardless, because d and the discrete Stokes theorem are
    // combinatorial and never see the metric.
    let lattice = LatticeComplex::<3, f64>::cubic_torus(6);
    let metric = CubicalReggeGeometry::from_graded_geometric(&lattice, [1.0; 3], [1.4, 1.0, 1.0]);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 7);
    let projection = m
        .leray_project_opts(&field, &HodgeDecomposeOptions::default())
        .unwrap();
    let u = projection.projected().as_slice();

    let div = sup(m.codifferential_of(u, 1).into_vec());
    assert!(
        div < 1e-9,
        "graded-metric Leray projection divergence {div:e} above solve exactness"
    );
}
