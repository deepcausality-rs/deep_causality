/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FloatType;
use crate::constants::*;
use crate::model::{Cell, rise_c};
use crate::model_types::{Reading, Summary};
use deep_causality_num::lower;

pub fn print_intro() {
    println!(
        "A lithium-ion cell charges from {:.0} % to {:.0} %. Its coolant pump stops after {PUMP_FAILS_AFTER_MIN} minutes.",
        lower(START_CHARGE_PCT),
        lower(TARGET_CHARGE_PCT),
    );
}

pub fn print_session(adaptive: bool) {
    println!();
    if adaptive {
        println!("The controller replaces its charge states when it detects the loss of cooling:");
    } else {
        println!("The controller keeps the charge law for a cooled cell:");
    }
    println!(" minute  cell °C  charge  current");
}

/// One line each time the charger changes its current.
pub fn print_minute(cell: &Cell, amps: FloatType) {
    println!(
        "{:7}  {:7.1}  {:4.1} %  {:4.0} A",
        cell.minute() + 1,
        lower(cell.cell_c()),
        lower(cell.charge_pct()),
        lower(amps),
    );
}

/// The moment the detector fires and the controller swaps its charge states.
pub fn print_swap(cell: &Cell, reading: &Reading, version: usize) {
    println!(
        "{:7}  >> the cell rose {:.2} K in the last minute; the pumped coolant allows {:.2} K.",
        cell.minute() + 1,
        lower(reading.cell_c - reading.previous_c),
        lower(rise_c(reading.amps, reading.previous_c, COOLING_PUMPED)),
    );
    println!(
        "         >> charge states replaced by the derated law, version {version}: 20 A below 40 °C, 10 A to 45 °C."
    );
}

pub fn print_summary(summary: &Summary) {
    println!(
        "Reached {:.1} % after {} minutes. The cell peaked at {:.1} °C and spent {} minutes above {:.0} °C.",
        lower(summary.charge_pct),
        summary.minutes,
        lower(summary.peak_c),
        summary.minutes_hot,
        lower(AGEING_ABOVE_C),
    );
}

pub fn print_comparison(fixed: &Summary, adaptive: &Summary) {
    println!();
    println!(
        "Replacing the charge states cost {} minutes and kept the cell {:.1} K cooler at its peak.",
        adaptive.minutes as i64 - fixed.minutes as i64,
        lower(fixed.peak_c - adaptive.peak_c),
    );
}
