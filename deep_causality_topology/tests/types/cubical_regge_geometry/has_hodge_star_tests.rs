/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Property tests for the cubical `HasHodgeStar<R>` implementation — Phase R4.3.
//!
//! Coverage matrix (closed-form expectations per design.md Decision 4):
//!
//! - **`UnitEdge`** — Hodge ⋆ is the identity matrix at every grade `k ∈ [0, D]`.
//!   Tested on 2D open + 2D periodic + 3D open + 3D periodic lattices.
//! - **`Uniform { length }`** — diagonal entry is `length^(D - 2k)` at every k-cell.
//!   Tested with `length = 2.0` on 2D and 3D lattices, covering both positive and
//!   negative exponents.
//! - **`PerAxis { lengths }`** — 2D `[a, b]` lattice: ⋆_0 = `a·b`, ⋆_1 alternates
//!   `b/a` (axis-0 edges) and `a/b` (axis-1 edges), ⋆_2 = `1/(a·b)`. Verified
//!   numerically against the closed-form expectation.
//! - **`PerEdge`** — invocation panics with the R4.4-deferred message (verified
//!   by `#[should_panic]`). Replaced by the real implementation in R4.4.
//!
//! Sparsity / shape invariants verified throughout: matrix is square
//! `num_cells(k) × num_cells(k)`, exactly `num_cells(k)` non-zero entries
//! (diagonal). The diagonal ⋆ is immutable for a fixed (geometry, lattice),
//! so it is memoized and served `Cow::Borrowed` after a build-once; a metric
//! reused on a differently-shaped lattice falls back to `Cow::Owned`.

use std::borrow::Cow;

use deep_causality_topology::utils_tests::{
    open_cube_3, open_square_3, per_axis_geometry, per_edge_uniform_per_axis, periodic_cube_3,
    periodic_square_3, unit_geometry,
};
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, HasHodgeStar, LatticeComplex};

const TOL: f64 = 1e-12;

/// Boundary clip factor `2^{-b}` of the corrected star (the
/// wall-hodge-star capability): one halving per open-axis boundary
/// incidence along the cell's inactive axes.
fn clip_factor<const D: usize>(
    lattice: &LatticeComplex<D, f64>,
    cell: &deep_causality_topology::LatticeCell<D>,
) -> f64 {
    let shape = lattice.shape();
    let periodic = lattice.periodic();
    let mut f = 1.0;
    for a in 0..D {
        let active = cell.orientation() & (1u32 << a) != 0;
        if active || periodic[a] {
            continue;
        }
        let p = cell.position()[a];
        if p == 0 || p + 1 == shape[a] {
            f *= 0.5;
        }
    }
    f
}

fn assert_diagonal<R>(matrix: &deep_causality_sparse::CsrMatrix<R>, n: usize)
where
    R: deep_causality_algebra::RealField + std::fmt::Debug,
{
    assert_eq!(matrix.shape(), (n, n), "matrix must be n × n");
    assert_eq!(
        matrix.values().len(),
        n,
        "diagonal matrix must have exactly n non-zero entries"
    );
    for (i, (row, col)) in matrix
        .row_indices()
        .windows(2)
        .enumerate()
        .flat_map(|(r, w)| (w[0]..w[1]).map(move |idx| (r, matrix.col_indices()[idx])))
        .enumerate()
    {
        let (expected_row, actual_col) = (i, col);
        assert_eq!(
            row, expected_row,
            "non-zero at row {row} differs from expected diagonal row {expected_row}"
        );
        assert_eq!(
            actual_col, expected_row,
            "row {row} has off-diagonal entry at col {actual_col}"
        );
    }
}

// -- UnitEdge ----------------------------------------------------------------------

#[test]
fn unit_edge_hodge_star_is_identity_on_2d_open_lattice() {
    // Boundary-corrected star: identity on the interior, dual volumes
    // clipped by 2^{-b} at open-axis boundary incidences.
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    for k in 0..=2 {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        let n = lattice.num_cells(k);
        assert_diagonal(star.as_ref(), n);
        let values = star.values();
        for (i, cell) in lattice.cells(k).enumerate() {
            let expected = clip_factor(&lattice, &cell);
            assert!(
                (values[i] - expected).abs() < TOL,
                "k={k} cell {i}: {} expected {expected}",
                values[i]
            );
        }
    }
}

#[test]
fn unit_edge_hodge_star_is_identity_on_2d_periodic_lattice() {
    let lattice = periodic_square_3();
    let geom = unit_geometry::<2>();
    for k in 0..=2 {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        for v in star.values() {
            assert!((*v - 1.0).abs() < TOL);
        }
    }
}

#[test]
fn unit_edge_hodge_star_is_identity_on_3d_open_lattice() {
    // Boundary-corrected star: identity on the interior, clipped at walls.
    let lattice = open_cube_3();
    let geom = unit_geometry::<3>();
    for k in 0..=3 {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        let values = star.values();
        for (i, cell) in lattice.cells(k).enumerate() {
            let expected = clip_factor(&lattice, &cell);
            assert!((values[i] - expected).abs() < TOL);
        }
    }
}

#[test]
fn unit_edge_hodge_star_is_identity_on_3d_periodic_lattice() {
    let lattice = periodic_cube_3();
    let geom = unit_geometry::<3>();
    for k in 0..=3 {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        for v in star.values() {
            assert!((*v - 1.0).abs() < TOL);
        }
    }
}

#[test]
fn unit_edge_hodge_star_is_served_borrowed_from_the_memo() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    // The diagonal ⋆ is immutable for this (geometry, lattice): the first
    // request builds and memoizes it, returning a borrow of the memo (no
    // per-call rebuild — the change that took the projection's δω off the
    // critical path).
    let first = geom.hodge_star_matrix(&lattice, 0).unwrap();
    assert!(matches!(first, Cow::Borrowed(_)));
    // A second request hits the memo: same backing matrix, still borrowed.
    let second = geom.hodge_star_matrix(&lattice, 0).unwrap();
    assert!(matches!(second, Cow::Borrowed(_)));
    assert!(core::ptr::eq(first.as_ref(), second.as_ref()));
}

#[test]
fn star_memo_falls_back_to_owned_on_a_different_lattice() {
    // The metric is complex-agnostic by its trait signature, so the memo
    // pins to the first lattice's fingerprint. The same metric applied to a
    // differently-shaped lattice rebuilds uncached rather than handing back a
    // wrongly-sized memo.
    let geom = unit_geometry::<2>();
    let lattice_a = LatticeComplex::<2, f64>::open([3, 3]);
    let lattice_b = LatticeComplex::<2, f64>::open([4, 4]);
    let a = geom.hodge_star_matrix(&lattice_a, 0).unwrap();
    assert!(matches!(a, Cow::Borrowed(_)));
    let b = geom.hodge_star_matrix(&lattice_b, 0).unwrap();
    assert!(matches!(b, Cow::Owned(_)));
    // The owned fallback is correctly sized for lattice_b (4×4 vertices).
    assert_eq!(b.as_ref().shape().0, 16);
}

// -- Uniform -----------------------------------------------------------------------

#[test]
fn uniform_hodge_star_is_length_to_the_d_minus_2k_at_every_cell_2d() {
    let lattice = open_square_3();
    let geom: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(2.0);
    // D = 2: exponents are D - 2k for k = 0, 1, 2 → 2, 0, -2.
    for (k, closed_form) in [(0usize, 4.0), (1, 1.0), (2, 0.25)] {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        let values = star.values();
        for (i, cell) in lattice.cells(k).enumerate() {
            let expected = closed_form * clip_factor(&lattice, &cell);
            assert!(
                (values[i] - expected).abs() < TOL,
                "k={k} cell {i}: {} expected {expected}",
                values[i]
            );
        }
    }
}

#[test]
fn uniform_hodge_star_is_length_to_the_d_minus_2k_at_every_cell_3d() {
    let lattice = open_cube_3();
    let geom: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::uniform(2.0);
    // D = 3: exponents are 3, 1, -1, -3 → 8, 2, 0.5, 0.125.
    for (k, closed_form) in [(0usize, 8.0), (1, 2.0), (2, 0.5), (3, 0.125)] {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        let values = star.values();
        for (i, cell) in lattice.cells(k).enumerate() {
            let expected = closed_form * clip_factor(&lattice, &cell);
            assert!(
                (values[i] - expected).abs() < TOL,
                "k={k} cell {i}: {} expected {expected}",
                values[i]
            );
        }
    }
}

// -- PerAxis -----------------------------------------------------------------------

#[test]
fn per_axis_hodge_star_2d_matches_closed_form_a_b() {
    // Per design.md Decision 4 (2D PerAxis with axes [a, b]):
    //   ⋆_0 entry = a · b  for every vertex
    //   ⋆_1 entry = b / a  for axis-0 edges, a / b for axis-1 edges
    //   ⋆_2 entry = 1 / (a · b)  for every 2-cube
    let a = 3.0_f64;
    let b = 5.0_f64;
    let lattice = open_square_3();
    let geom = per_axis_geometry::<2>([a, b]);

    // ⋆_0 (clipped at walls)
    let star0 = geom.hodge_star_matrix(&lattice, 0).unwrap();
    let values0 = star0.values();
    for (i, cell) in lattice.cells(0).enumerate() {
        let expected = a * b * clip_factor(&lattice, &cell);
        assert!((values0[i] - expected).abs() < TOL);
    }

    // ⋆_1 — per-edge: axis-0 (orientation 0b01) gives b/a; axis-1 (0b10) gives a/b.
    let star1 = geom.hodge_star_matrix(&lattice, 1).unwrap();
    let values = star1.values();
    for (i, cell) in lattice.cells(1).enumerate() {
        let closed_form = match cell.orientation() {
            0b01 => b / a, // edge along axis 0
            0b10 => a / b, // edge along axis 1
            other => panic!("unexpected 1-cell orientation {other:b}"),
        };
        let expected = closed_form * clip_factor(&lattice, &cell);
        assert!(
            (values[i] - expected).abs() < TOL,
            "edge {i} (orientation={:b}): got {}, expected {expected}",
            cell.orientation(),
            values[i]
        );
    }

    // ⋆_2 (2-cells have no inactive axes in 2D: never clipped)
    let star2 = geom.hodge_star_matrix(&lattice, 2).unwrap();
    for v in star2.values() {
        assert!((*v - 1.0 / (a * b)).abs() < TOL);
    }
}

#[test]
fn per_axis_degenerates_to_uniform_when_all_axes_equal() {
    // With a = b = length, PerAxis must produce the same Hodge ⋆ entries as
    // Uniform { length } at every grade.
    let length = 2.0_f64;
    let lattice = open_square_3();
    let per_axis = per_axis_geometry::<2>([length, length]);
    let uniform: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(length);
    for k in 0..=2 {
        let a = per_axis.hodge_star_matrix(&lattice, k).unwrap();
        let b = uniform.hodge_star_matrix(&lattice, k).unwrap();
        assert_eq!(a.values().len(), b.values().len());
        for (va, vb) in a.values().iter().zip(b.values().iter()) {
            assert!((*va - *vb).abs() < TOL);
        }
    }
}

#[test]
fn per_axis_3d_diagonal_entries_match_closed_form() {
    // 3D PerAxis with axes [a, b, c]:
    //   ⋆_0 = a·b·c (vertex → 3-cube dual)
    //   ⋆_1 axis-0 edge = (b·c)/a; axis-1 edge = (a·c)/b; axis-2 edge = (a·b)/c
    //   ⋆_2 face in axes {0,1} = c/(a·b); {0,2} = b/(a·c); {1,2} = a/(b·c)
    //   ⋆_3 = 1/(a·b·c)
    let a = 2.0_f64;
    let b = 3.0_f64;
    let c = 5.0_f64;
    let lattice = open_cube_3();
    let geom = per_axis_geometry::<3>([a, b, c]);

    let star0 = geom.hodge_star_matrix(&lattice, 0).unwrap();
    let values0 = star0.values();
    for (i, cell) in lattice.cells(0).enumerate() {
        let expected = a * b * c * clip_factor(&lattice, &cell);
        assert!((values0[i] - expected).abs() < TOL);
    }

    let star1 = geom.hodge_star_matrix(&lattice, 1).unwrap();
    for (i, cell) in lattice.cells(1).enumerate() {
        let closed_form = match cell.orientation() {
            0b001 => (b * c) / a,
            0b010 => (a * c) / b,
            0b100 => (a * b) / c,
            other => panic!("unexpected 1-cell orientation {other:b}"),
        };
        let expected = closed_form * clip_factor(&lattice, &cell);
        assert!((star1.values()[i] - expected).abs() < TOL);
    }

    let star2 = geom.hodge_star_matrix(&lattice, 2).unwrap();
    for (i, cell) in lattice.cells(2).enumerate() {
        let closed_form = match cell.orientation() {
            0b011 => c / (a * b),
            0b101 => b / (a * c),
            0b110 => a / (b * c),
            other => panic!("unexpected 2-cell orientation {other:b}"),
        };
        let expected = closed_form * clip_factor(&lattice, &cell);
        assert!((star2.values()[i] - expected).abs() < TOL);
    }

    let star3 = geom.hodge_star_matrix(&lattice, 3).unwrap();
    for v in star3.values() {
        assert!((*v - 1.0 / (a * b * c)).abs() < TOL);
    }
}

// -- Out of range ------------------------------------------------------------------

#[test]
fn hodge_star_for_k_greater_than_dimension_is_empty_matrix() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let star = geom.hodge_star_matrix(&lattice, 5).unwrap();
    assert_eq!(star.shape(), (0, 0));
    assert!(star.values().is_empty());
}

// -- PerEdge (R4.4) ----------------------------------------------------------------

#[test]
fn per_edge_with_uniform_lengths_matches_uniform_on_periodic_lattice() {
    // R4.4 internal consistency: when every per-edge length is the same scalar L,
    // the per-edge dual-cell formula must reduce to L^(D-2k) at every cell,
    // exactly matching the Uniform tier's closed form. Tested on a periodic
    // lattice so that every corner mask is valid and the formula reduces cleanly
    // without boundary-handling complications.
    let length = 1.0_f64;
    let lattice = periodic_cube_3();
    let per_edge = per_edge_uniform_per_axis::<3>(&lattice, [length; 3]);
    let uniform: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::uniform(length);
    for k in 0..=3 {
        let a = per_edge.hodge_star_matrix(&lattice, k).unwrap();
        let b = uniform.hodge_star_matrix(&lattice, k).unwrap();
        assert_eq!(a.values().len(), b.values().len(), "k = {k}");
        for (va, vb) in a.values().iter().zip(b.values().iter()) {
            assert!(
                (*va - *vb).abs() < TOL,
                "k = {k}: per-edge {va} vs uniform {vb}"
            );
        }
    }
}

#[test]
fn per_edge_with_uniform_per_axis_lengths_matches_per_axis_on_periodic_lattice() {
    // Stronger check: per-edge populated with axis-uniform but anisotropic
    // lengths must match the PerAxis tier on a periodic lattice.
    let lengths = [2.0_f64, 3.0, 5.0];
    let lattice = periodic_cube_3();
    let per_edge = per_edge_uniform_per_axis::<3>(&lattice, lengths);
    let per_axis = per_axis_geometry::<3>(lengths);
    for k in 0..=3 {
        let a = per_edge.hodge_star_matrix(&lattice, k).unwrap();
        let b = per_axis.hodge_star_matrix(&lattice, k).unwrap();
        assert_eq!(a.values().len(), b.values().len(), "k = {k}");
        for (va, vb) in a.values().iter().zip(b.values().iter()) {
            assert!(
                (*va - *vb).abs() < TOL,
                "k = {k}: per-edge {va} vs per-axis {vb}"
            );
        }
    }
}

#[test]
fn per_edge_2d_uniform_matches_closed_form_on_periodic_lattice() {
    // 2D periodic [a, b] direct closed-form check on the per-edge path.
    let a = 3.0_f64;
    let b = 5.0_f64;
    let lattice = periodic_square_3();
    let geom = per_edge_uniform_per_axis::<2>(&lattice, [a, b]);

    let star0 = geom.hodge_star_matrix(&lattice, 0).unwrap();
    for v in star0.values() {
        assert!(
            (*v - a * b).abs() < TOL,
            "PerEdge ⋆_0 entry {v} expected {}",
            a * b
        );
    }

    let star1 = geom.hodge_star_matrix(&lattice, 1).unwrap();
    for (i, cell) in lattice.cells(1).enumerate() {
        let expected = match cell.orientation() {
            0b01 => b / a,
            0b10 => a / b,
            other => panic!("unexpected 1-cell orientation {other:b}"),
        };
        assert!(
            (star1.values()[i] - expected).abs() < TOL,
            "PerEdge ⋆_1 edge {i} orientation={:b} got {} expected {expected}",
            cell.orientation(),
            star1.values()[i]
        );
    }

    let star2 = geom.hodge_star_matrix(&lattice, 2).unwrap();
    for v in star2.values() {
        assert!((*v - 1.0 / (a * b)).abs() < TOL);
    }
}

#[test]
fn per_edge_open_lattice_handles_boundary_without_panicking() {
    // Smoke test: open lattice has boundary cells where some corner masks
    // reference out-of-bounds edges. Verify the impl handles these gracefully
    // (returns finite, non-NaN values for every interior diagonal entry).
    let lattice = open_cube_3();
    let geom = per_edge_uniform_per_axis::<3>(&lattice, [1.0, 1.0, 1.0]);
    for k in 0..=3 {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        for v in star.values() {
            assert!(
                v.is_finite() && !v.is_nan(),
                "k = {k}: got non-finite entry {v}"
            );
            assert!(*v > 0.0, "k = {k}: got non-positive entry {v}");
        }
    }
}

#[test]
fn per_edge_hodge_star_is_served_borrowed_from_the_memo() {
    let lattice = periodic_cube_3();
    let geom = per_edge_uniform_per_axis::<3>(&lattice, [1.0, 1.0, 1.0]);
    // The PerEdge tier is the most expensive to build (per-cell dual-volume
    // averaging), so the memo matters most here; still served borrowed after
    // the first build.
    let first = geom.hodge_star_matrix(&lattice, 1).unwrap();
    assert!(matches!(first, Cow::Borrowed(_)));
    let second = geom.hodge_star_matrix(&lattice, 1).unwrap();
    assert!(matches!(second, Cow::Borrowed(_)));
    assert!(core::ptr::eq(first.as_ref(), second.as_ref()));
}

#[test]
fn per_edge_diagonal_entries_change_when_individual_edges_change() {
    // Behavioural test: prove the per-edge path actually responds to per-edge
    // data, not just to the aggregate axis statistics. Construct two per-edge
    // geometries on the same lattice where exactly one edge has a different
    // length; assert that at least one Hodge ⋆ diagonal entry differs.
    let lattice = periodic_square_3();
    let lens_a = vec![1.0_f64; per_edge_total_edges_2d(&lattice)];
    let mut lens_b = lens_a.clone();
    lens_b[0] = 2.0; // perturb a single edge

    let geom_a: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::from_edge_lengths(lens_a);
    let geom_b: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::from_edge_lengths(lens_b);

    let star_a = geom_a.hodge_star_matrix(&lattice, 0).unwrap();
    let star_b = geom_b.hodge_star_matrix(&lattice, 0).unwrap();
    let differs = star_a
        .values()
        .iter()
        .zip(star_b.values().iter())
        .any(|(a, b)| (a - b).abs() > TOL);
    assert!(
        differs,
        "Perturbing one edge length must change at least one Hodge ⋆_0 entry"
    );
}

fn per_edge_total_edges_2d(lattice: &LatticeComplex<2, f64>) -> usize {
    lattice.num_cells(1)
}
