/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This example's own tuned knobs. Everything shared with the corridor and weather examples —
//! the atmosphere, the carrier configuration, the vehicle, the propulsion constants, and the
//! powered-descent envelope — lives in `avionics_examples::shared::constants`.
//!
//! Every gated band is a `const` here because a gate is a plain `fn` pointer that captures nothing:
//! a band held anywhere else cannot be reached from the gate that enforces it. Bands marked
//! *earned* were pinned from the first measured run and now gate regressions.

/// The measured day's temperature departure, K. A cold day: the atmosphere the vehicle actually
/// flies, and the day whose dispersion row an informed guidance interpolates.
pub const MEASURED_D_TEMP: f64 = -32.0;
// The measured day's density scale is deliberately **not** a constant here. It is read from the
// day's interpolated dispersion row, so the atmosphere flown and the belief printed cannot disagree.
// The hand-set 1.15 that used to live here differed from the table's 1.1467 at this departure, and a
// two-decimal print format made the two look identical.

/// Steps for the Act-1 corridor leg (to blackout onset).
pub const ONSET_STEPS: usize = 400;
/// Horizon for the Act-2/3 coast-commit-burn leg. One march call: a leg boundary at ignition would
/// re-seed the flow and the fork would fork a state with the plume already discarded.
///
/// This is a **horizon, not a duration** — the leg stops on its `ignited` predicate. It must be
/// large enough for the coast to actually reach the ignition corridor: the vehicle enters this leg
/// near 52 km at Mach 6.8 and the corridor sits in the Jarvinen–Adams band (Mach 0.4–2.0), around
/// 33 km. A horizon that expires first pauses the march mid-coast, and the fork below would then
/// fork a state that has not ignited — every branch would read a zero preserved-drag fraction and
/// the flow witnesses would have nothing to measure.
pub const BURN_STEPS: usize = 2600;
/// Steps each forked branch continues for.
pub const BRANCH_STEPS: usize = 120;
/// Horizon for the supersonic burn leg, from the fork point down to the subsonic handover.
pub const BURN_OUT_STEPS: usize = 2600;
/// Flight Mach at which the supersonic-retropulsion envelope stops describing the physics and the
/// leg hands over to the subsonic landing burn. Below the Jarvinen-Adams dataset's floor (Mach 0.4)
/// with margin: there is no bow shock for the plume to interact with here.
pub const SUBSONIC_HANDOVER_MACH: f64 = 0.6;
/// Steps for the terminal leg after cutoff.
pub const TERMINAL_STEPS: usize = 3000;

/// The mid-burn throttle roster. On-axis magnitudes only — no angle of attack, the design note's §6
/// discipline pin.
///
/// **Spanning the band the A0 correlation resolves.** With the reference area derived from the flown
/// ballistic bundle (see `PLUME_S_REF_M2`), `C_T` at the fork runs from 0.45 at the 0.20 branch to
/// 1.90 at 0.85 — across the range where preserved drag actually collapses, rather than bunched in
/// its shallow end or saturated on its plateau.
///
/// The roster was previously capped near 0.44 because the envelope's dynamic `C_T` ceiling bound
/// there. That ceiling was computed against a reference area three times too small; corrected, it
/// sits at 1.35 and never binds, so the admissible band is the engine's own throttle range. Gate (4f)
/// still fails a roster whose branches collapse onto one realized throttle, so a future envelope
/// change that re-introduces clamping is a run failure rather than a silent one.
pub const ROSTER: [(&str, f64); 5] = [
    ("coast", 0.0),
    ("low", 0.20),
    ("mid", 0.40),
    ("high", 0.60),
    ("hard", 0.85),
];

/// (4f) Minimum separation between any two branches' **realized** throttles.
///
/// Sized well above the per-step drift of the envelope's ceiling, so it fails a genuine collapse
/// rather than the ordinary decay of the admissible throttle as dynamic pressure falls.
pub const ROSTER_THROTTLE_MIN_GAP: f64 = 0.02;

/// (4d) Cap on a branch's post-fork bond growth.
///
/// The second half of the fork-economics question: a fork that is O(1) to take is only cheap overall
/// if the branch's state does not then blow past the trunk's compression. A branch that cannot report
/// its growth records `-1` and fails this gate rather than passing on a missing value.
///
/// **Measured 2026-07-20: 0** — every branch's state re-quantized at the rank the fork carried, so
/// the copy-on-write fork stays cheap through the continuation, which is the M1 green result carried
/// forward onto plume-coupled state. A ceiling rather than a floor, so its headroom is the point: it
/// fires when a branch's rank runs away, not when the measurement drifts.
pub const MAX_BOND_GROWTH: f64 = 8.0;

/// (4d) Cap on the fan-out's per-step wall-clock relative to the trunk's.
///
/// The other half of design note §10(4d), which asks for the per-branch step cost against the trunk
/// alongside the sharing and bond witnesses. Branches run concurrently, so a roster of N costs about
/// one branch's time rather than N — the ratio reads as "how much longer one branch step took than
/// one trunk step" and should land near one while the fork stays cheap. A ratio that climbs means the
/// copy-on-write share is being paid for after all, at first write or in rank.
///
/// Timed by the example rather than the solver crate: a clock belongs to whoever owns the run.
pub const MAX_STEP_COST_RATIO: f64 = 3.0;

// ── Earned bands (pinned from the first measured run, then regressed) ───────────────────────

/// (4a) Minimum relative spread of the branch flow observables, over the evolved interior. The
/// corridor's bank branches agreed to three digits — that invariance is the explicit foil this
/// clears, and 0.02 is roughly twenty times it.
///
/// **Re-earned 2026-07-20 (third time): measured 0.0202**, pinned here with margin. This is a
/// *loosening* from 0.03 and the reason is a physics correction rather than a run that came in
/// under: the reference area `PLUME_S_REF_M2` was three times too small, so every branch's thrust
/// coefficient — and therefore its drag state and its trajectory — was wrong, and the roster was
/// pinned to an envelope ceiling computed from that same wrong area. Correcting both changed what
/// the branches fly. The 0.03 measured a different vehicle.
///
/// The band's own limitation is unchanged and documented at `CORDELL_MACH_MIN`: with the plume
/// geometry outside the Cordell-Braun envelope through the burn, the marched layer carries no plume
/// imprint, so this spread is throttle -> trajectory -> post-shock density rather than a plume
/// footprint.
pub const FLOW_SPREAD_MIN: f64 = 0.015;
/// (4c) Minimum departure of the realized trajectory from the frozen-drag prediction, m·s⁻¹.
///
/// **Re-earned 2026-07-20: measured 147.3 m/s**, pinned here with margin. The quantity changed units
/// and meaning, so the superseded 0.5 does not carry over: it bounded a difference of two closed
/// forms in m·s⁻² whose thrust terms cancelled identically, which restated the drag gate. This bounds
/// the separation of two **velocity increments** accumulated step by step over the same continuation
/// — the one the branch actually shed, and the one it would have shed with the drag closure frozen at
/// the fork's fraction.
pub const FROZEN_DRAG_SEPARATION_MIN: f64 = 100.0;
/// (4b) Minimum collapse of the preserved-drag fraction across the **burning** branches' realized
/// throttles.
///
/// **Re-earned 2026-07-20: measured 0.178** (0.1310 at the 0.16 branch down to −0.0472 at 0.38),
/// pinned here with margin. The earlier 0.5 is superseded by a correction rather than relaxed, and
/// two corrections moved it:
///
/// - the closure normalized its thrust coefficient against a **construction-time** 12 kPa while the
///   safety gate used the sensed ~2.2 kPa, which kept `C_T` in the correlation's shallow range and
///   produced fractions of 1.000 → 0.217. It reads the sensed dynamic pressure now, so `C_T` reaches
///   the range where the correlation actually collapses;
/// - the coast endpoint was a hardcoded `1.0`. A coasting branch has no plume and now reports no
///   fraction at all, so the collapse is measured across the branches that actually applied a
///   decrement.
///
/// The corrected roster also reaches the correlation's **negative** branch — the wake-type forebody
/// force past `C_T ≈ 2`, which is the drag reversal the design note's §3.2 sign-flip study exists to
/// find. A peak reduction folded from zero used to rectify it to zero.
pub const DRAG_COLLAPSE_MIN: f64 = 0.10;
/// (5) Minimum separation between the informed and uninformed guidances' **flown** landing
/// decisions — the altitude each lights its stopping burn at, m.
///
/// **Re-earned 2026-07-20: measured 23.44 m** (149.35 m informed against 125.91 m uninformed),
/// pinned here with margin. The quantity changed, so the superseded 1.0 does not carry over: it
/// bounded the difference between two interpolations of one CSV, computed before any march and
/// invariant to the entire descent.
///
/// The flown separation is close to but not equal to the 20.43 m difference in demanded margin,
/// because `ignition_altitude_kernel` solves a stopping distance rather than adding an offset — the
/// extra margin also changes the mass and speed the burn starts from. That the two numbers differ is
/// the point: one is arithmetic, the other is a flight.
pub const BELIEF_SEPARATION_MIN_M: f64 = 10.0;
/// (3) Minimum regime transitions across the whole descent: the rarefaction and comms cascade of
/// Acts 0-1 plus the Mach, thrust, and touchdown transitions of Acts 2-4.
pub const MIN_REGIME_TRANSITIONS: usize = 6;
// ── Corridor inheritance (gate 1): the window the day's dispersion row predicts ───────────────

/// Tolerance on the flown blackout window against the dispersion table's prediction for the measured
/// day, s.
///
/// **Earned 2026-07-20**: the flown window came in at 10.50 s onset and 59.60 s dwell against the
/// table's interpolated 10.54 s and 59.41 s — errors of 0.04 s and 0.19 s, both inside one compressed
/// flight step (`DT_FLIGHT` = 0.1 s) of the quantity being predicted. Pinned with margin.
///
/// This replaces a comparison against the *corridor's* recorded window, which cannot hold: the
/// corridor flies the standard day and this example flies a cold one, and a colder, denser atmosphere
/// ionizes earlier and dwells longer. Demanding equality there would have asked the descent to ignore
/// the weather it exists to consume.
pub const WINDOW_PREDICTION_TOL_S: f64 = 0.5;

/// (6) Tolerance around the commanded contact speed, m/s. Two-sided: a one-sided ceiling admits an
/// undershoot and admits a hover, which are exactly the tracking failures this gate claims to catch.
pub const TOUCHDOWN_SINK_TOL: f64 = 0.5;

/// (8) Carrier-rebuild cap across all legs.
pub const MAX_REBUILDS: usize = 6;
/// (9) Wall-clock budget for the whole example, s.
///
/// The corridor's 600 s figure was sized for a descent that stops at 47 km after ~476 coupled
/// steps. This example carries the same vehicle to the ground under a burn and forks a roster
/// mid-way, which is several thousand steps more — so the budget is sized for the flight actually
/// flown rather than inherited from a shorter one.
pub const WALL_CLOCK_BUDGET_S: f64 = 1_800.0;
