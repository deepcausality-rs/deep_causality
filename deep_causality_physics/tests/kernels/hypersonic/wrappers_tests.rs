/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::EffectValue;
use deep_causality_physics::{
    ElectronTemperature, EquilibriumConstant, IonizationFraction, NO_IONIZATION_ENERGY_EV,
    THETA_VIB_N2, Temperature, VibrationalTemperature, arrhenius_rate,
    dissociation_equilibrium_fraction, electron_density, electron_impact_ionization_n_rate,
    electron_impact_ionization_o_rate, finite_rate_ionization_fixed_point,
    n2_dissociation_equilibrium, no_dissociative_recombination_rate, o2_dissociation_equilibrium,
    park2t_ionization_surrogate, rankine_hugoniot_temperature, recovery_temperature,
    saha_ionization_fraction, vibrational_relaxation,
};

#[test]
fn test_vibrational_relaxation_wrapper() {
    let t_ve = VibrationalTemperature::<f64>::new(300.0).unwrap();
    let t_tr = Temperature::<f64>::new(7000.0).unwrap();
    let ok = vibrational_relaxation(t_ve, t_tr, 1.0, 14.0, THETA_VIB_N2, 1.0);
    assert!(ok.is_ok());
    if let EffectValue::Value(v) = ok.value() {
        assert!((v.value() - 7000.0).abs() < 1.0);
    } else {
        panic!("expected Value");
    }
    // Error arm: zero pressure.
    let err = vibrational_relaxation(t_ve, t_tr, 0.0, 14.0, THETA_VIB_N2, 1.0);
    assert!(!err.is_ok());
}

#[test]
fn test_arrhenius_rate_wrapper() {
    let t = Temperature::<f64>::new(7000.0).unwrap();
    let ok = arrhenius_rate(t, 9.03e9, 0.5, 32400.0);
    assert!(ok.is_ok());
    if let EffectValue::Value(v) = ok.value() {
        assert!(v.value() > 0.0);
    } else {
        panic!("expected Value");
    }
    let err = arrhenius_rate(Temperature::<f64>::new(0.0).unwrap(), 1.0, 0.0, 100.0);
    assert!(!err.is_ok());
}

#[test]
fn test_saha_wrapper() {
    let t = Temperature::<f64>::new(8000.0).unwrap();
    let ok = saha_ionization_fraction(t, 1.0e22, NO_IONIZATION_ENERGY_EV, 2.0);
    assert!(ok.is_ok());
    let err = saha_ionization_fraction(t, 0.0, NO_IONIZATION_ENERGY_EV, 2.0);
    assert!(!err.is_ok());
}

#[test]
fn test_surrogate_wrapper() {
    let t = Temperature::<f64>::new(8000.0).unwrap();
    let ok = park2t_ionization_surrogate(t, 1.0e22);
    assert!(ok.is_ok());
    if let EffectValue::Value(v) = ok.value() {
        assert!(v.value() > 0.0);
    } else {
        panic!("expected Value");
    }
    let err = park2t_ionization_surrogate(t, 0.0);
    assert!(!err.is_ok());
}

#[test]
fn test_electron_density_wrapper() {
    let alpha = IonizationFraction::<f64>::new(0.01).unwrap();
    let ok = electron_density(alpha, 1.0e22);
    assert!(ok.is_ok());
    let err = electron_density(alpha, -1.0);
    assert!(!err.is_ok());
}

#[test]
fn test_rankine_hugoniot_wrapper() {
    let t_inf = Temperature::<f64>::new(200.0).unwrap();
    let ok = rankine_hugoniot_temperature(t_inf, 25.0, 1.4);
    assert!(ok.is_ok());
    if let EffectValue::Value(v) = ok.value() {
        assert!(v.value() > 1.0e4);
    } else {
        panic!("expected Value");
    }
    let err = rankine_hugoniot_temperature(t_inf, 0.5, 1.4);
    assert!(!err.is_ok());
}

#[test]
fn test_recovery_temperature_wrapper() {
    let t_post = Temperature::<f64>::new(24500.0).unwrap();
    let ok = recovery_temperature(t_post, 2000.0, 1004.0);
    assert!(ok.is_ok());
    let err = recovery_temperature(Temperature::<f64>::new(300.0).unwrap(), 2000.0, 1004.0);
    assert!(!err.is_ok());
}

#[test]
fn test_finite_rate_network_wrappers() {
    let te = ElectronTemperature::<f64>::new(6_000.0).unwrap();
    assert!(no_dissociative_recombination_rate(te).is_ok());
    assert!(electron_impact_ionization_n_rate(te).is_ok());
    assert!(electron_impact_ionization_o_rate(te).is_ok());

    let t = Temperature::<f64>::new(6_000.0).unwrap();
    assert!(n2_dissociation_equilibrium(t).is_ok());
    assert!(o2_dissociation_equilibrium(t).is_ok());

    let k = EquilibriumConstant::<f64>::new(1.0e-8).unwrap();
    assert!(dissociation_equilibrium_fraction(k, 1.0e-6).is_ok());
    assert!(!dissociation_equilibrium_fraction(k, 0.0).is_ok());

    assert!(finite_rate_ionization_fixed_point(1.0e-10_f64, 0.0, 1.0e12).is_ok());
    assert!(!finite_rate_ionization_fixed_point(1.0e-10_f64, 0.0, 0.0).is_ok());
}
