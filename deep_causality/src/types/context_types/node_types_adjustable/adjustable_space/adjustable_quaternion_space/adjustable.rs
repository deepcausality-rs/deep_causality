use crate::errors::{AdjustmentError, UpdateError};
use crate::prelude::{Adjustable, AdjustableQuaternionSpace};
use dcl_data_structures::grid_type::ArrayGrid;
use dcl_data_structures::prelude::PointIndex;

impl Adjustable<f64> for AdjustableQuaternionSpace {
    fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), UpdateError> {
        // Create a 4D PointIndex for each of the updated x,y,z coordinates
        let p1 = PointIndex::new4d(0, 0, 0, 0);
        let p2 = PointIndex::new4d(0, 0, 0, 1);
        let p3 = PointIndex::new4d(0, 0, 0, 2);
        let p4 = PointIndex::new4d(0, 0, 0, 3);

        // The quaternion representing the rotation in [w, x, y, z] order

        // Get the data at the index position
        let new_w = array_grid.get(p1);
        let new_x = array_grid.get(p2);
        let new_y = array_grid.get(p3);
        let new_z = array_grid.get(p4);

        // Update the internal data
        self.quat[0] = new_w;
        self.quat[1] = new_x;
        self.quat[2] = new_y;
        self.quat[3] = new_z;

        Ok(())
    }

    fn adjust<const W: usize, const H: usize, const D: usize, const C: usize>(
        &mut self,
        array_grid: &ArrayGrid<f64, W, H, D, C>,
    ) -> Result<(), AdjustmentError> {
        // Create a 4D PointIndex for each of the updated x,y,z, t quaternion coordinates
        let p1 = PointIndex::new4d(0, 0, 0, 0);
        let p2 = PointIndex::new4d(0, 0, 0, 1);
        let p3 = PointIndex::new4d(0, 0, 0, 2);
        let p4 = PointIndex::new4d(0, 0, 0, 3);

        // Get the data at the index position
        let new_w = array_grid.get(p1);
        let new_x = array_grid.get(p2);
        let new_y = array_grid.get(p3);
        let new_z = array_grid.get(p4);

        // Calculate the adjusted data by adding the new data to the current data
        let adjusted_w = self.quat[0] + new_w;
        let adjusted_x = self.quat[1] + new_x;
        let adjusted_y = self.quat[2] + new_y;
        let adjusted_z = self.quat[3] + new_z;

        // Check if the adjusted data are safe to update i.e. not greater than max f64 value
        if adjusted_w > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new W data exceeds max f64 value ".into(),
            ));
        }

        if adjusted_x > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new X data exceeds max f64 value ".into(),
            ));
        }

        if adjusted_y > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new Y data exceeds max f64 value ".into(),
            ));
        }

        if adjusted_z > f64::MAX {
            return Err(AdjustmentError(
                "Adjustment failed, new Z data exceeds max f64 value ".into(),
            ));
        }

        // Update the internal data
        self.quat[0] = adjusted_w;
        self.quat[1] = adjusted_x;
        self.quat[2] = adjusted_y;
        self.quat[3] = adjusted_z;

        Ok(())
    }
}
