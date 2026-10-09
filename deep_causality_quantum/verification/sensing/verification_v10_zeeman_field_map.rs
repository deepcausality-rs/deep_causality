/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V10: the quadratic Zeeman bias of GAIN from its measured field maps.
//!
//! Hu et al., *Mapping the absolute magnetic field and evaluating the quadratic Zeeman effect
//! induced systematic error in an atom interferometer gravimeter*, Phys. Rev. A 96, 033414 (2017),
//! arXiv:1805.05159. The paper maps the field along GAIN's fountain at nominal (13 mA) and half
//! (6.5 mA) coil current (Fig. 6), splits it into the coil's field and the background (eqs. 5 to
//! 8), and integrates the quadratic Zeeman phase along the trajectory (eq. 9) to a gravity offset
//! of 2.04 µGal.
//!
//! The maps are digitised by `papers/digitised/digitise_hu2017_fig6.py`, so the digitisation error
//! enters the comparison: the verification moves the height axis by the 0.17 cm its dashed pulse lines
//! disagree with the printed heights, and rescales the field axis by one pixel of the 126 px
//! between its ticks 100 nT apart, the calibration every field value is read through, and takes
//! the quadrature of the moves as the uncertainty. The bias follows the field's variation along the
//! trajectory, which the rescaling stretches by 1/126; a uniform one-pixel shift of the field moves
//! the bias by about 1.4 · 10⁻⁴ of itself, the pixel's ratio to the mean field.
//!
//! The paper prints the quadratic Zeeman coefficient as 2π × 0.0575 Hz/nT²; the rubidium clock
//! transition's 575 Hz/G² is 0.0575 Hz/µT², and the printed unit puts the offset six orders of
//! magnitude off. This is a response-model check, not a discrimination: the mechanism path of QCL
//! must carry the field-map bias, at both currents, through its prediction.

#[path = "common/data.rs"]
mod data;
#[path = "common/fringe_qubit.rs"]
mod fringe_qubit;
#[path = "common/inverse.rs"]
mod inverse;
#[path = "common/report.rs"]
mod report;

use data::{load_csv, manifest_dir};
use deep_causality_context_store::ContextSnapshot;
use deep_causality_quantum::{
    ConfiguredExperiment, Hypothesis, ObservedContext, QuantumError, Response, ResponseModel,
};
use fringe_qubit::{mechanism, output_port, phase_channel, plant};
use inverse::phase_of;
use report::Report;
use std::error::Error;

/// Standard gravity, in m/s².
const G: f64 = 9.80665;
/// The time between pulses, in s (§III D).
const T: f64 = 0.26;
/// The printed pulse heights above the MOT centre, in m, and the apex.
const H_FIRST: f64 = 0.451;
const H_SECOND: f64 = 0.857;
const H_THIRD: f64 = 0.600;
const H_APEX: f64 = 0.861;
/// The quadratic Zeeman coefficient, 0.0575 Hz/µT², in Hz/nT².
const GAMMA: f64 = 0.0575e-6;
/// The rubidium D2 wavelength, in m.
const WAVELENGTH: f64 = 780.241e-9;
/// The printed offset at nominal current, in µGal.
const PRINTED_OFFSET: f64 = 2.04;
/// The printed statistics of the inferred coil field, in nT (Fig. 6(c)).
const PRINTED_COIL_MEAN: f64 = 5424.98;
const PRINTED_COIL_SD: f64 = 6.24;
/// The digitisation's resolution: the height disagreement of the dashed pulse lines, in cm; the
/// field axis's calibration, two ticks `TICK_SPAN_NT` apart and `TICK_SPAN_PX` pixels apart; and
/// one pixel of field, in nT.
const HEIGHT_ERROR_CM: f64 = 0.17;
const TICK_SPAN_NT: f64 = 100.0;
const TICK_SPAN_PX: f64 = 126.0;
const FIELD_PIXEL_NT: f64 = TICK_SPAN_NT / TICK_SPAN_PX;
/// One µGal, in m/s².
const MICRO_GAL: f64 = 1.0e-8;
/// Integration steps over the interferometer's `2T`.
const STEPS: usize = 200_000;

fn main() -> Result<(), Box<dyn Error>> {
    println!("V10 Hu et al. 2017: the quadratic Zeeman bias of GAIN from its field maps");
    let rows = load_csv(&manifest_dir().join("papers/digitised/hu2017_fig6.csv"))?;
    let heights: Vec<f64> = rows.iter().map(|r| r[0]).collect();
    let nominal: Vec<f64> = rows.iter().map(|r| r[1]).collect();
    let half: Vec<f64> = rows.iter().map(|r| r[2]).collect();
    let mut report = Report::default();

    // The trajectory through the first two printed heights, checked against the other two.
    let v0 = ((H_SECOND - H_FIRST) + G * T * T / 2.0) / T;
    let third = height_at(v0, 2.0 * T);
    let apex = H_FIRST + v0 * v0 / (2.0 * G);
    report.check(
        "the fountain through 45.1 cm and 85.7 cm passes the printed third pulse and apex",
        (third - H_THIRD).abs() < 5e-4 && (apex - H_APEX).abs() < 5e-4,
        format!(
            "third pulse {:.2} cm (printed 60.0), apex {:.2} cm (printed 86.1)",
            third * 100.0,
            apex * 100.0
        ),
    );

    let interp = |values: &[f64], shift_cm: f64| {
        let (h, v) = (heights.clone(), values.to_vec());
        move |z: f64| interpolate(&h, &v, z - shift_cm)
    };
    let nominal_field = interp(&nominal, 0.0);
    let half_field = interp(&half, 0.0);
    let at_nominal = bias(v0, &nominal_field, GAMMA);
    let at_half = bias(v0, &half_field, GAMMA);

    report.check(
        "the coefficient works in Hz/µT², not the printed Hz/nT²",
        (bias(v0, &nominal_field, GAMMA * 1e6) / at_nominal - 1e6).abs() < 1.0,
        format!(
            "in the printed unit the nominal map gives {:.3e} µGal against the printed 2.04",
            bias(v0, &nominal_field, GAMMA * 1e6)
        ),
    );

    // Digitisation uncertainty: the height axis moved both ways, and the field axis rescaled by a
    // pixel of its tick span, which stretches each value's distance from the mean by 1/126.
    let mean = nominal.iter().sum::<f64>() / nominal.len() as f64;
    let height_moves = [-HEIGHT_ERROR_CM, HEIGHT_ERROR_CM]
        .map(|dz| (bias(v0, &interp(&nominal, dz), GAMMA) - at_nominal).abs());
    let scaled: Vec<f64> = nominal
        .iter()
        .map(|b| mean + (b - mean) * (1.0 + FIELD_PIXEL_NT / TICK_SPAN_NT))
        .collect();
    let scale_move = (bias(v0, &interp(&scaled, 0.0), GAMMA) - at_nominal).abs();
    let uncertainty = height_moves[0].max(height_moves[1]).hypot(scale_move);
    report.check(
        "the nominal map reproduces the printed offset within the digitisation's uncertainty",
        (at_nominal - PRINTED_OFFSET).abs() <= uncertainty,
        format!(
            "{at_nominal:.3} µGal against the printed {PRINTED_OFFSET} (uncertainty {uncertainty:.3}: \
             height {:.3}, field scale {scale_move:.3})",
            height_moves[0].max(height_moves[1])
        ),
    );

    // The split into the coil's field and the background, eqs. 7 and 8.
    let coil: Vec<f64> = nominal
        .iter()
        .zip(&half)
        .map(|(n, h)| 2.0 * (n - h))
        .collect();
    let background: Vec<f64> = nominal
        .iter()
        .zip(&half)
        .map(|(n, h)| 2.0 * h - n)
        .collect();
    let (coil_mean, coil_sd) = mean_sd(&coil);
    let split_tolerance = 2.0 * 2.0f64.sqrt() * FIELD_PIXEL_NT;
    report.check(
        "the coil's field from eq. 7 has the printed mean and spread (Fig. 6(c))",
        (coil_mean - PRINTED_COIL_MEAN).abs() <= split_tolerance
            && (coil_sd - PRINTED_COIL_SD).abs() <= split_tolerance,
        format!(
            "mean {coil_mean:.2} nT, SD {coil_sd:.2} nT against {PRINTED_COIL_MEAN} and {PRINTED_COIL_SD} \
             (tolerance {split_tolerance:.2}, one pixel on each map)"
        ),
    );

    // The bias at coil current s is a s² + b s + c with B(s) = s B_sn + B_bg.
    let coil_field = interp(&coil, 0.0);
    let background_field = interp(&background, 0.0);
    let a = bias(v0, &coil_field, GAMMA);
    let c = bias(v0, &background_field, GAMMA);
    let b = bias(v0, &|z| coil_field(z) + background_field(z), GAMMA) - a - c;
    report.check(
        "the background's inhomogeneity carries the bias: the cross term dominates",
        b.abs() > 10.0 * (a.abs() + c.abs()),
        format!(
            "a {a:.4}, b {b:.4}, c {c:.4} µGal; shares {:.3}, {:.3}, {:.3}",
            a / (a + b + c),
            b / (a + b + c),
            c / (a + b + c)
        ),
    );

    // The mechanism path: a candidate whose response at current s is the phase of a s² + b s + c.
    let model = Zeeman {
        coefficients: (a, b, c),
        phase_per_offset: phase_per_acceleration() * MICRO_GAL,
    };
    let candidate = mechanism("quadratic Zeeman", CONTRAST)?;
    let (the_plant, observables) = (plant()?, [output_port()?]);
    let carried = |s: f64| -> Result<f64, QuantumError> {
        let e = ConfiguredExperiment::new("field step", 0.0, 1, Current(s), 0)?;
        let p = e.predict(&model, &candidate, &the_plant, &observables)?;
        Ok(phase_of(p, CONTRAST) / model.phase_per_offset)
    };
    let (full, halved) = (carried(1.0)?, carried(0.5)?);
    report.check(
        "QCL's prediction carries the field-map bias at both currents",
        (full - at_nominal).abs() < 1e-9 && (halved - at_half).abs() < 1e-9,
        format!(
            "predicted {full:.6} and {halved:.6} µGal; integrated {at_nominal:.6} and {at_half:.6}"
        ),
    );

    report.finish("V10").map_err(Into::into)
}

/// The fringe contrast of the mechanism; the check inverts the read-out, so any value serves.
const CONTRAST: f64 = 0.5;

/// The interferometer phase per m/s², `k_eff T²`.
fn phase_per_acceleration() -> f64 {
    4.0 * std::f64::consts::PI / WAVELENGTH * T * T
}

/// The height, in m, `tau` s after the first pulse.
fn height_at(v0: f64, tau: f64) -> f64 {
    H_FIRST + v0 * tau - G * tau * tau / 2.0
}

/// The gravity offset, in µGal, of eq. 9 for the field `field` (nT, of height in cm), with the
/// sensitivity function +1 for the first `T` and −1 for the second.
fn bias<F: Fn(f64) -> f64>(v0: f64, field: &F, gamma: f64) -> f64 {
    let dt = 2.0 * T / STEPS as f64;
    let integrand = |i: usize| {
        let tau = i as f64 * dt;
        let sign = if tau < T { 1.0 } else { -1.0 };
        let b = field(height_at(v0, tau) * 100.0);
        sign * b * b
    };
    let integral =
        (1..STEPS).map(integrand).sum::<f64>() * dt + 0.5 * dt * (integrand(0) + integrand(STEPS));
    2.0 * std::f64::consts::PI * gamma * integral / phase_per_acceleration() / MICRO_GAL
}

/// Linear interpolation of `values` over ascending `heights`, held at the ends.
fn interpolate(heights: &[f64], values: &[f64], z: f64) -> f64 {
    let n = heights.len();
    if z <= heights[0] {
        return values[0];
    }
    if z >= heights[n - 1] {
        return values[n - 1];
    }
    let i = heights.partition_point(|&h| h <= z);
    let w = (z - heights[i - 1]) / (heights[i] - heights[i - 1]);
    values[i - 1] + w * (values[i] - values[i - 1])
}

fn mean_sd(values: &[f64]) -> (f64, f64) {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n;
    (mean, var.sqrt())
}

/// The coil current, as a fraction of nominal.
#[derive(Debug, Clone, Copy)]
struct Current(f64);

impl ObservedContext for Current {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// The quadratic Zeeman response: the phase of `a s² + b s + c` µGal at coil current `s`.
struct Zeeman {
    coefficients: (f64, f64, f64),
    phase_per_offset: f64,
}

impl ResponseModel<f64, Current> for Zeeman {
    fn respond(
        &self,
        _: &Hypothesis<f64>,
        current: &Current,
    ) -> Result<Response<f64>, QuantumError> {
        let (a, b, c) = self.coefficients;
        let s = current.0;
        Ok(Response::Channel(phase_channel(
            self.phase_per_offset * (a * s * s + b * s + c),
        )?))
    }
}
