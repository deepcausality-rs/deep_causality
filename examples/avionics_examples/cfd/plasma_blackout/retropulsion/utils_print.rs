/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! All printing lives here; `main` stays the descent program. The final verdict renders itself
//! (`Display`); `main` prints it.

use crate::FloatType;

use crate::model::{BranchRow, DayBelief};
use avionics_examples::shared::constants::*;
use deep_causality_cfd::CompressiblePause;
use deep_causality_core::EffectLog;
use std::any::type_name;

pub fn print_intro() {
    println!("=== Plasma-retropulsion descent: blackout exit through ignition to touchdown ===");
    println!(
        "  Vehicle: {:.0} kg wet, {:.0} kg propellant, {:.0} kN central nozzle at Isp {:.0} s",
        VEHICLE_MASS_KG,
        PROPELLANT_KG,
        RETRO_THRUST_N / 1000.0,
        RETRO_ISP_S
    );
    println!(
        "  Drag authority: the cited Jarvinen-Adams A0 correlation (the M1 de-risk verdict is AMBER \
         on imprint fidelity); the marched fork carries flow realism and fork economics."
    );
    println!("  Precision: {}", type_name::<FloatType>());
}

pub fn print_plan(informed: &DayBelief, uninformed: &DayBelief) {
    println!("\n--- Act 0: PLAN — the measured day reads the dispersion table ---");
    println!(
        "  measured dT = {:+.1} K -> drift {:.2} +- {:.2} m, ignition margin {:.2} m (k = {:.0}){}",
        informed.d_temp,
        informed.drift_mean_m,
        informed.drift_sd_m,
        informed.margin_m,
        IGNITION_MARGIN_K,
        if informed.clamped {
            "  [clamped to the nearest tabulated row]"
        } else {
            ""
        }
    );
    println!(
        "  the day flown: density scale {:.2}, IMU bias departure {:.2}",
        informed.rho_scale, informed.bias_departure
    );
    println!(
        "  standard-day belief (the uninformed world) -> margin {:.2} m; separation {:.2} m",
        uninformed.margin_m,
        (informed.margin_m - uninformed.margin_m).abs()
    );
}

pub fn print_act<S>(title: &str, pause: &CompressiblePause<'_, FloatType, S>) {
    println!("\n--- Act: {title} ---");
    let f = pause.field();
    let g = |name: &str| {
        f.scalar(name)
            .and_then(|s| s.first().copied())
            .unwrap_or(0.0)
    };
    println!(
        "  steps {:>4} | altitude {:>8.2} km | Mach {:>5.2} | rebuilds {}",
        pause.step(),
        g("flight_altitude") / 1000.0,
        g("flight_mach"),
        pause.rebuilds()
    );
    println!(
        "  mass {:>7.1} kg | propellant {:>6.1} kg | throttle {:>4.2} | q {:>9.1} Pa | sink {:>7.1} m/s",
        g("mass"),
        g("propellant"),
        f.throttle_action().unwrap_or(0.0),
        g("q_inf"),
        g("descent_rate")
    );
    if let Some(r) = f.regime() {
        println!(
            "  regime: {} / {} / {}{}",
            r.model.name(),
            r.mach_regime.name(),
            r.thrust_state.name(),
            if r.touchdown { " / touchdown" } else { "" }
        );
    }
}

pub fn print_branches(rows: &[BranchRow]) {
    println!("\n--- Mid-burn throttle roster (forked from the marched, plume-coupled state) ---");
    println!(
        "  {:<10} {:>7} {:>8} {:>10} {:>11} {:>10} {:>10} {:>10}",
        "branch", "cmd", "flown", "preserved", "axial m/s2", "prop kg", "dv m/s", "dv frozen"
    );
    for r in rows {
        println!(
            "  {:<10} {:>7.2} {:>8.4} {:>10} {:>11.4} {:>10.2} {:>10.3} {:>10.3}",
            r.name,
            r.commanded_throttle,
            r.realized_throttle,
            r.preserved_fraction
                .map(|f| format!("{f:.4}"))
                .unwrap_or_else(|| "     —".into()),
            r.net_deceleration,
            r.propellant_used,
            r.dv_actual,
            r.dv_frozen
        );
    }
    println!(
        "  cmd is what each world published; flown is what the envelope admitted. Every other column \
         is read off the branch's own report — the axial deceleration from the summed force channel, \
         the two velocity increments accumulated step by step."
    );
}

pub fn print_provenance(log: &EffectLog) {
    let rendered = format!("{log}");
    let lines: Vec<&str> = rendered
        .lines()
        .skip(1)
        .filter(|l| !l.contains("bank correction bounded"))
        .collect();
    println!("\n--- Provenance ({} entries) ---", lines.len());
    for line in lines {
        let msg = line.split_once("] ").map(|(_, m)| m).unwrap_or(line);
        println!("    {msg}");
    }
}
