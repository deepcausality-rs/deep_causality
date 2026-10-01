/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The controller: one rule and one action per state, the state machine that pairs them, and
//! the two causaloids that decide the cooling regime.

use crate::FloatType;
use crate::constants::*;
use crate::model::{
    BUDGET_LIMIT, CELL_LIMIT, FAN, FULL, HALF, HEATER, RADIATOR, Reading, STOP, air_density,
    fan_heat_w, heat_in_w, passive_w_k, radiator_heat_w,
};
use deep_causality::{
    ActionError, CSM, CausalAction, CausalState, Causaloid, MonadicCausable, PropagatingEffect,
};
use std::error::Error;
use std::sync::atomic::Ordering;

pub type Rule = Causaloid<Reading, bool, (), ()>;
pub type Controller = CSM<Reading, bool, ()>;

/// The mechanism that cools the pack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Regime {
    Fan,
    Radiator,
}

// State identifiers, in the order the controller evaluates them each minute.
pub const COOL_ON: usize = 1;
pub const COOL_OFF: usize = 2;
pub const HEAT_ON: usize = 3;
pub const HEAT_OFF: usize = 4;
pub const CELL_FULL: usize = 5;
pub const CELL_HALF: usize = 6;
pub const CELL_STOP: usize = 7;
pub const BUDGET_FULL: usize = 8;
pub const BUDGET_HALF: usize = 9;
pub const BUDGET_STOP: usize = 10;
pub const STATES: [usize; 10] = [
    COOL_ON,
    COOL_OFF,
    HEAT_ON,
    HEAT_OFF,
    CELL_FULL,
    CELL_HALF,
    CELL_STOP,
    BUDGET_FULL,
    BUDGET_HALF,
    BUDGET_STOP,
];

/// The controller at launch: fan cooling, heater, and both charge limits.
pub fn controller() -> Controller {
    let state = |id: usize, rule: fn(Reading) -> PropagatingEffect<bool>| {
        CausalState::new(
            id,
            1,
            PropagatingEffect::pure(Reading::default()),
            rule_of(id, rule),
            None,
        )
    };
    let action =
        |f: fn() -> Result<(), ActionError>, descr: &'static str| CausalAction::new(f, descr, 1);
    CSM::new(&[
        (
            &state(COOL_ON, cell_needs_cooling),
            &action(fan_on, "fan on"),
        ),
        (
            &state(COOL_OFF, cell_cool_enough),
            &action(fan_off, "fan off"),
        ),
        (
            &state(HEAT_ON, cell_needs_heat),
            &action(heater_on, "heater on"),
        ),
        (
            &state(HEAT_OFF, cell_warm_enough),
            &action(heater_off, "heater off"),
        ),
        (
            &state(CELL_FULL, cell_in_core_window),
            &action(cell_full, "cell: full charge"),
        ),
        (
            &state(CELL_HALF, cell_in_derate_band),
            &action(cell_half, "cell: half charge"),
        ),
        (
            &state(CELL_STOP, cell_outside_window),
            &action(cell_stop, "cell: no charge"),
        ),
        (
            &state(BUDGET_FULL, budget_has_margin),
            &action(budget_full, "budget: full charge"),
        ),
        (
            &state(BUDGET_HALF, budget_is_tight),
            &action(budget_half, "budget: half charge"),
        ),
        (
            &state(BUDGET_STOP, budget_is_short),
            &action(budget_stop, "budget: no charge"),
        ),
    ])
}

/// Swaps the cooling states to `regime`'s mechanism while the controller runs. The rules stay
/// the same, because the envelope does; the actions change. The outgoing mechanism is switched
/// off first.
pub fn install_cooling(
    controller: &Controller,
    regime: Regime,
    version: usize,
) -> Result<(), Box<dyn Error>> {
    let (on, off, on_descr, off_descr, retire): (Act, Act, &'static str, &'static str, Act) =
        match regime {
            Regime::Fan => (fan_on, fan_off, "fan on", "fan off", radiator_close),
            Regime::Radiator => (
                radiator_open,
                radiator_close,
                "radiator open",
                "radiator closed",
                fan_off,
            ),
        };
    retire()?;
    let state = |id: usize, rule: fn(Reading) -> PropagatingEffect<bool>| {
        CausalState::new(
            id,
            version,
            PropagatingEffect::pure(Reading::default()),
            rule_of(id, rule),
            None,
        )
    };
    controller.update_single_state((
        state(COOL_ON, cell_needs_cooling),
        CausalAction::new(on, on_descr, version),
    ))?;
    controller.update_single_state((
        state(COOL_OFF, cell_cool_enough),
        CausalAction::new(off, off_descr, version),
    ))?;
    Ok(())
}

/// The two regime causaloids: the radiator overtakes the fan, and the fan overtakes the radiator.
pub fn regime_detectors() -> (Rule, Rule) {
    (
        Causaloid::new(
            11,
            radiator_beats_fan,
            "radiator removes more heat than the fan",
        ),
        Causaloid::new(
            12,
            fan_beats_radiator,
            "fan removes more heat than the radiator",
        ),
    )
}

/// The regime for this reading: the other mechanism once its detector holds, otherwise unchanged.
pub fn next_regime(
    regime: Regime,
    reading: &Reading,
    detectors: &(Rule, Rule),
) -> Result<Regime, deep_causality::CausalityError> {
    let (to_radiator, to_fan) = detectors;
    let detector = match regime {
        Regime::Fan => to_radiator,
        Regime::Radiator => to_fan,
    };
    let effect = detector.evaluate(&PropagatingEffect::pure(*reading));
    if let Some(err) = effect.error() {
        return Err(err.clone());
    }
    Ok(match (regime, effect.value()) {
        (Regime::Fan, Some(true)) => Regime::Radiator,
        (Regime::Radiator, Some(true)) => Regime::Fan,
        _ => regime,
    })
}

type Act = fn() -> Result<(), ActionError>;

fn rule_of(id: usize, rule: fn(Reading) -> PropagatingEffect<bool>) -> Rule {
    Causaloid::new(id as u64, rule, "balloon controller rule")
}

// =============================================================================
// Rules: each reads one sensor frame and says whether its state is active
// =============================================================================

fn cell_needs_cooling(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c >= COOL_ON_C)
}

fn cell_cool_enough(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c <= COOL_OFF_C)
}

fn cell_needs_heat(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c <= HEAT_ON_C)
}

fn cell_warm_enough(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c >= HEAT_OFF_C)
}

/// The charge window, measured at the cell now.
fn cell_in_core_window(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c >= COLD_DERATE_C && r.cell_c <= HOT_DERATE_C)
}

fn cell_in_derate_band(r: Reading) -> PropagatingEffect<bool> {
    let cold = r.cell_c >= CHARGE_MIN_C && r.cell_c < COLD_DERATE_C;
    let hot = r.cell_c > HOT_DERATE_C && r.cell_c <= CHARGE_MAX_C;
    PropagatingEffect::pure(cold || hot)
}

fn cell_outside_window(r: Reading) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(r.cell_c < CHARGE_MIN_C || r.cell_c > CHARGE_MAX_C)
}

/// The heat budget: heat coming in at full charge, against what the better of the two cooling
/// mechanisms removes with the cell at the top of the charge window, in the air measured now.
fn heat_budget(r: Reading) -> (FloatType, FloatType) {
    let density = air_density(r.pressure_kpa, r.air_c);
    let fan = fan_heat_w(density, CHARGE_MAX_C, r.air_c);
    let radiator = radiator_heat_w(CHARGE_MAX_C, r.air_c);
    let best = if fan > radiator { fan } else { radiator };
    let capacity = passive_w_k(density) * (CHARGE_MAX_C - r.air_c) + best;
    (heat_in_w(r.sun_w_m2, FULL_CHARGE_A), capacity)
}

fn budget_has_margin(r: Reading) -> PropagatingEffect<bool> {
    let (heat_in, capacity) = heat_budget(r);
    PropagatingEffect::pure(heat_in <= BUDGET_MARGIN * capacity)
}

fn budget_is_tight(r: Reading) -> PropagatingEffect<bool> {
    let (heat_in, capacity) = heat_budget(r);
    PropagatingEffect::pure(heat_in > BUDGET_MARGIN * capacity && heat_in <= capacity)
}

fn budget_is_short(r: Reading) -> PropagatingEffect<bool> {
    let (heat_in, capacity) = heat_budget(r);
    PropagatingEffect::pure(heat_in > capacity)
}

/// Each regime detector compares the two mechanisms at the cooling set point, in the air
/// measured now.
fn radiator_beats_fan(r: Reading) -> PropagatingEffect<bool> {
    let fan = fan_heat_w(air_density(r.pressure_kpa, r.air_c), COOL_ON_C, r.air_c);
    PropagatingEffect::pure(radiator_heat_w(COOL_ON_C, r.air_c) > REGIME_HYSTERESIS * fan)
}

fn fan_beats_radiator(r: Reading) -> PropagatingEffect<bool> {
    let fan = fan_heat_w(air_density(r.pressure_kpa, r.air_c), COOL_ON_C, r.air_c);
    PropagatingEffect::pure(fan > REGIME_HYSTERESIS * radiator_heat_w(COOL_ON_C, r.air_c))
}

// =============================================================================
// Actions: an action takes no arguments, so each one writes an actuator register
// =============================================================================

fn fan_on() -> Result<(), ActionError> {
    FAN.store(true, Ordering::Relaxed);
    Ok(())
}

fn fan_off() -> Result<(), ActionError> {
    FAN.store(false, Ordering::Relaxed);
    Ok(())
}

fn radiator_open() -> Result<(), ActionError> {
    RADIATOR.store(true, Ordering::Relaxed);
    Ok(())
}

fn radiator_close() -> Result<(), ActionError> {
    RADIATOR.store(false, Ordering::Relaxed);
    Ok(())
}

fn heater_on() -> Result<(), ActionError> {
    HEATER.store(true, Ordering::Relaxed);
    Ok(())
}

fn heater_off() -> Result<(), ActionError> {
    HEATER.store(false, Ordering::Relaxed);
    Ok(())
}

fn cell_full() -> Result<(), ActionError> {
    CELL_LIMIT.store(FULL, Ordering::Relaxed);
    Ok(())
}

fn cell_half() -> Result<(), ActionError> {
    CELL_LIMIT.store(HALF, Ordering::Relaxed);
    Ok(())
}

fn cell_stop() -> Result<(), ActionError> {
    CELL_LIMIT.store(STOP, Ordering::Relaxed);
    Ok(())
}

fn budget_full() -> Result<(), ActionError> {
    BUDGET_LIMIT.store(FULL, Ordering::Relaxed);
    Ok(())
}

fn budget_half() -> Result<(), ActionError> {
    BUDGET_LIMIT.store(HALF, Ordering::Relaxed);
    Ok(())
}

fn budget_stop() -> Result<(), ActionError> {
    BUDGET_LIMIT.store(STOP, Ordering::Relaxed);
    Ok(())
}
