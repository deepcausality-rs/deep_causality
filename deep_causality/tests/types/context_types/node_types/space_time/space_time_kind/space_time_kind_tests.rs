/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;
use std::fmt::Write;

#[test]
fn test_space_time_kind_variants_and_traits() {
    // Construct each variant
    let euclidean = EuclideanSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let lorentzian = LorentzianSpacetime::new(2, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let minkowski = MinkowskiSpacetime::new(3, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let tangent = TangentSpacetime::new(4, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);

    let variants = vec![
        SpaceTimeKind::Euclidean(euclidean),
        SpaceTimeKind::Lorentzian(lorentzian),
        SpaceTimeKind::Minkowski(minkowski),
        SpaceTimeKind::Tangent(tangent),
    ];

    for (i, variant) in variants.iter().enumerate() {
        // Identifiable
        assert_eq!(variant.id(), (i + 1) as u64);

        // Coordinate dimension
        assert_eq!(variant.dimension(), 4);

        // Coordinate access
        assert_eq!(*variant.coordinate(0).unwrap(), 1.0);
        assert_eq!(*variant.coordinate(1).unwrap(), 2.0);
        assert_eq!(*variant.coordinate(2).unwrap(), 3.0);
        assert_eq!(*variant.coordinate(3).unwrap(), 4.0);

        // Temporal
        assert_eq!(variant.time_unit(), 4.0);
        assert_eq!(variant.time_scale(), TimeScale::Second);

        // SpaceTemporal
        assert_eq!(*variant.t(), 4.0);

        // Display
        let mut out = String::new();
        write!(&mut out, "{}", variant).unwrap();
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
