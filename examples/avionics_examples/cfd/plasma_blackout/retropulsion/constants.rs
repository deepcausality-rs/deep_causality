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
/// The measured day's density scale, matching the departure.
pub const MEASURED_RHO_SCALE: f64 = 1.15;

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
/// **Pinned inside the band the envelope admits.** The safety gate caps the throttle at the value
/// where `C_T` reaches its stability limit against the *sensed* dynamic pressure, which at the fork
/// is about 0.44 and falls with `q∞` through the continuation. A roster reaching above that ceiling
/// does not fly the values it names: every entry above it clamps to the same admissible throttle and
/// produces one identical trajectory, reported several times. Gate (4f) now fails a roster that does
/// this, so the values here are chosen to stay under the ceiling with margin.
///
/// The upper values are also kept where the A0 correlation still *resolves*: it flattens above a
/// thrust coefficient near 0.46, so branches spaced across the saturated plateau would differ in
/// command and agree in every measured quantity.
pub const ROSTER: [(&str, f64); 5] = [
    ("coast", 0.0),
    ("floor", 0.16),
    ("low", 0.24),
    ("mid", 0.31),
    ("high", 0.38),
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

// ── Earned bands (pinned from the first measured run, then regressed) ───────────────────────

/// (4a) Minimum relative spread of the branch flow observables, over the evolved interior. The
/// corridor's bank branches agreed to three digits — that invariance is the explicit foil this
/// clears.
///
/// **Re-earned 2026-07-20: measured 0.0474**, pinned here with margin. Two corrections moved it, and
/// both raised it: the roster now flies five distinct throttles rather than collapsing three of them
/// onto one clamped value, and the closure normalizes on the sensed dynamic pressure so each branch
/// reaches a genuinely different drag state. The superseded 0.01 was earned when three of five
/// branches produced a bit-identical density and the whole measured spread was the coast branch.
///
/// What this measures is honest about its own mechanism: with the plume geometry outside the
/// Cordell-Braun envelope for the burn (see `CORDELL_MACH_MIN`), the marched layer carries no plume
/// imprint, so the spread is throttle -> trajectory -> post-shock density rather than a plume
/// footprint. It is still a flow observable that branch interventions move, and it is still the
/// corridor's branch-invariance that it clears.
pub const FLOW_SPREAD_MIN: f64 = 0.03;
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
/// (5) Minimum separation in demanded ignition margin between the informed and uninformed
/// guidance, m. **Earned.**
pub const BELIEF_SEPARATION_MIN_M: f64 = 1.0;
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
