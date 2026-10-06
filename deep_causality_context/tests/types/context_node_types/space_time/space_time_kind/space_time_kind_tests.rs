/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::*;
use std::fmt::Write;

#[test]
fn test_space_time_kind_variants_and_traits() {
    // Construct each variant
    // Arguments: id, x, y, z, t (time scale).
    let newtonian = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let minkowski = MinkowskiSpacetime::new(2, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let tangent = TangentSpacetime::new(3, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);
    let galilean = GalileanSpacetime::new(4, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let variants = [
        SpaceTimeKind::Newtonian(newtonian),
        SpaceTimeKind::Minkowski(minkowski),
        SpaceTimeKind::Tangent(tangent),
        SpaceTimeKind::Galilean(galilean),
    ];

    for (i, variant) in variants.iter().enumerate() {
        // Identifiable
        assert_eq!(variant.id(), (i + 1) as u64);

        // Coordinate dimension
        assert_eq!(variant.dimension(), 4);

        // Coordinate access, time first: 0 => t, 1 => x, 2 => y, 3 => z
        assert_eq!(*variant.coordinate(0).unwrap(), 4.0);
        assert_eq!(*variant.coordinate(1).unwrap(), 1.0);
        assert_eq!(*variant.coordinate(2).unwrap(), 2.0);
        assert_eq!(*variant.coordinate(3).unwrap(), 3.0);

        // Temporal
        assert_eq!(variant.time_unit(), 4.0);
        assert_eq!(variant.time_scale(), TimeScale::Second);

        // SpaceTemporal
        assert_eq!(*variant.t(), 4.0);

        // Display
        let mut out = String::new();
        write!(&mut out, "{variant}").unwrap();
        assert!(out.contains("id") || out.contains("x"));
    }
}

#[test]
fn test_space_time_kind_coordinate_out_of_bounds() {
    let minkowski = MinkowskiSpacetime::new(99, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let variant = SpaceTimeKind::Minkowski(minkowski);

    let result = variant.coordinate(10);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Coordinate index out of bounds"));
}
