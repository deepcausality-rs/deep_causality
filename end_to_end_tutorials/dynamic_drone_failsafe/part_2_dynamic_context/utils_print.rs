/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::model_context::{FailsafeState, Faults, Ground, LandNow};
use deep_causality::EffectLog;
use deep_causality_algebra::Real;
use deep_causality_num::lower;
use dynamic_drone_failsafe::{Command, Drone, FloatType, Outcome, Surface, Telemetry, Touchdown};

pub fn print_intro() {
    println!("Dynamic drone fail-safe, part 2: dynamic context");
    println!(
        "The same drone flies the same night. A thermal camera and a LiDAR now look at the ground below it."
    );
    println!(
        "The column \"below\" is the controller's judgement of the ground directly under the drone."
    );
    println!();
    println!(
        "{:>5} {:>8} {:>7} {:>5} {:>5} {:>5} {:>5} {:>6}  {:<38} {:<12} fail-safe",
        "time",
        "across",
        "along",
        "agl",
        "sats",
        "hdop",
        "loss",
        "cell",
        "confirmed faults",
        "below"
    );
}

pub fn print_second(
    drone: &Drone,
    t: &Telemetry,
    faults: &Faults,
    below: Option<Ground>,
    command: Command,
) {
    let (x, y) = drone.position();
    println!(
        "{:>4}s {:>7.0}m {:>6.0}m {:>4.0}m {:>5.0} {:>5.1} {:>4.0}% {:>5.2}V  {:<38} {:<12} {}",
        t.time_s(),
        lower(x),
        lower(y),
        lower(t.altitude_agl_m()),
        lower(t.satellites()),
        lower(t.hdop()),
        lower(t.link_loss_pct()),
        lower(t.min_cell_v()),
        describe(faults),
        below.map_or("not seen", ground),
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
}

/// Prints the controller's judgement of the ground around the touchdown point, one cell per patch,
/// from just downhill of the touchdown point to the tower pad uphill of it.
pub fn print_map(state: &FailsafeState, land_now: Option<&LandNow>, touchdown: (i64, i64)) {
    let land_now_at = land_now.map(|l| l.patch);
    println!();
    println!(
        "The ground as the controller judged it, one 4 m patch per cell, downhill to the left:"
    );
    println!();
    for j in (touchdown.1 - MAP_HALF_ALONG..=touchdown.1 + MAP_HALF_ALONG).rev() {
        let row: String = (touchdown.0 - MAP_DOWNHILL..=touchdown.0 + MAP_UPHILL)
            .map(|i| {
                if (i, j) == touchdown {
                    'X'
                } else if Some((i, j)) == land_now_at {
                    'L'
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
    println!("  L where the fail-safe chose to land now   X where the drone came down");
}

pub fn print_daytime(water: usize, still_water: usize) {
    println!();
    println!(
        "At night the creek is warmer than the land. Read with the daytime rule, where water is the"
    );
    println!(
        "cooler surface, {still_water} of the {water} patches judged to be water would still read as water."
    );
}

/// Prints where the fail-safe chose to land and how far the wind carried the drone from there.
pub fn print_closing(
    land_now: Option<&LandNow>,
    touchdown_s: usize,
    touchdown: (FloatType, FloatType),
) {
    println!();
    if let Some(l) = land_now {
        let (dx, dy) = (touchdown.0 - l.position.0, touchdown.1 - l.position.1);
        println!(
            "When the fail-safe chose to land, at {} s, the ground below was {}. The descent took {} s,",
            l.time_s,
            l.below.map_or("not seen", ground),
            touchdown_s - l.time_s,
        );
        println!(
            "and the night wind carried the drone {:.0} m downhill before it touched down.",
            lower(Real::sqrt(dx * dx + dy * dy)),
        );
    }
    println!(
        "The controller knew the ground, and nothing in it acted on what it knew. Part 3 adds action."
    );
}

/// Prints the faults and fail-safes from the controller's log. The log also holds each second's
/// telemetry, each second's judgement of the ground and each causaloid's trace; those are left out
/// here.
pub fn print_log(log: &EffectLog) {
    println!();
    println!("Faults and fail-safes from the controller's log:");
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
