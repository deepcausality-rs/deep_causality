/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::MetricFamily;

// =============================================================================
// Construction and equality
// =============================================================================

#[test]
fn test_flat_carries_no_parameters() {
    let a: MetricFamily<f64> = MetricFamily::Flat;
    let b: MetricFamily<f64> = MetricFamily::Flat;
    assert_eq!(a, b);
}

#[test]
fn test_schwarzschild_differs_by_mass() {
    let light = MetricFamily::Schwarzschild { mass: 1.0 };
    let heavy = MetricFamily::Schwarzschild { mass: 2.0 };

    assert_eq!(light, MetricFamily::Schwarzschild { mass: 1.0 });
    assert_ne!(light, heavy);
}

#[test]
fn test_kerr_differs_by_spin_at_equal_mass() {
    // Spin is the parameter that separates Kerr from Schwarzschild, so two Kerr metrics of equal
    // mass and unequal spin must not compare equal.
    let still = MetricFamily::Kerr {
        mass: 1.0,
        spin: 0.0,
    };
    let spinning = MetricFamily::Kerr {
        mass: 1.0,
        spin: 0.5,
    };

    assert_ne!(still, spinning);
}

#[test]
fn test_flrw_differs_by_each_parameter_independently() {
    let base = MetricFamily::Flrw {
        scale_factor: 1.0,
        curvature_k: 0.0,
    };
    let scaled = MetricFamily::Flrw {
        scale_factor: 2.0,
        curvature_k: 0.0,
    };
    let curved = MetricFamily::Flrw {
        scale_factor: 1.0,
        curvature_k: 1.0,
    };

    assert_ne!(base, scaled);
    assert_ne!(base, curved);
    assert_ne!(scaled, curved);
}

#[test]
fn test_families_of_different_form_are_distinct() {
    let flat: MetricFamily<f64> = MetricFamily::Flat;
    let schwarzschild = MetricFamily::Schwarzschild { mass: 1.0 };
    // A Kerr metric of zero spin is Schwarzschild as a tensor, but it is a different choice of
    // analytic form, and this type records the choice rather than the tensor.
    let kerr = MetricFamily::Kerr {
        mass: 1.0,
        spin: 0.0,
    };

    assert_ne!(flat, schwarzschild);
    assert_ne!(schwarzschild, kerr);
    assert_ne!(flat, kerr);
}

// =============================================================================
// The scalar is a parameter
// =============================================================================

#[test]
fn test_family_carries_any_scalar() {
    // No bound is placed on the parameter, so a family can be stated at whatever type the model
    // works in. f32 and f64 are two; nothing in the crate names either.
    let narrow = MetricFamily::Schwarzschild { mass: 1.0f32 };
    let wide = MetricFamily::Schwarzschild { mass: 1.0f64 };

    assert_eq!(narrow, MetricFamily::Schwarzschild { mass: 1.0f32 });
    assert_eq!(wide, MetricFamily::Schwarzschild { mass: 1.0f64 });
}

#[test]
fn test_family_is_copy() {
    let a = MetricFamily::Kerr {
        mass: 1.0,
        spin: 0.5,
    };
    let b = a;
    assert_eq!(a, b);
}

// =============================================================================
// Display
// =============================================================================

#[test]
fn test_display_names_the_form_and_its_parameters() {
    let flat: MetricFamily<f64> = MetricFamily::Flat;
    assert_eq!(format!("{flat}"), "Flat");

    let schwarzschild = MetricFamily::Schwarzschild { mass: 2.0 };
    assert_eq!(format!("{schwarzschild}"), "Schwarzschild(mass=2)");

    let kerr = MetricFamily::Kerr {
        mass: 2.0,
        spin: 0.5,
    };
    assert_eq!(format!("{kerr}"), "Kerr(mass=2, spin=0.5)");

    let flrw = MetricFamily::Flrw {
        scale_factor: 1.5,
        curvature_k: -1.0,
    };
    assert_eq!(format!("{flrw}"), "FLRW(a=1.5, k=-1)");
}

#[test]
fn test_debug_is_available() {
    let kerr = MetricFamily::Kerr {
        mass: 1.0,
        spin: 0.5,
    };
    let text = format!("{kerr:?}");
    assert!(text.contains("Kerr"));
    assert!(text.contains("mass"));
    assert!(text.contains("spin"));
}
