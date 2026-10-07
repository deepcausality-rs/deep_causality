/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration for the Tier-B Stage-4 RAM-C stagnation-line verification: the cited flight conditions
//! and the published reference anchor. `main.rs` computes the exact Rankine–Hugoniot post-shock state and the
//! resulting peak electron density; `print_utils.rs` gates it against RAM-C II.

use crate::FloatType;
use deep_causality_num::FromPrimitive;
use deep_causality_physics::{AVOGADRO_CONSTANT, MOLAR_GAS_CONSTANT};

// ── Flight conditions (RAM-C II freestream, cited; velocity and density derived)

/// One RAM-C II freestream station, as the literature states it: Mach number, static temperature
/// and dynamic pressure (Parent, Thoguluva Rajendran & Omprakas, "Electron Losses in Hypersonic
/// Flows", arXiv:2111.09432, §RAM-C-II; the Mach numbers also in Rodriguez Fuentes & Parent, Phys.
/// Fluids 37, 013609, 2025). Velocity and number density follow from these three values and the
/// U.S. Standard Atmosphere 1976 air molar mass, so they cannot drift from the stated condition.
#[derive(Debug, Clone, Copy)]
pub struct Freestream {
    /// Station label for the report.
    pub label: &'static str,
    /// Free-stream Mach number.
    pub mach: f64,
    /// Free-stream static temperature, K.
    pub t_inf: f64,
    /// Free-stream dynamic pressure `q = ½ρV²`, Pa.
    pub dynamic_pressure: f64,
}

/// Mean molar mass of sea-level dry air, kg/mol (U.S. Standard Atmosphere 1976, `M₀`).
pub const AIR_MOLAR_MASS: f64 = 28.9644e-3;
/// Free-stream ratio of specific heats (cold air) for the sound speed `a = √(γ_∞ R T)`.
pub const GAMMA_INF: f64 = 1.4;

impl Freestream {
    /// Free-stream velocity `V = M·√(γ_∞·R·T)`, m/s.
    pub fn velocity(&self) -> f64 {
        let r_specific = MOLAR_GAS_CONSTANT / AIR_MOLAR_MASS;
        self.mach * (GAMMA_INF * r_specific * self.t_inf).sqrt()
    }

    /// Free-stream heavy-particle number density `n = ρ·N_A/M` with `ρ = 2q/V²`, m⁻³.
    pub fn number_density(&self) -> f64 {
        let v = self.velocity();
        let rho = 2.0 * self.dynamic_pressure / (v * v);
        rho * AVOGADRO_CONSTANT / AIR_MOLAR_MASS
    }
}

/// The RAM-C II 71 km station: the cited freestream nearest the flight anchor (the station-1
/// Ka-band crossing at 71.93 km, [`RAMC_ANCHOR_ALTITUDE_KM`]). The 0.93 km between the two is
/// part of the comparison and is printed with it.
pub const ANCHOR_STATION: Freestream = Freestream {
    label: "RAM-C II 71 km",
    mach: 25.9,
    t_inf: 217.9,
    dynamic_pressure: 2_280.0,
};

/// The RAM-C II 61 km station: inside the span where the flight's station 1 is overdense at
/// Ka-band, so the flight datum is a lower bound there.
pub const LOWER_BOUND_STATION: Freestream = Freestream {
    label: "RAM-C II 61 km",
    mach: 23.9,
    t_inf: 255.9,
    dynamic_pressure: 8_000.0,
};

/// **Effective** post-shock ratio of specific heats for reacting air. Perfect-gas `1.4` over-predicts
/// `T₂` badly (≈30 000 K) because it ignores the dissociation/vibration that absorb the post-shock energy;
/// the engineering effective value for strongly-dissociated hypersonic air is `≈1.1–1.2`, which lands `T₂`
/// in the realistic ≈7500–8000 K band where RAM-C ionizes. Cited as an effective-γ closure, not perfect gas.
pub const GAMMA: f64 = 1.1;
/// Comms band as an angular frequency (GPS L-band ≈ 1.5 GHz → ω ≈ 9.4e9 rad/s).
pub const COMMS_BAND_RAD_S: f64 = 9.4e9;
/// Shock standoff on the stagnation line, m (≈0.05·nose radius for the RAM-C sphere-cone) — sets the
/// post-shock residence time `t_res = standoff / u₂` over which ionization lags equilibrium.
pub const STANDOFF_M: f64 = 0.0076;

// ── Park two-temperature ionization closure (the Gap-3 chemistry-fidelity controller) ──
/// Reduced mass `μ_sr` of the dominant relaxing collision pair, in amu — sets the Millikan–White
/// vibrational relaxation time `τ_vt` that controls how far the lagging `T_ve` catches up. Defined once
/// in `deep_causality_cfd`, next to the `Park2tClosure` it feeds; re-exported here so this harness and
/// the shared avionics world cannot drift to different values.
pub use deep_causality_cfd::REDUCED_MASS_AMU;
/// Standard atmosphere, Pa — converts the post-shock pressure to atm for the Millikan–White correlation.
pub const STANDARD_ATMOSPHERE_PA: f64 = 101_325.0;

// ── Post-shock relaxation profile (the smooth fitted-interior zone) ───────
/// QTT mode count for the 1-D relaxation profile (`2^L` points along the streamline).
pub const PROFILE_L: usize = 10;
/// Relaxation length as a fraction of the sampled streamwise extent.
pub const RELAX_LENGTH: f64 = 0.2;

// ── Published reference (Grantham 1970), defined once in `deep_causality_cfd` ─────
/// The RAM-C II station-1 Ka-band anchor, its altitude, and the allowance around it. Defined once in
/// `deep_causality_cfd` and re-exported here, so this harness and the plasma-blackout corridor
/// compare against the same datum with the same allowance. The allowance applies at
/// [`ANCHOR_STATION`] under the N₂–N₂ Millikan–White closure ([`REDUCED_MASS_AMU`]); it is a chosen
/// chemistry-model spread, independent of `μ_sr`, so the gate it sets is a `[tripwire]`.
pub use deep_causality_cfd::{
    RAMC_II_ALLOWANCE_DECADES, RAMC_II_ANCHOR_ALTITUDE_M, RAMC_II_NE_ANCHOR,
};

/// [`RAMC_II_ANCHOR_ALTITUDE_M`] in km, for the report.
pub const RAMC_ANCHOR_ALTITUDE_KM: f64 = RAMC_II_ANCHOR_ALTITUDE_M / 1000.0;

/// Lift an exact `f64` specification into the working precision.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}
