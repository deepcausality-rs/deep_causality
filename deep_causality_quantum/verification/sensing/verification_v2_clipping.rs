/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V2: the wrong-sign Coriolis shift, explained by detection clipping.
//!
//! Farah et al., *Effective velocity distribution in an atom gravimeter: effect of the convolution
//! with the response of the detection*, Phys. Rev. A 90, 023606 (2014), arXiv:1406.5998.
//! Unbalancing the east-west molasses beams moves gravity by +46(2) µGal per unit of imbalance
//! (§III), while an imbalance with less intensity in the East beam drives the atoms East and so
//! predicts a negative Coriolis shift. The paper explains the sign by the detection, whose finite
//! field of view clips the displaced cloud: the cloud moves 4.95(15) mm per unit of imbalance, and
//! a displacement moves gravity by 14.2(1.1) µGal/mm (§IV). The same imbalance along north-south,
//! where the detection does not clip, moves gravity by 10(5) µGal and the atoms by 1.8 (mm/s) per
//! unit (§VII, §VIII).
//!
//! The verification attributes the +46(2) µGal among the candidates through QCL:
//!
//! * Coriolis from the atoms' real velocity: the north-south velocity per unit of imbalance,
//!   1.8 mm/s, times the expected 9.72 µGal/(mm/s) (§VIII), with the sign the paper expects;
//! * the wavefront, bounded by the north-south sensitivity, 10(5) µGal;
//! * clipping of the displaced cloud: 4.95(15) mm times 14.2(1.1) µGal/mm;
//! * clipping with the real velocity's Coriolis shift, the sum of the two.
//!
//! Each prediction carries its inputs' uncertainty; a comparison adds the largest among the
//! candidates to the measurement's in quadrature. With Coriolis and the wavefront alone, no
//! candidate holds. With clipping added, the candidates that hold carry it. The planner finds that
//! a 180° turn, which flips Coriolis and clipping alike, separates nothing, while a cloud
//! displacement does. Two checks concern the source: its velocity-to-shift conversion in §VII
//! against the 9.72 µGal/(mm/s) it expects in §VIII, and its "kHz/mm", which only reads as Hz/mm.

#[path = "common/attribution.rs"]
mod attribution;
#[path = "common/fringe_qubit.rs"]
mod fringe_qubit;
#[path = "common/report.rs"]
mod report;

use attribution::{Measurement, attribute, describe};
use deep_causality_haft::Either;
use deep_causality_quantum::{
    Ambiguity, Experiment, Fringe, MinCostCover, design, separation_bits,
};
use report::Report;
use std::error::Error;

/// The agreement, in standard errors, and the floor, in bits.
const SIGMAS: f64 = 3.0;
const FLOOR_BITS: f64 = 5.0;

/// The measured sensitivity to the east-west imbalance, in µGal per unit (§III).
const MEASURED: (f64, f64) = (46.0, 2.0);
/// The north-south sensitivity, in µGal per unit, and the real velocity per unit, in mm/s.
const NORTH_SOUTH: (f64, f64) = (10.0, 5.0);
const REAL_VELOCITY: f64 = 1.8;
/// The Coriolis shift the paper expects per mm/s (§VIII), in µGal.
const EXPECTED_CORIOLIS: f64 = 9.72;
/// The cloud's displacement per unit of imbalance, in mm, and the clipping shift per mm, in µGal.
const DISPLACEMENT: (f64, f64) = (4.95, 0.15);
const CLIPPING: (f64, f64) = (14.2, 1.1);
/// §VII: velocities per unit of imbalance and per mm, in mm/s, and the shifts the paper converts
/// them to, in µGal.
const VELOCITY_PER_UNIT: (f64, f64) = (1.7, 18.6);
const VELOCITY_PER_MM: (f64, f64) = (1.15, 12.6);
/// §VI: the Doppler shift per mm of displacement as printed, read in Hz/mm, and the horizontal
/// Raman beam's angle to east-west, in degrees.
const DOPPLER_PER_MM: f64 = 2079.0;
const RAMAN_ANGLE_DEG: f64 = 45.0;
const WAVELENGTH: f64 = 780.241e-9;

fn main() -> Result<(), Box<dyn Error>> {
    println!("V2 Farah et al. 2014: the wrong-sign Coriolis shift and detection clipping");
    let mut report = Report::default();

    let coriolis = -REAL_VELOCITY * EXPECTED_CORIOLIS;
    let clipping = DISPLACEMENT.0 * CLIPPING.0;
    let clipping_u = clipping * (DISPLACEMENT.1 / DISPLACEMENT.0).hypot(CLIPPING.1 / CLIPPING.0);
    let measured = |predictions: Vec<f64>, prediction_u: f64| Measurement {
        name: "east-west imbalance".into(),
        value: MEASURED.0,
        standard_error: MEASURED.1.hypot(prediction_u),
        predictions,
    };

    let without = attribute(
        &["Coriolis", "wavefront"],
        &[measured(vec![coriolis, NORTH_SOUTH.0], NORTH_SOUTH.1)],
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "with Coriolis and the wavefront alone, no candidate explains +46 µGal",
        matches!(without.outcome, Either::Right(Ambiguity::NoSurvivor { .. })),
        format!(
            "QCL: {}; Coriolis predicts {coriolis:.1}, the wavefront at most {:.0} ± {:.0}",
            describe(&without.outcome),
            NORTH_SOUTH.0,
            NORTH_SOUTH.1
        ),
    );

    let names = [
        "Coriolis",
        "wavefront",
        "clipping",
        "clipping with Coriolis",
    ];
    let with = attribute(
        &names,
        &[measured(
            vec![coriolis, NORTH_SOUTH.0, clipping, clipping + coriolis],
            clipping_u,
        )],
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "with clipping added, every candidate that holds carries clipping",
        !with.holding.is_empty() && with.holding.iter().all(|h| h.starts_with("clipping")),
        format!(
            "QCL: {}; holding {:?}; clipping predicts {clipping:.1} ± {clipping_u:.1}, clipping \
             with Coriolis {:.1}",
            describe(&with.outcome),
            with.holding,
            clipping + coriolis
        ),
    );

    // The planner over Coriolis and clipping, both at the measured sensitivity: a turn flips both;
    // a 1 mm displacement of the cloud moves clipping alone.
    let fringe = Fringe::new(0.5, 1e-5)?;
    let read = |shift: f64| fringe.operating_point() + fringe.slope() * shift;
    let experiments = [
        ("turned 180°", [-MEASURED.0, -MEASURED.0]),
        (
            "cloud displaced 1 mm",
            [MEASURED.0, MEASURED.0 + CLIPPING.0],
        ),
    ];
    let per_draw: Vec<f64> = experiments
        .iter()
        .map(|(_, p)| separation_bits(read(p[0]), read(p[1]), 1).abs())
        .collect();
    let draws = (FLOOR_BITS / per_draw[1]).ceil() as u64;
    let probes = experiments
        .iter()
        .map(|(name, p)| Experiment::new(*name, 1.0, draws, vec![read(p[0]), read(p[1])]))
        .collect::<Result<Vec<_>, _>>()?;
    let plan = design(2, &probes, MinCostCover::new(FLOOR_BITS))?;
    let chosen: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    report.check(
        "a 180° turn does not separate Coriolis from clipping; a cloud displacement does",
        per_draw[0] == 0.0 && chosen == ["cloud displaced 1 mm"],
        format!(
            "bits per draw: turned {:.1e}, displaced {:.2e}; plan {chosen:?}",
            per_draw[0], per_draw[1]
        ),
    );

    // The source: §VII's conversions against §VIII's expectation, and the Doppler unit.
    let (per_unit, per_mm) = (
        VELOCITY_PER_UNIT.1 / VELOCITY_PER_UNIT.0,
        VELOCITY_PER_MM.1 / VELOCITY_PER_MM.0,
    );
    report.check(
        "§VII converts velocity to Coriolis shift at 10.9 µGal/(mm/s), not the 9.72 of §VIII",
        ((per_unit - EXPECTED_CORIOLIS) / EXPECTED_CORIOLIS).abs() > 0.1
            && ((per_mm - EXPECTED_CORIOLIS) / EXPECTED_CORIOLIS).abs() > 0.1,
        format!("{per_unit:.2} and {per_mm:.2} µGal/(mm/s) against {EXPECTED_CORIOLIS}"),
    );
    let velocity = DOPPLER_PER_MM * WAVELENGTH / (2.0 * RAMAN_ANGLE_DEG.to_radians().cos()) * 1e3;
    report.check(
        "the printed −2079 kHz/mm reads as Hz/mm",
        (velocity - VELOCITY_PER_MM.0).abs() < 0.02,
        format!(
            "2079 Hz/mm along a Raman beam at 45° is {velocity:.3} (mm/s)/mm against the printed \
             1.15; in kHz/mm it would be a thousand times that"
        ),
    );

    report.finish("V2").map_err(Into::into)
}
