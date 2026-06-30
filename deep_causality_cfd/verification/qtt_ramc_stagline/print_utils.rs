/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Measurement + gates for the RAM-C stagnation-line verification.

use crate::config::{COMMS_BAND_RAD_S, RAMC_NE_REFERENCE, T_INF};
use deep_causality_cfd::{PostShockState, StagnationOutcome};

/// Lower / upper bound of the post-shock "~10⁴ K" temperature band (K).
const T2_MIN: f64 = 5_000.0;
const T2_MAX: f64 = 60_000.0;
/// Peak electron density gate. The Gap-3 Park two-temperature controller (ionization off
/// `Tₐ = √(T_tr·T_ve)`, not the hot `T₂`) lands `n_e ≈ 1.1×10¹⁹` — within ~half a decade (~3×) of the
/// RAM-C II anchor `1×10¹⁹`, the production chemistry-spread band. This replaces the old ~2-decade
/// order-of-magnitude gate the single-temperature surrogate needed.
const NE_LO: f64 = 3.0e18;
const NE_HI: f64 = 3.0e19;
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
    let g1 = gate(
        "T2 in the ~10^4 K post-shock band",
        post.t2 > T2_MIN && post.t2 < T2_MAX,
    );
    let g2 = gate(
        "peak n_e within ~3× of RAM-C II (Park-2T controller)",
        out.electron_density > NE_LO && out.electron_density < NE_HI,
    );
    let g3 = gate("blackout onset (ω_p > comms band)", out.blackout);
    let g4 = gate("relaxation profile O(1) rank", profile_bond <= BOND_CAP);
    g1 && g2 && g3 && g4
}

fn gate(label: &str, pass: bool) -> bool {
    println!("  [{}] {label}", if pass { "PASS" } else { "FAIL" });
    pass
}

pub fn summary(out: &StagnationOutcome<f64>) {
    let decades = (out.electron_density / RAMC_NE_REFERENCE).log10();
    println!(
        "\n=== RAM-C stagnation line: peak n_e = {:.2e} m^-3, {:+.1} decades vs the RAM-C II anchor {:.0e}. ===",
        out.electron_density, decades, RAMC_NE_REFERENCE
    );
    println!(
        "Ionization is driven off the Park rate-controlling temperature Tₐ = √(T_tr·T_ve), with the lagging\n\
         vibrational-electron temperature T_ve relaxed from the free-stream value over the residence time by\n\
         the closed-form Millikan–White LER kernel — not the hot translational T₂. This is the Gap-3\n\
         chemistry-fidelity upgrade: it takes peak n_e from the single-temperature surrogate's ~12× over-\n\
         prediction down to ~1.1× of the RAM-C II anchor.\n\
         Disclaimer: still a two-temperature Saha surrogate. The T_e = T_ve lumping (a 3-T electron-energy\n\
         separation is ~2×) and the single associative-ionization channel (vs a finite-rate associative +\n\
         electron-impact + recombination network) remain open levers, and the exact landing is sensitive to\n\
         the Millikan–White τ_vt model (the documented ~2–5× chemistry-model spread). The effective γ = 1.1\n\
         lands T2 in the realistic ~8000 K reacting-air band. The post-shock T2 is the exact-RH transported\n\
         energy, retiring the Tier-A recovery-temperature reconstruction."
    );
}
