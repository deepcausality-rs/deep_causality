/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every tuned number in the corridor lives here: the carrier setup, the flight physics, the
//! navigation budget, the safety envelope, the branch study, and the three flight stations.
//!
//! **Tier-A labels.** Each simplification is stated where its number is defined; the largest ones
//! up front:
//!
//! * The carrier flow is the *incompressible* QTT rollout in nondimensional units (`U_inf = 1`,
//!   box `2π`). `T_tr` is a recovery-temperature reconstruction over a mandatory Rankine-Hugoniot
//!   jump, not a true post-shock thermodynamic path.
//! * Saha equilibrium at the frozen post-shock temperature drives near-full ionization at the
//!   Mach-25 station, so `n_e → n_tot` there. That over-predicts the RAM-C II `~1e19 m⁻³` anchor;
//!   the gate is a regression band against the Tier-A baseline, not a truth claim.
//! * The LER surrogate carries only the forward Park associative-ionization rate. It has no
//!   recombination channel, so the blackout exit cannot be reached chemically; see [`EXIT`].
//! * Bank enters the 2-D carrier as the projected forebody radius, the dominant flow-side effect
//!   of banking that this carrier can express.
//! * The truth vehicle flies with 5% more drag than the navigation model assumes. Dead reckoning
//!   integrates exactly that error through the blackout.
//! * Stagnation heating is a Sutton-Graves-form `q = K·u³` on the carrier's peak speed, with the
//!   density factor folded into `K` at a RAM-C-like MW/m² scale.
//! * Leg time is carrier time (`dt = 4 ms` per step), so navigation drifts are small; the gates
//!   compare them relatively, and the mechanism scales unchanged to a flight-length dwell.

// ── Carrier (the Gap-1/Gap-2 verification setup, reused verbatim)

/// Grid mode count; the box is `2^L × 2^L`.
pub const L: usize = 5;
/// Bond cap of the tensor-train round policy, i.e. the compression ceiling.
pub const CAP: usize = 16;
/// Kinematic viscosity of the incompressible carrier.
pub const NU: f64 = 0.05;
/// Solver time step (also the coupling and navigation step).
pub const DT: f64 = 0.004;
/// Brinkman penalization parameter; small means a hard wall.
pub const ETA: f64 = 0.016;
/// Free-stream speed of the nondimensional carrier.
pub const U_INF: f64 = 1.0;
/// Body-mask smoothing width in cells.
pub const SMOOTH_CELLS: f64 = 2.0;
/// Diffusivity transporting the carried ionization fraction.
pub const SCALAR_KAPPA: f64 = 0.05;

// ── Flight physics

/// Ratio of specific heats.
pub const GAMMA: f64 = 1.4;
/// Frozen-mixture specific heat at constant pressure, J·kg⁻¹·K⁻¹.
pub const C_P: f64 = 1004.0;
/// GPS L1 as an angular frequency: `ω = 2π · 1.57542 GHz`.
pub const COMMS_BAND_RAD_S: f64 = 9.899e9;
/// Vehicle characteristic length for the Knudsen number (RAM-C-like forebody scale, m).
pub const L_CHAR: f64 = 0.3;
/// The ④ channel's ballistic bundle `C_d·A/m`, scaled so the carrier's dynamic pressure carries a
/// RAM-C-like `~30 m/s²` drag into the navigation predict.
pub const CDA_OVER_M: f64 = 30.0;
/// Reference density of the ④ dynamic-pressure rescaling.
pub const RHO_REF: f64 = 1.0;
/// The truth vehicle's drag excess over the navigation model; the dead-reckoning error source.
pub const DRAG_MODEL_ERROR: f64 = 0.05;
/// Sutton-Graves-form heating coefficient in `q = K·u³` (density factor folded in).
pub const HEAT_COEFF: f64 = 1.5e6;
/// Standard gravity, m·s⁻².
pub const G0: f64 = 9.80665;

// ── Reference anchors (reported, with the Tier-A disclaimers)

/// The Tier-A saturation baseline the peak-station electron density is regression-gated against
/// (`n_e → n_tot` at the frozen Rankine-Hugoniot temperature).
pub const NE_BASELINE: f64 = 1.0e22;
/// The RAM-C II ~61 km peak electron density anchor. Reported as a cross-reference only; the
/// frozen-RH Saha limit over-predicts it.
pub const RAMC_NE_REFERENCE: f64 = 1.0e19;

// ── Navigation

/// Initial true position (a bound LEO-class state, m).
pub const TRUTH_R0: [f64; 3] = [7.0e6, 1.0e6, 2.0e6];
/// Initial true velocity (m/s).
pub const TRUTH_V0: [f64; 3] = [-1.0e3, 6.5e3, 3.0e3];
/// Initial INS error before the first fix folds (m per axis).
pub const NAV_INIT_ERR: [f64; 3] = [50.0, -30.0, 20.0];
/// Initial position uncertainty (50 m 1σ), m².
pub const P0_VAR: f64 = 2500.0;
/// ESKF process-noise diagonal.
pub const PROCESS_NOISE: f64 = 1.0e-4;
/// GNSS fix variance (5 m 1σ), m².
pub const GNSS_VAR: f64 = 25.0;
/// Through-plasma optical fix variance (50 m 1σ), m². No optical fix is published in this
/// corridor, so the budget is carried but unused.
pub const OPTICAL_VAR: f64 = 2500.0;

// ── Safety envelope + guidance

/// Heat-flux certification ceiling, W·m⁻² (margin over the ~4 MW/m² peak).
pub const MAX_HEAT_FLUX: f64 = 2.0e7;
/// G-load ceiling.
pub const MAX_G_LOAD: f64 = 15.0;
/// Bank-angle magnitude cap, rad.
pub const MAX_BANK_RAD: f64 = 0.5;
/// Guidance gain: bank command proportional to sensed heating. Deliberately aggressive, so the
/// cybernetic gate visibly bounds the command.
pub const GUIDANCE_GAIN: f64 = 2.0e-7;

// ── Branch study ──────────────────────────────────────────────────────────────────────────────

/// Steps each counterfactual branch continues past the shared blackout onset.
pub const BRANCH_STEPS: usize = 20;
/// Candidate bank angles for the branch study (degrees).
pub const BANK_ANGLES_DEG: [f64; 3] = [0.0, 30.0, 60.0];

// ── Flight stations ───────────────────────────────────────────────────────────────────────────

/// One flight station of the corridor: the reacting and rarefaction constants a leg marches under.
pub struct FlightCondition {
    pub name: &'static str,
    pub mach: f64,
    pub t_inf: f64,
    /// Total heavy-particle number density (m⁻³); the plasma scales with it.
    pub n_tot: f64,
    /// Mean free path at the station (m); the Knudsen and regime driver.
    pub mean_free_path: f64,
    pub steps: usize,
    /// Projected forebody radius as a fraction of the box. Bank enters here.
    pub radius_frac: f64,
}

/// Approach (~90 km). Thin air: the chemistry is frozen (`τ_ion ≫` dwell), the Knudsen number
/// sits in the slip band, the electron density stays below the L1 cutoff, and GNSS aids the INS.
pub const APPROACH: FlightCondition = FlightCondition {
    name: "approach_90km",
    mach: 8.0,
    t_inf: 250.0,
    n_tot: 1.0e19,
    mean_free_path: 0.025,
    steps: 40,
    radius_frac: 0.18,
};

/// Peak heating (61 km, the RAM-C II Mach-25 station). Ionization saturates within a few steps
/// and the sheath denies the link: blackout onset, dead reckoning, the branch study.
pub const PEAK: FlightCondition = FlightCondition {
    name: "peak_61km",
    mach: 25.0,
    t_inf: 250.0,
    n_tot: 1.0e22,
    mean_free_path: 2.0e-4,
    steps: 40,
    radius_frac: 0.18,
};

/// Exit (~30 km, decelerated). The sheath along the signal path has cleared; the first fix
/// collapses the accumulated drift.
///
/// **Tier-A label, the largest one.** The LER surrogate carries only the *forward* Park
/// associative-ionization rate, so it has no low-temperature recombination channel. At exit
/// temperatures the lagged fraction freezes instead of recombining, and through the Saha `√n`
/// scaling no `(T, n_tot)` station clears the GPS L1 cutoff chemically. The real mechanism is
/// dissociative recombination (`e⁻ + NO⁺ → N + O`, barrier-free), which clears the sheath quickly
/// once the shock weakens. This station therefore *imposes* the cleared line-of-sight plasma
/// density through its sheath `n_tot` (ionosphere-class, `1e16 m⁻³`). The denial flag, the
/// reacquisition, and the navigation response to it remain fully coupled and real.
pub const EXIT: FlightCondition = FlightCondition {
    name: "exit_30km",
    mach: 7.0,
    t_inf: 226.0,
    n_tot: 1.0e16,
    mean_free_path: 3.0e-6,
    steps: 40,
    radius_frac: 0.18,
};
