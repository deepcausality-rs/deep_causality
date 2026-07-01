// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! Trajectory FS-3 — the clock: is the regularising "two-time" parameter the proper time, and does a
//! forward `dτ/dt` kernel hit ns-level known relativistic offsets? (Gap-3 Resolution-3, de-risking item ⑤.)
//!
//! Two findings:
//!  (a) **Conceptual.** The fictitious time of FS-1 (eccentric anomaly / KS `s`, with `dt = (r/…)·ds`) is a
//!      *regularising reparametrisation*, NOT proper time. Proper time `τ` is a separate GR+SR integral.
//!      Resolution 1 conflates them ("τ↔t is what 2T physics is about"); they are distinct roles and the
//!      spec must carry **two** clocks — the linearising `s` and the relativistic `τ`.
//!  (b) **Numerical.** The missing forward clock kernel `dτ/dt = 1 + Φ/c² − v²/(2c²)` (Φ = −μ/r) reproduces
//!      the textbook GPS relativistic split to sub-µs/day — the canonical falsifiable anchor.
//!
//! Measurements (gates encode the finding; exit non-zero on regression):
//!   G1  gravitational (blueshift) offset of a GPS clock vs the geoid ≈ +45.7 µs/day  (|·| within 1.0).
//!   G2  velocity (time-dilation) offset of a GPS clock ≈ −7.2 µs/day                 (|·| within 0.5).
//!   G3  net GPS offset ≈ +38.5 µs/day                                                (|·| within 1.0).
//!   G4  the reentry-blackout carry: the accumulated τ−t offset over a 180 s GNSS-denied window at reentry
//!       conditions is finite and reported in ns and metres (×0.3 m/ns) — "carry the clock internally".

// The forward clock rate offset was PROMOTED out of this study into a reusable physics kernel
// (`relativistic_clock_offset_kernel`, capability ⑤). The study now consumes the shipped kernel.
use deep_causality_physics::{
    EARTH_GM, EARTH_RADIUS_EQUATORIAL, SPEED_OF_LIGHT, relativistic_clock_offset_kernel,
};

const SECONDS_PER_DAY: f64 = 86_400.0;
const GPS_SEMI_MAJOR_M: f64 = 26_560_000.0; // GPS orbit radius (~20,200 km altitude)
const METRES_PER_NS: f64 = 0.299_792_458; // 1 ns of clock error ⇒ this many metres of ranging error

fn gate(label: &str, pass: bool) -> bool {
    println!("  [{}] {label}", if pass { "PASS" } else { "FAIL" });
    pass
}

fn main() {
    println!("=== FS-3: the relativistic clock — forward dτ/dt vs the textbook GPS split ===\n");
    let mu = EARTH_GM;
    let c = SPEED_OF_LIGHT;
    let r_e = EARTH_RADIUS_EQUATORIAL;

    // (a) Conceptual finding — the two distinct clocks.
    println!(
        "Conceptual: the FS-1 linearising parameter s (eccentric anomaly / KS, dt = (r/na)·ds) is a\n\
         REGULARISING reparametrisation, not proper time. Proper time τ is the GR+SR integral below. The\n\
         trajectory spec must carry BOTH: s (to make the gravity core a matrix exponential) and τ (the\n\
         relativistic clock). Resolution-1's 'τ↔t is native' is true only for τ, not the linearising s.\n"
    );

    // (b) GPS split via the forward kernel — split the rate into its two physical pieces for reporting.
    let v_gps = (mu / GPS_SEMI_MAJOR_M).sqrt(); // circular orbital speed
    let grav_rate = (mu / r_e - mu / GPS_SEMI_MAJOR_M) / (c * c);
    let vel_rate = -(v_gps * v_gps) / (2.0 * c * c);
    let grav_us_day = grav_rate * SECONDS_PER_DAY * 1.0e6;
    let vel_us_day = vel_rate * SECONDS_PER_DAY * 1.0e6;
    // Sanity: the combined kernel equals the sum of the two pieces (geoid reference at rest).
    let net_rate = relativistic_clock_offset_kernel(GPS_SEMI_MAJOR_M, v_gps, r_e, 0.0, mu).unwrap();
    let net_us_day = net_rate * SECONDS_PER_DAY * 1.0e6;

    println!("GPS satellite clock vs a geoid clock (forward dτ/dt kernel):");
    println!("  orbital speed v = {v_gps:.1} m/s");
    println!(
        "  gravitational (higher potential → faster) : {grav_us_day:+.2} µs/day   [textbook +45.7]"
    );
    println!(
        "  velocity (time dilation → slower)         : {vel_us_day:+.2} µs/day   [textbook  −7.2]"
    );
    println!(
        "  net                                       : {net_us_day:+.2} µs/day   [textbook +38.5]\n"
    );

    // (c) Reentry GNSS-blackout carry: accumulate the τ−t offset over a 180 s denied window.
    let v_reentry = 7650.0; // RAM-C orbital reentry ~7.65 km/s
    let alt_reentry = 71_000.0; // ~71 km station (the RAM-C blackout altitude)
    let r_reentry = r_e + alt_reentry;
    let blackout_s = 180.0; // ~3 min GNSS-denial window
    let reentry_rate =
        relativistic_clock_offset_kernel(r_reentry, v_reentry, r_e, 0.0, mu).unwrap();
    let offset_ns = reentry_rate * blackout_s * 1.0e9;
    let ranging_m = offset_ns.abs() * METRES_PER_NS;
    println!("Reentry blackout carry (vehicle clock vs surface, {blackout_s:.0} s denied window):");
    println!(
        "  v = {v_reentry:.0} m/s, alt = {:.0} km",
        alt_reentry / 1000.0
    );
    println!(
        "  accumulated τ−t offset = {offset_ns:+.1} ns  ⇒  {ranging_m:.1} m ranging drift if uncorrected\n"
    );

    println!("--- FS-3 gates ---");
    let g1 = gate(
        "GPS gravitational offset ≈ +45.7 µs/day",
        (grav_us_day - 45.7).abs() < 1.0,
    );
    let g2 = gate(
        "GPS velocity offset ≈ −7.2 µs/day",
        (vel_us_day + 7.2).abs() < 0.5,
    );
    let g3 = gate(
        "GPS net offset ≈ +38.5 µs/day",
        (net_us_day - 38.5).abs() < 1.0,
    );
    let g4 = gate(
        "reentry-blackout clock carry is finite and bounded (10–200 ns)",
        offset_ns.abs() > 10.0 && offset_ns.abs() < 200.0,
    );

    if g1 && g2 && g3 && g4 {
        println!(
            "\n=== FINDING: the forward dτ/dt kernel (the missing capability ⑤) reproduces the textbook GPS\n\
             relativistic split to sub-µs/day, so ns-level onboard timing is feasible with the existing\n\
             constants. The linearising s and the proper time τ are DISTINCT clocks — the spec must carry\n\
             both. Over a 3-min reentry blackout the uncorrected clock drifts ~tens of metres, quantifying\n\
             why the correction must be carried internally (B3). ==="
        );
    } else {
        std::process::exit(1);
    }
}
