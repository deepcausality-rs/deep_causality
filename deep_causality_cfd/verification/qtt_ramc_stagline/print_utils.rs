/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Measurement + gates for the RAM-C stagnation-line verification.

use crate::FloatType;
use crate::config;
use crate::config::{COMMS_BAND_RAD_S, RAMC_ANCHOR_ALTITUDE_KM, RAMC_II_NE_ANCHOR};
use deep_causality_cfd::{EvidenceClass, PostShockState, StagnationOutcome};

/// Lower / upper bound of the post-shock "~10⁴ K" temperature band (K).
const T2_MIN: f64 = 5_000.0;
const T2_MAX: f64 = 60_000.0;
/// Peak electron density regression pin on the Park two-temperature controller. This is a tripwire on
/// the corrected value, not an agreement claim. At the cited RAM-C II 71 km freestream
/// (`fix-ramc-anchor-freestream`) with the N₂–N₂ reduced mass `μ = 14.007`, the controller lands
/// `n_e = 1.07×10¹⁷ m⁻³`, 1.97 decades below the station-1 Ka-band anchor. The band pins that value for
/// regression at roughly ±0.27 decade and is not widened toward the anchor; the summary reports the
/// offset as the result.
const NE_LO: f64 = 6.0e16;
const NE_HI: f64 = 2.0e17;
/// The smooth post-shock relaxation profile must stay `O(1)` rank.
const BOND_CAP: usize = 4;

/// Print the anchor station's flight condition and the intermediate figures of its prediction: the
/// residence time with the Saha-equilibrium bound, the single-temperature surrogate, the finite-rate
/// network with its channel-1 attribution, and the sheath-renewal A/B.
pub fn render_prediction(station: &config::Freestream, p: &crate::Prediction) {
    let dec = |ne: FloatType| (ne / RAMC_II_NE_ANCHOR).log10();
    println!(
        "Flight ({}): M = {}, T_inf = {} K, q = {} Pa -> V = {:.1} m/s, n_inf = {:.4e} m^-3; \
         γ_eff = {}\n",
        station.label,
        station.mach,
        station.t_inf,
        station.dynamic_pressure,
        station.velocity(),
        station.number_density(),
        config::GAMMA,
    );
    println!(
        "Residence time t_res = standoff/u2 = {:.3e} s  (Saha-equilibrium upper bound n_e = {:.3e} m^-3)",
        p.residence_time, p.ne_equilibrium
    );
    println!(
        "Single-T surrogate (ionizes at T₂, over-predicts): α = {:.3e}, n_e = {:.3e} m^-3 ({:+.1} dec vs RAM-C)\n",
        p.outcome_1t.ionization_fraction,
        p.outcome_1t.electron_density,
        dec(p.outcome_1t.electron_density)
    );
    println!(
        "Uncalibrated finite-rate network (RP-1232 Table II pairs; no Saha target):\n  \
         lagged atom pool: x_N = {:.3e}, x_O = {:.3e}\n  \
         channel 1 + pool: n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n  \
         full network:     n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n",
        p.x_n,
        p.x_o,
        p.ne_channel1,
        dec(p.ne_channel1),
        p.ne_network,
        dec(p.ne_network),
    );
    println!(
        "Sheath-renewal A/B under recombination (peak over the transit-age profile):\n  \
         renewal (kept):    n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n  \
         carried (marched): n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n",
        p.ne_network,
        dec(p.ne_network),
        p.ne_carried,
        dec(p.ne_carried),
    );
}

pub fn render(
    station: &config::Freestream,
    post: &PostShockState<f64>,
    out: &StagnationOutcome<f64>,
    profile_bond: usize,
) {
    println!(
        "Exact Rankine–Hugoniot post-shock state (the transported energy, no reconstruction):"
    );
    println!(
        "  T_inf -> T2 ............ {:.1} K -> {:.0} K",
        station.t_inf, post.t2
    );
    println!("  density ratio ρ2/ρ1 ... {:.3}", post.rho_ratio);
    println!("  velocity ratio u2/u1 .. {:.3}", post.u_ratio);
    println!("  pressure ratio p2/p1 .. {:.3e}", post.p_ratio);
    println!("  post-shock n_tot ...... {:.3e} m^-3", post.n_tot2);
    println!("\nStagnation-line blackout (Saha/Park-2T at the post-shock state):");
    println!("  ionization fraction α .. {:.3e}", out.ionization_fraction);
    println!(
        "  peak electron density .. {:.3e} m^-3",
        out.electron_density
    );
    println!(
        "  plasma frequency ω_p ... {:.3e} rad/s",
        out.plasma_frequency
    );
    println!(
        "  blackout (ω_p > comms) . {} (comms band {:.2e} rad/s)",
        out.blackout, COMMS_BAND_RAD_S
    );
    println!("  relaxation-profile bond  {profile_bond}  (smooth post-shock zone, O(1) rank)");
}

pub fn verify(
    post: &PostShockState<f64>,
    out: &StagnationOutcome<f64>,
    profile_bond: usize,
) -> bool {
    println!("\n--- RAM-C stagnation-line gates ---");
    // T2_MIN/T2_MAX are a loose physical expectation, not a cited value -> tripwire.
    let g1 = gate(
        "T2 in the ~10^4 K post-shock band",
        EvidenceClass::Tripwire,
        post.t2 > T2_MIN && post.t2 < T2_MAX,
    );
    // The corrected Park-2T controller lands 1.97 decades below the RAM-C II anchor. The bound pins that
    // corrected value for regression rather than asserting agreement, so the label is the weaker
    // Tripwire class and the offset is reported, not absorbed (spec: a prediction outside the anchor
    // band is reported, not re-admitted).
    let g2_decades = (out.electron_density / RAMC_II_NE_ANCHOR).log10();
    let g2 = gate(
        &format!(
            "peak n_e is the corrected Park-2T value ({g2_decades:+.2} dec vs RAM-C II; \
             a regression pin, reported, not re-admitted)"
        ),
        EvidenceClass::Tripwire,
        out.electron_density > NE_LO && out.electron_density < NE_HI,
    );
    let g3 = gate(
        "blackout onset (ω_p > comms band)",
        EvidenceClass::Tripwire,
        out.blackout,
    );
    let g4 = gate(
        "relaxation profile O(1) rank",
        EvidenceClass::Tripwire,
        profile_bond <= BOND_CAP,
    );
    g1 && g2 && g3 && g4
}

fn gate(label: &str, evidence: EvidenceClass, pass: bool) -> bool {
    println!(
        "  [{}] [{evidence}] {label}",
        if pass { "PASS" } else { "FAIL" }
    );
    pass
}

pub fn summary(
    station: &config::Freestream,
    out: &StagnationOutcome<f64>,
    ne_network: FloatType,
    ne_network_61km: FloatType,
) {
    let decades = (out.electron_density / RAMC_II_NE_ANCHOR).log10();
    let decades_network = (ne_network / RAMC_II_NE_ANCHOR).log10();
    println!(
        "\n=== RAM-C stagnation line: network n_e = {:.2e} m^-3 ({:+.2} dec), Park-2T controller \
         {:.2e} m^-3 ({:+.2} dec) vs the station-1 Ka-band anchor {:.3e} m^-3 at {RAMC_ANCHOR_ALTITUDE_KM} \
         km; 61 km network {:.2e} m^-3 above the flight lower bound. ===",
        ne_network,
        decades_network,
        out.electron_density,
        decades,
        RAMC_II_NE_ANCHOR,
        ne_network_61km
    );
    println!(
        "The freestream is the cited {} condition (Mach {}, {} K, q {:.2} kPa); velocity\n\
         and number density are derived from it. The anchor is the flight's station-1 Ka-band\n\
         critical-density crossing, which Grantham (1970) places at 71.93 km; below it the flight's nose\n\
         plasma is overdense, so the 61 km comparison is against a lower bound.\n\
         \n\
         Ionization in the Park-2T controller is driven off Tₐ = √(T_tr·T_ve), with T_ve relaxed from the\n\
         free-stream value over the residence time by the closed-form Millikan–White LER kernel, at the\n\
         N₂–N₂ reduced mass μ = 14.007. The controller's offset is reported as the result; its band is a\n\
         regression pin, not widened toward the anchor. Read the single-pair figure as a lower bound on\n\
         n_e: the bath also holds lighter partners (N, O) whose shorter τ_vt a mixture-weighted closure\n\
         would recover.\n\
         \n\
         Open levers: the T_e = T_ve lumping (a 3-T separation is ~2x), the single associative-ionization\n\
         channel, and the ~2-5x Millikan–White chemistry-model spread. The effective γ = 1.1 is a\n\
         reacting-air closure; T2 is the exact-RH transported energy at that γ.",
        station.label,
        station.mach,
        station.t_inf,
        station.dynamic_pressure / 1000.0,
    );
}

/// Gate the uncalibrated network prediction against the station-1 Ka-band anchor. The ±0.70-decade
/// width is a chemistry-model-spread allowance (rate sets spread 2x to 5x; production codes land 2x
/// to 3x on the RAM-C peak `n_e`); it is stated without a citation and is not derived from the
/// flight's own uncertainty, so the gate is a tripwire. The channel-1 measurement exists for
/// attribution: if the full network ever leaves its band, the two numbers say which channel moved.
pub fn verify_network(ne_channel1: FloatType, ne_network: FloatType) -> bool {
    // The RAM-C II anchor these bounds are centred on is external, but the band WIDTH is a chosen
    // allowance, so clearing it is evidence of non-regression, not of agreement with flight data.
    // The gate text says so; the label makes it machine-visible.
    let dec_network = (ne_network / RAMC_II_NE_ANCHOR).log10();
    let inside = gate(
        &format!(
            "network prediction inside the allowance: full network {:+.2} dec vs the {:.3e} m^-3 \
             station-1 Ka-band anchor at {RAMC_ANCHOR_ALTITUDE_KM} km, 0.93 km above the cited 71 km \
             freestream (allowance +-{:.2} dec, the chemistry-model spread)",
            dec_network,
            RAMC_II_NE_ANCHOR,
            config::RAMC_II_ALLOWANCE_DECADES,
        ),
        EvidenceClass::Tripwire,
        dec_network.abs() <= config::RAMC_II_ALLOWANCE_DECADES,
    );
    let refinement = gate(
        &format!(
            "electron impact is a refinement, not the driver: channel 1 + pool {:.3e} vs full \
             network {:.3e} m^-3 (the associative channel carries the prediction at RAM-C speeds)",
            ne_channel1, ne_network,
        ),
        EvidenceClass::Tripwire,
        ne_network >= ne_channel1 && ne_network < ne_channel1 * 10.0,
    );
    inside && refinement
}

/// The sheath-renewal A/B under recombination, superseding the forward-only
/// surrogate's A/B where renewal was load-bearing against runaway. Renewal
/// is kept: its clock is evaluated at the network fixed point, which is the
/// true Riccati relaxation rate `sqrt(production*beta)` of the two-way
/// balance near equilibrium, and it realizes the transit-age closure the
/// anchor gate is pinned on (each depth an independent parcel). The carried
/// arm rates its clock at the young carried population, so it approaches the
/// same fixed point more slowly. The gate asserts the property the
/// recombination channel was added for: the carried march self-limits at or
/// below the closed-form arm, where the forward-only surrogate ran away.
/// The carried arm lands below the network's ±0.70 renewal band; that offset is
/// reported, not gated.
pub fn verify_renewal_ab(ne_renewal: FloatType, ne_carried: FloatType) -> bool {
    let dec_carried = (ne_carried / RAMC_II_NE_ANCHOR).log10();
    // Runaway prevention is the property this A/B was added for: the carried march self-limits at or
    // below the renewal arm, where the forward-only surrogate ran away. That is the PASS condition.
    // The carried arm lands below the ±0.70 network band the renewal arm sits in, so the offset is
    // reported rather than gated (not widened to re-admit).
    gate(
        &format!(
            "carried mode self-limits at or below the renewal arm (no runaway): carried \
             {dec_carried:+.2} dec vs the anchor, below the network's ±{:.2}-dec renewal band \
             (reported, not re-admitted; the carried clock under-relaxes young sheath gas)",
            config::RAMC_II_ALLOWANCE_DECADES,
        ),
        EvidenceClass::Tripwire,
        ne_carried <= ne_renewal,
    )
}

/// Report the lower-bound station: the network's peak `n_e` at the cited 61 km freestream beside the
/// flight datum, which is a lower bound there.
pub fn render_lower_bound(station: &config::Freestream, lower: &crate::Prediction) {
    println!(
        "\nLower-bound station ({}: M = {}, T_inf = {} K, q = {} Pa -> V = {:.1} m/s, \
         n_inf = {:.4e} m^-3):",
        station.label,
        station.mach,
        station.t_inf,
        station.dynamic_pressure,
        station.velocity(),
        station.number_density(),
    );
    println!(
        "  T2 = {:.0} K, network n_e = {:.3e} m^-3 ({:+.2} dec vs the {:.3e} m^-3 flight lower bound), \
         Park-2T controller n_e = {:.3e} m^-3",
        lower.post.t2,
        lower.ne_network,
        (lower.ne_network / RAMC_II_NE_ANCHOR).log10(),
        RAMC_II_NE_ANCHOR,
        lower.outcome.electron_density,
    );
}

/// Gate the network at the 61 km station against the flight's lower bound. Station 1 is overdense
/// at Ka-band below the 71.93 km crossing, so the flight's peak electron density there is at least
/// the Ka-band datum. The bound is the flight measurement itself, with no allowance: a
/// `[reference]` gate.
pub fn verify_lower_bound(ne_network_61km: FloatType) -> bool {
    gate(
        &format!(
            "network n_e at the 61 km station meets the flight lower bound: {:.3e} >= {:.3e} m^-3 \
             (station 1 overdense at Ka-band below {RAMC_ANCHOR_ALTITUDE_KM} km; Grantham 1970, p. 18)",
            ne_network_61km, RAMC_II_NE_ANCHOR,
        ),
        EvidenceClass::Reference,
        ne_network_61km >= RAMC_II_NE_ANCHOR,
    )
}
