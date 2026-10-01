/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model_types::Faults;
use deep_causality::EffectLog;
use deep_causality_num::lower;
use dynamic_drone_failsafe::{Command, Drone, Outcome, Surface, Telemetry, Touchdown};

pub fn print_intro() {
    println!("Dynamic drone fail-safe, part 1: dynamic causality");
    println!(
        "An inspection drone flies a high-voltage line at night, 40 m above a 30 deg grass slope over a creek."
    );
    println!();
    println!(
        "{:>5} {:>8} {:>7} {:>5} {:>5} {:>5} {:>5} {:>6}  {:<38} fail-safe",
        "time", "across", "along", "agl", "sats", "hdop", "loss", "cell", "confirmed faults"
    );
}

pub fn print_second(drone: &Drone, t: &Telemetry, faults: &Faults, command: Command) {
    let (x, y) = drone.position();
    println!(
        "{:>4}s {:>7.0}m {:>6.0}m {:>4.0}m {:>5.0} {:>5.1} {:>4.0}% {:>5.2}V  {:<38} {}",
        t.time_s(),
        lower(x),
        lower(y),
        lower(t.altitude_agl_m()),
        lower(t.satellites()),
        lower(t.hdop()),
        lower(t.link_loss_pct()),
        lower(t.min_cell_v()),
        describe(faults),
        name(command),
    );
}

pub fn print_touchdown(time_s: usize, td: &Touchdown) {
    println!();
    println!(
        "Touchdown at {time_s} s, {:.0} m across and {:.0} m along: {} at {:.0} deg, nearest person {:.0} m away.",
        lower(td.position().0),
        lower(td.position().1),
        surface(td.surface()),
        lower(td.slope_deg()),
        lower(td.nearest_person_m()),
    );
    let verdict = match td.outcome() {
        Outcome::Safe => "The drone landed upright on ground it can stay on.".to_string(),
        Outcome::Ditched => "The drone came down in the water and is lost.".to_string(),
        Outcome::IntoRavine => "The drone came down into the ravine and is lost.".to_string(),
        Outcome::AmongPeople => "The drone came down among people.".to_string(),
        Outcome::TippedAndRolled {
            distance_m,
            came_to_rest_on,
        } => format!(
            "The ground was too steep: the drone tipped over and tumbled {:.0} m downhill, coming to rest on {}. It is lost.",
            lower(distance_m),
            surface(came_to_rest_on)
        ),
    };
    println!("{verdict}");
    println!();
    println!(
        "The controller found every fault in order and took the textbook fail-safe at each step. It never knew"
    );
    println!(
        "what was below it: the slope, the creek, the flat tower pad, the crew on the road. That is context, which part 2 adds."
    );
}

pub fn print_log(log: &EffectLog) {
    println!();
    println!("Decisions in the controller's log:");
    for message in log
        .messages()
        .filter(|m| m.contains(" faults: ") || m.contains(" fail-safe: "))
    {
        println!("  {message}");
    }
}

fn describe(f: &Faults) -> String {
    let names: Vec<&str> = [
        (f.gnss_lost, "GNSS lost"),
        (f.gnss_degraded && !f.gnss_lost, "GNSS degraded"),
        (f.link_lost, "link lost"),
        (f.battery_critical, "battery critical"),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .map(|(_, name)| *name)
    .collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

fn name(command: Command) -> &'static str {
    match command {
        Command::Continue => "continue",
        Command::Hold => "hold",
        Command::ReturnHome => "return home",
        Command::LandNow => "land now",
    }
}

fn surface(s: Surface) -> &'static str {
    match s {
        Surface::Water => "water",
        Surface::Grass => "grass",
        Surface::Road => "road",
        Surface::Rock => "rock",
        Surface::Pad => "a tower pad",
        Surface::Ravine => "the ravine",
    }
}
