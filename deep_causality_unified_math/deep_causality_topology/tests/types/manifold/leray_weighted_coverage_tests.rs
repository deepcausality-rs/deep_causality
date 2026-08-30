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

/// With no weighted rows at all the weighted entry point is a thin wrapper over
/// the binary open projection, so a failure of the inner call is the failure the
/// caller sees — here a field that is not a grade-1 form.
#[test]
fn weighted_open_projection_without_rows_propagates_the_binary_failure() {
    let m = manifold_2d([5, 5], [false, false]);
    let n1 = m.complex().num_cells(1);
    let short = random_field(n1 - 1, 29);

    let err = m
        .leray_project_open_weighted_opts(
            &short,
            &[0usize],
            &[],
            &[],
            &[],
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::DimensionMismatch(_)));
}

/// When every weighted row degenerates the call falls back to the binary open
/// path, and that path's own validation — here an out-of-range reference vertex,
/// which the weighted preamble never inspects — still reaches the caller.
#[test]
fn degenerate_rows_then_binary_path_reports_an_out_of_range_reference_vertex() {
    let m = manifold_2d([5, 5], [false, false]);
    let n0 = m.complex().num_cells(0);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 31);

    // The row's only entry is on a zeroed edge, so it drops during normalisation.
    let zeroed = [0usize];
    let row = CutFaceConstraint::new(
        vec![(0usize, 1.0)],
        0.0,
        1.0,
        CutConstraintKind::NoPenetration,
    );

    let err = m
        .leray_project_open_weighted_opts(
            &field,
            &zeroed,
            &[],
            &[n0 + 5],
            &[row],
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap_err();
    match err.0 {
        TopologyErrorEnum::InvalidInput(msg) => {
            assert!(
                msg.contains("reference vertex"),
                "unexpected message: {msg}"
            );
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

/// Warm-starting only the multiplier block leaves the potential block seeded at
/// zero. The converged projection is the same either way, since the guess only
/// changes where the iteration starts.
#[test]
fn lambda_only_warm_start_reaches_the_same_projection() {
    let m = manifold_2d([6, 6], [true, true]);
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 37);

    let row = CutFaceConstraint::new(
        vec![(2usize, 1.0), (5usize, -0.5)],
        0.0,
        1.0,
        CutConstraintKind::Tangential,
    );

    let (cold, lambda) = m
        .leray_project_constrained_weighted_warm(
            &field,
            &[],
            std::slice::from_ref(&row),
            &HodgeDecomposeOptions::default(),
            None,
            None,
        )
        .unwrap();
    assert_eq!(lambda.len(), 1, "one row survives normalisation");

    let (warm, _) = m
        .leray_project_constrained_weighted_warm(
            &field,
            &[],
            std::slice::from_ref(&row),
            &HodgeDecomposeOptions::default(),
            None,
            Some(&lambda),
        )
        .unwrap();

    for (a, b) in cold
        .projected()
        .as_slice()
        .iter()
        .zip(warm.projected().as_slice().iter())
    {
        assert!((a - b).abs() < 1e-9, "cold {a} vs lambda-warm {b}");
    }
}

/// A four-cycle whose grade-1 masses alternate in sign so every vertex's free
/// mass sums to zero. No potential degree of freedom is then active, and the
/// constrained gauge has no block to subtract a mean over, so the projection is
/// rejected instead of dividing by an empty block.
fn four_cycle_with_cancelling_edge_masses() -> deep_causality_topology::SimplicialManifold<f64, f64>
{
    use deep_causality_linear::CsrMatrix;
    use deep_causality_topology::{ReggeGeometry, Simplex, SimplicialComplex, Skeleton};

    let sk0 = Skeleton::new(0, (0..4).map(|i| Simplex::new(vec![i])).collect());
    // Sorted: [0,1], [0,3], [1,2], [2,3] — the cycle 0-1-2-3-0.
    let sk1 = Skeleton::new(
        1,
        vec![
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![0, 3]),
            Simplex::new(vec![1, 2]),
            Simplex::new(vec![2, 3]),
        ],
    );
    // Oriented consistently around the cycle, so every vertex row sums to zero.
    let d1 = CsrMatrix::from_triplets(
        4,
        4,
        &[
            (0, 0, -1i8),
            (1, 0, 1),
            (0, 1, 1),
            (3, 1, -1),
            (1, 2, -1),
            (2, 2, 1),
            (2, 3, -1),
            (3, 3, 1),
        ],
    )
    .unwrap();
    let cob = vec![d1.transpose()];

    let star_zero = CsrMatrix::from_triplets(
        4,
        4,
        &[(0, 0, 1.0f64), (1, 1, 1.0), (2, 2, 1.0), (3, 3, 1.0)],
    )
    .unwrap();
    // Alternating signs around the cycle: each vertex sees +1 and -1.
    let star_one = CsrMatrix::from_triplets(
        4,
        4,
        &[(0, 0, 1.0f64), (1, 1, -1.0), (2, 2, -1.0), (3, 3, 1.0)],
    )
    .unwrap();

    let complex = SimplicialComplex::new(vec![sk0, sk1], vec![d1], cob, vec![star_zero, star_one]);
    let regge = ReggeGeometry::new(CausalTensor::new(vec![1.0f64; 4], vec![4]).unwrap());
    let data = CausalTensor::new(vec![0.0f64; 8], vec![8]).unwrap();
    Manifold::with_metric(complex, data, Some(regge), 0).unwrap()
}

#[test]
fn constrained_gauge_with_no_active_potential_row_is_rejected() {
    let m = four_cycle_with_cancelling_edge_masses();
    let field = CausalTensor::new(vec![1.0f64, 2.0, 3.0, 4.0], vec![4]).unwrap();

    let row = CutFaceConstraint::new(
        vec![(0usize, 1.0)],
        0.0,
        1.0,
        CutConstraintKind::NoPenetration,
    );

    let err = m
        .leray_project_constrained_weighted_opts(
            &field,
            &[],
            &[row],
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap_err();
    match err.0 {
        TopologyErrorEnum::InvalidInput(msg) => {
            assert!(
                msg.contains("every edge is constrained"),
                "unexpected message: {msg}"
            );
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}
