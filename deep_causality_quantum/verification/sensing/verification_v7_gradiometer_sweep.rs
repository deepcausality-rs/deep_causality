/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V7: a gravity gradiometer's parameter sweep, and three single-cause attributions.
//!
//! Sorrentino et al., *Sensitivity limits of a Raman atom interferometer as a gravity
//! gradiometer*, Phys. Rev. A 89, 023607 (2014), arXiv:1312.3741. The paper measures the ellipse
//! phase angle's sensitivity to each experimental parameter (Table I) and explains three of them:
//!
//! * a tilt of the Raman mirror along east-west couples to gravity through the Coriolis force,
//!   −34θ by eq. (10); the measured slope is −37 ± 5 mrad/mrad along east-west (§IV C 3; Table I
//!   prints it without the sign) and −5 ± 2 along north-south;
//! * a magnetic pulse makes the angle depend on the bias solenoid's current through the quadratic
//!   Zeeman shift, ≃ 72 mrad/mA by eq. (12); the measured slope is 69 ± 1;
//! * the MOT beams' up/down intensity ratio moves the angle by 0.80 ± 0.06 mrad/%; the MOT position
//!   it shifts, 0.1 mm/%, explains a tenth of that through the gravity and magnetic gradients, so
//!   the paper names the clouds' temperature "most probably", by elimination.
//!
//! QCL attributes each measurement among the candidates through its control stage. The fourth check
//! compares Table I with the text's list of the parameters that dominate the long-term stability,
//! ranking each by its slope times its daily fluctuation.

#[path = "common/attribution.rs"]
mod attribution;
#[path = "common/fringe_qubit.rs"]
mod fringe_qubit;
#[path = "common/report.rs"]
mod report;

use attribution::{Measurement, attribute, describe};
use deep_causality_haft::Either;
use report::Report;
use std::error::Error;

/// The agreement, in standard errors, and the floor, in bits.
const SIGMAS: f64 = 3.0;
const FLOOR_BITS: f64 = 5.0;
/// Half a unit of the last printed digit of a prediction printed without an uncertainty.
const PRINTED_ROUNDING: f64 = 0.5;

/// Eq. (10)'s printed inputs: the clouds' launch velocities, in m/s, the latitude, in degrees, and
/// the time between pulses, in s.
const V_UPPER: f64 = 4.3;
const V_LOWER: f64 = 3.5;
const LATITUDE_DEG: f64 = 43.0;
const T: f64 = 0.160;
const EARTH_RATE: f64 = 7.292115e-5;
const WAVELENGTH: f64 = 780.241e-9;

fn main() -> Result<(), Box<dyn Error>> {
    println!("V7 Sorrentino et al. 2014: the gradiometer's parameter sweep");
    let mut report = Report::default();
    let with_rounding = |se: f64| se.hypot(PRINTED_ROUNDING);

    // The mirror tilt: Coriolis against no effect, along east-west and north-south.
    let k_eff = 4.0 * std::f64::consts::PI / WAVELENGTH;
    let from_inputs =
        -2.0 * EARTH_RATE * k_eff * T * T * (V_UPPER - V_LOWER) * LATITUDE_DEG.to_radians().cos();
    let tilt = attribute(
        &["Coriolis", "no tilt effect"],
        &[
            Measurement {
                name: "east-west tilt".into(),
                value: -37.0,
                standard_error: with_rounding(5.0),
                predictions: vec![-34.0, 0.0],
            },
            Measurement {
                name: "north-south tilt".into(),
                value: -5.0,
                standard_error: 2.0,
                predictions: vec![0.0, 0.0],
            },
        ],
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "the mirror tilt's slope is the Coriolis effect of eq. (10)",
        matches!(&tilt.outcome, Either::Left(s) if s.name == "Coriolis"),
        format!(
            "QCL: {}; eq. (10) from the printed inputs gives {from_inputs:.1} mrad/mrad against \
             the printed ≃ −34",
            describe(&tilt.outcome)
        ),
    );

    // The magnetic pulse: the quadratic Zeeman shift of eq. (12) against the solenoid without a
    // pulse, whose sensitivity at the working point is below 0.9 mrad/mA.
    let pulse = attribute(
        &["pulse quadratic Zeeman", "no pulse effect"],
        &[Measurement {
            name: "bias solenoid with pulse".into(),
            value: 69.0,
            standard_error: with_rounding(1.0),
            predictions: vec![72.0, 0.9],
        }],
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "the magnetic pulse's slope is the quadratic Zeeman shift of eq. (12)",
        matches!(&pulse.outcome, Either::Left(s) if s.name == "pulse quadratic Zeeman"),
        format!(
            "QCL: {}; 69 ± 1 against ≃ 72, {:.1} standard errors with the prediction's rounding",
            describe(&pulse.outcome),
            3.0 / with_rounding(1.0)
        ),
    );

    // The MOT intensity ratio: the quantified candidates both fail, which leaves the clouds'
    // temperature by elimination.
    let mot = attribute(
        &["MOT position through the gradients", "no effect"],
        &[Measurement {
            name: "MOT up/down intensity ratio".into(),
            value: 0.80,
            standard_error: 0.06,
            predictions: vec![0.08, 0.0],
        }],
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "no quantified candidate explains the MOT ratio's slope, which leaves the temperature",
        mot.holding.is_empty()
            && matches!(
                &mot.outcome,
                Either::Right(deep_causality_quantum::Ambiguity::NoSurvivor { .. })
            ),
        format!(
            "QCL: {}; the position shift explains at most 0.08 of 0.80 ± 0.06 mrad/%",
            describe(&mot.outcome)
        ),
    );

    // Table I against the text: slope times daily fluctuation, in µrad. A curvature is taken at the
    // working point the text gives; a quadratic dependence on the intensity ratio around its optimum
    // is taken as the curvature times the fluctuation squared.
    let mut impact: [(&str, f64, bool); 11] = [
        ("MOT power ratio", 0.80e3 * 2.0, true),
        ("bias solenoid, with pulse", 69.0e3 * 0.020, true),
        ("Raman total intensity", 0.30e3 * 2.0, false),
        ("Raman mirror east-west tilt", 37.0e3 * 0.010, true),
        ("probe power", 0.15e3 * 2.0, true),
        ("repumper power", 0.10e3 * 2.0, false),
        ("Raman intensity ratio", 20.0 * 2.0 * 2.0, false),
        ("MOT total power (bound)", 20.0 * 2.0, false),
        ("vertical MOT compensation coil", 1.0e3 * 0.020, true),
        ("bias solenoid, no pulse (bound)", 0.9e3 * 0.020, false),
        ("probe power ratio", 40.0 * 0.1, false),
    ];
    impact.sort_by(|a, b| b.1.total_cmp(&a.1));
    let rank = |name: &str| {
        impact
            .iter()
            .position(|row| row.0 == name)
            .map_or(0, |i| i + 1)
    };
    report.check(
        "the text's dominant parameters omit the third by Table I, Raman total intensity",
        rank("Raman total intensity") == 3 && !impact[2].2,
        format!(
            "ranked by µrad per day: {}; the text names {} and omits {}",
            impact
                .iter()
                .map(|(name, v, _)| format!("{name} {v:.0}"))
                .collect::<Vec<_>>()
                .join(", "),
            impact
                .iter()
                .filter(|row| row.2)
                .map(|row| format!("{} (rank {})", row.0, rank(row.0)))
                .collect::<Vec<_>>()
                .join(", "),
            impact
                .iter()
                .take(5)
                .filter(|row| !row.2)
                .map(|row| row.0)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );

    report.finish("V7").map_err(Into::into)
}
