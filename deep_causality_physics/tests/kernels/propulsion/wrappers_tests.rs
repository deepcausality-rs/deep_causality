/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `PropagatingEffect` wrapper coverage: each wrapper's success arm produces a
//! value and its error arm propagates a `CausalityError` (the kernels' own
//! rejections are exercised in the per-topic test files).

use deep_causality_physics::{
    Acceleration, Area, Density, FlowBranch, Force, Length, Mass, Pressure, Temperature,
    choked_mass_flow, cordell_braun_plume_boundary, ignition_altitude, inverse_area_mach,
    jarvinen_adams_baseline_axial_coefficient, momentum_flux_ratio, nozzle_exit_state,
    prandtl_meyer, propellant_mass_flow, srp_flow_regime_margin, srp_jet_edge_mach,
    srp_post_bow_shock_total_pressure, srp_preserved_drag_fraction, srp_terminal_shock_mach,
    srp_thrust_coefficient, srp_total_axial_force_coefficient, stopping_distance,
    suicide_burn_deceleration, tsiolkovsky_delta_v,
};

const M_INF: f64 = 2.0;
const P_INF: f64 = 1762.3;
const GAMMA: f64 = 1.4;

#[test]
fn test_performance_wrappers() {
    assert!(propellant_mass_flow(Force::new(1000.0_f64).unwrap(), 300.0).is_ok());
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
    assert!(inverse_area_mach(2.0_f64, GAMMA, FlowBranch::Supersonic).is_ok());
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
    assert!(stopping_distance(Speed_new(100.0), Acceleration::new(5.0).unwrap()).is_ok());
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
    assert!(srp_preserved_drag_fraction(1.0_f64).is_ok());
    assert!(!srp_preserved_drag_fraction(20.0_f64).is_ok());
    assert!(jarvinen_adams_baseline_axial_coefficient(1.5_f64).is_ok());
    assert!(!jarvinen_adams_baseline_axial_coefficient(3.0_f64).is_ok());
    assert!(srp_total_axial_force_coefficient(2.0_f64, 2.0).is_ok());
    assert!(!srp_total_axial_force_coefficient(20.0_f64, 2.0).is_ok());
    assert!(srp_flow_regime_margin(2.0_f64, 1.0).is_ok());
    assert!(!srp_flow_regime_margin(2.0_f64, 0.0).is_ok());
}

#[test]
fn test_plume_wrappers() {
    assert!(prandtl_meyer(2.0_f64, GAMMA).is_ok());
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
    assert!(pt_1.is_ok());
    assert!(!srp_post_bow_shock_total_pressure(Pressure::new(P_INF).unwrap(), 0.5, GAMMA).is_ok());
    let pt_1_val = *pt_1.value().unwrap();
    assert!(srp_terminal_shock_mach(Pressure::new(1.0e7_f64).unwrap(), pt_1_val, GAMMA).is_ok());
    assert!(
        !srp_terminal_shock_mach(
            Pressure::new(pt_1_val.value() * 0.5).unwrap(),
            pt_1_val,
            GAMMA
        )
        .is_ok()
    );
    let p_exit = Pressure::new(1.0e7_f64 / 40.0).unwrap();
    assert!(srp_jet_edge_mach(4.0_f64, p_exit, pt_1_val, GAMMA).is_ok());
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
    assert!(ok.is_ok());
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
    assert!(!bad.is_ok());
}

// Local helper: the descent/srp wrappers take `Speed<f64>`; keep the call
// sites terse.
#[allow(non_snake_case)]
fn Speed_new(v: f64) -> deep_causality_physics::Speed<f64> {
    deep_causality_physics::Speed::new(v).unwrap()
}
