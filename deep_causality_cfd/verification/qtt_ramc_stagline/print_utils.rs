/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Measurement + gates for the RAM-C stagnation-line verification.

use crate::FloatType;
use crate::config;
use crate::config::{COMMS_BAND_RAD_S, RAMC_NE_REFERENCE, T_INF};
use deep_causality_cfd::{EvidenceClass, PostShockState, StagnationOutcome};

/// Lower / upper bound of the post-shock "~10⁴ K" temperature band (K).
const T2_MIN: f64 = 5_000.0;
const T2_MAX: f64 = 60_000.0;
/// Peak electron density regression pin. This is a tripwire on the corrected value, not an agreement
/// claim. With the N₂–N₂ reduced mass corrected to `μ = 14.007` (`fix-ramc-vibrational-relaxation-pair`),
/// the Park two-temperature controller lands `n_e ≈ 5.3×10¹⁷`. That is 1.27 decades below the RAM-C II
/// anchor `1×10¹⁹`, not the `+0.0` the superseded `μ = 7.0` produced. The prediction no longer supports
/// an order-of-magnitude agreement claim, so per the audit (design D3) the band is not widened to
/// re-admit the old headline. It pins the corrected value for regression at roughly ±0.27 decade, and
/// the summary reports the offset as the result.
const NE_LO: f64 = 3.0e17;
const NE_HI: f64 = 1.0e18;
/// The smooth post-shock relaxation profile must stay `O(1)` rank.
const BOND_CAP: usize = 4;

pub fn render(post: &PostShockState<f64>, out: &StagnationOutcome<f64>, profile_bond: usize) {
    println!(
        "Exact Rankine–Hugoniot post-shock state (the transported energy, no reconstruction):"
    );
    println!(
        "  T_inf -> T2 ............ {T_INF:.0} K -> {:.0} K",
        post.t2
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
    // The corrected Park-2T controller lands 1.27 decades below the RAM-C II anchor. The bound now pins
    // that corrected value for regression rather than asserting agreement, so the label is the weaker
    // Tripwire class and the offset is reported, not absorbed (spec: a prediction outside the anchor
    // band is reported, not re-admitted to the old headline).
    let g2_decades = (out.electron_density / RAMC_NE_REFERENCE).log10();
    let g2 = gate(
        &format!(
            "peak n_e is the corrected Park-2T value ({g2_decades:+.2} dec vs RAM-C II; \
             reported, not re-admitted to the old +0.0 headline)"
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

pub fn summary(out: &StagnationOutcome<f64>) {
    let decades = (out.electron_density / RAMC_NE_REFERENCE).log10();
    println!(
        "\n=== RAM-C stagnation line: Park-2T peak n_e = {:.2e} m^-3, {:+.1} decades vs the RAM-C II anchor {:.0e}. ===",
        out.electron_density, decades, RAMC_NE_REFERENCE
    );
    println!(
        "Ionization is driven off the Park rate-controlling temperature Tₐ = √(T_tr·T_ve). The lagging\n\
         vibrational-electron temperature T_ve relaxes from the free-stream value over the residence time\n\
         by the closed-form Millikan–White LER kernel, not the hot translational T₂.\n\
         \n\
         The reduced mass is the N₂–N₂ pair, μ = 14.007. The earlier +0.0-decade headline was an artifact\n\
         of an invalid μ = 7.0 (the N–N atomic pair, which has no vibrational mode). Correcting it\n\
         lengthens τ_vt about 1.9x, cools Tₐ, and drops the Park-2T controller to 1.27 decades below the\n\
         anchor. That offset is reported here as the result; the band was not widened to restore the old\n\
         number. The uncalibrated finite-rate network's renewal arm still lands +0.35 decade, so its\n\
         order-of-magnitude prediction survives the correction while the closed-form controller does not.\n\
         Read the single-pair figure as a lower bound on n_e: the bath also holds lighter partners (N, O)\n\
         whose shorter τ_vt a mixture-weighted closure would recover.\n\
         \n\
         Open levers, unchanged by this fix: the T_e = T_ve lumping (a 3-T separation is ~2x), the single\n\
         associative-ionization channel, and the ~2-5x Millikan–White chemistry-model spread. The\n\
         effective γ = 1.1 lands T2 in the realistic ~8000 K reacting-air band, and that T2 is the\n\
         exact-RH transported energy."
    );
}

/// Gate the uncalibrated network prediction. The band is pinned from the
/// measurement (see `baseline.txt`), justified against the production-code
/// context: DPLR, LAURA, and US3D land 2x to 3x on the RAM-C peak `n_e`,
/// with a 2x to 5x chemistry-model spread between rate sets. The channel-1
/// measurement exists for attribution (design D7): if the full network ever
/// leaves its band, the two numbers say which channel moved.
pub fn verify_network(ne_channel1: FloatType, ne_network: FloatType) -> bool {
    let mut ok = true;
    // Every bound in this block is pinned from this harness's own measurement (see `baseline.txt`),
    // so all of them are tripwires. The RAM-C II anchor they are *centred* on is external, but the
    // band WIDTH is chosen — clearing it is evidence of non-regression, not of agreement with
    // flight data. The gate text says so; the label makes it machine-visible.
    let mut gate = |label: &str, pass: bool, detail: String| {
        println!(
            "  [{}] [{}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" },
            EvidenceClass::Tripwire
        );
        ok &= pass;
    };
    let dec_network = (ne_network / RAMC_NE_REFERENCE).log10();
    gate(
        "network prediction inside the earned band",
        dec_network.abs() <= config::NETWORK_BAND_DECADES,
        format!(
            "full network {:+.2} dec vs the flight anchor (band +-{:.2} dec, pinned from the \
             measurement; production codes sit at 2x to 3x)",
            dec_network,
            config::NETWORK_BAND_DECADES,
        ),
    );
    gate(
        "electron impact is a refinement, not the driver",
        ne_network >= ne_channel1 && ne_network < ne_channel1 * 10.0,
        format!(
            "channel 1 + pool {:.3e} vs full network {:.3e} m^-3 (the associative channel \
             carries the prediction at RAM-C speeds)",
            ne_channel1, ne_network,
        ),
    );
    ok
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
/// Under the corrected μ = 14 the carried arm lands -0.75 dec, below the
/// network's ±0.70 renewal band; that offset is reported, not gated (D3).
pub fn verify_renewal_ab(ne_renewal: FloatType, ne_carried: FloatType) -> bool {
    let dec_carried = (ne_carried / RAMC_NE_REFERENCE).log10();
    // Runaway prevention is the property this A/B was added for: the carried march self-limits at or
    // below the renewal arm, where the forward-only surrogate ran away. That is the PASS condition.
    // Under the corrected μ the carried arm lands -0.75 dec, below the ±0.70 network band the renewal
    // arm sits in, so the offset is reported rather than gated (design D3: not widened to re-admit).
    let pass = ne_carried <= ne_renewal;
    println!(
        "  [{}] [{}] carried mode self-limits at or below the renewal arm (no runaway): carried \
         {dec_carried:+.2} dec vs the anchor, below the network's ±{:.2}-dec renewal band (reported, \
         not re-admitted; the carried clock under-relaxes young sheath gas)",
        if pass { "PASS" } else { "FAIL" },
        EvidenceClass::Tripwire,
        config::NETWORK_BAND_DECADES,
    );
    pass
}
