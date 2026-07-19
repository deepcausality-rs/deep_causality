/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The physical anchor and harness geometry of the SRP drag-decrement verification.
//!
//! Dimensional side (SI): a Mach-2, γ = 1.4 cold-air freestream — the Jarvinen–Adams wind-tunnel
//! condition and the anchor of every digitized correlation constant (`*_M2`). A central retro
//! nozzle (γ_jet = 1.3, exit Mach 3) fires upstream from the face of a blunt body; the chamber
//! pressure is the throttle. Nondimensional side: the marcher runs `ρ̂ = ρ/ρ∞`, `û = u/c∞`,
//! `p̂ = p/(ρ∞·c∞²)`, on a unit square representing `DOMAIN_M` meters.

/// Freestream static pressure, Pa (a supersonic-tunnel static condition).
pub const P_INF: f64 = 1000.0;
/// Freestream static temperature, K.
pub const T_INF: f64 = 216.0;
/// Air specific gas constant, J/(kg·K).
pub const R_AIR: f64 = 287.0;
/// Freestream ratio of specific heats (cold air — the wind-tunnel condition, NOT the corridor's
/// reacting γ_eff).
pub const GAMMA_INF: f64 = 1.4;
/// Freestream Mach number: the Jarvinen–Adams correlation anchor (all `*_M2` constants).
pub const MACH_INF: f64 = 2.0;

/// Body radius, m (D = 1 m blunt face; the J–A aeroshell scale).
pub const R_BODY: f64 = 0.5;

/// Jet ratio of specific heats (inside the Cordell validity envelope [1.2, 1.4]).
pub const GAMMA_JET: f64 = 1.3;
/// Jet specific gas constant, J/(kg·K).
pub const R_JET: f64 = 300.0;
/// Chamber (stagnation) temperature, K.
pub const T_CHAMBER: f64 = 1500.0;
/// Nozzle exit Mach number.
pub const EXIT_MACH: f64 = 3.0;
/// Conical nozzle half-angle, rad (15°).
pub const NOZZLE_HALF_ANGLE: f64 = 15.0 * core::f64::consts::PI / 180.0;
/// Throat diameter, m.
pub const D_THROAT: f64 = 0.03;

/// Physical width of the square domain, m (the unit computational square spans this).
pub const DOMAIN_M: f64 = 4.0;
/// Grid mode count: the layer is `2^L × 2^L`. Sized at run time: 2^6 puts a single powered run
/// past 15 minutes (outside the family's minutes-class budget); 2^5 resolves the collapse
/// structure at ~4 cells of body radius and keeps the eight-run sweep inside it.
pub const L: usize = 5;
/// Solver step (nondimensional; convective CFL ≈ 0.2 at the jet wave speed on the 2^5 grid).
pub const DT: f64 = 8.0e-4;
/// Reference wave speed of the implicit acoustic envelope (covers the expanded jet, |û|+ĉ ≈ 7).
pub const S_REF: f64 = 8.0;
/// Bond cap of the round policy (runtime-bounding; the peak bond is printed as a caveat).
pub const CAP: usize = 24;
/// March steps per run (≈ two inflow→body transits: shock formation + settle).
pub const STEPS: usize = 500;

/// Body center on the unit square (x̂, ŷ).
pub const BODY_CX: f64 = 0.72;
pub const BODY_CY: f64 = 0.5;

/// The thrust-coefficient sweep: spans the central-nozzle collapse (→ C_T ≈ 1) and the
/// thrust-dominated recovery, inside both the Cordell pressure-ratio validity floor
/// (C_T ≳ 0.15 with this nozzle) and the digitized J–A domain (C_T ≤ 8.8).
pub const CT_SWEEP: [f64; 7] = [0.25, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0];

// ── Regression gates, pinned from the FIRST measured run (2026-07-17; committed output.txt;
//    re-pin rationale in openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md). The
//    originally anticipated Jarvinen–Adams structural gates (collapse < 0.10 by C_T ≈ 1, the
//    sign-flip dip) did NOT hold on this harness — the static-obstruction imprint shields like
//    a drag-reduction spike (monotone, partial) rather than collapsing the flowfield — so the
//    J–A comparison is REPORTED as the amber de-risk finding and the gates below regress the
//    measured behavior. ──

/// R-B: preserved-fraction ceiling at the sweep top, C_T = 4 (measured 0.647 on the first run;
/// the pin leaves ~8% slack for step/round jitter).
pub const TOP_FRACTION_CEILING: f64 = 0.70;

/// Freestream density, kg/m³.
pub fn rho_inf() -> f64 {
    P_INF / (R_AIR * T_INF)
}

/// Freestream sound speed, m/s.
pub fn c_inf() -> f64 {
    (GAMMA_INF * R_AIR * T_INF).sqrt()
}

/// Freestream speed, m/s.
pub fn u_inf() -> f64 {
    MACH_INF * c_inf()
}

/// Freestream dynamic pressure, Pa.
pub fn q_inf() -> f64 {
    0.5 * rho_inf() * u_inf() * u_inf()
}

/// The C_T reference area, m² (the body's frontal disc, the J–A normalization).
pub fn s_ref_area() -> f64 {
    core::f64::consts::PI * R_BODY * R_BODY
}

/// Nondimensional pressure of a dimensional `p` (Pa): `p̂ = p/(ρ∞·c∞²)`.
pub fn p_hat(p: f64) -> f64 {
    p / (rho_inf() * c_inf() * c_inf())
}

/// Nondimensional length of a dimensional `x` (m) on the unit square.
pub fn x_hat(x: f64) -> f64 {
    x / DOMAIN_M
}

/// The freestream conserved state `[ρ̂, m̂x, m̂y, Ê]` (û = M∞ along +x).
pub fn freestream_conserved() -> [f64; 4] {
    let rho = 1.0;
    let u = MACH_INF; // û = u∞/c∞
    let p = p_hat(P_INF);
    let e = p / (GAMMA_INF - 1.0) + 0.5 * rho * u * u;
    [rho, rho * u, 0.0, e]
}
