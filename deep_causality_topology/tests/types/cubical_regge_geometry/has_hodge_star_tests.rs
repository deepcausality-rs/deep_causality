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
//! `num_cells(k) × num_cells(k)`, exactly `num_cells(k)` non-zero entries (diagonal),
//! and `Cow::Owned` (cubical Hodge ⋆ is compute-on-demand).

use std::borrow::Cow;

use deep_causality_topology::utils_tests::{
    open_cube_3, open_square_3, per_axis_geometry, per_edge_uniform_per_axis, periodic_cube_3,
    periodic_square_3, unit_geometry,
};
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, HasHodgeStar};

const TOL: f64 = 1e-12;

fn assert_diagonal<R>(matrix: &deep_causality_sparse::CsrMatrix<R>, n: usize)
where
    R: deep_causality_num::RealField + std::fmt::Debug,
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
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    for k in 0..=2 {
        let star = geom.hodge_star_matrix(&lattice, k);
        let n = lattice.num_cells(k);
        assert_diagonal(star.as_ref(), n);
        for v in star.values() {
            assert!((*v - 1.0).abs() < TOL, "k={k} entry {v} expected 1.0");
        }
    }
}

#[test]
fn unit_edge_hodge_star_is_identity_on_2d_periodic_lattice() {
    let lattice = periodic_square_3();
    let geom = unit_geometry::<2>();
    for k in 0..=2 {
        let star = geom.hodge_star_matrix(&lattice, k);
        for v in star.values() {
            assert!((*v - 1.0).abs() < TOL);
        }
    }
}

#[test]
fn unit_edge_hodge_star_is_identity_on_3d_open_lattice() {
    let lattice = open_cube_3();
    let geom = unit_geometry::<3>();
    for k in 0..=3 {
        let star = geom.hodge_star_matrix(&lattice, k);
        for v in star.values() {
            assert!((*v - 1.0).abs() < TOL);
        }
    }
}

#[test]
fn unit_edge_hodge_star_is_identity_on_3d_periodic_lattice() {
    let lattice = periodic_cube_3();
    let geom = unit_geometry::<3>();
    for k in 0..=3 {
        let star = geom.hodge_star_matrix(&lattice, k);
        for v in star.values() {
            assert!((*v - 1.0).abs() < TOL);
        }
    }
}

#[test]
fn unit_edge_hodge_star_returns_owned_cow() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let star = geom.hodge_star_matrix(&lattice, 0);
    assert!(matches!(star, Cow::Owned(_)));
}

// -- Uniform -----------------------------------------------------------------------

#[test]
fn uniform_hodge_star_is_length_to_the_d_minus_2k_at_every_cell_2d() {
    let lattice = open_square_3();
    let geom: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(2.0);
    // D = 2: exponents are D - 2k for k = 0, 1, 2 → 2, 0, -2.
    for (k, expected) in [(0usize, 4.0), (1, 1.0), (2, 0.25)] {
        let star = geom.hodge_star_matrix(&lattice, k);
        for v in star.values() {
            assert!(
                (*v - expected).abs() < TOL,
                "k={k} entry {v} expected {expected}"
            );
        }
    }
}

#[test]
fn uniform_hodge_star_is_length_to_the_d_minus_2k_at_every_cell_3d() {
    let lattice = open_cube_3();
    let geom: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::uniform(2.0);
    // D = 3: exponents are 3, 1, -1, -3 → 8, 2, 0.5, 0.125.
    for (k, expected) in [(0usize, 8.0), (1, 2.0), (2, 0.5), (3, 0.125)] {
        let star = geom.hodge_star_matrix(&lattice, k);
        for v in star.values() {
            assert!(
                (*v - expected).abs() < TOL,
                "k={k} entry {v} expected {expected}"
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

    // ⋆_0
    let star0 = geom.hodge_star_matrix(&lattice, 0);
    for v in star0.values() {
        assert!((*v - a * b).abs() < TOL);
    }

    // ⋆_1 — per-edge: axis-0 (orientation 0b01) gives b/a; axis-1 (0b10) gives a/b.
    let star1 = geom.hodge_star_matrix(&lattice, 1);
    let values = star1.values();
    for (i, cell) in lattice.cells(1).enumerate() {
        let expected = match cell.orientation() {
            0b01 => b / a, // edge along axis 0
            0b10 => a / b, // edge along axis 1
            other => panic!("unexpected 1-cell orientation {other:b}"),
        };
        assert!(
            (values[i] - expected).abs() < TOL,
            "edge {i} (orientation={:b}): got {}, expected {expected}",
            cell.orientation(),
            values[i]
        );
    }

    // ⋆_2
    let star2 = geom.hodge_star_matrix(&lattice, 2);
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
        let a = per_axis.hodge_star_matrix(&lattice, k);
        let b = uniform.hodge_star_matrix(&lattice, k);
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

    let star0 = geom.hodge_star_matrix(&lattice, 0);
    for v in star0.values() {
        assert!((*v - a * b * c).abs() < TOL);
    }

    let star1 = geom.hodge_star_matrix(&lattice, 1);
    for (i, cell) in lattice.cells(1).enumerate() {
        let expected = match cell.orientation() {
            0b001 => (b * c) / a,
            0b010 => (a * c) / b,
            0b100 => (a * b) / c,
            other => panic!("unexpected 1-cell orientation {other:b}"),
        };
        assert!((star1.values()[i] - expected).abs() < TOL);
    }

    let star2 = geom.hodge_star_matrix(&lattice, 2);
    for (i, cell) in lattice.cells(2).enumerate() {
        let expected = match cell.orientation() {
            0b011 => c / (a * b),
            0b101 => b / (a * c),
            0b110 => a / (b * c),
            other => panic!("unexpected 2-cell orientation {other:b}"),
        };
        assert!((star2.values()[i] - expected).abs() < TOL);
    }

    let star3 = geom.hodge_star_matrix(&lattice, 3);
    for v in star3.values() {
        assert!((*v - 1.0 / (a * b * c)).abs() < TOL);
    }
}

// -- Out of range ------------------------------------------------------------------

#[test]
fn hodge_star_for_k_greater_than_dimension_is_empty_matrix() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let star = geom.hodge_star_matrix(&lattice, 5);
    assert_eq!(star.shape(), (0, 0));
    assert!(star.values().is_empty());
}

// -- PerEdge risk-gate marker (R4.4 lands the real impl) ---------------------------

#[test]
#[should_panic(expected = "deferred to R4.4")]
fn per_edge_hodge_star_panics_until_r4_4_lands() {
    // Per design.md Risk 1 + tasks.md R4.4.5: the PerEdge tier requires a
    // half-edge-average dual-cell derivation which is the ~1-week risk item.
    // R4.3 ships with an explicit panic gate to flag the deferred work;
    // R4.4 replaces this panic with the real implementation.
    let lattice = open_cube_3();
    let geom: CubicalReggeGeometry<3, f64> =
        per_edge_uniform_per_axis::<3>(&lattice, [1.0, 1.0, 1.0]);
    let _ = geom.hodge_star_matrix(&lattice, 1);
}
