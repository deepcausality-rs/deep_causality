/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::{AdjustmentError, UpdateError};
use crate::{Adjustable, GeoSpace};
use deep_causality_algebra::RealField;
use deep_causality_data_structures::{ArrayGrid, PointIndex};
use deep_causality_num::FromPrimitive;

// `Default` is not implied by `RealField`; `ArrayGrid<T, ..>` requires it to initialise
// its backing array, so it is bounded here rather than on the struct.
/// Grid positions 0, 1 and 2 hold latitude, longitude and altitude. A result with a latitude
/// outside `[−90°, 90°]` is refused and the position is unchanged: latitude and longitude are a
/// chart on the ellipsoid, not a vector space, and a latitude past a pole names no point.
impl<R: RealField + Default + FromPrimitive> Adjustable<R> for GeoSpace<R> {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 3D PointIndex for each of the updated x,y,z coordinates
        let p1 = PointIndex::new3d(0, 0, 0);
        let p2 = PointIndex::new3d(0, 0, 1);
        let p3 = PointIndex::new3d(0, 0, 2);

        // Get the data at the index position from the array grid
        // - `id`: A unique numeric identifier for the location (e.g., sensor ID, region ID)
        // - `lat`: Latitude in degrees (positive north, negative south)
        // - `lon`: Longitude in degrees (positive east, negative west)
        // - `alt`: Altitude in meters above the WGS84 ellipsoid (not above sea level)
        let new_lat = array_grid.get(p1);
        let new_lon = array_grid.get(p2);
        let new_alt = array_grid.get(p3);

        // Reject non-finite replacement coordinates (NaN, ±inf)
        if !new_lat.is_finite() {
            return Err(UpdateError(
                "Update failed, new lat value is not finite".into(),
            ));
        }

        if !new_lon.is_finite() {
            return Err(UpdateError(
                "Update failed, new lon value is not finite".into(),
            ));
        }

        if !new_alt.is_finite() {
            return Err(UpdateError(
                "Update failed, new alt value is not finite".into(),
            ));
        }

        super::check(new_lat, new_lon, new_alt).map_err(|e| UpdateError(e.0))?;

        // Replace the internal data with the new data
        self.lat = new_lat;
        self.lon = new_lon;
        self.alt = new_alt;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<R, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Create a 3D PointIndex for each of the updated x,y,z coordinates
        let p1 = PointIndex::new3d(0, 0, 0);
        let p2 = PointIndex::new3d(0, 0, 1);
        let p3 = PointIndex::new3d(0, 0, 2);

        // Get the data at the index position from the array grid
        // - `id`: A unique numeric identifier for the location (e.g., sensor ID, region ID)
        // - `lat`: Latitude in degrees (positive north, negative south)
        // - `lon`: Longitude in degrees (positive east, negative west)
        // - `alt`: Altitude in meters above the WGS84 ellipsoid (not above sea level)
        let new_lat = array_grid.get(p1);
        let new_lon = array_grid.get(p2);
        let new_alt = array_grid.get(p3);

        // Calculate the adjusted data by adding the new data to the current data
        let adjusted_lat = self.lat + new_lat;
        let adjusted_lon = self.lon + new_lon;
        let adjusted_alt = self.alt + new_alt;

        // Reject non-finite adjusted coordinates (NaN, ±inf)
        if !adjusted_lat.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new lat value is not finite".into(),
            ));
        }

        if !adjusted_lon.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new lon value is not finite".into(),
            ));
        }

        if !adjusted_alt.is_finite() {
            return Err(AdjustmentError(
                "Adjustment failed, new alt is not finite".into(),
            ));
        }

        super::check(adjusted_lat, adjusted_lon, adjusted_alt).map_err(|e| AdjustmentError(e.0))?;

        // Update the internal data with the adjusted data
        self.lat = adjusted_lat;
        self.lon = adjusted_lon;
        self.alt = adjusted_alt;

        Ok(())
    }
}
