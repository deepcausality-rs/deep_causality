/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the wall-bounded-ns capability: solver acceptance of
//! mixed-periodicity and all-walls manifolds, the typed construction
//! rejections (degenerate wall extents, non-positive wall masses), and
//! the march invariants (divergence at the solve's exactness, no-slip
//! exact at every step boundary).

use deep_causality_physics::DecNsSolver;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

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

/// The acceptance scenario: a periodic-x/wall-y solver constructs,
/// seeds, and marches; every step output is divergence-free at the
/// solve's exactness and exactly zero on the wall-tangential set.
#[test]
fn wall_bounded_solver_constructs_and_marches() {
    let m = manifold_2d([8, 6], [true, false], CubicalReggeGeometry::unit());
    let solver = DecNsSolver::new(&m, 0.05, 0.05, None).unwrap();

    let n0 = m.complex().num_cells(0);
    let mut vertex = vec![0.0; 2 * n0];
    for (vi, v) in m.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = (0.7 * x).sin() + 0.3 * y;
        vertex[2 * vi + 1] = (0.5 * y).cos() * 0.2;
    }
    let seed = CausalTensor::new(vertex, vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&seed).unwrap();

    let shape = m.complex().shape();
    for _ in 0..3 {
        let out = solver.step(&state).unwrap();
        assert!(
            out.divergence_residual() < 1e-9,
            "divergence residual {} above solve exactness",
            out.divergence_residual()
        );
        state = out.into_state();
        let u = state.as_one_form().as_slice();
        for (idx, cell) in m.complex().iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            let y = cell.position()[1];
            if axis == 0 && (y == 0 || y + 1 == shape[1]) {
                assert_eq!(u[idx], 0.0, "wall-tangential edge {idx} nonzero");
            }
        }
    }
}

/// A wall axis with a single vertex layer has no interior to march: the
/// construction is rejected with a typed error naming the requirement.
#[test]
fn degenerate_wall_extent_is_rejected() {
    let m = manifold_2d([8, 1], [true, false], CubicalReggeGeometry::unit());
    let err = DecNsSolver::new(&m, 0.05, 0.05, None).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("at least 2 vertex layers"),
        "unexpected error: {msg}"
    );
}

/// A walled lattice whose metric vends a non-positive (here infinite,
/// from a zero edge length) grade-1 mass cannot carry the corrected-star
/// solve: rejected with a typed error naming the boundary-corrected star.
#[test]
fn non_positive_wall_masses_are_rejected() {
    let lattice = LatticeComplex::<2, f64>::new([4, 3], [true, false]);
    let n1 = lattice.num_cells(1);
    let mut lengths = vec![1.0; n1];
    lengths[0] = 0.0;
    let m = manifold_2d(
        [4, 3],
        [true, false],
        CubicalReggeGeometry::from_edge_lengths(lengths),
    );
    let err = DecNsSolver::new(&m, 0.05, 0.05, None).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("boundary-corrected"),
        "unexpected error: {msg}"
    );
}

/// Fully periodic lattices skip the wall validation entirely — the
/// periodic construction path is unchanged (its behavioral regression
/// gate is the whole pre-existing periodic suite).
#[test]
fn periodic_construction_skips_wall_validation() {
    let m = manifold_2d([6, 6], [true, true], CubicalReggeGeometry::unit());
    assert!(DecNsSolver::new(&m, 0.05, 0.05, None).is_ok());
}
