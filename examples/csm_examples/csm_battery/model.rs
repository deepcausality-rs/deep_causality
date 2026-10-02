/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cell on charge, the charger the machine's actions set, and the heat balance both use.

use crate::FloatType;
use crate::constants::*;
use crate::model_types::{Reading, Summary};
use deep_causality_num::lift_usize;
use std::sync::atomic::{AtomicU8, Ordering};

/// The charger's current setting, in A. A causal action takes no arguments, so the actions set
/// the charger here and the session reads it back.
static CHARGER_A: AtomicU8 = AtomicU8::new(OFF_A);

/// Sets the charger to `amps`.
pub fn set_charger(amps: u8) {
    CHARGER_A.store(amps, Ordering::Relaxed);
}

/// The charger's current setting, in A.
pub fn charger_a() -> FloatType {
    lift_usize(usize::from(CHARGER_A.load(Ordering::Relaxed)))
}

/// How much the cell warms in one minute at `amps` from `cell_c`, shedding heat at `cooling`.
pub fn rise_c(amps: FloatType, cell_c: FloatType, cooling: FloatType) -> FloatType {
    HEATING_C_PER_A2 * amps * amps - cooling * (cell_c - AMBIENT_C)
}

/// The cell on charge. The coolant pump runs for the first minutes of the session and then stops.
pub struct Cell {
    minute: usize,
    cell_c: FloatType,
    previous_c: FloatType,
    last_amps: FloatType,
    charge_pct: FloatType,
    peak_c: FloatType,
    minutes_hot: usize,
}

impl Cell {
    /// A cell at ambient temperature, 20 % charged, with the charger off.
    pub fn plug_in() -> Self {
        set_charger(OFF_A);
        Self {
            minute: 0,
            cell_c: AMBIENT_C,
            previous_c: AMBIENT_C,
            last_amps: charger_a(),
            charge_pct: START_CHARGE_PCT,
            peak_c: AMBIENT_C,
            minutes_hot: 0,
        }
    }

    pub fn minute(&self) -> usize {
        self.minute
    }

    pub fn cell_c(&self) -> FloatType {
        self.cell_c
    }

    pub fn charge_pct(&self) -> FloatType {
        self.charge_pct
    }

    /// Whether the session is over: the cell reached the target charge or time ran out.
    pub fn done(&self) -> bool {
        self.charge_pct >= TARGET_CHARGE_PCT || self.minute >= MAX_MINUTES
    }

    /// What the sensors report now.
    pub fn reading(&self) -> Reading {
        Reading {
            cell_c: self.cell_c,
            previous_c: self.previous_c,
            amps: self.last_amps,
        }
    }

    /// One minute of charging at `amps`. The current adds charge and heats the cell; the coolant,
    /// or after the pump stops the air, carries heat away.
    pub fn charge_one_minute(&mut self, amps: FloatType) {
        let cooling = if self.minute < PUMP_FAILS_AFTER_MIN {
            COOLING_PUMPED
        } else {
            COOLING_PASSIVE
        };
        self.previous_c = self.cell_c;
        self.cell_c = self.cell_c + rise_c(amps, self.cell_c, cooling);
        let charged = self.charge_pct + amps / CELL_CAPACITY_AH * HUNDRED / MINUTES_PER_HOUR;
        self.charge_pct = if charged > HUNDRED { HUNDRED } else { charged };
        self.last_amps = amps;
        self.minute += 1;
        if self.cell_c > self.peak_c {
            self.peak_c = self.cell_c;
        }
        if self.cell_c > AGEING_ABOVE_C {
            self.minutes_hot += 1;
        }
    }

    pub fn summary(&self) -> Summary {
        Summary {
            minutes: self.minute,
            charge_pct: self.charge_pct,
            peak_c: self.peak_c,
            minutes_hot: self.minutes_hot,
        }
    }
}
