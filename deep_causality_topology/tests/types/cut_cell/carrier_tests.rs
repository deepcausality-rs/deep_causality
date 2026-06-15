/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Carrier + registry tests for cut cells (A1, A2): constructors, getters, the sparse
//! `cell-id -> CutCell` round-trip, and the cut-aware volume accessor (A3).

use deep_causality_topology::{
    CellClass, CubicalReggeGeometry, CutCell, CutCellRegistry, CutFaceFragment, LatticeCell,
    LatticeComplex, SourceGeometry,
};

#[test]
fn fluid_cell_is_full_and_unit_apertures() {
    let c = CutCell::<2, f64>::fluid(4.0);
    assert_eq!(c.class(), CellClass::Fluid);
    assert_eq!(c.fluid_volume(), 4.0);
    assert_eq!(c.full_volume(), 4.0);
    assert_eq!(c.volume_fraction(), 1.0);
    for a in 0..2 {
        assert_eq!(c.face_aperture(a, 0), Some(1.0));
        assert_eq!(c.face_aperture(a, 1), Some(1.0));
    }
    assert!(c.fragments().is_empty());
}

#[test]
fn solid_cell_is_empty_and_zero_apertures() {
    let c = CutCell::<3, f64>::solid(8.0);
    assert_eq!(c.class(), CellClass::Solid);
    assert_eq!(c.fluid_volume(), 0.0);
    assert_eq!(c.volume_fraction(), 0.0);
    for a in 0..3 {
        assert_eq!(c.face_aperture(a, 0), Some(0.0));
        assert_eq!(c.face_aperture(a, 1), Some(0.0));
    }
}

#[test]
fn cut_cell_preserves_fields_and_fragments() {
    let frag = CutFaceFragment::<2, f64>::new(0.5, [1.0, 0.0], [0.5, 0.25], SourceGeometry::Plane);
    let c = CutCell::cut(1.0, 0.25, [[0.0, 1.0], [0.5, 0.5]], vec![frag.clone()]);
    assert_eq!(c.class(), CellClass::Cut);
    assert_eq!(c.fluid_volume(), 0.25);
    assert_eq!(c.volume_fraction(), 0.25);
    assert_eq!(c.face_aperture(0, 0), Some(0.0));
    assert_eq!(c.face_aperture(0, 1), Some(1.0));
    assert_eq!(c.face_aperture(1, 0), Some(0.5));
    assert_eq!(c.face_aperture(0, 2), None);
    assert_eq!(c.face_aperture(2, 0), None);
    assert_eq!(c.fragments(), &[frag]);
}

#[test]
fn fragment_getters() {
    let f = CutFaceFragment::<3, f64>::new(
        2.0,
        [0.0, 1.0, 0.0],
        [1.0, 2.0, 3.0],
        SourceGeometry::Cylinder,
    );
    assert_eq!(f.area(), 2.0);
    assert_eq!(f.outward_normal(), &[0.0, 1.0, 0.0]);
    assert_eq!(f.centroid(), &[1.0, 2.0, 3.0]);
    assert_eq!(f.source(), SourceGeometry::Cylinder);
}

#[test]
fn registry_round_trip() {
    let mut reg = CutCellRegistry::<2, f64>::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
    assert!(reg.get(7).is_none());

    let prev = reg.insert(
        7,
        CutCell::cut(1.0, 0.3, [[0.0, 1.0], [0.3, 0.3]], Vec::new()),
    );
    assert!(prev.is_none());
    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());

    let got = reg.get(7).expect("inserted cell present");
    assert_eq!(got.fluid_volume(), 0.3);
    assert_eq!(got.class(), CellClass::Cut);

    // Overwrite returns the previous entry.
    let prev = reg.insert(7, CutCell::solid(1.0));
    assert!(prev.is_some());
    assert_eq!(reg.get(7).unwrap().class(), CellClass::Solid);

    // iter sees the single entry.
    assert_eq!(reg.iter().count(), 1);
}

#[test]
fn default_registry_is_empty() {
    let reg: CutCellRegistry<3, f64> = Default::default();
    assert!(reg.is_empty());
}

#[test]
fn clipped_cell_volume_falls_through_for_unregistered_cells() {
    // A 2x2 uniform lattice, spacing 2.0: every top cell has volume 4.0.
    let lattice = LatticeComplex::<2, f64>::square_torus(2);
    let geom = CubicalReggeGeometry::<2, f64>::uniform(2.0);
    let reg = CutCellRegistry::<2, f64>::new();

    for cell in lattice.iter_cells(2) {
        // No registry entry => full geometric volume.
        assert_eq!(reg.clipped_cell_volume(&geom, &lattice, &cell), 4.0);
    }

    // A registered cut overrides the volume for its top cell only.
    let mut reg = reg;
    let first = lattice.iter_cells(2).next().unwrap();
    let idx = lattice.iter_cells(2).position(|c| c == first).unwrap();
    reg.insert(
        idx,
        CutCell::cut(4.0, 1.5, [[1.0, 1.0], [1.0, 1.0]], Vec::new()),
    );
    assert_eq!(reg.clipped_cell_volume(&geom, &lattice, &first), 1.5);

    // Lower-grade cells always take the geometric fast path (vertex volume = 1).
    let vertex = LatticeCell::<2>::vertex([0, 0]);
    assert_eq!(reg.clipped_cell_volume(&geom, &lattice, &vertex), 1.0);
}
