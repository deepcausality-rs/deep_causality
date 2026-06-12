/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the constrained Leray projection (the no-slip-viscous
//! capability's projector): the M-orthogonal projection onto the
//! intersection of the divergence-free subspace with `{u : u|_E = 0}`.
//! Pinned here: both invariants hold simultaneously (constrained edges
//! exactly zero, full divergence at the solve's exactness — including
//! all-walls boxes whose corner vertices are structurally null rows),
//! idempotence, M-orthogonality of the removed component, the empty-set
//! delegation, and the typed rejections.

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HasHodgeStar, HodgeDecomposeOptions, LatticeComplex,
    Manifold,
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

/// Wall-tangential edges via the public lattice API (the no-slip set).
fn wall_tangential_edges<const D: usize>(complex: &LatticeComplex<D, f64>) -> Vec<usize> {
    let periodic = complex.periodic();
    let shape = complex.shape();
    complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            let axis = c.orientation().trailing_zeros() as usize;
            let pos = c.position();
            (0..D)
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

fn star_diag_1(m: &Manifold<LatticeComplex<2, f64>, f64>) -> Vec<f64> {
    let metric_binding = m.metric();
    let metric = metric_binding.as_ref().unwrap();
    let star: std::borrow::Cow<'_, CsrMatrix<f64>> =
        metric.hodge_star_matrix(m.complex(), 1).unwrap();
    let n1 = m.complex().num_cells(1);
    let mut diag = vec![0.0; n1];
    for (i, d) in diag.iter_mut().enumerate() {
        for e in star.row_indices()[i]..star.row_indices()[i + 1] {
            if star.col_indices()[e] == i {
                *d = star.values()[e];
            }
        }
    }
    diag
}

/// Both invariants at once on a mixed-periodicity channel: constrained
/// edges exactly zero, full divergence at the solve's exactness.
#[test]
fn constrained_projection_holds_both_invariants_on_a_channel() {
    let m = manifold_2d([8, 6], [true, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let edges = wall_tangential_edges(m.complex());
    let field = random_field(n1, 11);

    let p = m
        .leray_project_constrained_opts(&field, &edges, &HodgeDecomposeOptions::default())
        .unwrap();
    let u = p.projected().as_slice();

    for &e in &edges {
        assert_eq!(u[e], 0.0, "constrained edge {e} nonzero");
    }
    let div = sup(m.codifferential_of(u, 1).into_vec());
    assert!(div < 1e-9, "full divergence {div:e} above solve exactness");
}

/// All-walls box: the corner vertices lose every incident edge to the
/// constraint (structurally null rows) and the solve still produces both
/// invariants.
#[test]
fn all_walls_box_with_null_corner_rows_solves() {
    let m = manifold_2d(
        [7, 6],
        [false, false],
        CubicalReggeGeometry::per_axis([0.5, 0.25]),
    );
    let n1 = m.complex().num_cells(1);
    let edges = wall_tangential_edges(m.complex());
    let field = random_field(n1, 13);

    // Tight tolerance: the anisotropic clipped masses amplify the relative
    // CG residual through `M₀⁻¹` in the codifferential.
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-13),
        max_iterations: Some(10_000),
    };
    let p = m
        .leray_project_constrained_opts(&field, &edges, &opts)
        .unwrap();
    let u = p.projected().as_slice();
    for &e in &edges {
        assert_eq!(u[e], 0.0);
    }
    let div = sup(m.codifferential_of(u, 1).into_vec());
    assert!(div < 1e-9, "divergence {div:e}");
}

/// Idempotence: projecting the projection changes nothing beyond solve
/// tolerance (the result is already in the intersection subspace).
#[test]
fn constrained_projection_is_idempotent() {
    let m = manifold_2d([8, 6], [true, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let edges = wall_tangential_edges(m.complex());
    let field = random_field(n1, 17);
    let opts = HodgeDecomposeOptions::default();

    let once = m
        .leray_project_constrained_opts(&field, &edges, &opts)
        .unwrap();
    let twice = m
        .leray_project_constrained_opts(once.projected(), &edges, &opts)
        .unwrap();

    let gap = sup(once
        .projected()
        .as_slice()
        .iter()
        .zip(twice.projected().as_slice().iter())
        .map(|(a, b)| a - b));
    assert!(gap < 1e-9, "idempotence gap {gap:e}");
}

/// M-orthogonality: the removed component `v − u` is M-orthogonal to the
/// intersection subspace (witnessed by constrained projections of other
/// random fields) — the projection is the *orthogonal* one.
#[test]
fn removed_component_is_m_orthogonal_to_the_intersection() {
    let m = manifold_2d([8, 6], [true, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let edges = wall_tangential_edges(m.complex());
    let opts = HodgeDecomposeOptions::default();
    let mass = star_diag_1(&m);

    let field = random_field(n1, 19);
    let p = m
        .leray_project_constrained_opts(&field, &edges, &opts)
        .unwrap();
    // v = P_S(field): the projector acts on the constrained input; the
    // discarded part within the subspace is v − u.
    let mut v = field.as_slice().to_vec();
    for &e in &edges {
        v[e] = 0.0;
    }
    let removed: Vec<f64> = v
        .iter()
        .zip(p.projected().as_slice().iter())
        .map(|(a, b)| a - b)
        .collect();

    for seed in [23u64, 29, 31] {
        let witness = m
            .leray_project_constrained_opts(&random_field(n1, seed), &edges, &opts)
            .unwrap();
        let inner: f64 = removed
            .iter()
            .zip(witness.projected().as_slice().iter())
            .zip(mass.iter())
            .map(|((r, w), mw)| r * w * mw)
            .sum();
        assert!(
            inner.abs() < 1e-8,
            "seed {seed}: M-inner product {inner:e} not at rounding"
        );
    }
}

/// The empty edge set delegates to the plain projection bit-identically.
#[test]
fn empty_constraint_set_is_the_plain_projection() {
    let m = manifold_2d([8, 8], [true, true], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let field = random_field(n1, 37);
    let opts = HodgeDecomposeOptions::default();

    let constrained = m
        .leray_project_constrained_opts(&field, &[], &opts)
        .unwrap();
    let plain = m.leray_project_opts(&field, &opts).unwrap();
    assert_eq!(
        constrained.projected().as_slice(),
        plain.projected().as_slice()
    );
    assert_eq!(
        constrained.potential().as_slice(),
        plain.potential().as_slice()
    );
}

/// Typed rejections: out-of-range edge index, field-length mismatch, and
/// the fully-constrained degenerate case.
#[test]
fn constrained_projection_rejects_invalid_inputs() {
    let m = manifold_2d([6, 5], [true, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let opts = HodgeDecomposeOptions::default();
    let field = random_field(n1, 41);

    // Edge index out of range.
    let err = m
        .leray_project_constrained_opts(&field, &[n1 + 3], &opts)
        .unwrap_err();
    assert!(format!("{err}").contains("out of range"));

    // Field length mismatch.
    let short = random_field(n1 - 1, 43);
    let err = m
        .leray_project_constrained_opts(&short, &[0], &opts)
        .unwrap_err();
    assert!(format!("{err}").contains("grade-1 coefficients"));

    // Every edge constrained: no free block remains.
    let all: Vec<usize> = (0..n1).collect();
    let err = m
        .leray_project_constrained_opts(&field, &all, &opts)
        .unwrap_err();
    assert!(format!("{err}").contains("every edge is constrained"));
}

/// A starved iteration budget surfaces the typed non-convergence error.
#[test]
fn starved_budget_reports_nonconvergence() {
    let m = manifold_2d([8, 6], [true, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let edges = wall_tangential_edges(m.complex());
    let field = random_field(n1, 47);
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-14),
        max_iterations: Some(1),
    };
    let err = m
        .leray_project_constrained_opts(&field, &edges, &opts)
        .unwrap_err();
    assert!(format!("{err}").contains("did not converge"));
}
