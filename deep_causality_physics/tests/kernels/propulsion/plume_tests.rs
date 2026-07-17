/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cordell plume-model tests. The exact printed anchors (dissertation Table 13
//! jet-edge Mach, Fig. 54 terminal-shock Mach, and the terminal-shock standoff
//! normalized by body diameter) are asserted tightly; the composed absolute
//! plume radius — which depends on figure-read nozzle dimensions — is asserted
//! by physical bounds, ordering, and throttle response, matching how the
//! dissertation itself reports ~13% radial-extent error.

use deep_causality_physics::{
    FlowBranch, Length, Pressure, Temperature, cordell_braun_plume_boundary_kernel,
    inverse_area_mach_kernel, prandtl_meyer_kernel, srp_jet_edge_mach_kernel,
    srp_post_bow_shock_total_pressure_kernel, srp_terminal_shock_mach_kernel,
};

// Single-nozzle wind-tunnel conditions (dissertation Tables 1-2, 11; §4.2).
const M_INF: f64 = 2.0;
const P_INF: f64 = 1762.3; // Pa
const GAMMA: f64 = 1.4;
const R_GAS: f64 = 287.0; // J/(kg K)
const T_T_JET: f64 = 294.0; // K
// Nozzle geometry (Fig. 13): exit diameter 0.5 in, throat diameter 0.13 in,
// diverging-cone length 0.68 in, 15 deg conical half-angle.
const R_EXIT: f64 = 0.5 * 0.0254 / 2.0;
const D_THROAT: f64 = 0.13 * 0.0254;
const L_CONE: f64 = 0.68 * 0.0254;
const HALF_ANGLE: f64 = 15.0 * core::f64::consts::PI / 180.0;
const BODY_DIAMETER: f64 = 2.0 * 2.0 * 0.0254; // 60-deg sphere-cone, 2 in base radius

fn pt_jet_for(pr_over_pinf: f64) -> f64 {
    pr_over_pinf * P_INF
}

#[test]
fn test_prandtl_meyer_known_values() {
    // nu(1) = 0; nu(2, gamma=1.4) = 26.38 deg = 0.4604 rad (NACA 1135 tables).
    assert!(prandtl_meyer_kernel(1.0_f64, GAMMA).unwrap().abs() < 1e-12);
    let nu2 = prandtl_meyer_kernel(2.0_f64, GAMMA).unwrap();
    assert!((nu2 - 0.4604).abs() < 1e-3, "nu(2) = {nu2}");
    assert!(prandtl_meyer_kernel(0.9_f64, GAMMA).is_err());
}

#[test]
fn test_jet_edge_mach_reproduces_table_13() {
    // Dissertation Table 13, single nozzle: M_edge depends only on
    // P_T,jet/P_T,1 (the exit-Mach dependence cancels analytically), so these
    // are exact printed anchors. C_T 0.47 -> 3.86, 4.04 -> 5.63, 10.0 -> 6.53.
    let pt_1 =
        srp_post_bow_shock_total_pressure_kernel(Pressure::new(P_INF).unwrap(), M_INF, GAMMA)
            .unwrap();
    // Any supersonic exit Mach works; pick the nozzle's own from its area ratio.
    let ar = (R_EXIT / (D_THROAT / 2.0)).powi(2);
    let m_exit = inverse_area_mach_kernel(ar, GAMMA, FlowBranch::Supersonic).unwrap();
    for &(pr, m_edge_expected) in &[(712.4, 3.86), (6060.2, 5.63), (14988.2, 6.53)] {
        let pt_jet = pt_jet_for(pr);
        let p_exit = pt_jet
            / deep_causality_physics::isentropic_pressure_ratio_kernel(m_exit, GAMMA).unwrap();
        let m_edge =
            srp_jet_edge_mach_kernel(m_exit, Pressure::new(p_exit).unwrap(), pt_1, GAMMA).unwrap();
        assert!(
            (m_edge - m_edge_expected).abs() < 0.02,
            "PR {pr}: M_edge {m_edge} vs printed {m_edge_expected}"
        );
    }
}

#[test]
fn test_terminal_shock_mach_matches_fig_54() {
    // Fig. 54: terminal-shock Mach ~15.5 analytic at C_T = 10 (PR 14988.2).
    let pt_1 =
        srp_post_bow_shock_total_pressure_kernel(Pressure::new(P_INF).unwrap(), M_INF, GAMMA)
            .unwrap();
    let m_t =
        srp_terminal_shock_mach_kernel(Pressure::new(pt_jet_for(14988.2)).unwrap(), pt_1, GAMMA)
            .unwrap();
    assert!(
        (m_t - 15.5).abs() < 0.3,
        "terminal Mach {m_t} vs Fig. 54 ~15.5"
    );
    // Monotone: higher C_T (more overpressure) -> stronger terminal shock.
    let m_t_low =
        srp_terminal_shock_mach_kernel(Pressure::new(pt_jet_for(6060.2)).unwrap(), pt_1, GAMMA)
            .unwrap();
    assert!(m_t_low < m_t);
}

#[test]
fn test_terminal_shock_rejects_low_thrust() {
    // If the jet stagnation pressure does not exceed the post-bow-shock
    // stagnation pressure, no terminal shock forms (low-thrust regime).
    let pt_1 =
        srp_post_bow_shock_total_pressure_kernel(Pressure::new(P_INF).unwrap(), M_INF, GAMMA)
            .unwrap();
    assert!(
        srp_terminal_shock_mach_kernel(Pressure::new(pt_1.value() * 0.5).unwrap(), pt_1, GAMMA)
            .is_err()
    );
    // gamma <= 1 is rejected like every sibling kernel (no NaN pass-through).
    assert!(
        srp_terminal_shock_mach_kernel(Pressure::new(pt_jet_for(14988.2)).unwrap(), pt_1, 1.0)
            .is_err()
    );
}

fn plume_at(pr: f64) -> deep_causality_physics::PlumeGeometry<f64> {
    let ar = (R_EXIT / (D_THROAT / 2.0)).powi(2);
    let m_exit = inverse_area_mach_kernel(ar, GAMMA, FlowBranch::Supersonic).unwrap();
    cordell_braun_plume_boundary_kernel(
        Pressure::new(pt_jet_for(pr)).unwrap(),
        Temperature::new(T_T_JET).unwrap(),
        R_GAS,
        GAMMA,
        m_exit,
        HALF_ANGLE,
        Length::new(D_THROAT).unwrap(),
        Length::new(R_EXIT).unwrap(),
        Length::new(L_CONE).unwrap(),
        Pressure::new(P_INF).unwrap(),
        M_INF,
        GAMMA,
    )
    .unwrap()
}

#[test]
fn test_plume_geometry_standoff_matches_anchor() {
    // Terminal-shock standoff normalized by body diameter: anchor ~1.28
    // analytic at C_T = 4.04 (Fig. 55; consistently slightly underpredicted).
    let g = plume_at(6060.2);
    let standoff_norm = g.terminal_shock_standoff().value() / BODY_DIAMETER;
    assert!(
        (standoff_norm - 1.28).abs() < 0.15,
        "normalized standoff {standoff_norm} vs anchor ~1.28"
    );
    assert!(g.max_radius().value() > 0.0);
    assert!(g.penetration_length().value() >= g.terminal_shock_standoff().value());
}

#[test]
fn test_plume_geometry_responds_to_throttle() {
    // Dynamic-by-construction: two throttle settings -> two geometries, and a
    // larger jet drives a larger plume (radius, penetration, standoff all grow).
    let low = plume_at(6060.2); // C_T = 4.04
    let high = plume_at(14988.2); // C_T = 10.0
    assert!(high.max_radius().value() > low.max_radius().value());
    assert!(high.terminal_shock_standoff().value() > low.terminal_shock_standoff().value());
    assert!(high.penetration_length().value() > low.penetration_length().value());
    // Absolute radius in the same ballpark as the CFD bounds (0.039-0.078 m).
    assert!(high.max_radius().value() > 0.01 && high.max_radius().value() < 0.10);
}

#[test]
fn test_plume_rejects_outside_validity_envelope() {
    let ar = (R_EXIT / (D_THROAT / 2.0)).powi(2);
    let m_exit = inverse_area_mach_kernel(ar, GAMMA, FlowBranch::Supersonic).unwrap();
    let call = |mach_inf: f64, gamma_jet: f64, pr: f64| {
        cordell_braun_plume_boundary_kernel(
            Pressure::new(pt_jet_for(pr)).unwrap(),
            Temperature::new(T_T_JET).unwrap(),
            R_GAS,
            gamma_jet,
            m_exit,
            HALF_ANGLE,
            Length::new(D_THROAT).unwrap(),
            Length::new(R_EXIT).unwrap(),
            Length::new(L_CONE).unwrap(),
            Pressure::new(P_INF).unwrap(),
            mach_inf,
            GAMMA,
        )
    };
    // Freestream Mach outside [2, 4].
    assert!(call(5.0, GAMMA, 6060.2).is_err());
    // Jet gamma outside [1.2, 1.4].
    assert!(call(M_INF, 1.5, 6060.2).is_err());
    // Jet-penetration regime: exit pressure ratio below the blunt-flow
    // transition (very low thrust).
    assert!(call(M_INF, GAMMA, 50.0).is_err());
}
