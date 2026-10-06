/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::utils_test::test_utils_array_grid::{
    ARRAY_TYPES, get_slots_array_grid,
};
use deep_causality_context::*;
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_adjustable_geo_space_display_and_id() {
    let geo = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84).unwrap();
    let id = geo.id();
    assert_eq!(id, 1);
    assert!(format!("{geo}").contains("GeoSpace(id=1"));
    assert!(format!("{geo}").contains("lat=52.52"));
    assert!(format!("{geo}").contains("lon=13.40"));
    assert!(format!("{geo}").contains("alt=34.00"));
}

#[test]
fn test_geo_space_update_success() {
    let mut geo = GeoSpace::new(1, 0.0, 0.0, 0.0, VerticalDatum::WGS84).unwrap();
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
    let mut geo = GeoSpace::new(1, 45.0, 99.0, 99.0, VerticalDatum::WGS84).unwrap();
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
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.update(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_update_lon_fails_on_nan() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.update(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_update_alt_fails_on_nan() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // alt adjustment

    let result = geo.update(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_success() {
    let mut geo = GeoSpace::new(1, 50.0, 10.0, 100.0, VerticalDatum::WGS84).unwrap();
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
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_lat_fails_on_inf() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::INFINITY); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_lon_fails_on_nan() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_lon_fails_on_inf() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_alt_fails_on_nan() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

#[test]
fn test_geo_space_adjust_alt_fails_on_inf() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0); // lat adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0); // lon adjustment
    grid.set(PointIndex::new3d(0, 0, 2), f64::INFINITY); // alt adjustment

    let result = geo.adjust(&grid);
    assert!(result.is_err(), "Expected overflow to trigger an error");
}

fn grid(lat: f64, lon: f64, alt: f64) -> ArrayGrid<f64, 3, 3, 3, 1> {
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), lat);
    grid.set(PointIndex::new3d(0, 0, 1), lon);
    grid.set(PointIndex::new3d(0, 0, 2), alt);
    grid
}

#[test]
fn test_geo_space_update_refuses_a_latitude_past_a_pole_and_keeps_the_position() {
    let mut geo = GeoSpace::new(1, 10.0, 20.0, 30.0, VerticalDatum::WGS84).unwrap();
    for lat in [90.000_001, -90.000_001, 180.0] {
        let err = geo
            .update(&grid(lat, 0.0, 0.0))
            .expect_err("latitude accepted");
        assert!(err.to_string().contains("latitude"), "{err}");
        assert_eq!((geo.lat(), geo.lon(), geo.alt()), (10.0, 20.0, 30.0));
    }
}

#[test]
fn test_geo_space_update_accepts_the_poles_and_any_longitude() {
    let mut geo = GeoSpace::new(1, 0.0, 0.0, 0.0, VerticalDatum::WGS84).unwrap();
    assert!(geo.update(&grid(90.0, 540.0, 0.0)).is_ok());
    assert_eq!((geo.lat(), geo.lon()), (90.0, 540.0));
    assert!(geo.update(&grid(-90.0, -720.0, 0.0)).is_ok());
    assert_eq!((geo.lat(), geo.lon()), (-90.0, -720.0));
}

#[test]
fn test_geo_space_adjust_refuses_a_result_past_a_pole_and_keeps_the_position() {
    // 89° + 2° is not a latitude: adding degrees past the pole names no point.
    let mut geo = GeoSpace::new(1, 89.0, 10.0, 100.0, VerticalDatum::WGS84).unwrap();
    let err = geo
        .adjust(&grid(2.0, 0.0, 0.0))
        .expect_err("latitude accepted");
    assert!(err.to_string().contains("latitude"), "{err}");
    assert_eq!((geo.lat(), geo.lon(), geo.alt()), (89.0, 10.0, 100.0));
}

#[test]
fn test_geo_space_adjust_reaches_a_pole_exactly() {
    let mut geo = GeoSpace::new(1, -89.0, 10.0, 100.0, VerticalDatum::WGS84).unwrap();
    assert!(geo.adjust(&grid(-1.0, 0.0, 0.0)).is_ok());
    assert_eq!(geo.lat(), -90.0);
}

#[test]
fn test_geo_space_update_replaces_lat_lon_alt_with_grid_positions_0_to_2_of_every_grid() {
    for array_type in ARRAY_TYPES {
        let mut geo = GeoSpace::new(1, 1.0, 2.0, 3.0, VerticalDatum::WGS84).unwrap();
        let grid = get_slots_array_grid(array_type, [10.0, 20.0, 30.0]);

        assert!(geo.update(&grid).is_ok(), "{array_type:?}");
        assert_eq!(
            [geo.lat(), geo.lon(), geo.alt()],
            [10.0, 20.0, 30.0],
            "{array_type:?}"
        );
    }
}

#[test]
fn test_geo_space_adjust_adds_grid_positions_0_to_2_of_every_grid_to_lat_lon_alt() {
    for array_type in ARRAY_TYPES {
        let mut geo = GeoSpace::new(1, 1.0, 2.0, 3.0, VerticalDatum::WGS84).unwrap();
        let grid = get_slots_array_grid(array_type, [10.0, 20.0, 30.0]);

        assert!(geo.adjust(&grid).is_ok(), "{array_type:?}");
        assert_eq!(
            [geo.lat(), geo.lon(), geo.alt()],
            [11.0, 22.0, 33.0],
            "{array_type:?}"
        );
    }
}
