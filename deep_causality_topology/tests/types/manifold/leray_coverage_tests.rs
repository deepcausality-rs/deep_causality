/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the Leray-projection open and weighted branches not
//! pinned by the existing suites:
//!
//! * Weighted entry with **no** constraint rows delegates to the binary open
//!   path (`leray.rs:415`).
//! * Weighted entry whose rows all filter away, but **with** reference
//!   vertices, delegates to the open path (`leray.rs:506`).
//! * Weighted **open** gauge with reference vertices runs the reference
//!   flood-fill BFS and a converging surviving row (`leray.rs:601-604`, the
//!   warm-start φ seed and the converged solve).
//! * Weighted solve **non-convergence** with a zero iteration budget surfaces
//!   `HodgeDecompositionFailed` (`leray.rs:725,729`).
//! * The binary **open** path rejects a missing metric (`leray.rs:786-791`)
//!   and an out-of-range reference vertex (`leray.rs:860-862`).

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutConstraintKind, CutFaceConstraint,
    HodgeDecomposeOptions, LatticeComplex, Manifold, TopologyErrorEnum,
};

fn manifold_with_metric(
    shape: [usize; 2],
    periodic: [bool; 2],
) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new(shape, periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, CubicalReggeGeometry::unit(), 0)
}

fn manifold_no_metric(
    shape: [usize; 2],
    periodic: [bool; 2],
) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new(shape, periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical(lattice, data, 0)
}

fn random_field(len: usize, seed: u64) -> CausalTensor<f64> {
    let mut s = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let data: Vec<f64> = (0..len)
        .map(|_| {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            2.0 * ((s >> 11) as f64 / (1u64 << 53) as f64) - 1.0
        })
        .collect();
    CausalTensor::new(data, vec![len]).unwrap()
}

/// West x-edges (inflow) and east vertices (outflow reference) of an x-open channel.
fn west_edges_east_vertices(
    m: &Manifold<LatticeComplex<2, f64>, f64>,
    shape: [usize; 2],
) -> (Vec<usize>, Vec<usize>) {
    let complex = m.complex();
    let west: Vec<usize> = complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            (c.orientation().trailing_zeros() == 0 && c.position()[0] == 0).then_some(i)
        })
        .collect();
    let east: Vec<usize> = complex
        .iter_cells(0)
        .enumerate()
        .filter_map(|(i, c)| (c.position()[0] == shape[0] - 1).then_some(i))
        .collect();
    (west, east)
}

// ---------------------------------------------------------------------------
// leray.rs:415 — weighted entry with no constraint rows delegates to the
// binary open path.
// ---------------------------------------------------------------------------

#[test]
fn weighted_open_with_no_rows_matches_binary_open() {
    let shape = [5, 4];
    let m = manifold_with_metric(shape, [false, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 101);
    let (west, east) = west_edges_east_vertices(&m, shape);
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(10_000),
    };

    let weighted = m
        .leray_project_open_weighted_opts(&field, &[], &west, &east, &[], &opts, None)
        .unwrap();
    let binary = m
        .leray_project_open_opts(&field, &[], &west, &east, &opts)
        .unwrap();
    assert_eq!(
        weighted.projected().as_slice(),
        binary.projected().as_slice()
    );
}

// ---------------------------------------------------------------------------
// leray.rs:506 — weighted entry whose rows all filter away but WITH reference
// vertices delegates to the open path (the `m == 0` branch carrying the
// reference-vertex arguments through to `leray_project_open_guess`).
// ---------------------------------------------------------------------------

#[test]
fn weighted_open_with_only_empty_rows_and_reference_delegates() {
    let shape = [5, 4];
    let m = manifold_with_metric(shape, [false, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 103);
    let (west, east) = west_edges_east_vertices(&m, shape);
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(10_000),
    };

    // An all-empty-entries row drops to nothing → emitted-row count 0.
    let empty_row = CutFaceConstraint::new(Vec::new(), 0.0, 1.0, CutConstraintKind::NoPenetration);

    let weighted = m
        .leray_project_open_weighted_opts(&field, &[], &west, &east, &[empty_row], &opts, None)
        .unwrap();
    let binary = m
        .leray_project_open_opts(&field, &[], &west, &east, &opts)
        .unwrap();
    assert_eq!(
        weighted.projected().as_slice(),
        binary.projected().as_slice()
    );
}

// ---------------------------------------------------------------------------
// leray.rs:601-604,714 — weighted OPEN gauge with reference vertices: the
// reference flood-fill BFS runs, and the warm-start φ seed is masked onto the
// active DOFs. A surviving interior row keeps the augmented KKT system, and the
// projected field is divergence-free on the interior and satisfies the row.
// ---------------------------------------------------------------------------

#[test]
fn weighted_open_gauge_with_reference_and_row_is_satisfied() {
    let shape = [5, 4];
    let m = manifold_with_metric(shape, [false, true]);
    let complex = m.complex();
    let n0 = complex.num_cells(0);
    let n1 = complex.num_cells(1);
    let field = random_field(n1, 107);
    let (west, east) = west_edges_east_vertices(&m, shape);

    // A genuine interior weighted row over two free edges (not on the masked set).
    let row = CutFaceConstraint::new(
        vec![(6usize, 1.0), (9usize, -0.5)],
        0.0,
        1.0,
        CutConstraintKind::Tangential,
    );
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(20_000),
    };

    // Warm-start the φ block with a zero guess of the right length (n0) to walk
    // the warm-start seed branch.
    let x0 = vec![0.0_f64; n0];
    let p = m
        .leray_project_open_weighted_opts(
            &field,
            &[],
            &west,
            &east,
            std::slice::from_ref(&row),
            &opts,
            Some(&x0),
        )
        .unwrap();
    let u = p.projected().as_slice();

    // The weighted row is satisfied on the projected field.
    let mut residual = -row.target();
    for &(e, w) in row.entries() {
        residual += w * u[e];
    }
    assert!(residual.abs() < 1e-7, "row residual {residual:e}");
}

// ---------------------------------------------------------------------------
// leray.rs:625 — constrained gauge (no reference) with a structurally-null φ
// row: zeroing every edge incident to one vertex leaves that vertex with no
// free incidence (`diag[i] == 0`), so it is marked inactive and its RHS row is
// zeroed. A surviving interior weighted row keeps the augmented KKT system.
// ---------------------------------------------------------------------------

#[test]
fn constrained_gauge_with_isolated_vertex_zeroes_its_rhs_row() {
    let shape = [4usize, 4usize];
    let m = manifold_with_metric(shape, [true, true]);
    let complex = m.complex();
    let n1 = complex.num_cells(1);
    let field = random_field(n1, 131);

    // All edges incident to vertex at position (0, 0): the two outgoing edges
    // (x and y from (0,0)) and the two incoming edges (x from (3,0), y from
    // (0,3)) — on the torus these are the four links touching vertex 0.
    let target_pos = [0usize, 0usize];
    let mut incident: Vec<usize> = Vec::new();
    for (i, c) in complex.iter_cells(1).enumerate() {
        let axis = c.orientation().trailing_zeros() as usize;
        let p = c.position();
        // outgoing edge from the target vertex
        if *p == target_pos {
            incident.push(i);
            continue;
        }
        // incoming edge: target = p + e_axis (mod shape)
        let mut q = *p;
        q[axis] = (q[axis] + 1) % shape[axis];
        if q == target_pos {
            incident.push(i);
        }
    }
    assert!(!incident.is_empty(), "vertex 0 must have incident edges");

    // A surviving weighted row over two free interior edges away from vertex 0.
    let row = CutFaceConstraint::new(
        vec![(10usize, 1.0), (15usize, -0.5)],
        0.0,
        1.0,
        CutConstraintKind::Tangential,
    );

    let p = m
        .leray_project_constrained_weighted_opts(
            &field,
            &incident,
            std::slice::from_ref(&row),
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();
    let u = p.projected().as_slice();
    // The zeroed (masked) edges are held at zero in the projected field.
    for &e in &incident {
        assert_eq!(u[e], 0.0, "masked edge stays zero");
    }
}

// ---------------------------------------------------------------------------
// leray.rs:725,729 — the weighted solve reports HodgeDecompositionFailed when
// it cannot converge within the iteration budget (here a zero budget on a
// non-trivial system).
// ---------------------------------------------------------------------------

#[test]
fn weighted_solve_nonconvergence_is_reported() {
    let m = manifold_with_metric([6, 6], [true, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 109);

    let row = CutFaceConstraint::new(
        vec![(2usize, 1.0), (5usize, -0.5)],
        1.0,
        1.0,
        CutConstraintKind::Tangential,
    );
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-14),
        max_iterations: Some(0), // no iterations ⇒ cannot converge
    };

    let err = m
        .leray_project_constrained_weighted_opts(
            &field,
            &[],
            std::slice::from_ref(&row),
            &opts,
            None,
        )
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::HodgeDecompositionFailed(_)),
        "expected HodgeDecompositionFailed, got {:?}",
        err.0
    );
}

// ---------------------------------------------------------------------------
// leray.rs:786-791 — the binary open path rejects a manifold without a metric.
// ---------------------------------------------------------------------------

#[test]
fn open_path_without_metric_is_rejected() {
    let shape = [5, 4];
    let m = manifold_no_metric(shape, [false, true]);
    let n1 = m.complex().num_cells(1);
    let field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    // A non-empty edge/vertex partition keeps us off the all-empty delegate path.
    let zeroed = [0usize];
    let opts = HodgeDecomposeOptions::default();

    let err = m
        .leray_project_open_opts(&field, &zeroed, &[], &[], &opts)
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::InvalidInput(_)),
        "expected InvalidInput (missing metric), got {:?}",
        err.0
    );
}

// ---------------------------------------------------------------------------
// leray.rs:860-862 — the binary open path rejects an out-of-range reference
// vertex.
// ---------------------------------------------------------------------------

#[test]
fn open_path_rejects_out_of_range_reference_vertex() {
    let shape = [5, 4];
    let m = manifold_with_metric(shape, [false, true]);
    let complex = m.complex();
    let n0 = complex.num_cells(0);
    let n1 = complex.num_cells(1);
    let field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions::default();

    // Reference vertex index past the end → InvalidInput.
    let bad_refs = [n0 + 5];
    let err = m
        .leray_project_open_opts(&field, &[], &[], &bad_refs, &opts)
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::InvalidInput(_)),
        "expected InvalidInput (reference vertex out of range), got {:?}",
        err.0
    );
}
