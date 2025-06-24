// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

// SPDX-License-Identifier: MIT

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::*;

#[test]
fn test_adjustable_space_kind_geo() {
    let mut space = AdjustableSpaceKind::Geo(AdjustableGeoSpace::new(1, 0.0, 0.0, 0.0));
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 2.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), 2.0);

    assert!(space.update(&grid).is_ok());
    assert!(space.adjust(&grid).is_ok());

    assert_eq!(space.coordinate(0), &4.0); // lat: 2 + 2
    assert_eq!(space.coordinate(1), &4.0); // lon: 2 + 2
    assert_eq!(space.coordinate(2), &4.0); // alt: 2 + 2
}

#[test]
fn test_adjustable_space_kind_euclidean() {
    let mut space = AdjustableSpaceKind::Euclidean(AdjustableEuclideanSpace::new(1, 0.0, 0.0, 0.0));
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array3D);

    assert!(space.update(&grid).is_ok());
    assert_eq!(space.coordinate(0), &0.0); // 0.0 from init
    assert_eq!(space.coordinate(1), &0.0);
    assert_eq!(space.coordinate(2), &0.0);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    assert!(space.update(&grid).is_ok());
    assert_eq!(space.coordinate(0), &1.0); // 0.0 replaced with 1.0
    assert_eq!(space.coordinate(1), &1.0);
    assert_eq!(space.coordinate(2), &1.0);

    assert!(space.adjust(&grid).is_ok());

    assert_eq!(space.coordinate(0), &2.0); // 1.0 adjusted by 1.0 = 2.0
    assert_eq!(space.coordinate(1), &2.0);
    assert_eq!(space.coordinate(2), &2.0);
}

#[test]
fn test_adjustable_space_kind_ecef() {
    let mut space = AdjustableSpaceKind::Ecef(AdjustableEcefSpace::new(1, 100.0, 100.0, 100.0));
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 50.0);
    grid.set(PointIndex::new3d(0, 0, 1), 50.0);
    grid.set(PointIndex::new3d(0, 0, 2), 50.0);

    assert!(space.update(&grid).is_ok());
    assert_eq!(space.coordinate(0), &50.0); // 100.0 Replaced by 50
    assert_eq!(space.coordinate(1), &50.0);
    assert_eq!(space.coordinate(2), &50.0);

    grid.set(PointIndex::new3d(0, 0, 0), 10.0);
    grid.set(PointIndex::new3d(0, 0, 1), 20.0);
    grid.set(PointIndex::new3d(0, 0, 2), 30.0);
    assert!(space.adjust(&grid).is_ok());

    assert_eq!(space.coordinate(0), &60.0); // 50 adjusted by 10
    assert_eq!(space.coordinate(1), &70.0); // 50 adjusted by 20
    assert_eq!(space.coordinate(2), &80.0); // 50 adjusted by 30
}

#[test]
fn test_adjustable_space_kind_ned() {
    let mut space = AdjustableSpaceKind::Ned(AdjustableNedSpace::new(1, 5.0, 5.0, 5.0));
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    assert!(space.update(&grid).is_ok());
    assert_eq!(space.coordinate(0), &1.0); // 5.0 replaced with 1.0
    assert_eq!(space.coordinate(1), &1.0);
    assert_eq!(space.coordinate(2), &1.0);

    grid.set(PointIndex::new3d(0, 0, 0), 0.25);
    grid.set(PointIndex::new3d(0, 0, 1), -1.0);
    grid.set(PointIndex::new3d(0, 0, 2), -0.50);

    assert!(space.adjust(&grid).is_ok());

    assert_eq!(space.coordinate(0), &1.25);
    assert_eq!(space.coordinate(1), &0.0);
    assert_eq!(space.coordinate(2), &0.50);
}

#[test]
fn test_adjustable_space_kind_quaternion() {
    let mut space = AdjustableSpaceKind::Quaternion(AdjustableQuaternionSpace::new(1, [0.0; 4]));
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.25);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.25);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.25);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.25);

    assert!(space.update(&grid).is_ok());
    assert!(space.adjust(&grid).is_ok());

    assert_eq!(space.coordinate(0), &0.5); // 0.25 + 0.25
    assert_eq!(space.coordinate(1), &0.5);
    assert_eq!(space.coordinate(2), &0.5);
    assert_eq!(space.coordinate(3), &0.5);
}

#[test]
fn test_adjustable_space_kind_ids_and_dimensions() {
    let geo = AdjustableGeoSpace::new(101, 10.0, 20.0, 30.0);
    let ned = AdjustableNedSpace::new(102, 1.0, 2.0, 3.0);
    let quat = AdjustableQuaternionSpace::new(103, [0.25, 0.25, 0.25, 0.25]);

    let g = AdjustableSpaceKind::Geo(geo);
    let n = AdjustableSpaceKind::Ned(ned);
    let q = AdjustableSpaceKind::Quaternion(quat);

    assert_eq!(g.id(), 101);
    assert_eq!(g.dimension(), 3);
    assert_eq!(g.coordinate(1), &20.0);

    assert_eq!(n.id(), 102);
    assert_eq!(n.dimension(), 3);
    assert_eq!(n.coordinate(2), &3.0);

    assert_eq!(q.id(), 103);
    assert_eq!(q.dimension(), 4);
    assert_eq!(q.coordinate(3), &0.25);
}

#[test]
fn test_adjustable_space_kind_identifiable() {
    let euc = AdjustableEuclideanSpace::new(42, 1.0, 2.0, 3.0);
    let kind = AdjustableSpaceKind::Euclidean(euc);
    assert_eq!(kind.id(), 42);
}

#[test]
fn test_adjustable_space_kind_coordinate_trait() {
    let ecef = AdjustableEcefSpace::new(7, 4.0, 5.0, 6.0);
    let kind = AdjustableSpaceKind::Ecef(ecef);

    assert_eq!(kind.dimension(), 3);
    assert_eq!(kind.coordinate(0), &4.0);
    assert_eq!(kind.coordinate(1), &5.0);
    assert_eq!(kind.coordinate(2), &6.0);
}

#[test]
fn test_adjustable_space_kind_display_trait() {
    let ned = AdjustableNedSpace::new(99, 100.0, 200.0, 300.0);
    let kind = AdjustableSpaceKind::Ned(ned);

    let output = format!("{kind}");

    dbg!(&output);
    assert!(output.contains("AdjustableNedSpace"));
    assert!(output.contains("N=100.000"));
}

#[test]
fn test_adjustable_space_kind_debug_trait() {
    let geo = AdjustableGeoSpace::new(12, 1.1, 2.2, 3.3);
    let kind = AdjustableSpaceKind::Geo(geo);

    let debug = format!("{:?}", kind);
    assert!(debug.contains("AdjustableGeoSpace"));
}

#[test]
fn test_adjustable_space_kind_partial_eq_trait() {
    let q1 = AdjustableQuaternionSpace::new(1, [1.0, 0.0, 0.0, 0.0]);
    let q2 = AdjustableQuaternionSpace::new(1, [1.0, 0.0, 0.0, 0.0]);

    let kind1 = AdjustableSpaceKind::Quaternion(q1);
    let kind2 = AdjustableSpaceKind::Quaternion(q2);

    assert_eq!(kind1, kind2);
}

#[test]
fn test_adjustable_space_kind_clone_trait() {
    let geo = AdjustableGeoSpace::new(8, 10.0, 20.0, 30.0);
    let kind = AdjustableSpaceKind::Geo(geo);
    let cloned = kind.clone();

    assert_eq!(kind, cloned);
}
