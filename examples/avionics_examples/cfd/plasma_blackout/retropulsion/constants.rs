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

/// The mid-burn throttle roster: a coast branch, two candidates straddling the drag sign-flip band,
/// a nominal branch, and an engine-degraded contingency. On-axis magnitudes only — no angle of
/// attack, the design note's §6 discipline pin.
pub const ROSTER: [(&str, f64); 5] = [
    ("coast", 0.0),
    ("sign_flip_low", 0.25),
    ("sign_flip_high", 0.55),
    ("nominal", 0.80),
    ("engine_degraded", 0.45),
];

/// The axial aerodynamic drag the branch scoring normalizes against, N. The forebody drag the A0
/// preserved-drag fraction scales.
pub const BASE_AXIAL_DRAG_N: f64 = -18_000.0;

// ── Earned bands (pinned from the first measured run, then regressed) ───────────────────────

/// (4a) Minimum relative spread of the branch flow observables, over the evolved interior. The
/// corridor's bank branches agreed to three digits — that invariance is the explicit foil this
/// clears. **Re-earned 2026-07-20: measured 0.018** once `RetroThrust` was aimed along the true
/// flight velocity. The earlier 0.105 was measured against a thrust vector pinned to a fixed axis,
/// so it is superseded rather than relaxed — the physics changed, and a band earned under the old
/// physics does not carry over.
pub const FLOW_SPREAD_MIN: f64 = 0.01;
/// (4c) Minimum departure of the realized deceleration from the frozen-drag prediction, m·s⁻².
/// **Earned 2026-07-20: measured 1.610**, pinned here with margin.
pub const FROZEN_DRAG_SEPARATION_MIN: f64 = 0.5;
/// (4b) Minimum collapse of the preserved-drag fraction from coast to hardest throttle.
/// **Earned 2026-07-20** from the first measured roster.
pub const DRAG_COLLAPSE_MIN: f64 = 0.5;
/// (5) Minimum separation in demanded ignition margin between the informed and uninformed
/// guidance, m. **Earned.**
pub const BELIEF_SEPARATION_MIN_M: f64 = 1.0;
/// (3) Minimum regime transitions across the whole descent: the rarefaction and comms cascade of
/// Acts 0-1 plus the Mach, thrust, and touchdown transitions of Acts 2-4.
pub const MIN_REGIME_TRANSITIONS: usize = 6;
/// (6) Touchdown descent-rate limit, m·s⁻¹, sensed at the `TOUCHDOWN_ALTITUDE_M` floor. Distinct
/// from the envelope's whole-leg descent-rate axis, which is sized for the supersonic entry to the
/// powered leg.
///
/// **Earned 2026-07-20: measured 2.0 m/s** against a commanded `CONTACT_SPEED_MS` of 2.0, pinned
/// here with margin. What this gate now measures is *tracking*: whether the stopping burn arrives at
/// its commanded contact condition from 157 m and 49 m/s, not whether a setpoint exists. A guidance
/// that overshoots, undershoots, or saturates fails it. Three earlier figures are superseded rather
/// than relaxed, each by a correction rather than by loosening:
///
/// - **160 m/s**, when the terminal leg still flew under the supersonic envelope. That envelope's
///   `C_T ≤ 3` axis is the Jarvinen–Adams **bow-shock-stability** bound, and applying it below the
///   dataset's own Mach floor imposed a constraint with no physics behind it: there is no bow shock
///   for the plume to destabilize at Mach 0.1. The terminal leg now flies its own subsonic envelope.
/// - **7.4 m/s at 10.5 km**, when the terminal guidance burned continuously from the subsonic
///   handover. `a_cmd = v²/2h + g` degenerates to `a_cmd ≈ g` for large `h`, so the vehicle nulled
///   its descent rate at altitude and hovered until the propellant ran out. The guidance now flies
///   a stopping burn: it coasts to the ignition altitude, then burns once.
/// - **13.4 m/s**, when the stopping law still targeted zero velocity at *zero* altitude while this
///   gate sampled at the 15 m floor — the vehicle was reported one floor-height of braking short of
///   its target. The law now aims at the touchdown plane, and at a commanded contact speed rather
///   than at rest, which is how a lander is actually flown.
pub const TOUCHDOWN_SINK_MAX: f64 = 5.0;
/// (8) Carrier-rebuild cap across all legs.
pub const MAX_REBUILDS: usize = 6;
/// (9) Wall-clock budget for the whole example, s.
///
/// The corridor's 600 s figure was sized for a descent that stops at 47 km after ~476 coupled
/// steps. This example carries the same vehicle to the ground under a burn and forks a roster
/// mid-way, which is several thousand steps more — so the budget is sized for the flight actually
/// flown rather than inherited from a shorter one.
pub const WALL_CLOCK_BUDGET_S: f64 = 1_800.0;
