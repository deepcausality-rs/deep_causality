/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    IonizationFraction, NO_IONIZATION_ENERGY_EV, Temperature, electron_density_kernel,
    park2t_ionization_surrogate_kernel, saha_ionization_fraction_kernel,
};

#[test]
fn test_saha_fraction_in_unit_interval() {
    let alpha = saha_ionization_fraction_kernel(
        Temperature::<f64>::new(8000.0).unwrap(),
        1.0e22,
        NO_IONIZATION_ENERGY_EV,
        2.0,
    )
    .unwrap();
    assert!(alpha.value() >= 0.0 && alpha.value() <= 1.0);
}

#[test]
fn test_saha_fraction_monotonic_in_temperature() {
    // Higher temperature ⇒ more ionization at fixed density.
    let lo = saha_ionization_fraction_kernel(
        Temperature::<f64>::new(6000.0).unwrap(),
        1.0e22,
        NO_IONIZATION_ENERGY_EV,
        2.0,
    )
    .unwrap();
    let hi = saha_ionization_fraction_kernel(
        Temperature::<f64>::new(10000.0).unwrap(),
        1.0e22,
        NO_IONIZATION_ENERGY_EV,
        2.0,
    )
    .unwrap();
    assert!(hi.value() > lo.value());
}

#[test]
fn test_surrogate_is_saha_for_no_channel() {
    // The Tier-A surrogate is the Saha equilibrium for the NO channel (E ≈ 9.26 eV, g = 2).
    let t = Temperature::<f64>::new(8000.0).unwrap();
    let surrogate = park2t_ionization_surrogate_kernel(t, 1.0e22).unwrap();
    let saha = saha_ionization_fraction_kernel(t, 1.0e22, NO_IONIZATION_ENERGY_EV, 2.0).unwrap();
    assert_eq!(surrogate.value(), saha.value());
}

#[test]
fn test_surrogate_produces_electrons_at_reentry() {
    // Gate (vi): the ionized-species target is strictly positive at reentry conditions.
    let t = Temperature::<f64>::new(8000.0).unwrap();
    let alpha = park2t_ionization_surrogate_kernel(t, 1.0e22).unwrap();
    assert!(alpha.value() > 0.0);
    let n_e = electron_density_kernel(alpha, 1.0e22).unwrap();
    assert!(n_e.value() > 0.0);
}

#[test]
fn test_electron_density_is_alpha_times_density() {
    let alpha = IonizationFraction::<f64>::new(0.01).unwrap();
    let n_e = electron_density_kernel(alpha, 1.0e22).unwrap();
    assert!((n_e.value() - 1.0e20).abs() / 1.0e20 < 1e-12);
}

#[test]
fn test_saha_rejects_bad_inputs() {
    let t = Temperature::<f64>::new(8000.0).unwrap();
    assert!(saha_ionization_fraction_kernel(t, 0.0, NO_IONIZATION_ENERGY_EV, 2.0).is_err());
    assert!(saha_ionization_fraction_kernel(t, 1.0e22, 0.0, 2.0).is_err());
    assert!(saha_ionization_fraction_kernel(t, 1.0e22, NO_IONIZATION_ENERGY_EV, 0.0).is_err());
}

#[test]
fn test_electron_density_rejects_negative_density() {
    let alpha = IonizationFraction::<f64>::new(0.01).unwrap();
    assert!(electron_density_kernel(alpha, -1.0).is_err());
}
