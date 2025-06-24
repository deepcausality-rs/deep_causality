// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::*;

#[test]
fn test_adjustable_geo_space_display_and_id() {
    let geo = AdjustableGeoSpace::new(1, 52.52, 13.40, 34.0);
    let id = geo.id();
    assert_eq!(id, 1);
    assert_eq!(
        format!("{}", geo),
        "AdjustableGeoSpace(id=\"1\", lat=52.520000, lon=13.400000, alt=34m)"
    );
}

#[test]
fn test_adjustable_geo_space_distance() {
    let g1 = AdjustableGeoSpace::new(1, 52.520008, 13.404954, 34.0); // Berlin
    let g2 = AdjustableGeoSpace::new(2, 48.856613, 2.352222, 35.0); // Paris

    let d = g1.distance(&g2);
    let km = d / 1000.0;
    assert!(km > 875.0 && km < 885.0, "Distance was {:.2} km", km);
}

#[test]
fn test_geo_space_update() {
    let mut geo = AdjustableGeoSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 52.52); // lat
    grid.set(PointIndex::new3d(0, 0, 1), 13.40); // lon
    grid.set(PointIndex::new3d(0, 0, 2), 34.0); // alt

    let result = geo.update(&grid);
    assert!(result.is_ok());
    assert_eq!(geo.lat(), 52.52);
    assert_eq!(geo.lon(), 13.40);
    assert_eq!(geo.alt(), 34.0);
}

#[test]
fn test_geo_space_update_allows_zero_values() {
    let mut geo = AdjustableGeoSpace::new(1, 99.0, 99.0, 99.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt

    let result = geo.update(&grid);
    assert!(result.is_ok());
    assert_eq!(geo.lat(), 0.0);
    assert_eq!(geo.lon(), 0.0);
    assert_eq!(geo.alt(), 0.0);
}

#[test]
fn test_geo_space_adjust() {
    let mut geo = AdjustableGeoSpace::new(1, 50.0, 10.0, 100.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0); // lat delta
    grid.set(PointIndex::new3d(0, 0, 1), 2.0); // lon delta
    grid.set(PointIndex::new3d(0, 0, 2), 3.0); // alt delta

    let result = geo.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(geo.lat(), 51.0);
    assert_eq!(geo.lon(), 12.0);
    assert_eq!(geo.alt(), 103.0);
}

#[test]
fn test_geo_space_adjust_fails_on_overflow() {
    let mut geo = AdjustableGeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::MAX); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), f64::MAX); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), f64::MAX); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}
