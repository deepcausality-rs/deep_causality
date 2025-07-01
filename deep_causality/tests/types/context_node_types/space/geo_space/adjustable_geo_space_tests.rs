/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::*;

#[test]
fn test_adjustable_geo_space_display_and_id() {
    let geo = GeoSpace::new(1, 52.52, 13.40, 34.0);
    let id = geo.id();
    assert_eq!(id, 1);
    assert!(format!("{geo}").contains("GeoSpace(id=1"));
    assert!(format!("{geo}").contains("lat=52.52"));
    assert!(format!("{geo}").contains("lon=13.40"));
    assert!(format!("{geo}").contains("alt=34.00"));
}

#[test]
fn test_geo_space_update_success() {
    let mut geo = GeoSpace::new(1, 0.0, 0.0, 0.0);
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
    let mut geo = GeoSpace::new(1, 99.0, 99.0, 99.0);
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
fn test_geo_space_update_lat_fails_on_nan() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.update(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_update_lon_fails_on_nan() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.update(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_update_alt_fails_on_nan() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // alt adjustment

    let result = geo.update(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_success() {
    let mut geo = GeoSpace::new(1, 50.0, 10.0, 100.0);
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
fn test_geo_space_adjust_lat_fails_on_nan() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_lat_fails_on_inf() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::INFINITY); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_lon_fails_on_nan() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_lon_fails_on_inf() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_alt_fails_on_nan() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_alt_fails_on_inf() {
    let mut geo = GeoSpace::new(1, f64::MAX, f64::MAX, f64::MAX);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), f64::INFINITY); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}
