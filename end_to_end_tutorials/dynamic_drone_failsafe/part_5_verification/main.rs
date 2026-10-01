/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Dynamic drone fail-safe, part 5: verification
//!
//! The controllers of parts 1, 3 and 4, unchanged, each fly the same randomised scenarios: any
//! time of day, any wind with gusts at touchdown, the line anywhere on the slope, the faults in
//! any order and at any time, the terrace anywhere, and up to eight people around the point where
//! the fix drops out. The campaign counts how every flight ended and bounds each rate at 95 %
//! confidence. Every scenario is a pure function of its number and can be flown again alone.
//!
//! ```text
//! drone_failsafe_part_5                     1000 scenarios
//! drone_failsafe_part_5 campaign <count>    <count> scenarios
//! drone_failsafe_part_5 scenario <id>       one scenario, with each controller's log
//! ```

mod constants;
mod model;
mod model_config;
mod model_types;
// The parts' own `main` and printing use items part 5 does not.
#[allow(dead_code)]
mod part_1;
#[allow(dead_code)]
mod part_3;
#[allow(dead_code)]
mod part_4;
mod utils_print;

use crate::constants::SCENARIOS;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => run(SCENARIOS),
        [command, count] if command == "campaign" => run(count.parse()?),
        [command, id] if command == "scenario" => {
            let (scenario, flights) = model::fly_scenario(id.parse()?)?;
            utils_print::print_replay(&scenario, &flights);
            Ok(())
        }
        _ => Err("usage: drone_failsafe_part_5 [campaign <count> | scenario <id>]".into()),
    }
}

fn run(count: usize) -> Result<(), Box<dyn Error>> {
    utils_print::print_intro(count);
    let flown = model::campaign(count)?;
    utils_print::print_report(&flown);
    let path = utils_print::write_csv(&flown)?;
    utils_print::print_record(&path);
    Ok(())
}
