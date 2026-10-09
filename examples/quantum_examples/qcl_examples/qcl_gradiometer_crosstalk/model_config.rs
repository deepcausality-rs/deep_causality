/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The configuration: the instrument, the worlds the observations come from, the candidates fitted
//! to each world's passive read-out, and the experiments.

use crate::constants::{
    BASELINE, CONTRAST, CYCLE_TIME, DARK_BACKGROUND, DECLARED_SHOTS, EARTH_RATE, GRAVITY_GRADIENT,
    INTERROGATION_TIME, LATITUDE_DEG, LEAK, PLATFORM_SETUP_TIME, SENSITIVITY_G, SETUP_TIME,
    STANDARD_GRAVITY, STEP_RATE, WAVELENGTH, WHITE_NOISE_RANGE,
};
use crate::model::fit;
use crate::model_types::{Cause, Cloud, Fringe, Instrument, Mechanism, Setting};
use crate::{C, FloatType};
use deep_causality_algebra::Real;
use deep_causality_num::{One, Zero, lift_count};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    ConfiguredExperiment, InterferometerModel, Observable, Projection, QuantumError, QuantumPlant,
};
use deep_causality_tensor::CausalTensor;

/// The candidates' names, in the order the verdicts and tables report them.
pub const CANDIDATES: [(&str, Mechanism); 4] = [
    ("H1 leak A->B", Mechanism::LeakAToB),
    ("H2 leak B->A", Mechanism::LeakBToA),
    ("H3 shared phase", Mechanism::Benign),
    ("H4 rotation", Mechanism::Rotation),
];

/// The instrument's numbers, and its interferometer model for planning in time.
///
/// The operating rotation is the horizontal component of Earth's rate at the site; the centrifugal
/// phase at a rate `Ω` is `2 b k_eff T² Ω²`.
///
/// # Errors
///
/// The interferometer model's refusal of a parameter.
pub fn instrument() -> Result<(Instrument, InterferometerModel<FloatType>), QuantumError> {
    let two = FloatType::one() + FloatType::one();
    let k_eff = two * (FloatType::pi() + FloatType::pi()) / WAVELENGTH;
    let t2 = INTERROGATION_TIME * INTERROGATION_TIME;
    let phase_per_gradient = k_eff * t2 * BASELINE;
    let centrifugal = |rate: FloatType| two * phase_per_gradient * rate * rate;
    let latitude = LATITUDE_DEG * FloatType::pi() / lift_count::<FloatType>(180);
    let operating_rate = EARTH_RATE * latitude.cos();

    let interferometer = InterferometerModel::new(
        k_eff,
        INTERROGATION_TIME,
        CONTRAST,
        SENSITIVITY_G * STANDARD_GRAVITY,
        CYCLE_TIME,
        WHITE_NOISE_RANGE,
        SETUP_TIME,
    )?;
    let instrument = Instrument {
        contrast: CONTRAST,
        dark_background: DARK_BACKGROUND,
        leak: LEAK,
        operating_rotation_phase: centrifugal(operating_rate),
        step_rotation_phase: centrifugal(STEP_RATE),
        phase_per_gradient,
    };
    Ok((instrument, interferometer))
}

/// The worlds the observations are drawn from: one per candidate, and the two-way leak, which no
/// candidate is. Each cloud's fringe is centred, and the differential phase is the gravity
/// gradient's plus the quarter fringe the instrument operates at, where the ellipse is closest to
/// a circle.
pub fn worlds(instrument: &Instrument) -> Vec<Cause> {
    let half = FloatType::one() / (FloatType::one() + FloatType::one());
    let fringe = Fringe {
        centre_a: half,
        centre_b: half,
        phase: GRAVITY_GRADIENT * instrument.phase_per_gradient + FloatType::pi() * half,
    };
    [
        ("leak A->B", Mechanism::LeakAToB),
        ("leak B->A", Mechanism::LeakBToA),
        ("shared phase only", Mechanism::Benign),
        ("rotation", Mechanism::Rotation),
        ("leak both ways", Mechanism::LeakBothWays),
    ]
    .into_iter()
    .map(|(name, mechanism)| Cause {
        name,
        mechanism,
        fringe,
    })
    .collect()
}

/// The four candidates, each fitted to `passive`, the passive read-out of the world in force.
///
/// # Errors
///
/// [`fit`]'s, when a candidate cannot reproduce the read-out.
pub fn candidates(
    passive: &[FloatType; 4],
    instrument: &Instrument,
) -> Result<Vec<Cause>, QuantumError> {
    CANDIDATES
        .iter()
        .map(|&(name, mechanism)| {
            Ok(Cause {
                name,
                mechanism,
                fringe: fit(mechanism, passive, instrument)?,
            })
        })
        .collect()
}

/// The experiment family. Each is priced at its setup time; planning adds the integration time of
/// the shots it sizes.
///
/// # Errors
///
/// [`ConfiguredExperiment::new`]'s refusal of a cost.
pub fn experiments() -> Result<Vec<ConfiguredExperiment<FloatType, Setting>>, QuantumError> {
    let exp = |name: &str, cost: FloatType, setting: Setting| {
        ConfiguredExperiment::new(name, cost, DECLARED_SHOTS, setting, 0)
    };
    Ok(vec![
        exp("E0 passive", FloatType::zero(), Setting::Passive)?,
        exp(
            "E1 brighten A, read B",
            SETUP_TIME,
            Setting::Brighten(Cloud::A),
        )?,
        exp(
            "E2 brighten B, read A",
            SETUP_TIME,
            Setting::Brighten(Cloud::B),
        )?,
        exp(
            "E3 rotation step",
            PLATFORM_SETUP_TIME,
            Setting::RotationStep,
        )?,
    ])
}

/// The two read-outs as a two-qubit register in `|00⟩`, which the configuration requires and the
/// structural predictions do not use.
///
/// # Errors
///
/// The plant's refusal of the state.
pub fn plant() -> Result<QuantumPlant<FloatType>, QuantumError> {
    let (one, zero) = (
        Complex::new(FloatType::one(), FloatType::zero()),
        Complex::new(FloatType::zero(), FloatType::zero()),
    );
    QuantumPlant::from_ket(&CausalTensor::from_slice(&[one, zero, zero, zero], &[4]))
}

/// The projector onto both read-outs excited, `|11⟩⟨11|`.
///
/// # Errors
///
/// The projection's refusal of the operator.
pub fn both_excited() -> Result<Observable<FloatType, 4>, QuantumError> {
    let zero: C = Complex::new(FloatType::zero(), FloatType::zero());
    let mut data = vec![zero; 16];
    data[15] = Complex::new(FloatType::one(), FloatType::zero());
    Ok(Observable::new(
        "both excited",
        Projection::<FloatType, 4>::new(CausalTensor::from_slice(&data, &[4, 4]))?,
    ))
}
