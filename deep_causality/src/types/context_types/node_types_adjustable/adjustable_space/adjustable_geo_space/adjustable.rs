use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;
use crate::errors::{AdjustmentError, UpdateError};
use crate::prelude::{Adjustable, AdjustableGeoSpace};

impl Adjustable<f64> for AdjustableGeoSpace {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
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

        // Replace the internal data with the new data
        self.lat = new_lat;
        self.lon = new_lon;
        self.alt = new_alt;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
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

        // Check if the adjusted data are safe to update i.e. not greater than max f64 value
        if adjusted_lat > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new lat data exceeds max f64 value ".into(),
            ));
        }

        if adjusted_lon > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new lon data exceeds max f64 value ".into(),
            ));
        }

        if adjusted_alt > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new alt data exceeds max f64 value ".into(),
            ));
        }

        // Update the internal data with the adjusted data
        self.lat = adjusted_lat;
        self.lon = adjusted_lon;
        self.alt = adjusted_alt;

        Ok(())
    }
}
