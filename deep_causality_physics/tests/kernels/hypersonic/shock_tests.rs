/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Temperature, rankine_hugoniot_temperature_kernel, recovery_temperature_kernel,
};

#[test]
fn test_rankine_hugoniot_reaches_ionization_band_at_mach25() {
    // Gate (iii): the mandatory RH jump lands peak T_post in the ~10⁴ K band at M ≈ 25,
    // not the cold (~200 K) freestream / isentropic value.
    let t_inf = Temperature::<f64>::new(200.0).unwrap();
    let t_post = rankine_hugoniot_temperature_kernel(t_inf, 25.0, 1.4).unwrap();
    assert!(t_post.value() > 1.0e4, "T_post = {}", t_post.value());
    assert!(t_post.value() < 1.0e5);
}

#[test]
fn test_rankine_hugoniot_identity_at_mach1() {
    // A Mach-1 "shock" is no jump: T_post = T_inf.
    let t_inf = Temperature::<f64>::new(300.0).unwrap();
    let t_post = rankine_hugoniot_temperature_kernel(t_inf, 1.0, 1.4).unwrap();
    assert!((t_post.value() - 300.0).abs() < 1e-9);
}

#[test]
fn test_rankine_hugoniot_rejects_bad_inputs() {
    let t_inf = Temperature::<f64>::new(200.0).unwrap();
    assert!(rankine_hugoniot_temperature_kernel(t_inf, 0.5, 1.4).is_err()); // M < 1
    assert!(rankine_hugoniot_temperature_kernel(t_inf, 25.0, 1.0).is_err()); // γ ≤ 1
}

#[test]
fn test_recovery_temperature_subtracts_kinetic_term() {
    // T_tr = T_post − ½|u|²/c_p.
    let t_post = Temperature::<f64>::new(24500.0).unwrap();
    let t_tr = recovery_temperature_kernel(t_post, 2000.0, 1004.0).unwrap();
    let expected = 24500.0 - 0.5 * 2000.0 * 2000.0 / 1004.0;
    assert!((t_tr.value() - expected).abs() < 1e-6);
}

#[test]
fn test_recovery_temperature_overcooling_errors() {
    // Subtracting more enthalpy than available drives T below 0 ⇒ rejected.
    let t_post = Temperature::<f64>::new(300.0).unwrap();
    assert!(recovery_temperature_kernel(t_post, 2000.0, 1004.0).is_err());
}

#[test]
fn test_recovery_temperature_rejects_bad_cp() {
    let t_post = Temperature::<f64>::new(24500.0).unwrap();
    assert!(recovery_temperature_kernel(t_post, 2000.0, 0.0).is_err());
}
