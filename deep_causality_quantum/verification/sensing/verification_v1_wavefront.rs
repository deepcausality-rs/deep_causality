/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V1: the wavefront bias of a gravimeter, from a temperature scan down to 50 nK.
//!
//! Karcher et al., *Improving the accuracy of atom interferometers with ultracold sources*, New J.
//! Phys. 20, 113041 (2018), arXiv:1804.04909. The gravimeter (T = 80 ms) measures g(T) − g(1.8 µK)
//! from 50 nK to 7 µK (Fig. 2), averaging the wave vector up and down, which removes the
//! one-photon light shift; it repeats the scan turned by 180°, with the same behaviour, which rules
//! out Coriolis; and it changes the atom number fivefold at 650 nK, which bounds interactions.
//! A Monte Carlo model gives each Zernike polynomial's shift against temperature (Fig. 4); fits
//! of the scan with increasing numbers of polynomials extrapolate to zero temperature (Table I),
//! −56(13) nm/s² with five.
//!
//! The verification checks, against the paper:
//!
//! * the defocus study's −63 nm/s² at 1.8 µK and the 3 nm requirement it implies;
//! * Table I's extrapolations, refitted from the digitised Figs. 2 and 4, and two inconsistencies
//!   between the figures: Fig. 4 computes each of Fig. 2's temperatures at 1.5 times its value,
//!   and the drawn fit of Fig. 2 fits the points better than any combination of Fig. 4's
//!   responses at those temperatures can;
//! * that QCL attributes the scan to the wavefront, rejecting a wavefront linear in temperature
//!   fitted above 2 µK, Coriolis of a cloud centred on the detection, and the one-photon light shift
//!   under the k average; and that the linear extrapolation misses Table I. The Coriolis shift of a
//!   centred cloud does not depend on its temperature, and the k average removes the light shift,
//!   so both predict a flat scan;
//! * that the atom-number change bounds interactions at one standard error, and at three leaves them
//!   as a possible cause of the rise at 650 nK;
//! * that QCL's planner needs the turn, the atom-number change and a Rabi change at constant pulse
//!   area to separate the candidates the scan cannot, and that a light shift with the wavefront's
//!   response to temperature, which no experiment of the paper separates, leaves an ambiguity.
//!
//! Figs. 2 and 4 are read from the PDF's vector paths by
//! `papers/digitised/extract_karcher2018_figs.py`.

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
use deep_causality_haft::Either;
use deep_causality_linear::{DenseMatrix, MatrixView, inverse};
use deep_causality_quantum::{Experiment, MinCostCover, design, separation_bits};
use report::Report;
use std::error::Error;

/// The agreement, in standard errors, and the floor, in bits.
const SIGMAS: f64 = 3.0;
const FLOOR_BITS: f64 = 5.0;
/// The contrast of the planner's candidates; the planner compares them on one fringe.
const CONTRAST: f64 = 0.5;
/// The rubidium D2 wavelength, in nm, and the time between pulses, in s.
const WAVELENGTH_NM: f64 = 780.241;
const T: f64 = 0.080;

/// The Zernike orders Fig. 4 plots, in the order Table I adds them.
const ORDERS: [f64; 5] = [2.0, 4.0, 6.0, 8.0, 10.0];
/// Table I: the number of polynomials, the extrapolation to zero temperature and its uncertainty,
/// in nm/s², and the correlation coefficient. The rows with six and seven polynomials need orders
/// Fig. 4 does not plot.
const TABLE_I: [(usize, f64, f64, f64); 4] = [
    (2, 37.0, 17.0, 0.587),
    (3, 9.0, 18.0, 0.764),
    (4, -19.0, 18.0, 0.871),
    (5, -56.0, 13.0, 0.963),
];
/// The defocus study: the shift of a 20 nm peak-to-peak defocus at 1.8 µK, in nm/s², and the
/// peak-to-peak, in nm, that keeps the bias below 10 nm/s².
const DEFOCUS_PV: f64 = 20.0;
const DEFOCUS_SHIFT: f64 = -63.0;
const DEFOCUS_TEMPERATURE: f64 = 1.8;
const REQUIRED_PV: f64 = 3.0;
const BIAS_LIMIT: f64 = 10.0;
/// The atom-number change at 650 nK: the atom numbers, the shift and its standard error, in nm/s²,
/// and the printed bound, in nm/s² per thousand atoms.
const ATOMS: (f64, f64) = (25_000.0, 5_000.0);
const ATOM_SHIFT: (f64, f64) = (-7.0, 12.0);
const ATOM_TEMPERATURE: f64 = 0.65;
const BOUND: f64 = 1.0;
/// The temperature above which the linear wavefront is fitted, in µK.
const LINEAR_ABOVE: f64 = 2.0;
/// The ratio of Fig. 4's temperatures to Fig. 2's, and the separation, in µK, below which two of
/// Fig. 4's markers share a temperature.
const TEMPERATURE_SCALE: f64 = 1.5;
const COINCIDENT: f64 = 0.002;

/// A point of Fig. 2: temperature, shift, standard error, the drawn fit and half its band.
struct Point {
    temperature: f64,
    shift: f64,
    error: f64,
    fit: f64,
    band: f64,
}

/// A weighted least-squares fit: the coefficients, their covariance and the χ².
struct Fit {
    coefficients: Vec<f64>,
    covariance: DenseMatrix<f64>,
    chi_squared: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("V1 Karcher et al. 2018: the wavefront bias from a temperature scan to 50 nK");
    let mut report = Report::default();
    let dir = manifest_dir().join("papers/digitised");
    let rows = load_csv(&dir.join("karcher2018_fig2.csv"))?;
    let all: Vec<Point> = rows
        .iter()
        .map(|r| Point {
            temperature: r[0],
            shift: r[1],
            error: r[2],
            fit: r[3],
            band: r[4],
        })
        .collect();
    let reference = all
        .iter()
        .find(|p| p.error == 0.0)
        .ok_or("Fig. 2 has a reference")?;
    let points: Vec<&Point> = all.iter().filter(|p| p.error > 0.0).collect();
    let fig4 = load_csv(&dir.join("karcher2018_fig4.csv"))?;
    let responses: Vec<Vec<(f64, f64)>> = ORDERS
        .iter()
        .map(|&n| {
            let series: Vec<(f64, f64)> = fig4
                .iter()
                .filter(|r| r[0] == n)
                .map(|r| (r[1], r[2]))
                .collect();
            if series.is_empty() {
                Err(format!("Fig. 4 has no markers for order {n}"))
            } else {
                Ok(series)
            }
        })
        .collect::<Result<_, _>>()?;
    // The marker nearest `temperature`; where Fig. 4 has two markers at one temperature, their mean.
    // Every series holds a marker, so `at` holds the nearest one at least.
    let response = |order: usize, temperature: f64| -> f64 {
        let series = &responses[order];
        let nearest = nearest_marker(series, temperature).0;
        let at: Vec<f64> = series
            .iter()
            .filter(|m| (m.0 - nearest).abs() < COINCIDENT)
            .map(|m| m.1)
            .collect();
        at.iter().sum::<f64>() / at.len() as f64
    };

    // The defocus study, read from Fig. 4 at its own 1.8 µK.
    let defocus = response(0, DEFOCUS_TEMPERATURE);
    report.check(
        "a 20 nm defocus shifts g by −63 nm/s² at 1.8 µK, so 3 nm (λ/260) keeps it below 10",
        (defocus - DEFOCUS_SHIFT).abs() < 0.5
            && DEFOCUS_SHIFT.abs() * REQUIRED_PV / DEFOCUS_PV < BIAS_LIMIT
            && (WAVELENGTH_NM / 260.0 - REQUIRED_PV).abs() < 0.05,
        format!(
            "Fig. 4, n = 2 at 1.8 µK: {defocus:.1} nm/s²; 3 nm gives {:.2} nm/s²; λ/260 = {:.2} nm",
            DEFOCUS_SHIFT.abs() * REQUIRED_PV / DEFOCUS_PV,
            WAVELENGTH_NM / 260.0
        ),
    );

    // Fig. 4's temperature grid against Fig. 2's.
    let ratios: Vec<f64> = all
        .iter()
        .map(|p| nearest_marker(&responses[1], TEMPERATURE_SCALE * p.temperature).0 / p.temperature)
        .collect();
    let close = |ratio: f64, tolerance: f64| (ratio / TEMPERATURE_SCALE - 1.0).abs() < tolerance;
    let within = |tolerance: f64| ratios.iter().filter(|&&r| close(r, tolerance)).count();
    let lowest = all
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.temperature.total_cmp(&b.1.temperature))
        .map(|(i, _)| i)
        .ok_or("Fig. 2 has points")?;
    report.check(
        "Fig. 4 computes each of Fig. 2's temperatures above the lowest at 1.5 times its value",
        ratios
            .iter()
            .enumerate()
            .all(|(i, &r)| i == lowest || close(r, 0.015)),
        format!(
            "{} of {} within 0.6 % of 1.5, {} within 1.5 %, the lowest at {:.3}; at the reference, \
             1.76 µK, Fig. 4 reads {:.1} nm/s² for the defocus at 2.64 µK, against {defocus:.1} at \
             its 1.8 µK",
            within(0.006),
            ratios.len(),
            within(0.015),
            ratios[lowest],
            response(0, TEMPERATURE_SCALE * reference.temperature)
        ),
    );

    // Table I, refitted: each point against the responses at Fig. 4's temperature for it.
    let basis = |count: usize, temperature: f64| -> Vec<f64> {
        (0..count)
            .map(|k| {
                response(k, TEMPERATURE_SCALE * temperature)
                    - response(k, TEMPERATURE_SCALE * reference.temperature)
            })
            .collect()
    };
    let shifts: Vec<f64> = points.iter().map(|p| p.shift).collect();
    let errors: Vec<f64> = points.iter().map(|p| p.error).collect();
    let mut rows_out = Vec::new();
    let mut agree = true;
    let mut five = None;
    for &(count, printed, printed_u, printed_r) in &TABLE_I {
        let design_matrix: Vec<Vec<f64>> =
            points.iter().map(|p| basis(count, p.temperature)).collect();
        let fit = weighted_fit(&design_matrix, &shifts, &errors)?;
        let at_zero: Vec<f64> = (0..count)
            .map(|k| -response(k, TEMPERATURE_SCALE * reference.temperature))
            .collect();
        let extrapolation = dot(&at_zero, &fit.coefficients);
        let scale = fit.chi_squared / (points.len() - count) as f64;
        let u = (quadratic_form(&fit.covariance, &at_zero)? * scale).sqrt();
        let fitted: Vec<f64> = design_matrix
            .iter()
            .map(|row| dot(row, &fit.coefficients))
            .collect();
        agree &= (extrapolation - printed).abs() < u.hypot(printed_u);
        rows_out.push(format!(
            "{count}: {extrapolation:.1}({u:.1}) against {printed}({printed_u}), R {:.2} against \
             {printed_r}",
            pearson(&shifts, &fitted)
        ));
        if count == 5 {
            five = Some(fit.chi_squared);
        }
    }
    report.check(
        "the refit reproduces Table I's extrapolations within one standard error",
        agree,
        format!("{}; R does not reproduce", rows_out.join("; ")),
    );

    let drawn: f64 = points
        .iter()
        .map(|p| ((p.shift - p.fit) / p.error).powi(2))
        .sum();
    let best = five.ok_or("Table I has a row with five polynomials")?;
    report.check(
        "the drawn fit fits better than any combination of Fig. 4's five responses",
        drawn < best,
        format!("χ² of the drawn fit {drawn:.1}; the least χ² over the five responses {best:.1}"),
    );

    // The scan: the wavefront against the candidates it rules out.
    let (slope, slope_u) = linear_above(&points, reference.temperature);
    let names = [
        "wavefront, five Zernike polynomials",
        "wavefront linear in temperature above 2 µK",
        "Coriolis, cloud centred on the detection",
        "one-photon light shift under the k average",
    ];
    let measurements: Vec<Measurement> = points
        .iter()
        .map(|p| {
            let offset = p.temperature - reference.temperature;
            Measurement {
                name: format!("{:.3} µK", p.temperature),
                value: p.shift,
                standard_error: p.error.hypot(p.band.max(slope_u * offset.abs())),
                predictions: vec![p.fit, slope * offset, 0.0, 0.0],
            }
        })
        .collect();
    let scan = attribute(&names, &measurements, SIGMAS, FLOOR_BITS)?;
    report.check(
        "the scan attributes to the wavefront alone",
        matches!(&scan.outcome, Either::Left(s) if s.name == names[0]),
        format!(
            "QCL: {}; holding {:?}",
            describe(&scan.outcome),
            scan.holding
        ),
    );

    let linear = -slope * reference.temperature;
    let linear_u = slope_u * reference.temperature;
    let (printed, printed_u) = (TABLE_I[3].1, TABLE_I[3].2);
    let miss = (linear - printed).abs() / linear_u.hypot(printed_u);
    report.check(
        "the linear extrapolation from above 2 µK misses Table I's −56(13)",
        miss > SIGMAS,
        format!(
            "slope {slope:.2} ± {slope_u:.2} nm/s²/µK extrapolates to {linear:.1} ± {linear_u:.1}, \
             {miss:.1} standard errors from −56(13)"
        ),
    );

    // The atom-number change, against interactions carrying the rise at 650 nK. If the rise were
    // interactions, at the same atom number at 650 nK and at the reference, the shift at 650 nK
    // would be at least the rise, and four fifths of it would go with four fifths of the atoms.
    let per_thousand = (ATOM_SHIFT.0.abs() + ATOM_SHIFT.1) / ((ATOMS.0 - ATOMS.1) / 1e3);
    let (rise, rise_u) = interpolate(&all, ATOM_TEMPERATURE);
    let fraction = 1.0 - ATOMS.1 / ATOMS.0;
    let atoms = attribute(
        &[
            "no interactions",
            "interactions carrying the rise at 650 nK",
        ],
        &[Measurement {
            name: "atom number 25,000 to 5,000".into(),
            value: ATOM_SHIFT.0,
            standard_error: ATOM_SHIFT.1.hypot(fraction * rise_u),
            predictions: vec![0.0, -fraction * rise],
        }],
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "the bound on interactions is one standard error; at three, they stay possible",
        per_thousand < BOUND && atoms.holding.len() == 2,
        format!(
            "(|−7| + 12)/20 = {per_thousand:.2} nm/s² per thousand atoms; interactions carrying the \
             {rise:.1} nm/s² rise predict {:.1}; QCL: {}",
            -fraction * rise,
            describe(&atoms.outcome)
        ),
    );

    planner(&mut report, TABLE_I[3].1.abs())?;

    let rival = attribute(
        &[
            names[0],
            "two-photon light shift, the same response to temperature",
        ],
        &points
            .iter()
            .map(|p| Measurement {
                name: format!("{:.3} µK", p.temperature),
                value: p.shift,
                standard_error: p.error.hypot(p.band),
                predictions: vec![p.fit, p.fit],
            })
            .collect::<Vec<_>>(),
        SIGMAS,
        FLOOR_BITS,
    )?;
    report.check(
        "a light shift with the wavefront's response to temperature leaves an ambiguity",
        matches!(rival.outcome, Either::Right(_)) && rival.holding.len() == 2,
        format!("QCL: {}", describe(&rival.outcome)),
    );

    report.finish("V1").map_err(Into::into)
}

/// The planner over the candidates the scan cannot separate, each odd in the wave vector, as it
/// must be to survive the k average, and each shifting g by `shift` nm/s² at the nominal setting.
fn planner(report: &mut Report, shift: f64) -> Result<(), Box<dyn Error>> {
    let k_eff_t2 = 4.0 * std::f64::consts::PI / (WAVELENGTH_NM * 1e-9) * T * T * 1e-9;
    let read = |s: f64| 0.5 * (1.0 + CONTRAST * (k_eff_t2 * s).sin());
    // Wavefront, Coriolis with a temperature-dependent detected velocity, interactions, and the
    // two-photon light shift.
    let settings = [
        ("k reversed", [-shift, -shift, -shift, -shift]),
        ("turned 180°", [shift, -shift, shift, shift]),
        (
            "atom number divided by five",
            [shift, shift, shift / 5.0, shift],
        ),
        (
            "Rabi frequency halved at constant pulse area",
            [shift, shift, shift, shift / 2.0],
        ),
    ];
    let weakest = separation_bits(read(shift), read(shift / 2.0), 1);
    if !weakest.is_finite() || weakest <= 0.0 {
        return Err("a shift and its half do not separate at any draw count".into());
    }
    let draws = (FLOOR_BITS / weakest).ceil() as u64;
    let probes = settings
        .iter()
        .map(|(name, p)| Experiment::new(*name, 1.0, draws, p.iter().map(|&s| read(s)).collect()))
        .collect::<Result<Vec<_>, _>>()?;
    let plan = design(4, &probes, MinCostCover::new(FLOOR_BITS))?;
    let chosen: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    report.check(
        "the cover needs the turn, the atom-number change and a Rabi change; k reversal separates none",
        plan.is_complete()
            && chosen
                == [
                    "turned 180°",
                    "atom number divided by five",
                    "Rabi frequency halved at constant pulse area",
                ],
        format!("plan {chosen:?} at {draws} draws each; the paper ran the first two"),
    );
    Ok(())
}

/// The weighted least-squares fit of `values` with standard errors `errors` on the rows of
/// `design`.
fn weighted_fit(
    design: &[Vec<f64>],
    values: &[f64],
    errors: &[f64],
) -> Result<Fit, Box<dyn Error>> {
    let k = design.first().map_or(0, Vec::len);
    let weights: Vec<f64> = errors.iter().map(|e| e.powi(-2)).collect();
    let normal: Vec<f64> = (0..k * k)
        .map(|ij| {
            let (i, j) = (ij / k, ij % k);
            design
                .iter()
                .zip(&weights)
                .map(|(row, w)| w * row[i] * row[j])
                .sum()
        })
        .collect();
    let rhs: Vec<f64> = (0..k)
        .map(|i| {
            design
                .iter()
                .zip(values)
                .zip(&weights)
                .map(|((row, y), w)| w * row[i] * y)
                .sum()
        })
        .collect();
    let covariance = inverse(&DenseMatrix::from_vec(normal, k, k)?)?;
    let coefficients = (0..k)
        .map(|i| {
            (0..k)
                .map(|j| covariance.get(i, j).map(|c| c * rhs[j]))
                .sum::<Result<f64, _>>()
        })
        .collect::<Result<Vec<f64>, _>>()?;
    let chi_squared = design
        .iter()
        .zip(values)
        .zip(&weights)
        .map(|((row, y), w)| w * (y - dot(row, &coefficients)).powi(2))
        .sum();
    Ok(Fit {
        coefficients,
        covariance,
        chi_squared,
    })
}

/// The weighted slope through the reference of the points above [`LINEAR_ABOVE`], and its
/// standard error.
fn linear_above(points: &[&Point], reference: f64) -> (f64, f64) {
    let (sxy, sxx) =
        points
            .iter()
            .filter(|p| p.temperature > LINEAR_ABOVE)
            .fold((0.0, 0.0), |(sxy, sxx), p| {
                let (x, w) = (p.temperature - reference, p.error.powi(-2));
                (sxy + w * x * p.shift, sxx + w * x * x)
            });
    (sxy / sxx, sxx.powf(-0.5))
}

/// The drawn fit and half its band at `temperature`, interpolated in the logarithm of temperature
/// between the fit's vertices.
fn interpolate(points: &[Point], temperature: f64) -> (f64, f64) {
    let i = points
        .partition_point(|p| p.temperature <= temperature)
        .clamp(1, points.len() - 1);
    let (a, b) = (&points[i - 1], &points[i]);
    let t = (temperature.ln() - a.temperature.ln()) / (b.temperature.ln() - a.temperature.ln());
    (a.fit + t * (b.fit - a.fit), a.band + t * (b.band - a.band))
}

/// The marker of the non-empty `series` nearest `temperature`; of two equally near, the first.
fn nearest_marker(series: &[(f64, f64)], temperature: f64) -> (f64, f64) {
    series.iter().copied().fold(series[0], |best, m| {
        if (m.0 - temperature).abs() < (best.0 - temperature).abs() {
            m
        } else {
            best
        }
    })
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// `vᵀ M v`.
fn quadratic_form(m: &DenseMatrix<f64>, v: &[f64]) -> Result<f64, Box<dyn Error>> {
    let mut sum = 0.0;
    for (i, vi) in v.iter().enumerate() {
        for (j, vj) in v.iter().enumerate() {
            sum += vi * m.get(i, j)? * vj;
        }
    }
    Ok(sum)
}

/// The correlation coefficient of `a` and `b`.
fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let (va, vb): (f64, f64) = (
        a.iter().map(|x| (x - ma).powi(2)).sum(),
        b.iter().map(|y| (y - mb).powi(2)).sum(),
    );
    cov / (va * vb).sqrt()
}
