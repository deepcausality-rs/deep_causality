/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The verified safety envelope the gate enforces (\[6\]), and the cybernetic-loop witness that
//! carries it.

use crate::CfdScalar;
use deep_causality_haft::{CyberneticLoop, HKT5Unbound, NoConstraint, Satisfies};

/// The optional powered-descent axes of a [`SafetyEnvelope`] (change
/// `plasma-retropulsion-cfd-contracts`, capability `powered-descent-envelope`). Present only for a
/// burn-phase world; absent (`SafetyEnvelope::burn == None`) for the corridor, where the gate
/// behaves exactly as before. Carries the throttle floor/ceiling, the maximum thrust coefficient
/// `max_ct` (the *dynamic* throttle cap — the admissible ceiling is the static ceiling min'd with
/// the throttle at which `C_T = T/(q∞·S_ref)` reaches `max_ct`), the ignition dynamic-pressure
/// window `[q_min, q_max]` (stored for M4's ignition-corridor commit, not enforced by the gate),
/// the propellant floor, and the maximum descent rate.
#[derive(Debug, Clone, Copy)]
pub struct BurnEnvelope<R: CfdScalar> {
    /// Minimum admissible throttle command.
    pub throttle_floor: R,
    /// Maximum admissible throttle command (the static ceiling).
    pub throttle_ceiling: R,
    /// Maximum admissible thrust coefficient `C_T` (the dynamic, q-dependent throttle cap).
    pub max_ct: R,
    /// Ignition dynamic-pressure window lower bound (Pa) — stored for M4, not gated here.
    pub q_min: R,
    /// Ignition dynamic-pressure window upper bound (Pa) — stored for M4, not gated here.
    pub q_max: R,
    /// Minimum admissible propellant mass (kg); a positive throttle at or below it is a breach.
    pub propellant_floor: R,
    /// Maximum admissible descent rate (m/s).
    pub max_descent_rate: R,
}

impl<R: CfdScalar> BurnEnvelope<R> {
    /// A burn envelope with all six powered-descent axes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        throttle_floor: R,
        throttle_ceiling: R,
        max_ct: R,
        q_min: R,
        q_max: R,
        propellant_floor: R,
        max_descent_rate: R,
    ) -> Self {
        Self {
            throttle_floor,
            throttle_ceiling,
            max_ct,
            q_min,
            q_max,
            propellant_floor,
            max_descent_rate,
        }
    }
}

/// The verified safety envelope — the cybernetic loop's Context `C`. A bank-angle correction is
/// admissible only inside it; the gate clamps into `[−max_bank, max_bank]` and refuses (yields
/// [`BankCorrection::NoSafeAction`]) once the sensed heat flux or g-load exceeds its ceiling.
///
/// The optional [`burn`](Self::burn) axes carry the powered-descent limits; they are `None` for the
/// corridor, so the gate stays bit-identical until a burn-phase world attaches them.
#[derive(Debug, Clone, Copy)]
pub struct SafetyEnvelope<R: CfdScalar> {
    /// Maximum admissible wall heat flux (W·m⁻²).
    pub max_heat_flux: R,
    /// Maximum admissible g-load (multiples of g).
    pub max_g_load: R,
    /// Maximum admissible bank-angle magnitude (rad).
    pub max_bank_rad: R,
    /// The optional powered-descent axes; `None` for the corridor (gate unchanged).
    pub burn: Option<BurnEnvelope<R>>,
}

impl<R: CfdScalar> SafetyEnvelope<R> {
    /// An envelope bounded by a heat-flux ceiling, a g-load ceiling, and a bank-angle magnitude.
    /// The powered-descent axes are absent (`burn: None`); attach them with [`with_burn`](Self::with_burn).
    pub fn new(max_heat_flux: R, max_g_load: R, max_bank_rad: R) -> Self {
        Self {
            max_heat_flux,
            max_g_load,
            max_bank_rad,
            burn: None,
        }
    }

    /// Attach the powered-descent axes, returning the extended envelope (the corridor never calls
    /// this, so its gate is untouched).
    pub fn with_burn(mut self, burn: BurnEnvelope<R>) -> Self {
        self.burn = Some(burn);
        self
    }
}

/// The sensed coupled state the gate observes — the loop's Sensor `S`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SensedState<R: CfdScalar> {
    pub(crate) peak_heat_flux: R,
    pub(crate) g_load: R,
}

/// The estimated margins — the loop's Belief `B` (headroom to each envelope ceiling).
#[derive(Debug, Clone, Copy)]
pub(crate) struct MarginBelief<R: CfdScalar> {
    pub(crate) thermal_margin: R,
    pub(crate) g_margin: R,
}

/// The bounded bank-angle correction — the loop's Action `A`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BankCorrection<R: CfdScalar> {
    /// A bank-angle command clamped into the envelope (rad).
    Clamped(R),
    /// No admissible correction exists — the envelope is already breached (the loop's Entropy `E`).
    NoSafeAction,
}

/// The five-slot agent carrying `(S, B, C, A, E)` for the [`CyberneticLoop`] witness.
pub(crate) struct GuidanceAgent<S, B, C, A, E>(pub S, pub B, pub C, pub A, pub E);

/// The HKT-5 witness realizing the corridor's cybernetic guidance loop.
pub(crate) struct GuidanceWitness;

impl HKT5Unbound for GuidanceWitness {
    type Constraint = NoConstraint;
    type Type<S, B, C, A, E> = GuidanceAgent<S, B, C, A, E>;
}

impl CyberneticLoop<GuidanceWitness> for GuidanceWitness {
    fn control_step<S, B, C, A, E, FObserve, FDecide>(
        agent: GuidanceAgent<S, B, C, A, E>,
        sensor_input: S,
        observe_fn: FObserve,
        decide_fn: FDecide,
    ) -> Result<A, E>
    where
        S: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        E: Satisfies<NoConstraint>,
        FObserve: Fn(S, &C) -> B,
        FDecide: Fn(B, &C) -> A,
    {
        // The agent carries the single fixed Context (its 3rd slot); observe and decide both borrow
        // it. The Action encodes the refusal (NoSafeAction), so this witness is infallible in `E`.
        let GuidanceAgent(_s, _b, context, _a, _e) = agent;
        let belief = observe_fn(sensor_input, &context);
        Ok(decide_fn(belief, &context))
    }
}
