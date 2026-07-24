/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration for the Tier-B Stage-4 RAM-C stagnation-line verification: the flight condition and the
//! published reference anchors. `main.rs` computes the exact Rankine–Hugoniot post-shock state and the
//! resulting peak electron density; `print_utils.rs` gates it against RAM-C II.

use crate::FloatType;
use deep_causality_num::FromPrimitive;

// ── Flight condition (RAM-C II, ~71 km station) ──────────────────────────
/// Free-stream Mach number (`M ≈ 25` orbital reentry).
pub const MACH: f64 = 25.0;
/// **Effective** post-shock ratio of specific heats for reacting air. Perfect-gas `1.4` over-predicts
/// `T₂` badly (≈30 000 K) because it ignores the dissociation/vibration that absorb the post-shock energy;
/// the engineering effective value for strongly-dissociated hypersonic air is `≈1.1–1.2`, which lands `T₂`
/// in the realistic ≈8000 K band where RAM-C ionizes. Cited as an effective-γ closure, not perfect gas.
pub const GAMMA: f64 = 1.1;
/// Free-stream (ambient) temperature, K.
pub const T_INF: f64 = 250.0;
/// Free-stream heavy-particle number density, m⁻³ (RAM-C II ~71 km: ρ∞ ≈ 6.4e-5 kg/m³, air mass
/// ≈ 4.8e-26 kg → n∞ ≈ 1.3e21).
pub const NUMBER_DENSITY: f64 = 1.3e21;
/// Comms band as an angular frequency (GPS L-band ≈ 1.5 GHz → ω ≈ 9.4e9 rad/s).
pub const COMMS_BAND_RAD_S: f64 = 9.4e9;
/// Free-stream velocity, m/s (RAM-C orbital reentry ≈ 7.65 km/s).
pub const FREESTREAM_VELOCITY: f64 = 7650.0;
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

// ── Published reference cross-references (reported, with disclaimers) ─────
/// RAM-C II peak electron density near the 71 km station, m⁻³ (order-of-magnitude anchor).
pub const RAMC_NE_REFERENCE: f64 = 1.0e19;

/// Acceptance band of the uncalibrated finite-rate network prediction, in decades around the
/// flight anchor. The width is a chemistry-model-spread allowance — production codes (DPLR/LAURA/US3D)
/// sit at 2x to 3x, rate sets spread 2x to 5x — and is therefore independent of `μ_sr`. Re-confirmed
/// under the corrected N₂–N₂ closure (`fix-ramc-vibrational-relaxation-pair`): the network **renewal**
/// arm moved from +0.48 to **+0.35 dec** (the correction pulls the over-predicting network *toward* the
/// anchor), still inside ±0.70. The width is not re-tuned; only its verdict is re-measured.
pub const NETWORK_BAND_DECADES: f64 = 0.7;

/// Lift an exact `f64` specification into the working precision.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}
