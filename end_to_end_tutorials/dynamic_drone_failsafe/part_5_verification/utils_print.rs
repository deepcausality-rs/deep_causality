/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::model::{paired, risk_ratio, sign_test_ln_p, upper_bound};
use crate::model_types::{Controller, Ending, Flight, Flown, Scenario};
use deep_causality_algebra::Real;
use deep_causality_num::{lift_usize, lower};
use dynamic_drone_failsafe::{FloatType, manifest_dir};
use std::io::Write;
use std::path::PathBuf;

pub fn print_intro(count: usize) {
    println!("Dynamic drone fail-safe, part 5: verification");
    println!(
        "The controllers of parts 1, 3 and 4, unchanged, each fly the same {count} randomised scenarios."
    );
    println!("Each scenario draws, uniformly:");
    println!("  the local time at launch, from midnight to midnight;");
    println!("  a wind of 0 to 6 m/s from any direction, and a gust at touchdown of up to 2 m;");
    println!("  the line anywhere from 24 to 108 m across the grass slope;");
    println!(
        "  the fix lost 20 to 90 s after launch, and the link lost from 20 s before to 30 s after;"
    );
    println!(
        "  in a third of scenarios the fix comes back 10 to 60 s later, and so, independently, does the link;"
    );
    println!(
        "  in half of them a cell fails up to 60 s after the later loss, and the battery dies 25 s after that;"
    );
    println!(
        "  the terrace anywhere on the slope, and a stand of 18 m trees near where the fix drops out;"
    );
    println!(
        "  up to 8 people within 40 m across and 60 m along of that point, some perhaps under the trees."
    );
}

/// Prints how every flight ended, by controller; the rates of harm to people and of losing the
/// drone, bounded, for all scenarios and for those with and without a failed cell; and the
/// scenarios in which part 4 neither landed safely nor resumed its mission.
pub fn print_report(flown: &[Flown]) {
    let n = flown.len();
    let count = |c: usize, e: Ending| flown.iter().filter(|(_, f)| f[c].ending == e).count();
    println!();
    println!(
        "{:<44} {:>14} {:>14} {:>14}",
        "How the flight ended", "part 1", "part 3", "part 4"
    );
    println!(
        "{:<44} {:>14} {:>14} {:>14}",
        "", "textbook", "context and", "with the"
    );
    println!(
        "{:<44} {:>14} {:>14} {:>14}",
        "", "fail-safe", "action", "Effect Ethos"
    );
    for ending in Ending::ALL {
        let cells: Vec<String> = (0..Controller::ALL.len())
            .map(|c| {
                format!(
                    "{:>5} {:>6.1} %",
                    count(c, ending),
                    percent(count(c, ending), n)
                )
            })
            .collect();
        println!(
            "{:<44} {:>14} {:>14} {:>14}",
            describe(ending),
            cells[0],
            cells[1],
            cells[2]
        );
    }

    println!();
    println!(
        "Rates, each with its one-sided {:.0} % upper confidence bound:",
        lower(CONFIDENCE * HUNDRED)
    );
    let classes: [(&str, Vec<&Flown>); 3] = [
        ("all scenarios", flown.iter().collect()),
        (
            "battery healthy",
            flown.iter().filter(|(s, _)| !s.cell_fails()).collect(),
        ),
        (
            "a cell fails",
            flown.iter().filter(|(s, _)| s.cell_fails()).collect(),
        ),
    ];
    for (label, selected) in classes {
        let k = selected.len();
        println!("  {label}, {k} scenarios:");
        for (c, controller) in Controller::ALL.iter().enumerate() {
            let near = selected
                .iter()
                .filter(|(_, f)| f[c].ending == Ending::NearPerson)
                .count();
            let lost = selected
                .iter()
                .filter(|(_, f)| f[c].ending.drone_lost())
                .count();
            println!(
                "    {:<30} near a person {:>5.1} % (at most {:>5.2} %)   drone lost {:>5.1} % (at most {:>5.2} %)",
                name(*controller),
                percent(near, k),
                lower(upper_bound(near, k) * HUNDRED),
                percent(lost, k),
                lower(upper_bound(lost, k) * HUNDRED),
            );
        }
    }

    print_paired(flown);

    let failures: Vec<&Flown> = flown
        .iter()
        .filter(|(_, f)| !matches!(f[2].ending, Ending::Safe | Ending::Resumed))
        .collect();
    println!();
    if failures.is_empty() {
        println!("Part 4 landed safely or resumed its mission in every scenario.");
        return;
    }
    println!(
        "Part 4 neither landed safely nor resumed its mission in {} scenarios{}:",
        failures.len(),
        if failures.len() > MAX_LISTED {
            format!("; the first {MAX_LISTED}")
        } else {
            String::new()
        }
    );
    for (scenario, flights) in failures.iter().take(MAX_LISTED) {
        println!(
            "  scenario {:>4}: {}; {}, {}.",
            scenario.id,
            conditions(scenario),
            describe(flights[2].ending),
            from_people(flights[2].nearest_person_m),
        );
    }
    println!(
        "Fly any scenario again, with each controller's log: cargo run --release -p dynamic_drone_failsafe --example drone_failsafe_part_5 -- scenario <id>"
    );
}

/// Prints part 4 against each other controller on the same scenarios: where only one of the two came
/// down near a person, how many times less often part 4 did, with its lower confidence bound, and
/// the chance of so uneven a split were the two equally safe.
fn print_paired(flown: &[Flown]) {
    let candidate = Controller::ALL.len() - 1;
    println!();
    println!(
        "Paired on the same scenarios, part 4 against each other controller, all {} scenarios:",
        flown.len()
    );
    for (baseline, controller) in Controller::ALL[..candidate].iter().enumerate() {
        let p = paired(flown, baseline, candidate);
        println!(
            "  {}: only it came down near a person in {} scenarios, only part 4 in {}, both in {}.",
            name(*controller),
            p.only_baseline,
            p.only_candidate,
            p.both,
        );
        match risk_ratio(p) {
            Some((ratio, at_least)) => println!(
                "    Part 4 came down near a person {:.1} times less often; at least {:.1} times less with {:.0} % confidence.",
                lower(ratio),
                lower(at_least),
                lower(CONFIDENCE * HUNDRED),
            ),
            None => println!(
                "    One of the two never came down near a person, so the ratio has no finite estimate."
            ),
        }
        if p.only_baseline + p.only_candidate == 0 {
            println!("    No scenario tells the two apart.");
            continue;
        }
        println!(
            "    Were the two equally safe, a split this uneven would arise with probability {}.",
            probability(sign_test_ln_p(p))
        );
    }
}

/// A probability given by its natural logarithm, in decimals down to a thousandth and in powers of
/// ten below.
fn probability(ln_p: FloatType) -> String {
    let log10 = lower(ln_p) / std::f64::consts::LN_10;
    if log10 >= -3.0 {
        return format!("{:.3}", 10f64.powf(log10));
    }
    let mut exponent = log10.floor();
    let mut mantissa = 10f64.powf(log10 - exponent);
    // 9.96 would print as 10.0.
    if mantissa >= 9.95 {
        mantissa /= 10.0;
        exponent += 1.0;
    }
    format!("{mantissa:.1}e{}", exponent as i32)
}

/// Prints one scenario's conditions, how each controller's flight ended, and each controller's log.
pub fn print_replay(scenario: &Scenario, flights: &[Flight]) {
    println!("Dynamic drone fail-safe, part 5: scenario {}", scenario.id);
    println!("{}.", conditions(scenario));
    println!("{}.", timeline(scenario));
    for (controller, flight) in Controller::ALL.iter().zip(flights) {
        println!();
        println!(
            "{}: at {} s it {}, {}.",
            name(*controller),
            flight.time_s,
            describe(flight.ending),
            from_people(flight.nearest_person_m),
        );
        for entry in &flight.log {
            println!("  {entry}");
        }
    }
}

/// Writes one row per scenario: its conditions and, per controller, how the flight ended, when, and
/// how far from the nearest person. The file is named by the number of scenarios, so a campaign of
/// another size never overwrites it.
pub fn write_csv(flown: &[Flown]) -> std::io::Result<PathBuf> {
    let path = manifest_dir()
        .join("part_5_verification")
        .join(format!("campaign_{}.csv", flown.len()));
    let mut file = std::io::BufWriter::new(std::fs::File::create(&path)?);
    write!(
        file,
        "scenario,start_hour,wind_across_m_s,wind_along_m_s,gust_across_m,gust_along_m,line_across_m,fix_lost_s,fix_restored_s,link_lost_s,link_restored_s,cell_failed_s,crew"
    )?;
    for c in ["part_1", "part_3", "part_4"] {
        write!(file, ",{c}_ending,{c}_time_s,{c}_nearest_person_m")?;
    }
    writeln!(file)?;
    let optional = |t: Option<usize>| t.map_or(String::new(), |t| t.to_string());
    for (s, flights) in flown {
        let m = s.mission;
        let f = m.faults();
        write!(
            file,
            "{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.1},{},{},{},{},{},{}",
            s.id,
            lower(m.start_hour()),
            lower(m.wind_m_s().0),
            lower(m.wind_m_s().1),
            lower(m.touchdown_gust_m().0),
            lower(m.touchdown_gust_m().1),
            lower(m.line_across_m()),
            f.gnss_lost_at_s(),
            optional(f.gnss_restored_at_s()),
            f.link_lost_at_s(),
            optional(f.link_restored_at_s()),
            optional(f.battery_fault_at_s()),
            s.crew,
        )?;
        for flight in flights {
            write!(
                file,
                ",{:?},{},{:.1}",
                flight.ending,
                flight.time_s,
                lower(smaller_than_far(flight.nearest_person_m)),
            )?;
        }
        writeln!(file)?;
    }
    file.flush()?;
    Ok(path)
}

pub fn print_record(path: &std::path::Path) {
    println!();
    println!(
        "Every scenario and how each flight ended: {}",
        path.display()
    );
}

/// A scenario's conditions in one line.
fn conditions(scenario: &Scenario) -> String {
    let m = scenario.mission;
    let (wx, wy) = m.wind_m_s();
    let hour = lower(m.start_hour());
    format!(
        "launch at {:02}:{:02}, wind {:.1} m/s, the line {:.0} m across, {} {}, {}",
        hour.floor() as u32,
        ((hour - hour.floor()) * 60.0).floor() as u32,
        lower(Real::sqrt(wx * wx + wy * wy)),
        lower(m.line_across_m()),
        scenario.crew,
        if scenario.crew == 1 {
            "person"
        } else {
            "people"
        },
        if scenario.cell_fails() {
            "a cell fails"
        } else {
            "battery healthy"
        },
    )
}

/// A scenario's fault timeline in one sentence.
fn timeline(scenario: &Scenario) -> String {
    let f = scenario.mission.faults();
    let back = |at: Option<usize>| {
        at.map_or("never comes back".to_string(), |t| {
            format!("comes back at {t} s")
        })
    };
    format!(
        "The fix degrades at {} s, drops out at {} s and {}; the link drops at {} s and {}; {}",
        f.gnss_degraded_at_s(),
        f.gnss_lost_at_s(),
        back(f.gnss_restored_at_s()),
        f.link_lost_at_s(),
        back(f.link_restored_at_s()),
        f.battery_fault_at_s()
            .map_or("no cell fails".to_string(), |t| format!(
                "a cell fails at {t} s"
            )),
    )
}

fn describe(ending: Ending) -> &'static str {
    match ending {
        Ending::Resumed => "resumed its mission",
        Ending::Safe => "landed upright, clear of people",
        Ending::NearPerson => "came down within 10 m of a person",
        Ending::Ditched => "dropped into the creek",
        Ending::IntoRavine => "fell into the ravine",
        Ending::TippedAndRolled => "tipped over on steep ground",
        Ending::HitTrees => "flew into the trees",
        Ending::Fell => "fell when its battery died",
    }
}

fn name(controller: Controller) -> &'static str {
    match controller {
        Controller::Textbook => "Part 1, textbook fail-safe",
        Controller::ContextAndAction => "Part 3, context and action",
        Controller::EffectEthos => "Part 4, with the Effect Ethos",
    }
}

/// How far the drone came down from the nearest person, in words.
fn from_people(distance_m: FloatType) -> String {
    if smaller_than_far(distance_m) < ZERO {
        "with no one at risk".to_string()
    } else {
        format!("{:.0} m from the nearest person", lower(distance_m))
    }
}

/// A distance to the nearest person, with "no one at risk" shown as -1.
fn smaller_than_far(distance_m: FloatType) -> FloatType {
    if distance_m > lift_usize::<FloatType>(100_000) {
        -ONE
    } else {
        distance_m
    }
}

fn percent(k: usize, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    lower(lift_usize::<FloatType>(k) / lift_usize::<FloatType>(n) * HUNDRED)
}
