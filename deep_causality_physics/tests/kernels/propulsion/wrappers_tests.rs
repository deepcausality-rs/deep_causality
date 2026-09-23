/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `PropagatingEffect` wrapper coverage: each wrapper's success arm produces a
//! value and its error arm propagates a `CausalityError` (the kernels' own
//! rejections are exercised in the per-topic test files).

use deep_causality_physics::{
    Acceleration, Area, Density, FlowBranch, Force, Length, Mass, Pressure, Temperature,
    choked_mass_flow, cordell_braun_plume_boundary, cordell_braun_plume_boundary_kernel,
    ignition_altitude, inverse_area_mach, inverse_area_mach_kernel,
    jarvinen_adams_baseline_axial_coefficient, jarvinen_adams_baseline_axial_coefficient_kernel,
    momentum_flux_ratio, nozzle_exit_state, prandtl_meyer, prandtl_meyer_kernel,
    propellant_mass_flow, propellant_mass_flow_kernel, srp_flow_regime_margin,
    srp_flow_regime_margin_kernel, srp_jet_edge_mach, srp_jet_edge_mach_kernel,
    srp_post_bow_shock_total_pressure, srp_post_bow_shock_total_pressure_kernel,
    srp_preserved_drag_fraction, srp_preserved_drag_fraction_kernel, srp_terminal_shock_mach,
    srp_terminal_shock_mach_kernel, srp_thrust_coefficient, srp_total_axial_force_coefficient,
    srp_total_axial_force_coefficient_kernel, stopping_distance, stopping_distance_kernel,
    suicide_burn_deceleration, tsiolkovsky_delta_v,
};

const M_INF: f64 = 2.0;
const P_INF: f64 = 1762.3;
const GAMMA: f64 = 1.4;

#[test]
fn test_performance_wrappers() {
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = propellant_mass_flow(Force::new(1000.0_f64).unwrap(), 300.0);
    assert_eq!(
        effect.value_cloned().unwrap(),
        propellant_mass_flow_kernel(Force::new(1000.0_f64).unwrap(), 300.0).unwrap(),
        "propellant_mass_flow must carry the value its kernel produced"
    );
    assert!(!propellant_mass_flow(Force::new(1000.0_f64).unwrap(), 0.0).is_ok());
    assert!(
        tsiolkovsky_delta_v(
            300.0_f64,
            Mass::new(1000.0).unwrap(),
            Mass::new(400.0).unwrap()
        )
        .is_ok()
    );
    assert!(
        !tsiolkovsky_delta_v(
            300.0_f64,
            Mass::new(400.0).unwrap(),
            Mass::new(1000.0).unwrap()
        )
        .is_ok()
    );
}

#[test]
fn test_nozzle_wrappers() {
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = inverse_area_mach(2.0_f64, GAMMA, FlowBranch::Supersonic);
    assert_eq!(
        effect.value_cloned().unwrap(),
        inverse_area_mach_kernel(2.0_f64, GAMMA, FlowBranch::Supersonic).unwrap(),
        "inverse_area_mach must carry the value its kernel produced"
    );
    assert!(!inverse_area_mach(0.5_f64, GAMMA, FlowBranch::Supersonic).is_ok());
    assert!(
        nozzle_exit_state(
            Pressure::new(2.0e6_f64).unwrap(),
            Temperature::new(3000.0).unwrap(),
            4.0,
            GAMMA,
            287.0
        )
        .is_ok()
    );
    assert!(
        !nozzle_exit_state(
            Pressure::new(0.0_f64).unwrap(),
            Temperature::new(3000.0).unwrap(),
            4.0,
            GAMMA,
            287.0
        )
        .is_ok()
    );
}

#[test]
fn test_descent_wrappers() {
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = stopping_distance(Speed_new(100.0), Acceleration::new(5.0).unwrap());
    assert_eq!(
        effect.value_cloned().unwrap(),
        stopping_distance_kernel(Speed_new(100.0), Acceleration::new(5.0).unwrap()).unwrap(),
        "stopping_distance must carry the value its kernel produced"
    );
    assert!(!stopping_distance(Speed_new(100.0), Acceleration::new(-1.0).unwrap()).is_ok());
    assert!(
        ignition_altitude(
            Speed_new(200.0),
            Acceleration::new(30.0).unwrap(),
            Acceleration::new(9.8).unwrap(),
            Length::new(50.0).unwrap()
        )
        .is_ok()
    );
    assert!(
        !ignition_altitude(
            Speed_new(200.0),
            Acceleration::new(9.0).unwrap(),
            Acceleration::new(9.8).unwrap(),
            Length::new(50.0).unwrap()
        )
        .is_ok()
    );
    assert!(
        suicide_burn_deceleration(
            Speed_new(100.0),
            Length::new(600.0).unwrap(),
            Acceleration::new(9.8).unwrap()
        )
        .is_ok()
    );
    assert!(
        !suicide_burn_deceleration(
            Speed_new(100.0),
            Length::new(0.0).unwrap(),
            Acceleration::new(9.8).unwrap()
        )
        .is_ok()
    );
}

#[test]
fn test_srp_wrappers() {
    assert!(
        srp_thrust_coefficient(
            Force::new(1000.0_f64).unwrap(),
            Pressure::new(500.0).unwrap(),
            Area::new(0.5).unwrap()
        )
        .is_ok()
    );
    assert!(
        !srp_thrust_coefficient(
            Force::new(1000.0_f64).unwrap(),
            Pressure::new(0.0).unwrap(),
            Area::new(0.5).unwrap()
        )
        .is_ok()
    );
    assert!(
        momentum_flux_ratio(
            Density::new(2.0_f64).unwrap(),
            Speed_new(100.0),
            Density::new(0.5).unwrap(),
            Speed_new(200.0)
        )
        .is_ok()
    );
    assert!(
        !momentum_flux_ratio(
            Density::new(2.0_f64).unwrap(),
            Speed_new(100.0),
            Density::new(0.0).unwrap(),
            Speed_new(200.0)
        )
        .is_ok()
    );
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = srp_preserved_drag_fraction(1.0_f64);
    assert_eq!(
        effect.value_cloned().unwrap(),
        srp_preserved_drag_fraction_kernel(1.0_f64).unwrap(),
        "srp_preserved_drag_fraction must carry the value its kernel produced"
    );
    assert!(!srp_preserved_drag_fraction(20.0_f64).is_ok());
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = jarvinen_adams_baseline_axial_coefficient(1.5_f64);
    assert_eq!(
        effect.value_cloned().unwrap(),
        jarvinen_adams_baseline_axial_coefficient_kernel(1.5_f64).unwrap(),
        "jarvinen_adams_baseline_axial_coefficient must carry the value its kernel produced"
    );
    assert!(!jarvinen_adams_baseline_axial_coefficient(3.0_f64).is_ok());
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = srp_total_axial_force_coefficient(2.0_f64, 2.0);
    assert_eq!(
        effect.value_cloned().unwrap(),
        srp_total_axial_force_coefficient_kernel(2.0_f64, 2.0).unwrap(),
        "srp_total_axial_force_coefficient must carry the value its kernel produced"
    );
    assert!(!srp_total_axial_force_coefficient(20.0_f64, 2.0).is_ok());
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = srp_flow_regime_margin(2.0_f64, 1.0);
    assert_eq!(
        effect.value_cloned().unwrap(),
        srp_flow_regime_margin_kernel(2.0_f64, 1.0).unwrap(),
        "srp_flow_regime_margin must carry the value its kernel produced"
    );
    assert!(!srp_flow_regime_margin(2.0_f64, 0.0).is_ok());
}

#[test]
fn test_plume_wrappers() {
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = prandtl_meyer(2.0_f64, GAMMA);
    assert_eq!(
        effect.value_cloned().unwrap(),
        prandtl_meyer_kernel(2.0_f64, GAMMA).unwrap(),
        "prandtl_meyer must carry the value its kernel produced"
    );
    assert!(!prandtl_meyer(0.5_f64, GAMMA).is_ok());
    assert!(
        choked_mass_flow(
            Area::new(1.0e-4_f64).unwrap(),
            Pressure::new(2.0e6).unwrap(),
            Temperature::new(3000.0).unwrap(),
            GAMMA,
            287.0
        )
        .is_ok()
    );
    assert!(
        !choked_mass_flow(
            Area::new(0.0_f64).unwrap(),
            Pressure::new(2.0e6).unwrap(),
            Temperature::new(3000.0).unwrap(),
            GAMMA,
            287.0
        )
        .is_ok()
    );
    let pt_1 = srp_post_bow_shock_total_pressure(Pressure::new(P_INF).unwrap(), M_INF, GAMMA);
    // Delegation, not merely success: `assert!(pt_1.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        pt_1.value_cloned().unwrap(),
        srp_post_bow_shock_total_pressure_kernel(Pressure::new(P_INF).unwrap(), M_INF, GAMMA)
            .unwrap(),
        "srp_post_bow_shock_total_pressure must carry the value its kernel produced"
    );
    assert!(!srp_post_bow_shock_total_pressure(Pressure::new(P_INF).unwrap(), 0.5, GAMMA).is_ok());
    let pt_1_val = *pt_1.value().unwrap();
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = srp_terminal_shock_mach(Pressure::new(1.0e7_f64).unwrap(), pt_1_val, GAMMA);
    assert_eq!(
        effect.value_cloned().unwrap(),
        srp_terminal_shock_mach_kernel(Pressure::new(1.0e7_f64).unwrap(), pt_1_val, GAMMA).unwrap(),
        "srp_terminal_shock_mach must carry the value its kernel produced"
    );
    assert!(
        !srp_terminal_shock_mach(
            Pressure::new(pt_1_val.value() * 0.5).unwrap(),
            pt_1_val,
            GAMMA
        )
        .is_ok()
    );
    let p_exit = Pressure::new(1.0e7_f64 / 40.0).unwrap();
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = srp_jet_edge_mach(4.0_f64, p_exit, pt_1_val, GAMMA);
    assert_eq!(
        effect.value_cloned().unwrap(),
        srp_jet_edge_mach_kernel(4.0_f64, p_exit, pt_1_val, GAMMA).unwrap(),
        "srp_jet_edge_mach must carry the value its kernel produced"
    );
    assert!(!srp_jet_edge_mach(0.5_f64, p_exit, pt_1_val, GAMMA).is_ok());
}

#[test]
fn test_plume_boundary_wrapper() {
    let r_exit = 0.5_f64 * 0.0254 / 2.0;
    let d_throat = 0.13 * 0.0254;
    let l_cone = 0.68 * 0.0254;
    let half = 15.0 * core::f64::consts::PI / 180.0;
    let ar = (r_exit / (d_throat / 2.0)).powi(2);
    let m_exit =
        deep_causality_physics::inverse_area_mach_kernel(ar, GAMMA, FlowBranch::Supersonic)
            .unwrap();
    let ok = cordell_braun_plume_boundary(
        Pressure::new(6060.2_f64 * P_INF).unwrap(),
        Temperature::new(294.0).unwrap(),
        287.0,
        GAMMA,
        m_exit,
        half,
        Length::new(d_throat).unwrap(),
        Length::new(r_exit).unwrap(),
        Length::new(l_cone).unwrap(),
        Pressure::new(P_INF).unwrap(),
        M_INF,
        GAMMA,
    );
    // Delegation, not merely success: `assert!(ok.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        ok.value_cloned().unwrap(),
        cordell_braun_plume_boundary_kernel(
            Pressure::new(6060.2_f64 * P_INF).unwrap(),
            Temperature::new(294.0).unwrap(),
            287.0,
            GAMMA,
            m_exit,
            half,
            Length::new(d_throat).unwrap(),
            Length::new(r_exit).unwrap(),
            Length::new(l_cone).unwrap(),
            Pressure::new(P_INF).unwrap(),
            M_INF,
            GAMMA
        )
        .unwrap(),
        "cordell_braun_plume_boundary must carry the value its kernel produced"
    );
    let bad = cordell_braun_plume_boundary(
        Pressure::new(6060.2_f64 * P_INF).unwrap(),
        Temperature::new(294.0).unwrap(),
        287.0,
        GAMMA,
        m_exit,
        half,
        Length::new(d_throat).unwrap(),
        Length::new(r_exit).unwrap(),
        Length::new(l_cone).unwrap(),
        Pressure::new(P_INF).unwrap(),
        5.0, // Mach outside [2, 4]
        GAMMA,
    );
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = bad.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Physical Invariant Broken"),
        "expected a Physical Invariant Broken refusal, got {err}"
    );
}

// Local helper: the descent/srp wrappers take `Speed<f64>`; keep the call
// sites terse.
#[allow(non_snake_case)]
fn Speed_new(v: f64) -> deep_causality_physics::Speed<f64> {
    deep_causality_physics::Speed::new(v).unwrap()
}
