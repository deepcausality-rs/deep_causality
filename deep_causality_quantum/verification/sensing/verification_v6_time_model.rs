/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V6: the instrument-time model against a transportable gravimeter's long-term data.
//!
//! Ménoret et al., *Gravity measurements below 10⁻⁹ g with a transportable absolute quantum
//! gravimeter*, Sci. Rep. 8, 12300 (2018), arXiv:1809.04908. The AQG runs at T = 60 ms, contrast
//! 40 % and about 2 Hz. In Talence, at about 600 nm/s²/√Hz over five days, 10-minute averages
//! scatter by 25.2 nm/s² and 1-hour averages by 10.7; in Larzac, at 750 nm/s²/√Hz over a month,
//! 1-day averages scatter by 9.4. The Allan deviation is white between 100 and 2000 s. A tilt scan
//! from 0 to 1.5 mrad moves gravity by about 10 µm/s².
//!
//! QCL sizes integration on `σ(τ) = S/√τ` through effective draws, and refuses to size averaging
//! the white-noise range does not hold. The verification checks the scatter the time model predicts,
//! the planner's sizing on both sides of the range, and the tilt response on the mechanism path.
//! A scatter is compared within two standard errors of a standard deviation estimated from `N`
//! averages, `σ/√(2(N − 1))`, the paper giving none of its own.

#[path = "common/fringe_qubit.rs"]
mod fringe_qubit;
#[path = "common/inverse.rs"]
mod inverse;
#[path = "common/report.rs"]
mod report;

use deep_causality_context_store::ContextSnapshot;
use deep_causality_quantum::{
    ConfiguredExperiment, Experiment, Hypothesis, InterferometerModel, MinCostCover,
    ObservedContext, QuantumError, Response, ResponseModel, ShotEstimate, design,
};
use fringe_qubit::{mechanism, output_port, phase_channel, plant};
use inverse::phase_of;
use report::Report;
use std::error::Error;

/// The rubidium D2 wavelength, in m.
const WAVELENGTH: f64 = 780.241e-9;
/// The interrogation time, in s, and the contrast: "τ = 10 µs, T = 60 ms, C = 40%".
const T: f64 = 0.060;
const CONTRAST: f64 = 0.40;
/// The cycle, in s: "a 2 Hz repetition rate".
const CYCLE: f64 = 0.5;
/// The white-noise range, in s: "white-noise signature between 100 and 2000 s" (Methods).
const WHITE_NOISE_RANGE: f64 = 2000.0;
/// The sensitivities, in nm/s²/√Hz: Talence, and Larzac during the 2017 campaign.
const S_TALENCE: f64 = 600.0;
const S_LARZAC: f64 = 750.0;
/// The scatters of the averages, in nm/s², and the averages behind them: five days of 10-minute
/// and 1-hour averages in Talence, a month of 1-day averages in Larzac.
const SCATTER_10_MIN: f64 = 25.2;
const SCATTER_1_HOUR: f64 = 10.7;
const SCATTER_1_DAY: f64 = 9.4;
const AVERAGES_10_MIN: f64 = 5.0 * 144.0;
const AVERAGES_1_HOUR: f64 = 5.0 * 24.0;
/// The end of the tilt scan, in rad, and the gravity change the paper gives for it, in m/s².
const TILT_SCAN_END: f64 = 1.5e-3;
const TILT_RESPONSE: f64 = 10.0e-6;
/// Standard gravity, in m/s².
const G: f64 = 9.80665;
/// The floor, in bits.
const FLOOR_BITS: f64 = 5.0;
/// One nm/s², in m/s².
const NANO: f64 = 1.0e-9;

fn main() -> Result<(), Box<dyn Error>> {
    println!("V6 Ménoret et al. 2018: the instrument-time model against the AQG's long-term data");
    let mut report = Report::default();
    let k_eff = 4.0 * std::f64::consts::PI / WAVELENGTH;
    let aqg = |s: f64| {
        InterferometerModel::new(k_eff, T, CONTRAST, s * NANO, CYCLE, WHITE_NOISE_RANGE, 0.0)
    };
    let (talence, larzac) = (aqg(S_TALENCE)?, aqg(S_LARZAC)?);

    // The scatter of τ-second averages: the read-out's standard error over the draws τ holds,
    // carried off the fringe.
    let predicted = |model: &InterferometerModel<f64>, tau: f64| -> Result<f64, QuantumError> {
        let draws = tau / model.instrument_time()?.shot_time();
        let read_out = ShotEstimate::from_effective_draws(0.5, draws)?;
        Ok(read_out.standard_error() / model.fringe().slope() / NANO)
    };
    for (label, tau, measured, averages) in [
        ("10-minute", 600.0, SCATTER_10_MIN, AVERAGES_10_MIN),
        ("1-hour", 3600.0, SCATTER_1_HOUR, AVERAGES_1_HOUR),
    ] {
        let sigma = predicted(&talence, tau)?;
        let spread = measured / (2.0 * (averages - 1.0)).sqrt();
        report.check(
            &format!("the time model predicts the scatter of Talence's {label} averages"),
            (sigma - measured).abs() <= 2.0 * spread,
            format!(
                "predicted {sigma:.1} nm/s², measured {measured} ± {spread:.1} (from {averages} averages)"
            ),
        );
    }
    println!("       the 1-hour averages lie past the 2000 s white-noise range the paper states");

    // The 1-day averages: white noise predicts far less scatter than Larzac shows, and the
    // planner refuses to size averaging past the white-noise range.
    let one_day = predicted(&larzac, 86_400.0)?;
    report.check(
        "white noise does not explain Larzac's 1-day scatter",
        SCATTER_1_DAY > 3.0 * one_day,
        format!("predicted {one_day:.2} nm/s², measured {SCATTER_1_DAY}"),
    );
    let time = larzac.instrument_time()?;
    let needs = |tau: f64| {
        // The gravity difference whose separation reaches the floor after τ seconds:
        // τ = 8 ln 2 · F · S² / Δg².
        S_LARZAC * NANO * (8.0 * std::f64::consts::LN_2 * FLOOR_BITS / tau).sqrt()
    };
    let pair = |delta_g: f64| -> Result<Vec<Experiment<f64>>, QuantumError> {
        let fringe = larzac.fringe();
        let p = |g: f64| fringe.operating_point() + fringe.slope() * g;
        Ok(vec![Experiment::new(
            "averaging",
            0.0,
            1,
            vec![p(0.0), p(delta_g)],
        )?])
    };
    let objective = MinCostCover::new(FLOOR_BITS).timed(time);
    let within = design(2, &pair(needs(600.0))?, objective)?;
    let beyond = design(2, &pair(needs(86_400.0))?, objective)?;
    let sized = within.entries().first().map_or(0.0, |e| e.cost);
    report.check(
        "the planner sizes a separation inside the white-noise range and refuses one past it",
        within.is_complete() && !beyond.is_complete(),
        format!(
            "10 minutes' separation: planned at {sized:.0} s; a day's: uncovered {:?}, the range \
             holding {} draws",
            beyond.uncovered(),
            time.max_shots()
        ),
    );

    // The tilt response on the mechanism path: −g(1 − cos θ), quadratic in θ.
    let tilted = |theta: f64| -> Result<f64, QuantumError> {
        let model = Tilt {
            phase_per_acceleration: k_eff * T * T,
        };
        let e = ConfiguredExperiment::new("tilt", 0.0, 1, Angle(theta), 0)?;
        let p = e.predict(
            &model,
            &mechanism("tilt", CONTRAST)?,
            &plant()?,
            &[output_port()?],
        )?;
        Ok(phase_of(p, CONTRAST) / model.phase_per_acceleration)
    };
    let (end, halfway) = (tilted(TILT_SCAN_END)?, tilted(TILT_SCAN_END / 2.0)?);
    report.check(
        "a 1.5 mrad tilt moves gravity by about 10 µm/s², quadratically",
        (end.abs() - TILT_RESPONSE).abs() <= 0.5 * TILT_RESPONSE
            && (end / halfway - 4.0).abs() < 1e-3,
        format!(
            "{:.2} µm/s² at 1.5 mrad (the paper: about 10, to one significant figure); {:.2} at \
             0.75 mrad, a ratio of {:.4}",
            end * 1e6,
            halfway * 1e6,
            end / halfway
        ),
    );

    report.finish("V6").map_err(Into::into)
}

/// A tilt from vertical, in rad.
#[derive(Debug, Clone, Copy)]
struct Angle(f64);

impl ObservedContext for Angle {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// The tilt's response: the phase of `−g(1 − cos θ)`.
struct Tilt {
    phase_per_acceleration: f64,
}

impl ResponseModel<f64, Angle> for Tilt {
    fn respond(&self, _: &Hypothesis<f64>, angle: &Angle) -> Result<Response<f64>, QuantumError> {
        let offset = -G * (1.0 - angle.0.cos());
        Ok(Response::Channel(phase_channel(
            self.phase_per_acceleration * offset,
        )?))
    }
}
