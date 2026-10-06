/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_context::utils_test::test_utils_array_grid::{
    ARRAY_TYPES, get_slots_array_grid,
};
use deep_causality_context::*;
use deep_causality_data_structures::ArrayType;

/// Grid values for positions 0 to 3, distinct so that a misplaced read cannot pass.
const GRID: [f64; 4] = [40.0, 10.0, 20.0, 30.0];

/// The grid position and the error-message name of each coordinate.
const AXES: [(usize, &str); 4] = [(0, "T"), (1, "X"), (2, "Y"), (3, "Z")];

/// The event `(t, x, y, z) = (4, 1, 2, 3)`.
fn spacetime() -> GalileanSpacetime<f64> {
    GalileanSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second)
}

fn t_x_y_z(s: &GalileanSpacetime<f64>) -> [f64; 4] {
    [s.time_unit(), s.x(), s.y(), s.z()]
}

#[test]
fn test_update_replaces_t_x_y_z_with_grid_positions_0_to_3() {
    for array_type in ARRAY_TYPES {
        let mut s = spacetime();
        let grid = get_slots_array_grid(array_type, GRID);

        assert!(s.update(&grid).is_ok(), "{array_type:?}");
        assert_eq!(t_x_y_z(&s), GRID, "{array_type:?}");
    }
}

#[test]
fn test_adjust_adds_grid_positions_0_to_3_to_t_x_y_z() {
    for array_type in ARRAY_TYPES {
        let mut s = spacetime();
        let grid = get_slots_array_grid(array_type, GRID);

        assert!(s.adjust(&grid).is_ok(), "{array_type:?}");
        assert_eq!(t_x_y_z(&s), [44.0, 11.0, 22.0, 33.0], "{array_type:?}");
    }
}

#[test]
fn test_update_and_adjust_reject_a_non_finite_coordinate_and_keep_the_event() {
    for array_type in ARRAY_TYPES {
        for (position, axis) in AXES {
            for non_finite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut values = GRID;
                values[position] = non_finite;
                let grid = get_slots_array_grid(array_type, values);
                let mut s = spacetime();

                let err = s.update(&grid).unwrap_err().to_string();
                assert!(
                    err.contains(&format!("new {axis} is not a finite value")),
                    "{array_type:?}: {err}"
                );
                assert_eq!(t_x_y_z(&s), [4.0, 1.0, 2.0, 3.0], "{array_type:?}");

                let err = s.adjust(&grid).unwrap_err().to_string();
                assert!(
                    err.contains(&format!("adjusted {axis} is not a finite value")),
                    "{array_type:?}: {err}"
                );
                assert_eq!(t_x_y_z(&s), [4.0, 1.0, 2.0, 3.0], "{array_type:?}");
            }
        }
    }
}

#[test]
fn test_adjust_rejects_a_finite_value_whose_sum_overflows() {
    for (position, axis) in AXES {
        let mut s =
            GalileanSpacetime::new(1, f64::MAX, f64::MAX, f64::MAX, f64::MAX, TimeScale::Second);
        let mut values = [0.0; 4];
        values[position] = f64::MAX;
        let grid = get_slots_array_grid(ArrayType::Array3D, values);

        let err = s.adjust(&grid).unwrap_err().to_string();
        assert!(
            err.contains(&format!("adjusted {axis} is not a finite value")),
            "{err}"
        );
    }
}
