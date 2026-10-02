/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The controller: three charge states, the two charge laws that fill them, and the causaloid
//! that detects a loss of cooling.

use crate::constants::*;
use crate::model::{rise_c, set_charger};
use crate::model_types::Reading;
use deep_causality::{
    ActionError, CSM, CausalAction, CausalState, CausalityError, Causaloid, MonadicCausable,
    PropagatingEffect, StateAction,
};
use deep_causality_context::UpdateError;

pub type Rule = Causaloid<Reading, bool, (), ()>;
pub type Controller = CSM<Reading, bool, ()>;

// State identifiers, one per charge setting, in the order the controller evaluates them.
pub const FULL: usize = 1;
pub const HALF: usize = 2;
pub const STOP: usize = 3;
pub const STATES: [usize; 3] = [FULL, HALF, STOP];

type Law = fn(Reading) -> PropagatingEffect<bool>;
type Act = fn() -> Result<(), ActionError>;

/// The controller at plug-in, running the charge law for a cooled cell.
pub fn controller() -> Controller {
    let law = cooled_law(1);
    let pairs: Vec<_> = law.iter().map(|(state, action)| (state, action)).collect();
    CSM::new(&pairs)
}

/// Replaces the three charge states with the derated law while the controller runs. The state
/// identifiers stay; the rules and actions behind them change.
pub fn install_derated_law(controller: &Controller, version: usize) -> Result<(), UpdateError> {
    derated_law(version)
        .into_iter()
        .try_for_each(|state_action| controller.update_single_state(state_action))
}

/// The causaloid that detects a loss of cooling.
pub fn cooling_loss_detector() -> Rule {
    Causaloid::new(
        10,
        cooling_lost,
        "the cell warms faster than the pumped coolant allows",
    )
}

/// Whether `detector` finds the cooling lost in `reading`.
pub fn cooling_lost_in(detector: &Rule, reading: &Reading) -> Result<bool, CausalityError> {
    let effect = detector.evaluate(&PropagatingEffect::pure(*reading));
    if let Some(err) = effect.error() {
        return Err(err.clone());
    }
    Ok(effect.value() == Some(&true))
}

/// The charge law for a cooled cell: 50 A below 45 °C, 25 A up to 55 °C, none above.
fn cooled_law(version: usize) -> [StateAction<Reading, bool, ()>; 3] {
    [
        state_action(FULL, version, below_45_c, full_current, "50 A below 45 °C"),
        state_action(
            HALF,
            version,
            from_45_below_55_c,
            half_current,
            "25 A from 45 to 55 °C",
        ),
        state_action(
            STOP,
            version,
            from_55_c,
            no_current,
            "no current from 55 °C",
        ),
    ]
}

/// The charge law for a cell without its pump: 20 A below 40 °C, 10 A up to 45 °C, none above.
fn derated_law(version: usize) -> [StateAction<Reading, bool, ()>; 3] {
    [
        state_action(
            FULL,
            version,
            below_40_c,
            derated_current,
            "20 A below 40 °C",
        ),
        state_action(
            HALF,
            version,
            from_40_below_45_c,
            trickle_current,
            "10 A from 40 to 45 °C",
        ),
        state_action(
            STOP,
            version,
            from_45_c,
            no_current,
            "no current from 45 °C",
        ),
    ]
}

fn state_action(
    id: usize,
    version: usize,
    law: Law,
    act: Act,
    description: &'static str,
) -> StateAction<Reading, bool, ()> {
    let rule = Causaloid::new(id as u64, law, description);
    (
        CausalState::new(
            id,
            version,
            PropagatingEffect::pure(Reading::default()),
            rule,
            None,
        ),
        CausalAction::new(act, description, version),
    )
}

// =============================================================================
// Rules: each reads one minute of sensor data and says whether its state is active
// =============================================================================

fn below_45_c(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c < COOLED_FULL_BELOW_C)
}

fn from_45_below_55_c(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c >= COOLED_FULL_BELOW_C && r.cell_c < COOLED_HALF_BELOW_C)
}

fn from_55_c(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c >= COOLED_HALF_BELOW_C)
}

fn below_40_c(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c < DERATED_FULL_BELOW_C)
}

fn from_40_below_45_c(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c >= DERATED_FULL_BELOW_C && r.cell_c < DERATED_HALF_BELOW_C)
}

fn from_45_c(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c >= DERATED_HALF_BELOW_C)
}

/// The cooling-loss rule: over the last minute the cell rose more than the pumped coolant allows
/// for the current that flowed.
fn cooling_lost(r: Reading) -> PropagatingEffect<bool> {
    let allowed = rise_c(r.amps, r.previous_c, COOLING_PUMPED);
    PropagatingEffect::pure(r.cell_c - r.previous_c > allowed + COOLING_LOSS_MARGIN_C)
}

// =============================================================================
// Actions: each sets the charger
// =============================================================================

fn full_current() -> Result<(), ActionError> {
    set_charger(FULL_A);
    Ok(())
}

fn half_current() -> Result<(), ActionError> {
    set_charger(HALF_A);
    Ok(())
}

fn derated_current() -> Result<(), ActionError> {
    set_charger(DERATED_A);
    Ok(())
}

fn trickle_current() -> Result<(), ActionError> {
    set_charger(TRICKLE_A);
    Ok(())
}

fn no_current() -> Result<(), ActionError> {
    set_charger(OFF_A);
    Ok(())
}
