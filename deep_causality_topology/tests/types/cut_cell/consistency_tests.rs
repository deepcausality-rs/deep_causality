/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Consistency + composition tests (A6, A7):
//!
//! - the cut-aware dual clip reproduces the Stage-3 integer `boundary_clip` (`2^{-b}`) on an
//!   axis-aligned cut — pinned both against the closed-form `2^{-b}` and against the actual
//!   shipped Hodge-star diagonal;
//! - a registry built from an analytic primitive is sparse (only cut/solid recorded);
//! - a cut cell composes with graded `PerEdge` edge lengths — its full volume is the
//!   closed-form measure from the graded lengths, so the cut rides the verified second-order
//!   graded substrate (`graded-metrics`).

use deep_causality_topology::{
    CellClass, ChainComplex, CubicalReggeGeometry, CutCellRegistry, HasHodgeStar, LatticeComplex,
    Primitive,
};

const TOL: f64 = 1e-12;

// -- A6: the dual clip reproduces the Stage-3 wall clip ---------------------------------

#[test]
fn empty_registry_dual_clip_matches_open_star_2d() {
    // On an open (wall-bounded) lattice the walls ARE the axis-aligned cut: the cut-aware
    // dual fluid fraction must equal the boundary-clipped Hodge-star diagonal at every grade.
    let lattice = LatticeComplex::<2, f64>::square_open(4);
    let geom = CubicalReggeGeometry::<2, f64>::unit();
    let reg = CutCellRegistry::<2, f64>::new();

    for k in 0..=2 {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        let values = star.values();
        for (i, cell) in lattice.cells(k).enumerate() {
            let clip = reg.dual_fluid_fraction(&lattice, &cell);
            assert!(
                (clip - values[i]).abs() < TOL,
                "k={k} cell {i}: dual clip {clip} != star boundary clip {}",
                values[i]
            );
        }
    }
}

#[test]
fn empty_registry_dual_clip_matches_open_star_3d() {
    let lattice = LatticeComplex::<3, f64>::open([3, 3, 3]);
    let geom = CubicalReggeGeometry::<3, f64>::unit();
    let reg = CutCellRegistry::<3, f64>::new();

    for k in 0..=3 {
        let star = geom.hodge_star_matrix(&lattice, k).unwrap();
        let values = star.values();
        for (i, cell) in lattice.cells(k).enumerate() {
            let clip = reg.dual_fluid_fraction(&lattice, &cell);
            assert!((clip - values[i]).abs() < TOL, "k={k} cell {i}");
        }
    }
}

#[test]
fn corner_vertex_dual_clip_is_two_to_minus_b() {
    // A vertex at the b-fold open corner has dual fluid fraction 2^{-b}, exactly the
    // boundary_clip integer pattern the generalized continuous clip must reduce to.
    let lattice = LatticeComplex::<3, f64>::open([3, 3, 3]);
    let reg = CutCellRegistry::<3, f64>::new();

    // 3-fold corner vertex [0,0,0]: 2^{-3} = 1/8.
    let corner = deep_causality_topology::LatticeCell::<3>::vertex([0, 0, 0]);
    assert!((reg.dual_fluid_fraction(&lattice, &corner) - 0.125).abs() < TOL);
    // Interior vertex: no clip.
    let interior = deep_causality_topology::LatticeCell::<3>::vertex([1, 1, 1]);
    assert!((reg.dual_fluid_fraction(&lattice, &interior) - 1.0).abs() < TOL);
}

#[test]
fn plane_cut_coincident_with_a_cell_boundary_reproduces_the_wall_clip() {
    // On a periodic lattice place a plane x = 2 (coincident with a cell boundary). The x > 2
    // half is solid; the x < 2 half fluid; no cell straddles. A vertex on the plane then has
    // its +x dual half in solid ⇒ dual fluid fraction 1/2 = the Stage-3 wall clip.
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let geom = CubicalReggeGeometry::<2, f64>::unit();
    let prim = Primitive::<2, f64>::halfspace([1.0, 0.0], 2.0);
    let reg = CutCellRegistry::from_primitive(&lattice, &geom, &prim).unwrap();

    // Only fully-solid cells are recorded (the plane is on a boundary; nothing is partial).
    assert!(!reg.is_empty());
    for (_, cut) in reg.iter() {
        assert_eq!(cut.class(), CellClass::Solid);
    }

    // A vertex on the cut plane (x = 2) has its +x dual half in solid ⇒ fraction 1/2.
    let on_plane = deep_causality_topology::LatticeCell::<2>::vertex([2, 1]);
    assert!((reg.dual_fluid_fraction(&lattice, &on_plane) - 0.5).abs() < TOL);
    // A vertex deep in the fluid (x = 3: both incident columns x∈[2,3],[3,4] fluid) is unclipped.
    let in_fluid = deep_causality_topology::LatticeCell::<2>::vertex([3, 1]);
    assert!((reg.dual_fluid_fraction(&lattice, &in_fluid) - 1.0).abs() < TOL);
}

// -- B4: immersed no-slip edge set -----------------------------------------------------

#[test]
fn solid_incident_edges_are_the_cube_boundary_edges() {
    // One solid cell in a periodic 4×4 lattice: its no-slip set is exactly the four edges of
    // that cube (B4 staircase immersed no-slip / no-penetration).
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let mut reg = CutCellRegistry::<2, f64>::new();
    let solid = deep_causality_topology::LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == solid).unwrap();
    reg.insert(idx, deep_causality_topology::CutCell::<2, f64>::solid(1.0));

    let edge_idx = |axis: usize, pos: [usize; 2]| {
        lattice
            .cells(1)
            .position(|c| c.orientation().trailing_zeros() as usize == axis && *c.position() == pos)
            .unwrap()
    };
    // Cube [1,1] boundary: x-edges at [1,1] & [1,2]; y-edges at [1,1] & [2,1].
    let mut expected = [
        edge_idx(0, [1, 1]),
        edge_idx(0, [1, 2]),
        edge_idx(1, [1, 1]),
        edge_idx(1, [2, 1]),
    ];
    expected.sort_unstable();

    let got = reg.solid_incident_edges(&lattice);
    assert_eq!(
        got,
        expected.to_vec(),
        "solid no-slip set must be the cube's 4 edges"
    );

    // A distant edge is not constrained.
    let far = edge_idx(0, [3, 3]);
    assert!(!got.contains(&far));
}

#[test]
fn empty_registry_has_no_immersed_edges() {
    let lattice = LatticeComplex::<3, f64>::cubic_torus(4);
    let reg = CutCellRegistry::<3, f64>::new();
    assert!(reg.solid_incident_edges(&lattice).is_empty());
}

// -- A7: registry sparsity + graded composition ----------------------------------------

#[test]
fn from_primitive_is_sparse() {
    // A small disk near one corner of a 6x6 cell grid touches only a few cells.
    let lattice = LatticeComplex::<2, f64>::square_torus(6);
    let geom = CubicalReggeGeometry::<2, f64>::unit();
    let prim = Primitive::<2, f64>::ball([0.0, 0.0], 1.5);
    let reg = CutCellRegistry::from_primitive(&lattice, &geom, &prim).unwrap();

    // Sized to the boundary, not the 36-cell volume.
    assert!(!reg.is_empty());
    assert!(
        reg.len() < 36,
        "registry should be sparse, got {}",
        reg.len()
    );
}

#[test]
fn cylinder_cut_geometry_recovers_the_exact_disk_area() {
    // D4 cheap CI regression: the clipped fluid volumes of a disk cut out of a uniform grid
    // sum to the exact (domain − π r²) area, with no heavy march. This pins the cut-cylinder
    // geometry the wake harness builds.
    let n = 16;
    let lattice = LatticeComplex::<2, f64>::square_torus(n);
    let geom = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let r = 3.5;
    let disk = Primitive::<2, f64>::ball([8.0, 8.0], r);
    let reg = CutCellRegistry::from_primitive(&lattice, &geom, &disk).unwrap();

    // Total fluid area = Σ clipped cell volume over every top cell.
    let fluid: f64 = lattice
        .cells(2)
        .map(|cell| reg.clipped_cell_volume(&geom, &lattice, &cell))
        .sum();
    let expected = (n * n) as f64 - std::f64::consts::PI * r * r;
    assert!(
        (fluid - expected).abs() < 1e-9,
        "cut disk fluid area {fluid} != domain − π r² = {expected}"
    );

    // The body has a solid interior (fully-dry cells) and a cut ring.
    let solids = reg
        .iter()
        .filter(|(_, c)| c.class() == CellClass::Solid)
        .count();
    let cuts = reg
        .iter()
        .filter(|(_, c)| c.class() == CellClass::Cut)
        .count();
    assert!(
        solids > 0,
        "a disk of radius {r} must have solid interior cells"
    );
    assert!(cuts > 0, "a disk must have a ring of partially-cut cells");
}

#[test]
fn cut_cell_composes_with_graded_edge_lengths() {
    // Geometric grading along axis 0; a plane perpendicular to axis 1 cuts every column at a
    // graded x-width. Each recorded cut cell's FULL volume must be the closed-form measure
    // computed from the graded edge lengths — i.e. geom.cell_volume of that top cell — so the
    // cut rides the verified second-order graded substrate.
    let lattice = LatticeComplex::<2, f64>::square_torus(5);
    let geom =
        CubicalReggeGeometry::<2, f64>::from_graded_geometric(&lattice, [1.0, 1.0], [1.2, 1.0]);

    // Plane y = 2.5 (axis 1 is ungraded, unit spacing) cuts the row of cells spanning it.
    let prim = Primitive::<2, f64>::halfspace([0.0, 1.0], 2.5);
    let reg = CutCellRegistry::from_primitive(&lattice, &geom, &prim).unwrap();

    assert!(!reg.is_empty());
    let mut saw_cut = false;
    for (i, cell) in lattice.cells(2).enumerate() {
        if let Some(cut) = reg.get(i) {
            let geometric_volume = geom.cell_volume(&lattice, &cell);
            assert!(
                (cut.full_volume() - geometric_volume).abs() < TOL,
                "cut full volume {} != graded cell_volume {geometric_volume}",
                cut.full_volume()
            );
            if cut.class() == CellClass::Cut {
                saw_cut = true;
                // The clipped fluid measure is a proper fraction of the graded full measure.
                assert!(cut.fluid_volume() > 0.0);
                assert!(cut.fluid_volume() < cut.full_volume());
            }
        }
    }
    assert!(
        saw_cut,
        "the plane should produce at least one partially-cut cell"
    );
}
