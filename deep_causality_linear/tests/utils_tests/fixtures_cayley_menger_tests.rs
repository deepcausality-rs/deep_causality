/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Cayley-Menger fixtures are the regression that keeps elimination pivoting. They are checked
//! against the geometry they describe, not against this crate.

use deep_causality_linear::utils_tests::fixtures_cayley_menger::*;

#[test]
fn test_tetrahedron_matrix_has_the_cayley_menger_shape() {
    let (d, r, c) = regular_unit_tetrahedron();
    assert_eq!((r, c), (5, 5));
    assert_eq!(d[0], 0.0, "the (0,0) entry is zero by construction");
    for j in 1..5 {
        assert_eq!(d[j], 1.0, "the border is one");
        assert_eq!(d[j * 5], 1.0, "and symmetric");
    }
    for i in 1..5 {
        assert_eq!(
            d[i * 5 + i],
            0.0,
            "the interior diagonal is zero: d(p, p) = 0"
        );
        for j in 1..5 {
            if i != j {
                assert_eq!(
                    d[i * 5 + j],
                    1.0,
                    "unit edges, so every squared distance is one"
                );
            }
        }
    }
}

#[test]
fn test_tetrahedron_matrix_is_symmetric() {
    let (d, _, _) = regular_unit_tetrahedron();
    for i in 0..5 {
        for j in 0..5 {
            assert_eq!(d[i * 5 + j], d[j * 5 + i], "asymmetric at ({i}, {j})");
        }
    }
}

#[test]
fn test_tetrahedron_volume_is_the_exact_value() {
    // vol^2 = det / 288 = 4 / 288 = 1/72, so vol = 1/sqrt(72) = sqrt(2)/12.
    let vol_sq = cm_determinant_to_volume_squared(TETRAHEDRON_CM_DETERMINANT, 5);
    assert!((vol_sq - 1.0 / 72.0).abs() < 1e-15, "vol^2 was {vol_sq}");
    let vol = vol_sq.sqrt();
    let exact = 2.0_f64.sqrt() / 12.0;
    assert!(
        (vol - exact).abs() < 1e-15,
        "vol {vol} against sqrt(2)/12 {exact}"
    );
    assert!((vol - TETRAHEDRON_VOLUME).abs() < 1e-15);
}

#[test]
fn test_right_triangle_area_is_one_half() {
    // A k=2 simplex: the divisor is (-1)^3 * 2^2 * (2!)^2 = -16.
    let area_sq = cm_determinant_to_volume_squared(RIGHT_TRIANGLE_CM_DETERMINANT, 4);
    assert!((area_sq - 0.25).abs() < 1e-15, "area^2 was {area_sq}");
    assert!((area_sq.sqrt() - 0.5).abs() < 1e-15);
}

#[test]
fn test_right_triangle_matrix_carries_the_hypotenuse() {
    let (d, _, _) = right_triangle();
    assert_eq!(d[0], 0.0);
    // Legs of length one, hypotenuse squared of two.
    assert_eq!(d[6], 1.0, "row 1, column 2");
    assert_eq!(d[7], 1.0, "row 1, column 3");
    assert_eq!(
        d[11], 2.0,
        "row 2, column 3: the squared hypotenuse, by Pythagoras"
    );
}

#[test]
fn test_the_divisor_matches_the_simplex_dimension() {
    // k = 3: (-1)^4 * 2^3 * (3!)^2 = 288.
    assert!((cm_determinant_to_volume_squared(288.0, 5) - 1.0).abs() < 1e-12);
    // k = 2: (-1)^3 * 2^2 * (2!)^2 = -16.
    assert!((cm_determinant_to_volume_squared(-16.0, 4) - 1.0).abs() < 1e-12);
    // k = 1, a segment: (-1)^2 * 2^1 * (1!)^2 = 2.
    assert!((cm_determinant_to_volume_squared(2.0, 3) - 1.0).abs() < 1e-12);
}

#[test]
fn test_an_unpivoted_elimination_would_return_zero_here() {
    // The defect this fixture guards against, stated as the property that makes it possible: the
    // leading entry is zero, so an elimination taking mat[i][i] as its pivot divides by zero or
    // bails, and returns zero for a matrix whose determinant is 4.
    let (d, _, _) = regular_unit_tetrahedron();
    assert_eq!(d[0], 0.0);
    assert_ne!(TETRAHEDRON_CM_DETERMINANT, 0.0);
}
