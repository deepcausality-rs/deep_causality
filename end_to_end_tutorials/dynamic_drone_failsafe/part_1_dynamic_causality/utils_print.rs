/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::model_types::Faults;
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
        "Touchdown at {time_s} s, {:.0} m along the line, {:.0} m from the nearest person.",
        lower(td.position().1),
        lower(td.nearest_person_m()),
    );
    let verdict = match td.outcome() {
        Outcome::Safe => "It landed upright and can be recovered.".to_string(),
        Outcome::Ditched => "It dropped into the creek and was lost.".to_string(),
        Outcome::HitTrees => "It flew into the trees and was lost.".to_string(),
        Outcome::Fell => "Its battery died in the air, and it fell and was lost.".to_string(),
        Outcome::IntoRavine => "It fell into the ravine and was lost.".to_string(),
        Outcome::AmongPeople => "It came down among the crew.".to_string(),
        Outcome::TippedAndRolled {
            distance_m,
            came_to_rest_on,
        } => format!(
            "It touched down on {:.0} deg {}, tipped over, tumbled {:.0} m {} and was lost.",
            lower(td.slope_deg()),
            surface(td.surface()),
            lower(distance_m),
            resting_place(came_to_rest_on),
        ),
    };
    println!("{verdict}");
    println!();
    println!(
        "The controller found every fault in order and took the textbook fail-safe at each step."
    );
    println!(
        "It never knew what lay below: the steep grass, the creek, the flat tower pads, the crew."
    );
    println!("Part 2 adds that knowledge as context.");
}

/// Prints the faults and fail-safes from the controller's log. The log also holds each second's
/// telemetry and each causaloid's trace; those are left out here.
pub fn print_log(log: &EffectLog) {
    println!();
    println!("Faults and fail-safes from the controller's log:");
    for message in log
        .messages()
        .filter(|m| m.starts_with("t=") && !m.contains(": Telemetry:"))
    {
        println!("  {message}");
    }
}

fn describe(f: &Faults) -> String {
    let names: Vec<&str> = [
        (f.gnss_lost, "fix lost"),
        (f.gnss_degraded && !f.gnss_lost, "fix degraded"),
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

fn resting_place(s: Surface) -> &'static str {
    match s {
        Surface::Water => "into the creek",
        Surface::Grass => "down the grass",
        Surface::Road => "onto the road",
        Surface::Rock => "down the rock",
        Surface::Pad => "onto a tower pad",
        Surface::Ravine => "into the ravine",
        Surface::Trees => "into the trees",
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
        Surface::Trees => "trees",
    }
}
