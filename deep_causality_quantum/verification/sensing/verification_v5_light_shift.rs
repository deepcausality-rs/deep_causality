/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V5: the two-photon light shift of a gravimeter, attributed by its dependence on the Rabi
//! frequency.
//!
//! Gauguet et al., *Off-resonant Raman transitions impact in an atom interferometer*, Phys. Rev. A
//! 78, 043615 (2008), arXiv:0809.0149. On the T = 50 ms gravimeter (§III B), the off-resonant
//! Raman pairs shift the interferometer phase by eq. (7), which the counter-propagating pair alone
//! reduces to eq. (8): about 22 mrad at a Rabi frequency of 40 kHz, plus 11 mrad from the
//! co-propagating transitions, 33 mrad in all. The phase is odd in the wave vector and linear in
//! the Rabi frequency at constant pulse area, so the paper alternates four configurations, the wave
//! vector up and down at the Rabi frequencies Ω and Ω′, and plots the phase difference against
//! Ω′/Ω (Fig. 7). A linear fit gives 32 mrad; measurements at one ratio disagree by up to ±10 %,
//! which the paper traces to the Raman beams' polarisation.
//!
//! The verification checks, against the paper:
//!
//! * eq. (8) from the printed Doppler shifts, and the gravity shift the 33 mrad corresponds to;
//! * that QCL's planner needs both the wave-vector reversal and the Rabi change to separate the
//!   light shift from a shift independent of the wave vector, such as quadratic Zeeman, and from
//!   one odd in the wave vector but independent of the Rabi frequency;
//! * that Fig. 7's points with their error bars alone reject every candidate, and with the printed
//!   ±10 % added leave eq. (7) as the survivor over eq. (8) alone and quadratic Zeeman; the same
//!   holds with the pooled scatter of the repeated ratios in place of ±10 %;
//! * the fit's 32 mrad, and the spread of the repeated measurements against the printed ±10 %.
//!
//! Every number used depends on the Doppler shifts' magnitudes only, so the sign convention on
//! which eq. (2) and the text differ does not enter. Fig. 7 is read from the PDF's vector paths by
//! `papers/digitised/extract_gauguet2008_fig7.py`.

#[path = "common/attribution.rs"]
mod attribution;
#[path = "common/data.rs"]
mod data;
#[path = "common/fringe_qubit.rs"]
mod fringe_qubit;
#[path = "common/report.rs"]
mod report;

use attribution::{Attribution, Measurement, attribute, describe};
use data::{load_csv, manifest_dir};
use deep_causality_haft::Either;
use deep_causality_quantum::{Ambiguity, Experiment, MinCostCover, design, separation_bits};
use report::Report;
use std::error::Error;

/// The agreement, in standard errors, and the floor, in bits.
const SIGMAS: f64 = 3.0;
const FLOOR_BITS: f64 = 5.0;
/// Half a unit of the last printed digit of a prediction printed without an uncertainty, in mrad.
const PRINTED_ROUNDING: f64 = 0.5;
/// The contrast of the planner's candidates; the planner compares them on one fringe.
const CONTRAST: f64 = 0.5;

/// §III B: the Rabi frequency and the Doppler shifts at the first and last pulses as printed, in
/// Hz; the first pulse's delay after release and the time between pulses, in s.
const RABI: f64 = 40.0e3;
const DOPPLER_PRINTED: (f64, f64) = (400.0e3, 3.0e6);
const FIRST_PULSE: f64 = 0.017;
const T: f64 = 0.050;
const GRAVITY: f64 = 9.81;
const WAVELENGTH: f64 = 780.241e-9;
/// §III B: the counter- and co-propagating light shifts, their sum, and the fit of Fig. 7, in mrad;
/// the gravity shift the sum corresponds to, in g.
const COUNTER: f64 = 22.0;
const CO: f64 = 11.0;
const EXPECTED: f64 = 33.0;
const FITTED: f64 = 32.0;
const GRAVITY_SHIFT: f64 = 8.0e-8;
/// The printed disagreement between measurements at one ratio.
const REPRODUCIBILITY: f64 = 0.10;

/// A point of Fig. 7: the ratio Ω′/Ω, the phase difference and its error bar, in mrad.
type Point = (f64, f64, f64);

fn main() -> Result<(), Box<dyn Error>> {
    println!("V5 Gauguet et al. 2008: the two-photon light shift on the T = 50 ms gravimeter");
    let mut report = Report::default();

    // Eq. (8), in rad, from the printed Doppler shifts and from the pulses' timing.
    let eq8 = |first: f64, last: f64| RABI / (4.0 * first) - RABI / (4.0 * last);
    let doppler = |t: f64| 2.0 * GRAVITY * t / WAVELENGTH;
    let printed = eq8(DOPPLER_PRINTED.0, DOPPLER_PRINTED.1) * 1e3;
    let (first, last) = (doppler(FIRST_PULSE), doppler(FIRST_PULSE + 2.0 * T));
    let timed = eq8(first, last) * 1e3;
    report.check(
        "eq. (8) gives about 22 mrad for the counter-propagating transition",
        (printed - COUNTER).abs() < PRINTED_ROUNDING,
        format!(
            "{printed:.1} mrad from the printed 400 kHz and 3 MHz; the pulses' timing gives \
             Doppler shifts of {:.1} kHz and {:.2} MHz and {timed:.1} mrad",
            first * 1e-3,
            last * 1e-6
        ),
    );
    let k_eff = 4.0 * std::f64::consts::PI / WAVELENGTH;
    let shift = EXPECTED * 1e-3 / (k_eff * T * T) / GRAVITY;
    report.check(
        "33 mrad corresponds to about 8·10⁻⁸ g",
        (shift / GRAVITY_SHIFT - 1.0).abs() < 0.5 / 8.0,
        format!("33 mrad / (k_eff T²) = {shift:.2e} g"),
    );

    planner(&mut report)?;

    let points: Vec<Point> =
        load_csv(&manifest_dir().join("papers/digitised/gauguet2008_fig7.csv"))?
            .into_iter()
            .map(|row| (row[0], row[1], row[2]))
            .collect();
    let names = [
        "two-photon light shift, eq. (7)",
        "counter-propagating light shift alone, eq. (8)",
        "quadratic Zeeman",
    ];
    let bars = attribute_points(&names, &points, |_| 0.0)?;
    report.check(
        "with Fig. 7's error bars alone, no candidate holds",
        matches!(bars.outcome, Either::Right(Ambiguity::NoSurvivor { .. })),
        format!("QCL: {}", describe(&bars.outcome)),
    );

    let ten_percent = attribute_points(&names, &points, |value| REPRODUCIBILITY * value.abs())?;
    report.check(
        "with the printed ±10 % added, the light shift of eq. (7) survives",
        matches!(&ten_percent.outcome, Either::Left(s) if s.name == names[0]),
        format!(
            "QCL: {}; holding {:?}",
            describe(&ten_percent.outcome),
            ten_percent.holding
        ),
    );

    let scatter = pooled_scatter(&points);
    let pooled = attribute_points(&names, &points, |_| scatter)?;
    report.check(
        "with the repeated ratios' pooled scatter instead, eq. (7) still survives",
        matches!(&pooled.outcome, Either::Left(s) if s.name == names[0]),
        format!(
            "pooled scatter {scatter:.2} mrad; QCL: {}",
            describe(&pooled.outcome)
        ),
    );

    // The linear fit, unweighted, through the points and the reference at ratio 1.
    let slope = least_squares_slope(&points);
    report.check(
        "the linear fit of Fig. 7 gives 32 mrad, against 33 expected",
        (slope - FITTED).abs() < PRINTED_ROUNDING
            && (EXPECTED - slope).abs() < REPRODUCIBILITY * EXPECTED,
        format!(
            "slope {slope:.2} mrad per unit ratio; 22 + 11 = {}",
            COUNTER + CO
        ),
    );

    let spreads = spreads(&points);
    let widest = spreads
        .iter()
        .fold((0.0, 0.0), |a, &b| if b.1 > a.1 { b } else { a });
    report.check(
        "the measurements at ratio ½ disagree by more than the printed ±10 %",
        widest.1 > REPRODUCIBILITY && (widest.0 - 0.5f64).abs() < 0.01,
        format!(
            "half the range over the mean, per repeated ratio: {}",
            spreads
                .iter()
                .map(|(ratio, spread)| format!("{ratio:.3}: ±{:.1} %", spread * 100.0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );

    report.finish("V5").map_err(Into::into)
}

/// The attribution of Fig. 7's points among `names`: eq. (7), eq. (8) and quadratic Zeeman. Each
/// point's standard error adds its error bar, `extra(value)` and the predictions' rounding in
/// quadrature.
fn attribute_points<F: Fn(f64) -> f64>(
    names: &[&str],
    points: &[Point],
    extra: F,
) -> Result<Attribution, Box<dyn Error>> {
    let measurements: Vec<Measurement> = points
        .iter()
        .map(|&(ratio, value, bar)| Measurement {
            name: format!("Ω′/Ω = {ratio:.3}"),
            value,
            standard_error: bar
                .hypot(extra(value))
                .hypot(PRINTED_ROUNDING * (1.0 - ratio)),
            predictions: vec![-EXPECTED * (1.0 - ratio), -COUNTER * (1.0 - ratio), 0.0],
        })
        .collect();
    attribute(names, &measurements, SIGMAS, FLOOR_BITS)
}

/// The planner over the light shift, a shift odd in the wave vector but independent of the Rabi
/// frequency, and quadratic Zeeman, each 33 mrad at the nominal setting.
fn planner(report: &mut Report) -> Result<(), Box<dyn Error>> {
    let read = |mrad: f64| 0.5 * (1.0 + CONTRAST * (mrad * 1e-3).sin());
    let settings = [
        ("k reversed", [-EXPECTED, -EXPECTED, EXPECTED]),
        (
            "Rabi halved at constant pulse area",
            [EXPECTED / 2.0, EXPECTED, EXPECTED],
        ),
    ];
    let rabi_bits = separation_bits(read(settings[1].1[0]), read(settings[1].1[1]), 1);
    let draws = (FLOOR_BITS / rabi_bits).ceil() as u64;
    let probes = settings
        .iter()
        .map(|(name, p)| Experiment::new(*name, 1.0, draws, p.iter().map(|&m| read(m)).collect()))
        .collect::<Result<Vec<_>, _>>()?;
    let plan = design(3, &probes, MinCostCover::new(FLOOR_BITS))?;
    let chosen: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    report.check(
        "the cover needs both the wave-vector reversal and the Rabi change: the four configurations",
        chosen.len() == 2 && plan.is_complete(),
        format!(
            "plan {chosen:?} at {draws} draws each; resolved {:?}",
            plan.entries().iter().map(|e| &e.resolves).collect::<Vec<_>>()
        ),
    );
    Ok(())
}

/// The slope of the unweighted least-squares line through `points` and the reference `(1, 0)`.
fn least_squares_slope(points: &[Point]) -> f64 {
    let xy: Vec<(f64, f64)> = points
        .iter()
        .map(|p| (p.0, p.1))
        .chain([(1.0, 0.0)])
        .collect();
    let n = xy.len() as f64;
    let (mx, my) = (
        xy.iter().map(|p| p.0).sum::<f64>() / n,
        xy.iter().map(|p| p.1).sum::<f64>() / n,
    );
    xy.iter().map(|(x, y)| (x - mx) * (y - my)).sum::<f64>()
        / xy.iter().map(|(x, _)| (x - mx).powi(2)).sum::<f64>()
}

/// The points grouped by ratio, each group's values.
fn groups(points: &[Point]) -> Vec<(f64, Vec<f64>)> {
    let mut groups: Vec<(f64, Vec<f64>)> = Vec::new();
    for &(ratio, value, _) in points {
        match groups.iter_mut().find(|(r, _)| (r - ratio).abs() < 1e-3) {
            Some((_, values)) => values.push(value),
            None => groups.push((ratio, vec![value])),
        }
    }
    groups
}

/// Per repeated ratio, half the range of its values over their mean's magnitude.
fn spreads(points: &[Point]) -> Vec<(f64, f64)> {
    groups(points)
        .into_iter()
        .filter(|(_, values)| values.len() > 1)
        .map(|(ratio, values)| {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let (lo, hi) = values
                .iter()
                .fold((f64::MAX, f64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
            (ratio, (hi - lo) / 2.0 / mean.abs())
        })
        .collect()
}

/// The pooled standard deviation of the repeated ratios' values about their group means.
fn pooled_scatter(points: &[Point]) -> f64 {
    let (squares, freedom) = groups(points)
        .iter()
        .fold((0.0, 0usize), |(s, f), (_, values)| {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            (
                s + values.iter().map(|v| (v - mean).powi(2)).sum::<f64>(),
                f + values.len() - 1,
            )
        });
    (squares / freedom as f64).sqrt()
}
