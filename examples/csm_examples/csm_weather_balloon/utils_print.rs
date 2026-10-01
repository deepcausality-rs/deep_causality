/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::MIN_CHARGING_SUN_W_M2;
use crate::model::{Actuators, FULL, Flight, HALF, Reading};
use crate::model_config::Regime;
use deep_causality_num::lower;

pub fn print_header() {
    println!("Weather balloon: one battery envelope from the launch pad to 20 km and back");
    println!(
        "Charge window 0-45 °C (half rate below 10 °C and above 40 °C); cooling at 35 °C; heater at 5 °C"
    );
    println!();
    println!(
        "{:<12} {:<9} {:>7} {:>8} {:>6} {:>7} {:>5}  {:<9} {:<6} charge (cell / budget)",
        "time", "phase", "alt km", "air °C", "sun", "cell °C", "pack", "cooling", "heater"
    );
}

pub fn print_event(flight: &Flight, r: &Reading, act: &Actuators) {
    let cooling = match (act.fan, act.radiator) {
        (true, _) => "fan",
        (_, true) => "radiator",
        _ => "off",
    };
    let charge = if r.sun_w_m2 < MIN_CHARGING_SUN_W_M2 {
        "none".to_string()
    } else {
        format!(
            "{} ({} / {})",
            limit(act.charge_limit()),
            limit(act.cell_limit),
            limit(act.budget_limit)
        )
    };
    println!(
        "{:<12} {:<9} {:>7.1} {:>8.1} {:>6.0} {:>7.1} {:>4.0}%  {:<9} {:<6} {charge}",
        clock(flight),
        phase(flight),
        lower(flight.altitude_m()) / 1000.0,
        lower(r.air_c),
        lower(r.sun_w_m2),
        lower(r.cell_c),
        lower(flight.charge_fraction()) * 100.0,
        cooling,
        if act.heater { "on" } else { "off" },
    );
}

pub fn print_regime_change(flight: &Flight, r: &Reading, regime: Regime, version: usize) {
    let mechanism = match regime {
        Regime::Fan => "fan",
        Regime::Radiator => "radiator",
    };
    println!(
        "{:<12} {:<9} {:>7.1} {:>8.1}  >> cooling states now use the {mechanism} (version {version})",
        clock(flight),
        phase(flight),
        lower(flight.altitude_m()) / 1000.0,
        lower(r.air_c),
    );
}

pub fn print_summary(flight: &Flight) {
    let (coldest, warmest) = flight.cell_range_c();
    println!();
    println!(
        "Landed. Cell stayed between {:.1} °C and {:.1} °C; pack charge never fell below {:.0} % and ended at {:.0} %.",
        lower(coldest),
        lower(warmest),
        lower(flight.lowest_charge_fraction()) * 100.0,
        lower(flight.charge_fraction()) * 100.0,
    );
}

/// Local solar time as `day hh:mm`.
fn clock(flight: &Flight) -> String {
    let minutes = lower(flight.local_hour()) * 60.0;
    let total = minutes.round() as u64;
    let day = if total >= 24 * 60 { "day 2" } else { "day 1" };
    format!("{day} {:02}:{:02}", (total / 60) % 24, total % 60)
}

/// Phase of the day at the balloon's local solar time. The sun rises at 06:00 and sets at 18:00.
fn phase(flight: &Flight) -> &'static str {
    let hour = lower(flight.local_hour()) % 24.0;
    match hour {
        h if (6.0..11.0).contains(&h) => "morning",
        h if (11.0..13.0).contains(&h) => "noon",
        h if (13.0..17.0).contains(&h) => "afternoon",
        h if (17.0..19.0).contains(&h) => "evening",
        _ => "night",
    }
}

fn limit(level: u8) -> &'static str {
    match level {
        FULL => "full",
        HALF => "half",
        _ => "stop",
    }
}
