/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the interior branches of `leray_project_open_weighted_guess`
//! reached by synthetic `CutFaceConstraint` rows (built directly, bypassing the
//! cut-cell registry): a row whose edge index is out of range, a row whose
//! entries all reference fixed (masked) edges (so the row degenerates to nothing
//! and the call falls back to the binary open/constrained path), and the
//! every-edge-constrained abort of the constrained gauge.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutConstraintKind, CutFaceConstraint,
    HodgeDecomposeOptions, LatticeComplex, Manifold, TopologyErrorEnum,
};

fn manifold_2d(shape: [usize; 2], periodic: [bool; 2]) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new(shape, periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, CubicalReggeGeometry::unit(), 0)
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

/// A non-empty weighted row whose single entry edge index is out of range must
/// be rejected (the per-entry bound check inside the row-normalisation loop).
#[test]
fn weighted_row_with_out_of_range_edge_is_rejected() {
    let m = manifold_2d([6, 6], [true, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 11);

    let bad_row = CutFaceConstraint::new(
        vec![(n1 + 3, 1.0)], // edge index past the end
        0.0,
        1.0,
        CutConstraintKind::NoPenetration,
    );

    let err = m
        .leray_project_constrained_weighted_opts(
            &field,
            &[],
            &[bad_row],
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::InvalidInput(_)));
}

/// A weighted row whose only entries reference fixed (zeroed) edges drops every
/// entry during normalisation, leaving the emitted-row count at zero — the call
/// must then delegate to the binary constrained path and still succeed.
#[test]
fn weighted_row_over_only_fixed_edges_degenerates_to_binary_path() {
    let m = manifold_2d([6, 6], [true, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 13);

    // Zero edges 0 and 1; build a row that only touches those edges.
    let zeroed = [0usize, 1usize];
    let row = CutFaceConstraint::new(
        vec![(0usize, 1.0), (1usize, 1.0)],
        0.0,
        1.0,
        CutConstraintKind::Tangential,
    );

    let weighted = m
        .leray_project_constrained_weighted_opts(
            &field,
            &zeroed,
            &[row],
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();

    // The reference: the binary constrained path with the same zeroed set. The
    // degenerate weighted call must reproduce it bit-for-bit.
    let binary = m
        .leray_project_constrained_opts(&field, &zeroed, &HodgeDecomposeOptions::default())
        .unwrap();
    assert_eq!(
        weighted.projected().as_slice(),
        binary.projected().as_slice()
    );
}

/// An empty-entries weighted row is skipped (the `entries.is_empty()` continue),
/// again degenerating to the binary path.
#[test]
fn empty_entries_weighted_row_is_skipped() {
    let m = manifold_2d([6, 6], [true, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 17);

    let empty_row = CutFaceConstraint::new(Vec::new(), 0.0, 1.0, CutConstraintKind::NoPenetration);

    let weighted = m
        .leray_project_constrained_weighted_opts(
            &field,
            &[],
            &[empty_row],
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();
    let binary = m
        .leray_project_constrained_opts(&field, &[], &HodgeDecomposeOptions::default())
        .unwrap();
    assert_eq!(
        weighted.projected().as_slice(),
        binary.projected().as_slice()
    );
}

/// Constrained gauge (no reference vertices) with a surviving weighted row: this
/// drives the augmented-KKT branch through the constrained-gauge RHS path (the
/// block-mean subtraction over active φ rows and the divergence-free invariant)
/// rather than the open-gauge branch.
#[test]
fn constrained_gauge_weighted_row_is_divergence_free_and_satisfied() {
    let m = manifold_2d([6, 6], [true, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 23);

    // A genuine weighted row over two free interior edges (none zeroed).
    let row = CutFaceConstraint::new(
        vec![(2usize, 1.0), (5usize, -0.5)],
        0.0,
        1.0,
        CutConstraintKind::Tangential,
    );

    let p = m
        .leray_project_constrained_weighted_opts(
            &field,
            &[],
            std::slice::from_ref(&row),
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();
    let u = p.projected().as_slice();

    // The row is satisfied on the projected state.
    let mut residual = -row.target();
    for &(e, w) in row.entries() {
        residual += w * u[e];
    }
    assert!(residual.abs() < 1e-9, "row residual {residual:e}");

    // And the field is divergence-free to the solve's exactness.
    let div = m
        .codifferential_of(u, 1)
        .into_vec()
        .into_iter()
        .fold(0.0_f64, |acc, x| acc.max(x.abs()));
    assert!(div < 1e-8, "divergence {div:e}");
}
