// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::*;

#[test]
fn test_construction_and_accessors() {
    let space = AdjustableEuclideanSpace::new(1, 1.0, 2.0, 3.0);
    assert_eq!(space.id(), 1);
    assert_eq!(space.x(), 1.0);
    assert_eq!(space.y(), 2.0);
    assert_eq!(space.z(), 3.0);
}

#[test]
fn test_coordinate_trait() {
    let space = AdjustableEuclideanSpace::new(0, 1.1, 2.2, 3.3);
    assert_eq!(space.dimension(), 3);
    assert_eq!(*space.coordinate(0), 1.1);
    assert_eq!(*space.coordinate(1), 2.2);
    assert_eq!(*space.coordinate(2), 3.3);
}

#[test]
#[should_panic(expected = "AdjustableEuclideanSpace: index out of bounds")]
fn test_coordinate_index_out_of_bounds() {
    let space = AdjustableEuclideanSpace::new(0, 1.0, 2.0, 3.0);
    let _ = space.coordinate(3);
}

#[test]
fn test_display_trait() {
    let space = AdjustableEuclideanSpace::new(5, 1.234, 5.678, 9.876);
    let output = format!("{}", space);
    assert!(output.contains("EuclideanSpace(id=5"));
    assert!(output.contains("x=1.234"));
    assert!(output.contains("y=5.678"));
    assert!(output.contains("z=9.876"));
}

#[test]
fn test_clone_and_eq() {
    let a = AdjustableEuclideanSpace::new(10, 1.0, 2.0, 3.0);
    let b = a.clone();
    let c = AdjustableEuclideanSpace::new(10, 1.0, 2.0, 3.0);
    let d = AdjustableEuclideanSpace::new(11, 1.0, 2.0, 3.0);
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_ne!(a, d);
}

#[test]
fn test_metric_trait_distance() {
    struct TestSpace(AdjustableEuclideanSpace);

    impl Metric<f64> for TestSpace {
        fn distance(&self, other: &Self) -> f64 {
            let dx = self.0.x() - other.0.x();
            let dy = self.0.y() - other.0.y();
            let dz = self.0.z() - other.0.z();
            (dx * dx + dy * dy + dz * dz).sqrt()
        }
    }

    let a = TestSpace(AdjustableEuclideanSpace::new(0, 0.0, 0.0, 0.0));
    let b = TestSpace(AdjustableEuclideanSpace::new(1, 3.0, 4.0, 0.0));
    assert_eq!(a.distance(&b), 5.0); // 3-4-5 triangle
}

#[test]
fn test_adjustable_trait_update_and_adjust() {
    let mut space = AdjustableEuclideanSpace::new(1, 1.0, 2.0, 3.0);

    // Use matching layout from your successful test suite
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 10.0); // x
    grid.set(PointIndex::new3d(0, 0, 1), 20.0); // y
    grid.set(PointIndex::new3d(0, 0, 2), 30.0); // z

    let update_result = space.update(&grid);
    assert!(update_result.is_ok());
    assert_eq!(space.x(), 10.0);
    assert_eq!(space.y(), 20.0);
    assert_eq!(space.z(), 30.0);

    let adjust_result = space.adjust(&grid);
    assert!(adjust_result.is_ok());
    assert_eq!(space.x(), 20.0);
    assert_eq!(space.y(), 40.0);
    assert_eq!(space.z(), 60.0);
}

#[test]
fn test_spatial_trait_marker() {
    fn assert_spatial<T: Spatial<f64>>() {}
    assert_spatial::<AdjustableEuclideanSpace>();
}
