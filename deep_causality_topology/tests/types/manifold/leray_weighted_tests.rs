/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the **generalized constrained** Leray projection
//! (`leray_project_constrained_weighted_opts`, `add-aperture-resolved-noslip` Group B): the
//! M-orthogonal projection onto `{divergence-free} ∩ {Cᵀu = b}` for aperture-weighted wall rows.
//!
//! The headline is the **single-cut-cell formulation gate** (design Decision 2, Phase 1): build one
//! cut cell of known geometry, derive its rows with `cut_face_constraints`, project an arbitrary
//! field, and assert the reconstructed fragment velocity is zero to tolerance — the cheap validator
//! the axis-aligned reduction (which has no cut cells) cannot provide. Plus: divergence-free,
//! binary-equivalence (empty rows ≡ the staircase path, bit-identical), warm-start agreement, and
//! the no-penetration on/off ablation.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, CutConstraintKind,
    CutFaceConstraint, CutFaceFragment, HodgeDecomposeOptions, LatticeCell, LatticeComplex,
    Manifold, SourceGeometry,
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

/// A registry with a single `Cut` cell at `base` carrying a `+y` half-space wall and asymmetric
/// apertures (so the row weights are non-trivial).
fn single_cut_cell(complex: &LatticeComplex<2, f64>, base: [usize; 2]) -> CutCellRegistry<2, f64> {
    let cell = LatticeCell::<2>::new(base, 0b11);
    let idx = complex.cells(2).position(|c| c == cell).unwrap();
    let fragment =
        CutFaceFragment::<2, f64>::new(1.0, [0.0, 1.0], [0.0, 0.0], SourceGeometry::Plane);
    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.4, [[0.5, 0.5], [0.3, 1.0]], vec![fragment]),
    );
    reg
}

/// The residual `Σ wₑ uₑ − target` of one constraint row evaluated on a projected field.
fn row_residual(row: &CutFaceConstraint<f64>, u: &[f64]) -> f64 {
    let mut s = 0.0;
    for &(e, w) in row.entries() {
        s += w * u[e];
    }
    s - row.target()
}

// -- the single-cut-cell formulation gate (design Decision 2, Phase 1) ---------------------------

#[test]
fn single_cut_cell_drives_fragment_velocity_to_zero() {
    let m = manifold_2d([6, 6], [true, true], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let reg = single_cut_cell(m.complex(), [2, 2]);
    let rows = reg.cut_face_constraints(m.complex());
    assert_eq!(
        rows.len(),
        2,
        "a 2D cut cell yields 1 no-pen + 1 tangential row"
    );

    let field = random_field(n1, 7);
    let p = m
        .leray_project_constrained_weighted_opts(
            &field,
            &[],
            &rows,
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();
    let u = p.projected().as_slice();

    // The formulation gate: every wall-frame component of the fragment velocity is zero.
    for row in &rows {
        let r = row_residual(row, u);
        assert!(
            r.abs() < 1e-9,
            "fragment velocity residual {r:e} (kind {:?}) above tolerance",
            row.kind()
        );
    }
    // And the field stays divergence-free.
    let div = sup(m.codifferential_of(u, 1).into_vec());
    assert!(
        div < 1e-9,
        "interior divergence {div:e} above solve exactness"
    );
}

#[test]
fn weighted_projection_is_divergence_free_with_a_solid_pin() {
    // Mix a binary pin (zeroed edge) with the weighted rows: both invariants must hold.
    let m = manifold_2d(
        [8, 6],
        [true, false],
        CubicalReggeGeometry::per_axis([0.7, 1.1]),
    );
    let n1 = m.complex().num_cells(1);
    let reg = single_cut_cell(m.complex(), [3, 2]);
    let rows = reg.cut_face_constraints(m.complex());
    let zeroed = vec![0usize, 5usize];

    let field = random_field(n1, 23);
    let p = m
        .leray_project_constrained_weighted_opts(
            &field,
            &zeroed,
            &rows,
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();
    let u = p.projected().as_slice();

    for &e in &zeroed {
        assert_eq!(u[e], 0.0, "binary-pinned edge {e} must stay zero");
    }
    for row in &rows {
        assert!(
            row_residual(row, u).abs() < 1e-9,
            "weighted row not satisfied"
        );
    }
    let div = sup(m.codifferential_of(u, 1).into_vec());
    assert!(
        div < 1e-9,
        "interior divergence {div:e} above solve exactness"
    );
}

// -- binary equivalence: empty rows are bit-identical to the staircase path ----------------------

#[test]
fn empty_rows_are_bit_identical_to_the_constrained_path() {
    let m = manifold_2d([8, 6], [true, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let zeroed = vec![1usize, 4, 9, 12];
    let field = random_field(n1, 31);
    let opts = HodgeDecomposeOptions::default();

    let binary = m
        .leray_project_constrained_opts(&field, &zeroed, &opts)
        .unwrap();
    let weighted_empty: &[CutFaceConstraint<f64>] = &[];
    let via_weighted = m
        .leray_project_constrained_weighted_opts(&field, &zeroed, weighted_empty, &opts, None)
        .unwrap();

    assert_eq!(
        binary.projected().as_slice(),
        via_weighted.projected().as_slice(),
        "empty weighted rows must reproduce the binary staircase projection bit-for-bit"
    );
}

// -- open gauge (inflow/outflow reference) composed with weighted rows --------------------------

/// Wall-tangential edges of a wall-bounded box (the binary no-slip set).
fn wall_edges(complex: &LatticeComplex<2, f64>) -> Vec<usize> {
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

#[test]
fn open_empty_rows_are_bit_identical_to_the_open_path() {
    // With a reference vertex (open gauge) and no weighted rows, the weighted entry point must
    // reproduce the binary open projection bit-for-bit.
    let m = manifold_2d([8, 6], [false, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let zeroed = wall_edges(m.complex());
    let reference = vec![0usize];
    let field = random_field(n1, 61);
    let opts = HodgeDecomposeOptions::default();

    let binary = m
        .leray_project_open_opts(&field, &zeroed, &[], &reference, &opts)
        .unwrap();
    let empty: &[CutFaceConstraint<f64>] = &[];
    let weighted = m
        .leray_project_open_weighted_opts(&field, &zeroed, &[], &reference, empty, &opts, None)
        .unwrap();

    assert_eq!(
        binary.projected().as_slice(),
        weighted.projected().as_slice(),
        "empty rows must reproduce the binary open projection bit-for-bit"
    );
}

#[test]
fn open_gauge_weighted_rows_are_satisfied_on_the_state() {
    // The state projection path: an immersed body in a wall-bounded box with an outflow reference.
    // The weighted rows (the body no-slip) must hold on the projected state — the property the
    // per-stage rate projection alone cannot guarantee through the re-entry's gradient correction.
    let m = manifold_2d([8, 8], [false, false], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let zeroed = wall_edges(m.complex());
    let reference = vec![0usize];
    let reg = single_cut_cell(m.complex(), [4, 4]);
    let rows = reg.cut_face_constraints(m.complex());
    let field = random_field(n1, 67);

    let p = m
        .leray_project_open_weighted_opts(
            &field,
            &zeroed,
            &[],
            &reference,
            &rows,
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();
    let u = p.projected().as_slice();

    for &e in &zeroed {
        assert_eq!(u[e], 0.0, "wall edge {e} must stay zero");
    }
    for row in &rows {
        assert!(
            row_residual(row, u).abs() < 1e-9,
            "aperture-resolved body row not satisfied on the open-gauge state (kind {:?})",
            row.kind()
        );
    }
}

// -- warm-start agreement -----------------------------------------------------------------------

#[test]
fn warm_start_matches_the_cold_weighted_solve() {
    let m = manifold_2d([6, 6], [true, true], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let reg = single_cut_cell(m.complex(), [2, 2]);
    let rows = reg.cut_face_constraints(m.complex());
    let field = random_field(n1, 41);
    let opts = HodgeDecomposeOptions::default();

    let cold = m
        .leray_project_constrained_weighted_opts(&field, &[], &rows, &opts, None)
        .unwrap();
    let guess = cold.potential().as_slice().to_vec();
    let warm = m
        .leray_project_constrained_weighted_opts(&field, &[], &rows, &opts, Some(&guess))
        .unwrap();

    let gap = sup(cold
        .projected()
        .as_slice()
        .iter()
        .zip(warm.projected().as_slice())
        .map(|(a, b)| a - b));
    assert!(gap < 1e-9, "warm and cold solves disagree by {gap:e}");
}

#[test]
fn lambda_warm_start_matches_the_cold_weighted_solve() {
    // Warming BOTH the φ block and the λ (multiplier) block must give the same projection as cold —
    // the per-stage hot-path optimization changes only the CG iteration count, not the result.
    let m = manifold_2d([6, 6], [true, true], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let reg = single_cut_cell(m.complex(), [2, 2]);
    let rows = reg.cut_face_constraints(m.complex());
    let opts = HodgeDecomposeOptions::default();

    // Step 1: a cold solve yields the φ potential and the λ multipliers.
    let field1 = random_field(n1, 41);
    let (p1, lambda1) = m
        .leray_project_constrained_weighted_warm(&field1, &[], &rows, &opts, None, None)
        .unwrap();
    let phi1 = p1.potential().as_slice().to_vec();

    // Step 2: a nearby field, solved cold and warm (φ + λ seeded from step 1). The two must agree.
    let field2 = random_field(n1, 42);
    let cold2 = m
        .leray_project_constrained_weighted_opts(&field2, &[], &rows, &opts, None)
        .unwrap();
    let (warm2, lambda2) = m
        .leray_project_constrained_weighted_warm(
            &field2,
            &[],
            &rows,
            &opts,
            Some(&phi1),
            Some(&lambda1),
        )
        .unwrap();

    let gap = sup(cold2
        .projected()
        .as_slice()
        .iter()
        .zip(warm2.projected().as_slice())
        .map(|(a, b)| a - b));
    assert!(gap < 1e-9, "λ-warm and cold solves disagree by {gap:e}");
    assert_eq!(lambda1.len(), rows.len(), "one multiplier per emitted row");
    assert_eq!(
        lambda2.len(),
        rows.len(),
        "multiplier vector is reusable across steps"
    );

    // A stale λ guess of the wrong length is ignored (falls back to a zero seed), still converging.
    let (warm_badlen, _) = m
        .leray_project_constrained_weighted_warm(
            &field2,
            &[],
            &rows,
            &opts,
            Some(&phi1),
            Some(&[1.0, 2.0, 3.0, 4.0, 5.0]),
        )
        .unwrap();
    let gap2 = sup(cold2
        .projected()
        .as_slice()
        .iter()
        .zip(warm_badlen.projected().as_slice())
        .map(|(a, b)| a - b));
    assert!(
        gap2 < 1e-9,
        "wrong-length λ guess must be ignored, not corrupt the solve"
    );
}

// -- no-penetration ablation (tasks.md 4.3, validated cheaply on one cell) -----------------------

#[test]
fn tangential_only_ablation_still_enforces_tangential_no_slip() {
    let m = manifold_2d([6, 6], [true, true], CubicalReggeGeometry::unit());
    let n1 = m.complex().num_cells(1);
    let reg = single_cut_cell(m.complex(), [2, 2]);
    let all_rows = reg.cut_face_constraints(m.complex());
    let tangential: Vec<CutFaceConstraint<f64>> = all_rows
        .iter()
        .filter(|r| r.kind() == CutConstraintKind::Tangential)
        .cloned()
        .collect();
    assert_eq!(tangential.len(), 1, "2D leaves one tangential row");

    let field = random_field(n1, 53);
    let p = m
        .leray_project_constrained_weighted_opts(
            &field,
            &[],
            &tangential,
            &HodgeDecomposeOptions::default(),
            None,
        )
        .unwrap();
    let u = p.projected().as_slice();

    for row in &tangential {
        assert!(
            row_residual(row, u).abs() < 1e-9,
            "tangential row not satisfied under the no-penetration-off ablation"
        );
    }
    let div = sup(m.codifferential_of(u, 1).into_vec());
    assert!(
        div < 1e-9,
        "interior divergence {div:e} above solve exactness"
    );
}
