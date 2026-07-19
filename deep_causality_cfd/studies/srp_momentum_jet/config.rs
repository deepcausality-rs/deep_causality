/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The physical anchor and harness geometry of the SRP momentum-jet study.
//!
//! Every freestream, body, and domain constant is carried **verbatim** from
//! `verification/srp_drag_decrement/config.rs` so that at the default L = 5 / cap 24 / 500-step
//! configuration the unpowered baseline is the same run to the last rounded digit (gauge
//! forebody force 1.584393e-2 on the committed first run; this study's first run measured a
//! delta of 4.1e-9) and any difference in the powered sweep is attributable to the jet model
//! alone. The one modeling change is the jet: a small nozzle-exit **patch** pinned to a
//! supersonic upstream-firing exit state (momentum-carrying inflow; the plume forms in the
//! marched field), instead of the verification's whole-envelope pin at ambient pressure (the
//! static obstruction).
//!
//! March length, grid level, and bond cap are runtime-dialable (`SRP_MJ_*` environment
//! variables, parsed in `main.rs`) because the de-risk follow-up needs robustness companions:
//! a longer settle with a time-averaged tail read (the committed harness reads a single
//! terminal snapshot), a cap-32 run (exact at L = 5 — no bond exceeds its natural dimension,
//! so tensor-train truncation is off), and an L = 6 point (halved numerical dissipation,
//! ν = ½·s_ref·Δx).

/// Freestream static pressure, Pa (a supersonic-tunnel static condition).
pub const P_INF: f64 = 1000.0;
/// Freestream static temperature, K.
pub const T_INF: f64 = 216.0;
/// Air specific gas constant, J/(kg·K).
pub const R_AIR: f64 = 287.0;
/// Freestream ratio of specific heats (cold air — the wind-tunnel condition).
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
/// Nozzle exit Mach number (fixed; the throttle is the exit pressure, i.e. chamber pressure).
pub const EXIT_MACH: f64 = 3.0;

/// Physical width of the square domain, m (the unit computational square spans this).
pub const DOMAIN_M: f64 = 4.0;
/// Reference wave speed of the implicit acoustic envelope (covers the exit state, |û|+ĉ ≈ 6.8).
pub const S_REF: f64 = 8.0;

/// Body center on the unit square (x̂, ŷ).
pub const BODY_CX: f64 = 0.72;
pub const BODY_CY: f64 = 0.5;

/// The thrust-coefficient sweep: the verification's seven points (row-by-row comparison with
/// the pinned-envelope amber table) extended through the upper range where the fixed-nozzle
/// exit-pressure ratio approaches the Jarvinen–Adams transition variable (p_e/p∞ ≈ 7 — the
/// sharp jet-penetration → blunt-flow transition; Fig. 18 p. 35, Conclusion 3 p. 145). The
/// upper points carry the measured domain-blockage finding: the upstream probe leaves the
/// freestream long before the transition ratio is reached, so the transition regime is
/// unreachable on this domain — a disclosed harness limit, not a gated regression.
pub const CT_SWEEP: [f64; 10] = [0.25, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0];

// ── Regression bands, pinned from the FIRST measured v3 run (2026-07-17; committed
//    output.txt; the momentum-jet counterpart of the reverted verification's R-A/R-B pins).
//    They regress the measured structure — monotone drag AUGMENTATION with a frozen
//    stagnation interface — which is the de-risk finding, not a physics truth claim: the
//    J–A collapse needs jet penetration this harness's dissipation floor cannot carry
//    (openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md is the authority). ──

/// R-A′: the annulus fraction is monotone non-decreasing in C_T (measured 1.031 → 3.614).
/// Structural; enforced in `main.rs` at the default configuration only.
/// R-B′: annulus-fraction band at the sweep top, C_T = 8 (measured 3.614 on the first run;
/// ~±10% slack for step/round jitter).
pub const TOP_ANNULUS_FRACTION_BAND: (f64, f64) = (3.2, 4.0);
/// R-C′: the frozen-interface witness — centerline interface position at every swept C_T
/// (measured 0.469–0.531 across the 32× thrust range; one-cell slack each side).
pub const INTERFACE_X_BAND: (f64, f64) = (0.44, 0.56);

/// Jet exit patch half-height in **cells of the L = 5 grid** (patch spans `|ŷ − ŷ_c| ≤ h·Δx̂₅`,
/// one L-5 cell wide). Fixing the patch in physical units (0.25 m tall, r_jet/R_body = 0.25)
/// keeps the injected momentum flux identical across grid levels, so an L = 6 companion run
/// varies only the resolution, not the nozzle.
pub const JET_HALF_HEIGHT_L5_CELLS: f64 = 1.0;
/// The L = 5 cell width the patch geometry is anchored to.
pub const DX_L5: f64 = 1.0 / 32.0;

/// The committed unpowered baseline of the verification harness (first run, 2026-07-17,
/// L = 5 / cap 24 / 500 steps / terminal snapshot): the identity witness this study's
/// matching-configuration baseline must land on.
pub const VERIFICATION_BASELINE: f64 = 1.584393e-2;

/// Freestream density, kg/m³.
pub fn rho_inf() -> f64 {
    P_INF / (R_AIR * T_INF)
}

/// Freestream sound speed, m/s.
pub fn c_inf() -> f64 {
    (GAMMA_INF * R_AIR * T_INF).sqrt()
}

/// Nondimensional freestream dynamic pressure `q̂ = ½·ρ̂∞·û∞²` (ρ̂∞ = 1, û∞ = M∞).
pub fn q_hat_inf() -> f64 {
    0.5 * MACH_INF * MACH_INF
}

/// Nondimensional frontal span of the body, `D̂ = 2·R̂` — the 2-D (per-unit-depth) C_T
/// reference length. The 2-D analog of the J–A frontal-area normalization, declared as *the*
/// depth convention of this study (the axisymmetric disc area does not exist in plane flow).
pub fn d_hat_body() -> f64 {
    2.0 * x_hat(R_BODY)
}

/// Nondimensional pressure of a dimensional `p` (Pa): `p̂ = p/(ρ∞·c∞²)`.
pub fn p_hat(p: f64) -> f64 {
    p / (rho_inf() * c_inf() * c_inf())
}

/// Nondimensional length of a dimensional `x` (m) on the unit square.
pub fn x_hat(x: f64) -> f64 {
    x / DOMAIN_M
}

/// The nondimensional jet-patch half-height (grid-independent).
pub fn jet_half_height_hat() -> f64 {
    JET_HALF_HEIGHT_L5_CELLS * DX_L5
}

/// The freestream conserved state `[ρ̂, m̂x, m̂y, Ê]` (û = M∞ along +x).
pub fn freestream_conserved() -> [f64; 4] {
    let rho = 1.0;
    let u = MACH_INF; // û = u∞/c∞
    let p = p_hat(P_INF);
    let e = p / (GAMMA_INF - 1.0) + 0.5 * rho * u * u;
    [rho, rho * u, 0.0, e]
}

/// Isentropic nozzle-exit static temperature at `EXIT_MACH`, K.
pub fn t_exit() -> f64 {
    T_CHAMBER / (1.0 + 0.5 * (GAMMA_JET - 1.0) * EXIT_MACH * EXIT_MACH)
}

/// Nondimensional exit velocity `û_e = −u_e/c∞` (upstream-firing).
pub fn u_hat_exit() -> f64 {
    -(EXIT_MACH * (GAMMA_JET * R_JET * t_exit()).sqrt()) / c_inf()
}

/// One throttle point of the fixed-geometry nozzle: the exit static pressure `p̂_e` that makes
/// the patch's momentum thrust per unit depth,
/// `T̂′ = ĥ·(ρ̂_e·û_e² + p̂_e − p̂∞)`, equal `C_T·q̂∞·D̂` — the 2-D thrust-coefficient
/// definition consistent with the harness's per-depth drag reading. `h_hat_eff` is the
/// **realized** pinned exit height on the grid: the mask codec samples at grid nodes
/// (`mask_from_fn` evaluates at `k·Δx`), so the caller counts the pinned node-rows and passes
/// `rows·Δx̂` — the C_T abscissa is sized to the mask the marcher actually sees, not to the
/// nominal predicate span (a half-cell fencepost at L = 5 would otherwise inflate the injected
/// thrust ~1.5×). With the exit Mach fixed, `ρ̂_e·û_e² = γ_jet·M_e²·p̂_e`, so the solve is
/// closed-form and the sweep moves the nozzle from overexpanded (p_e < p∞, deep throttle) to
/// underexpanded (p_e > p∞), as a real fixed nozzle throttles. The injected momentum flux is
/// then known analytically and exactly (the patch is hard-pinned every step).
/// Returns `(p̂_e, ρ̂_e, target conserved state)`.
pub fn jet_exit_state(ct: f64, h_hat_eff: f64) -> (f64, f64, [f64; 4]) {
    let p_e = (ct * q_hat_inf() * d_hat_body() / h_hat_eff + p_hat(P_INF))
        / (1.0 + GAMMA_JET * EXIT_MACH * EXIT_MACH);
    let rho_e = p_e * c_inf() * c_inf() / (R_JET * t_exit());
    let u_e = u_hat_exit();
    let e_e = p_e / (GAMMA_INF - 1.0) + 0.5 * rho_e * u_e * u_e;
    (p_e, rho_e, [rho_e, rho_e * u_e, 0.0, e_e])
}
