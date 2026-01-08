/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{PointIndex, PointIndexType};

#[test]
fn test_point_index_type_values() {
    assert_eq!(PointIndexType::OneD as u8, 0);
    assert_eq!(PointIndexType::TwoD as u8, 1);
    assert_eq!(PointIndexType::ThreeD as u8, 2);
    assert_eq!(PointIndexType::FourD as u8, 3);
}

#[test]
fn test_point_index_1d_creation() {
    let point = PointIndex::new1d(5);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 0);
    assert_eq!(point.z, 0);
    assert_eq!(point.t, 0);
    assert!(matches!(point.point_type(), PointIndexType::OneD));
}

#[test]
fn test_point_index_2d_creation() {
    let point = PointIndex::new2d(5, 10);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 10);
    assert_eq!(point.z, 0);
    assert_eq!(point.t, 0);
    assert!(matches!(point.point_type(), PointIndexType::TwoD));
}

#[test]
fn test_point_index_3d_creation() {
    let point = PointIndex::new3d(5, 10, 15);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 10);
    assert_eq!(point.z, 15);
    assert_eq!(point.t, 0);
    assert!(matches!(point.point_type(), PointIndexType::ThreeD));
}

#[test]
fn test_point_index_4d_creation() {
    let point = PointIndex::new4d(5, 10, 15, 20);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 10);
    assert_eq!(point.z, 15);
    assert_eq!(point.t, 20);
    assert!(matches!(point.point_type(), PointIndexType::FourD));
}

#[test]
fn test_point_index_copy_clone() {
    let point = PointIndex::new4d(1, 2, 3, 4);

    // Test Copy
    let copied = point;
    assert_eq!(copied.x, 1);
    assert_eq!(point.x, 1); // Original still accessible due to Copy

    // Test Clone
    let cloned = point;
    assert_eq!(cloned.x, 1);
    assert_eq!(point.x, 1);
}

#[test]
fn test_point_index_debug() {
    let point_1d = PointIndex::new1d(5);
    let point_2d = PointIndex::new2d(5, 10);
    let point_3d = PointIndex::new3d(5, 10, 15);
    let point_4d = PointIndex::new4d(5, 10, 15, 20);

    assert!(format!("{point_1d:?}").contains("PointIndex"));
    assert!(format!("{point_2d:?}").contains("PointIndex"));
    assert!(format!("{point_3d:?}").contains("PointIndex"));
    assert!(format!("{point_4d:?}").contains("PointIndex"));
}

#[test]
fn test_point_index_display() {
    let point_1d = PointIndex::new1d(5);
    let point_2d = PointIndex::new2d(5, 10);
    let point_3d = PointIndex::new3d(5, 10, 15);
    let point_4d = PointIndex::new4d(5, 10, 15, 20);

    assert_eq!(format!("{point_1d}"), "(x:5)");
    assert_eq!(format!("{point_2d}"), "(x:5, y:10)");
    assert_eq!(format!("{point_3d}"), "(x:5, y:10, z:15)");
    assert_eq!(format!("{point_4d}"), "(x:5, y:10, z:15, t:20)");
}

#[test]
fn test_point_index_edge_cases() {
    // Test with zero values
    let point = PointIndex::new4d(0, 0, 0, 0);
    assert_eq!(format!("{point}"), "(x:0, y:0, z:0, t:0)");

    // Test with max usize values
    let max = usize::MAX;
    let point = PointIndex::new4d(max, max, max, max);
    assert_eq!(point.x, max);
    assert_eq!(point.y, max);
    assert_eq!(point.z, max);
    assert_eq!(point.t, max);
}

#[test]
fn test_point_type_consistency() {
    let point_1d = PointIndex::new1d(5);
    let point_2d = PointIndex::new2d(5, 10);
    let point_3d = PointIndex::new3d(5, 10, 15);
    let point_4d = PointIndex::new4d(5, 10, 15, 20);

    // Test that point types remain consistent after operations
    // `PointIndex` implements the `Copy` trait
    let cloned_1d = point_1d;
    let cloned_2d = point_2d;
    let cloned_3d = point_3d;
    let cloned_4d = point_4d;

    assert!(matches!(cloned_1d.point_type(), PointIndexType::OneD));
    assert!(matches!(cloned_2d.point_type(), PointIndexType::TwoD));
    assert!(matches!(cloned_3d.point_type(), PointIndexType::ThreeD));
    assert!(matches!(cloned_4d.point_type(), PointIndexType::FourD));
}
