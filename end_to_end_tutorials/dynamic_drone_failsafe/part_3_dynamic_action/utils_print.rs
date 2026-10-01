/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::model_types::{FailsafeState, Faults, Ground, Maneuver};
use deep_causality::EffectLog;
use deep_causality_num::lower;
use dynamic_drone_failsafe::{Command, Drone, FloatType, Outcome, Surface, Touchdown};

pub fn print_intro() {
    println!("Dynamic drone fail-safe, part 3: dynamic action");
    println!(
        "The same drone flies the same night. A fail-safe machine now turns each fail-safe into a maneuver over the ground."
    );
    println!(
        "The column \"below\" is the controller's judgement of the ground directly under the drone."
    );
    println!();
    println!(
        "{:>5} {:>8} {:>7} {:>5}  {:<38} {:<10} {:<10} maneuver",
        "time", "across", "along", "agl", "confirmed faults", "below", "fail-safe"
    );
}

pub fn print_second(
    drone: &Drone,
    faults: &Faults,
    below: Option<Ground>,
    failsafe: Command,
    maneuver: Maneuver,
) {
    let (x, y) = drone.position();
    println!(
        "{:>4}s {:>7.0}m {:>6.0}m {:>4.0}m  {:<38} {:<10} {:<10} {}",
        drone.time_s(),
        lower(x),
        lower(y),
        lower(drone.altitude_agl_m()),
        describe(faults),
        below.map_or("not seen", ground),
        name(failsafe),
        maneuver_name(maneuver),
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
        Outcome::IntoRavine => "It fell into the ravine and was lost.".to_string(),
        Outcome::AmongPeople => format!(
            "It landed {:.0} m from a member of the crew.",
            lower(td.nearest_person_m())
        ),
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
}

/// Prints the controller's judgement of the ground around the touchdown point, one cell per patch.
pub fn print_map(state: &FailsafeState, touchdown: (i64, i64)) {
    println!();
    println!(
        "The ground as the controller judged it, one 4 m patch per cell, downhill to the left:"
    );
    println!();
    for j in (touchdown.1 - MAP_HALF_ALONG..=touchdown.1 + MAP_HALF_ALONG).rev() {
        let row: String = (touchdown.0 - MAP_HALF_ACROSS..=touchdown.0 + MAP_HALF_ACROSS)
            .map(|i| {
                if (i, j) == touchdown {
                    'X'
                } else {
                    state.ground.get(&(i, j)).map_or(' ', symbol)
                }
            })
            .flat_map(|c| [c, ' '])
            .collect();
        println!("  {}", row.trim_end());
    }
    println!();
    println!("  . safe   / too steep   ~ water   P person   ? unsure   (blank) not seen");
    println!("  X where the drone came down");
}

/// Prints how close to a person the chosen patch lay, by the controller's own map.
pub fn print_closing(state: &FailsafeState, person_m: Option<FloatType>) {
    println!();
    if state.target.patch.is_none() {
        println!("The fail-safe machine found no safe patch and landed where the drone was.");
        return;
    }
    println!(
        "The fail-safe machine chose the safe patch nearest the drone and landed on it. The patch"
    );
    println!("passed every test the controller ran: flat, dry, no one on it.");
    if let Some(d) = person_m {
        println!(
            "The controller's own map showed a person on a patch {:.0} m away from it.",
            lower(d)
        );
    }
    println!("No test asked how close to a person the drone may land. Part 4 adds that check.");
}

/// Prints the faults, fail-safes and maneuvers from the controller's log. The log also holds each
/// second's telemetry, each second's judgement of the ground, each causaloid's trace and each
/// machine evaluation; those are left out here.
pub fn print_log(log: &EffectLog) {
    println!();
    println!("Faults, fail-safes and maneuvers from the controller's log:");
    for message in log
        .messages()
        .filter(|m| m.starts_with("t=") && !m.contains(": Telemetry:") && !m.contains(": Ground:"))
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

fn ground(g: Ground) -> &'static str {
    match g {
        Ground::Safe => "safe",
        Ground::Steep => "too steep",
        Ground::Water => "water",
        Ground::Person => "a person",
        Ground::Unsure => "unsure",
    }
}

fn symbol(g: &Ground) -> char {
    match g {
        Ground::Safe => '.',
        Ground::Steep => '/',
        Ground::Water => '~',
        Ground::Person => 'P',
        Ground::Unsure => '?',
    }
}

fn maneuver_name(m: Maneuver) -> &'static str {
    match m {
        Maneuver::None => "none",
        Maneuver::HoldOver { .. } => "hold over the ground",
        Maneuver::ReturnHome => "fly home",
        Maneuver::LandOn { .. } => "land on the chosen patch",
        Maneuver::ChooseTarget => "choose a landing patch",
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
