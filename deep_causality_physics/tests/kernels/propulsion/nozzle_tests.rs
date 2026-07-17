/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    FlowBranch, Pressure, Temperature, area_mach_ratio_kernel, inverse_area_mach_kernel,
    nozzle_exit_state_kernel,
};

#[test]
fn test_inverse_area_mach_published_table_values() {
    // NACA Report 1135 isentropic-flow tables, gamma = 1.4, A/A* = 2.0:
    // subsonic root M = 0.3059, supersonic root M = 2.1972.
    let m_sub = inverse_area_mach_kernel(2.0_f64, 1.4, FlowBranch::Subsonic).unwrap();
    let m_sup = inverse_area_mach_kernel(2.0_f64, 1.4, FlowBranch::Supersonic).unwrap();
    assert!((m_sub - 0.305904).abs() < 1e-4);
    assert!((m_sup - 2.197198).abs() < 1e-4);
}

#[test]
fn test_inverse_area_mach_round_trip_both_branches() {
    for &(mach, branch) in &[
        (0.25_f64, FlowBranch::Subsonic),
        (0.8, FlowBranch::Subsonic),
        (1.5, FlowBranch::Supersonic),
        (3.0, FlowBranch::Supersonic),
        (6.0, FlowBranch::Supersonic),
    ] {
        let ar = area_mach_ratio_kernel(mach, 1.4).unwrap();
        let m = inverse_area_mach_kernel(ar, 1.4, branch).unwrap();
        assert!(
            (m - mach).abs() < 1e-9,
            "round trip failed: mach {mach}, recovered {m}"
        );
    }
}

#[test]
fn test_inverse_area_mach_sonic_throat() {
    // A/A* = 1 is the throat: both branches return M = 1 exactly.
    assert_eq!(
        inverse_area_mach_kernel(1.0_f64, 1.4, FlowBranch::Subsonic).unwrap(),
        1.0
    );
    assert_eq!(
        inverse_area_mach_kernel(1.0_f64, 1.4, FlowBranch::Supersonic).unwrap(),
        1.0
    );
}

#[test]
fn test_inverse_area_mach_rejects_bad_domain() {
    assert!(inverse_area_mach_kernel(0.9_f64, 1.4, FlowBranch::Supersonic).is_err());
    assert!(inverse_area_mach_kernel(2.0_f64, 1.0, FlowBranch::Supersonic).is_err());
    assert!(inverse_area_mach_kernel(f64::NAN, 1.4, FlowBranch::Subsonic).is_err());
    // Absurd area ratio beyond the subsonic bracket (A/A* > ~5.8e8): rejected,
    // not silently converged to the bracket floor.
    assert!(inverse_area_mach_kernel(1.0e10_f64, 1.4, FlowBranch::Subsonic).is_err());
}

#[test]
fn test_nozzle_exit_state_expansion_ratio_four() {
    // Area-Mach solution at eps = 4, gamma = 1.4 (Anderson, "Modern
    // Compressible Flow," Ch. 5 quasi-1D relations): M_e = 2.9402,
    // p0/pe = 33.572, T0/Te = 2.7289.
    let p0 = 2.0e6_f64; // Pa
    let t0 = 3000.0; // K
    let r_s = 287.0; // J/(kg K)
    let state = nozzle_exit_state_kernel(
        Pressure::new(p0).unwrap(),
        Temperature::new(t0).unwrap(),
        4.0,
        1.4,
        r_s,
    )
    .unwrap();
    assert!((state.mach() - 2.940179).abs() < 1e-4);
    assert!((p0 / state.pressure().value() - 33.5717).abs() < 1e-2);
    assert!((t0 / state.temperature().value() - 2.72893).abs() < 1e-4);
    // Internal consistency: rho_e = p_e/(R_s T_e) and u_e = M_e * sqrt(g R_s T_e).
    let t_e = state.temperature().value();
    assert!((state.density().value() - state.pressure().value() / (r_s * t_e)).abs() < 1e-9);
    assert!((state.velocity().value() - state.mach() * (1.4 * r_s * t_e).sqrt()).abs() < 1e-6);
}

#[test]
fn test_nozzle_exit_state_rejects_bad_chamber() {
    assert!(
        nozzle_exit_state_kernel(
            Pressure::new(0.0_f64).unwrap(),
            Temperature::new(3000.0).unwrap(),
            4.0,
            1.4,
            287.0
        )
        .is_err()
    );
    assert!(
        nozzle_exit_state_kernel(
            Pressure::new(2.0e6_f64).unwrap(),
            Temperature::new(3000.0).unwrap(),
            4.0,
            1.4,
            0.0
        )
        .is_err()
    );
    assert!(
        nozzle_exit_state_kernel(
            Pressure::new(2.0e6_f64).unwrap(),
            Temperature::new(3000.0).unwrap(),
            0.5,
            1.4,
            287.0
        )
        .is_err()
    );
}
