/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V3: Coriolis against wavefront aberration, and the wavefront's ambiguity under a temperature
//! scan.
//!
//! Louchet-Chauvet et al., *The influence of transverse motion within an atomic gravimeter*, New
//! J. Phys. 13, 065025 (2011). The gravimeter interleaves four configurations, the wave vector up
//! and down at the nominal and the half Rabi frequency (§3.2), and separates the Coriolis shift
//! from the wavefront aberration by turning the experiment by 180° (§4.2). A temperature scan from
//! 2 to 6 µK fits wavefronts of increasing order whose extrapolations to zero temperature disagree
//! (§4.4, Fig. 9). Table 1 sums the corrections.
//!
//! The harness checks, against the paper:
//!
//! * the budget's sums, and the two inconsistencies of the source: eq. (4) as printed carries a
//!   spurious factor 2, and the device's total uncertainty, 5.1 µGal, is not the quadrature of its
//!   rows;
//! * that QCL's planner finds the turn separating Coriolis from the wavefront, and the wave-vector
//!   reversal and the Rabi step separating nothing;
//! * that the turn's published Coriolis value, 0.5 ± 0.4 µGal, does not resolve a Coriolis shift
//!   from none;
//! * that the fits of Fig. 9 hold against the measured points, every fit of panel (a) and, as the
//!   paper says, most of panel (c), which leaves the wavefront ambiguous rather than measured.
//!
//! Fig. 9 is read from the PDF's vector paths by `papers/digitised/extract_louchet2011_fig9.py`.

#[path = "common/attribution.rs"]
mod attribution;
#[path = "common/data.rs"]
mod data;
#[path = "common/fringe_qubit.rs"]
mod fringe_qubit;
#[path = "common/report.rs"]
mod report;

use attribution::{Measurement, attribute, describe};
use data::{load_csv, manifest_dir};
use deep_causality_context_store::ContextSnapshot;
use deep_causality_haft::Either;
use deep_causality_quantum::{
    ConfiguredExperiment, Experiment, Hypothesis, MinCostCover, ObservedContext, QuantumError,
    Response, ResponseModel, design, separation_bits,
};
use fringe_qubit::{mechanism, output_port, phase_channel, plant};
use report::Report;
use std::error::Error;

/// The rubidium D2 wavelength, in m, and the time between pulses, in s (§2).
const WAVELENGTH: f64 = 780.241e-9;
const T: f64 = 0.070;
/// The contrast of the planner's candidates; the planner compares them on one fringe.
const CONTRAST: f64 = 0.5;
/// One µGal, in m/s².
const MICRO_GAL: f64 = 1.0e-8;
/// The floor, in bits, and the agreement, in standard errors.
const FLOOR_BITS: f64 = 5.0;
const SIGMAS: f64 = 3.0;
/// Table 1: the environmental rows and the device rows, each a bias and an uncertainty, in µGal,
/// and the printed totals.
const ENVIRONMENT: [(f64, f64); 3] = [(-18.8, 0.5), (-1.7, 0.5), (-3.8, 0.1)];
const DEVICE: [(f64, f64); 6] = [
    (-12.1, 0.8),
    (0.0, 0.1),
    (4.2, 0.5),
    (-12.3, 0.5),
    (0.5, 0.4),
    (0.0, 4.0),
];
const PRINTED_ENVIRONMENT: f64 = -24.3;
const PRINTED_DEVICE: (f64, f64) = (-19.7, 5.1);
const PRINTED_TOTAL: (f64, f64) = (-44.0, 5.2);
/// The Coriolis correction and its uncertainty, in µGal (Table 1, §4.2).
const CORIOLIS: (f64, f64) = (0.5, 0.4);

fn main() -> Result<(), Box<dyn Error>> {
    println!("V3 Louchet-Chauvet et al. 2011: Coriolis, wavefront, and the temperature scan");
    let mut report = Report::default();

    budget(&mut report);
    protocol(&mut report);
    planner(&mut report)?;

    // The turn's published Coriolis value against a Coriolis shift and none.
    let coriolis = attribute(
        &["Coriolis 0.5 µGal", "no Coriolis"],
        &[Measurement {
            name: "half-difference of the two orientations".into(),
            value: CORIOLIS.0,
            standard_error: CORIOLIS.1,
            predictions: vec![CORIOLIS.0, 0.0],
        }],
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "the turn's 0.5 ± 0.4 µGal does not resolve a Coriolis shift from none",
        coriolis.holding.len() == 2 && matches!(coriolis.outcome, Either::Right(_)),
        format!("QCL: {}", describe(&coriolis.outcome)),
    );

    // The temperature scan: every fit of Fig. 9 against the measured points.
    let digitised = manifest_dir().join("papers/digitised");
    let points = load_csv(&digitised.join("louchet2011_fig9_points.csv"))?;
    let fits = load_fits(&digitised.join("louchet2011_fig9_fits.csv"))?;
    let names: Vec<&str> = fits.iter().map(|(name, _)| name.as_str()).collect();
    let scan = points
        .iter()
        .skip(1) // the reference point at 2 µK, zero by definition
        .map(|row| Measurement {
            name: format!("{:.2} µK", row[0]),
            value: row[1],
            standard_error: row[2],
            predictions: fits
                .iter()
                .map(|(_, curve)| interpolate(curve, row[0]))
                .collect(),
        })
        .collect::<Vec<_>>();
    let scanned = attribute(&names, &scan, SIGMAS, FLOOR_BITS)?;
    let holds = |name: &str| scanned.holding.iter().any(|h| h == name);
    let (refits, fakes): (Vec<&Fit>, Vec<&Fit>) =
        fits.iter().partition(|(name, _)| !name.starts_with("fake"));
    let fakes_holding = fakes.iter().filter(|(name, _)| holds(name)).count();
    let rejected: Vec<&str> = names.iter().copied().filter(|name| !holds(name)).collect();
    report.check(
        "every fit of Fig. 9(a), and most of Fig. 9(c), hold against the measured points",
        refits.iter().all(|(name, _)| holds(name)) && 2 * fakes_holding > fakes.len(),
        format!(
            "at {SIGMAS} standard errors: {} of {} fits of (a), {fakes_holding} of {} of (c) \
             (the paper: \"most of the corresponding fits agree reasonably well\"); rejected: {rejected:?}",
            refits.iter().filter(|(name, _)| holds(name)).count(),
            refits.len(),
            fakes.len()
        ),
    );

    let at_zero = |name: &str| {
        fits.iter()
            .find(|(n, _)| n == name)
            .and_then(|(_, curve)| (curve[0].0 <= 1e-3).then_some(curve[0].1))
    };
    let linear = at_zero("linear").ok_or("the linear fit reaches 0 K")?;
    report.check(
        "the extracted linear fit extrapolates to the printed 0.6(5) µGal",
        (linear - 0.6).abs() < 0.05,
        format!("{linear:.2} µGal at 0 K"),
    );
    let reached: Vec<f64> = scanned.holding.iter().filter_map(|h| at_zero(h)).collect();
    let (low, high) = reached
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let resolution = scan.iter().map(|m| m.standard_error).sum::<f64>() / scan.len() as f64;
    report.check(
        "the scan leaves the wavefront ambiguous: no point value at zero temperature",
        matches!(scanned.outcome, Either::Right(_)) && high - low > 10.0 * resolution,
        format!(
            "QCL: {}; the holding fits that reach 0 K extrapolate from {low:.2} to {high:.2} µGal, \
             against a mean standard error of {resolution:.2}; {} fits leave the plot above 0 K \
             (the text: −4 to 15)",
            describe(&scanned.outcome),
            names.iter().filter(|name| at_zero(name).is_none()).count()
        ),
    );

    report.finish("V3").map_err(Into::into)
}

/// Table 1's sums, and its device uncertainty against the quadrature of its rows.
fn budget(report: &mut Report) {
    let sum = |rows: &[(f64, f64)]| rows.iter().map(|r| r.0).sum::<f64>();
    let quadrature = |rows: &[(f64, f64)]| rows.iter().map(|r| r.1 * r.1).sum::<f64>().sqrt();
    let (environment, device) = (sum(&ENVIRONMENT), sum(&DEVICE));
    report.check(
        "Table 1's corrections sum to its printed totals",
        (environment - PRINTED_ENVIRONMENT).abs() < 0.05
            && (device - PRINTED_DEVICE.0).abs() < 0.05
            && (environment + device - PRINTED_TOTAL.0).abs() < 0.05,
        format!(
            "environment {environment:.1}, device {device:.1}, total {:.1} µGal",
            environment + device
        ),
    );
    let device_u = quadrature(&DEVICE);
    report.check(
        "the device's printed 5.1 µGal is not the quadrature of its rows",
        (device_u - PRINTED_DEVICE.1).abs() > 0.5,
        format!(
            "quadrature {device_u:.2} µGal against the printed {}; the total's {} follows from the \
             printed 5.1 and the environment's {:.1}",
            PRINTED_DEVICE.1,
            PRINTED_TOTAL.1,
            quadrature(&ENVIRONMENT)
        ),
    );
}

/// Eqs. (3) and (4) against eq. (2): the four configurations' phases from arbitrary shifts.
fn protocol(report: &mut Report) {
    let (gravity, coriolis, wavefront, light_shift, independent) = (100.0, 3.0, 5.0, 7.0, 11.0);
    // Eq. (2), with the light shift linear in the Rabi frequency.
    let phase = |up: bool, rabi: f64| {
        let dependent = gravity + coriolis + wavefront + light_shift * rabi;
        if up {
            dependent + independent
        } else {
            -dependent + independent
        }
    };
    let (u1, d1, u2, d2) = (
        phase(true, 1.0),
        phase(false, 1.0),
        phase(true, 0.5),
        phase(false, 0.5),
    );
    let eq3 = (u2 - d2) - (u1 - d1) / 2.0;
    let eq4_printed = 2.0 * ((u1 - d1) - (u2 - d2));
    let eq4 = (u1 - d1) - (u2 - d2);
    report.check(
        "eq. (3) holds; eq. (4) as printed doubles the light shift",
        (eq3 - (gravity + coriolis + wavefront)).abs() < 1e-12
            && (eq4_printed - 2.0 * light_shift).abs() < 1e-12
            && (eq4 - light_shift).abs() < 1e-12,
        format!(
            "eq. (3) gives {eq3} for Φg + ΦC + ΦWF = {}; printed eq. (4) gives {eq4_printed} for \
             ΦLS2 = {light_shift}, and without its factor 2, {eq4}",
            gravity + coriolis + wavefront
        ),
    );
}

/// The planner over Coriolis and the wavefront, both at the same passive shift: Coriolis flips
/// under the turn; neither depends on the wave vector's sign in the half-difference, or on the
/// Rabi frequency.
fn planner(report: &mut Report) -> Result<(), Box<dyn Error>> {
    let model = TurnModel {
        phase_per_ugal: 4.0 * std::f64::consts::PI / WAVELENGTH * T * T * MICRO_GAL,
    };
    let candidates = [
        mechanism("Coriolis", CONTRAST)?,
        mechanism("wavefront", CONTRAST)?,
    ];
    let settings = [
        ("k reversed", Setting::KReversed),
        ("Rabi halved", Setting::RabiHalved),
        ("turned 180°", Setting::Turned),
    ];
    let (the_plant, observables) = (plant()?, [output_port()?]);
    let predictions = settings
        .iter()
        .map(|(name, setting)| {
            let e = ConfiguredExperiment::new(*name, 1.0, 1, *setting, 0)?;
            candidates
                .iter()
                .map(|h| e.predict(&model, h, &the_plant, &observables))
                .collect::<Result<Vec<f64>, QuantumError>>()
        })
        .collect::<Result<Vec<_>, QuantumError>>()?;
    let per_draw: Vec<f64> = predictions
        .iter()
        .map(|p| separation_bits(p[0], p[1], 1).abs())
        .collect();
    let draws = (FLOOR_BITS / per_draw[2]).ceil() as u64;
    let probes = settings
        .iter()
        .zip(&predictions)
        .map(|((name, _), p)| Experiment::new(*name, 1.0, draws, p.clone()))
        .collect::<Result<Vec<_>, QuantumError>>()?;
    let plan = design(2, &probes, MinCostCover::new(FLOOR_BITS))?;
    let chosen: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    report.check(
        "only the turn separates Coriolis from the wavefront",
        chosen == ["turned 180°"] && per_draw[0] == 0.0 && per_draw[1] == 0.0,
        format!(
            "bits per draw: k reversed {:.1e}, Rabi halved {:.1e}, turned {:.2e}; plan {chosen:?}",
            per_draw[0], per_draw[1], per_draw[2]
        ),
    );
    Ok(())
}

/// A fit of Fig. 9: its name and its polyline of `(temperature µK, Δg µGal)`.
type Fit = (String, Vec<(f64, f64)>);

/// The fits of Fig. 9, each a polyline ascending in temperature.
fn load_fits(path: &std::path::Path) -> Result<Vec<Fit>, Box<dyn Error>> {
    let mut fits: Vec<Fit> = Vec::new();
    for line in std::fs::read_to_string(path)?.lines().skip(1) {
        let cells: Vec<&str> = line.split(',').collect();
        let (name, t, g) = (cells[0], cells[1].parse::<f64>()?, cells[2].parse::<f64>()?);
        match fits.last_mut() {
            Some((last, curve)) if last == name => curve.push((t, g)),
            _ => fits.push((name.to_string(), vec![(t, g)])),
        }
    }
    Ok(fits)
}

/// Linear interpolation along an ascending polyline, held at its ends.
fn interpolate(curve: &[(f64, f64)], t: f64) -> f64 {
    let i = curve.partition_point(|&(x, _)| x <= t);
    if i == 0 {
        return curve[0].1;
    }
    if i == curve.len() {
        return curve[curve.len() - 1].1;
    }
    let ((x0, y0), (x1, y1)) = (curve[i - 1], curve[i]);
    y0 + (t - x0) / (x1 - x0) * (y1 - y0)
}

/// What the planner's experiments set.
#[derive(Debug, Clone, Copy)]
enum Setting {
    /// The wave vector reversed, its half-difference taken.
    KReversed,
    /// The Rabi frequency halved at constant pulse area.
    RabiHalved,
    /// The experiment turned by 180° about vertical.
    Turned,
}

impl ObservedContext for Setting {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// Coriolis and the wavefront, each 0.5 µGal in the passive configuration: only Coriolis flips
/// under the turn.
struct TurnModel {
    phase_per_ugal: f64,
}

impl ResponseModel<f64, Setting> for TurnModel {
    fn respond(
        &self,
        candidate: &Hypothesis<f64>,
        setting: &Setting,
    ) -> Result<Response<f64>, QuantumError> {
        let flips = candidate.name() == "Coriolis" && matches!(setting, Setting::Turned);
        let ugal = if flips { -CORIOLIS.0 } else { CORIOLIS.0 };
        Ok(Response::Channel(phase_channel(
            self.phase_per_ugal * ugal,
        )?))
    }
}
