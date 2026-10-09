/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The configuration: the instrument, the candidates, the experiments as instrument contexts at
//! the time they run, and the runs.

use crate::FloatType;
use crate::constants::{
    AMBIENT_TEMPERATURE, ATOM_TEMPERATURE, BIAS_FIELD, CLIPPING_SLOPE_UGAL_PER_MM,
    CLOUD_DISPLACEMENT, CONTRAST, DECLARED_SHOTS, FIELD_STEP, FLOOR_BITS, INTERROGATION_TIME,
    LIGHT_SHIFT_CO_MRAD, LIGHT_SHIFT_COUNTER_MRAD, MICRO_GAL, OFFSET_UGAL, POSITIVE_OFFSET_UGAL,
    PROTOCOL_DEGRADATION, PROTOCOL_SENSITIVITY_UGAL, RABI_FREQUENCY, RABI_STEP, REPETITION_RATE,
    SETUP_ACCELEROMETER, SETUP_CLOUD, SETUP_FIELD, SETUP_K_REVERSAL, SETUP_RABI, SETUP_TEMPERATURE,
    SETUP_TILT, SETUP_TURN, STANDARD_GRAVITY, TIDE_AMPLITUDE_UGAL, TIDE_PERIOD, TIDE_PHASE,
    TILT_STEP, WAVELENGTH, WHITE_NOISE_RANGE, ZEEMAN_BACKGROUND_SHARE, ZEEMAN_COIL_SHARE,
    ZEEMAN_CROSS_SHARE,
};
use crate::model::{calibrated, read_out, tide_at};
use crate::model_types::{Cause, Physics, Scenario, Session, Systematic, TideSignal, ZeemanShares};
use deep_causality_algebra::Real;
use deep_causality_context::TimeScale;
use deep_causality_num::{One, Zero, lift_count, to_count};
use deep_causality_quantum::{
    ConfiguredExperiment, EnvironmentReading, InterferometerConfiguration, InterferometerModel,
    QuantumError, WaveVector, interferometer_context, record_environment, separation_bits,
};

/// The candidates' systematics, in the order the verdicts report them.
pub const CANDIDATES: [Systematic; 7] = [
    Systematic::Coriolis,
    Systematic::QuadraticZeeman,
    Systematic::Tilt,
    Systematic::MirrorVibration,
    Systematic::Wavefront,
    Systematic::LightShift,
    Systematic::Clipping,
];

/// The name of the candidate whose systematic is `systematic`.
pub fn candidate_name(systematic: Systematic) -> &'static str {
    match systematic {
        Systematic::Coriolis => "H1 Coriolis",
        Systematic::QuadraticZeeman => "H2 quadratic Zeeman",
        Systematic::Tilt => "H3 tilt",
        Systematic::MirrorVibration => "H4 mirror vibration",
        Systematic::Wavefront => "H5 wavefront",
        Systematic::LightShift => "H6 light shift",
        Systematic::Clipping => "H7 clipping",
    }
}

/// The runs: each of the seven systematics as the cause of a −5 µGal offset; a +5 µGal offset from
/// the light shift, where the baseline refuses tilt; the contexts without the tide; and clipping
/// as the cause with clipping left off the list.
pub fn scenarios() -> Vec<Scenario> {
    let mut runs: Vec<Scenario> = CANDIDATES
        .iter()
        .map(|&truth| Scenario {
            label: candidate_name(truth),
            truth,
            offset_ugal: OFFSET_UGAL,
            records_tide: true,
            without: None,
        })
        .collect();
    runs.extend([
        Scenario {
            label: "a positive offset",
            truth: Systematic::LightShift,
            offset_ugal: POSITIVE_OFFSET_UGAL,
            records_tide: true,
            without: None,
        },
        Scenario {
            label: "no tide in the context",
            truth: Systematic::Coriolis,
            offset_ugal: OFFSET_UGAL,
            records_tide: false,
            without: None,
        },
        Scenario {
            label: "clipping left off the list",
            truth: Systematic::Clipping,
            offset_ugal: OFFSET_UGAL,
            records_tide: true,
            without: Some(Systematic::Clipping),
        },
    ]);
    runs
}

/// The numbers every response is built from.
///
/// # Errors
///
/// The quadratic Zeeman shares' refusal.
pub fn physics() -> Result<Physics, QuantumError> {
    let per_metre = lift_count::<FloatType>(1000);
    Ok(Physics {
        contrast: CONTRAST,
        gravity: STANDARD_GRAVITY,
        bias_field: BIAS_FIELD,
        rabi_frequency: RABI_FREQUENCY,
        zeeman_shares: ZeemanShares::new(
            ZEEMAN_COIL_SHARE,
            ZEEMAN_CROSS_SHARE,
            ZEEMAN_BACKGROUND_SHARE,
        )?,
        light_shift_field_share: LIGHT_SHIFT_CO_MRAD
            / (LIGHT_SHIFT_COUNTER_MRAD + LIGHT_SHIFT_CO_MRAD),
        clipping_slope: CLIPPING_SLOPE_UGAL_PER_MM * MICRO_GAL * per_metre,
        tide: TideSignal {
            amplitude: TIDE_AMPLITUDE_UGAL * MICRO_GAL,
            phase: TIDE_PHASE,
            period: TIDE_PERIOD,
        },
    })
}

/// The instrument model. One configuration's sensitivity is the four-configuration protocol's,
/// 70 µGal at 1 s, divided by `√10`; the setup time of a configuration change is each
/// experiment's own cost, so the model carries none.
///
/// # Errors
///
/// The model's refusal of a parameter.
pub fn interferometer() -> Result<InterferometerModel<FloatType>, QuantumError> {
    let two = FloatType::one() + FloatType::one();
    let k_eff = two * (FloatType::pi() + FloatType::pi()) / WAVELENGTH;
    let sensitivity = PROTOCOL_SENSITIVITY_UGAL * MICRO_GAL / PROTOCOL_DEGRADATION.sqrt();
    InterferometerModel::new(
        k_eff,
        INTERROGATION_TIME,
        CONTRAST,
        sensitivity,
        FloatType::one() / REPETITION_RATE,
        WHITE_NOISE_RANGE,
        FloatType::zero(),
    )
}

/// The candidates for an offset of `offset` m/s², each calibrated to produce it in the passive
/// configuration as closely as its physics allows, with `without` left off.
pub fn candidates(offset: FloatType, without: Option<Systematic>, physics: &Physics) -> Vec<Cause> {
    CANDIDATES
        .iter()
        .filter(|&&s| Some(s) != without)
        .map(|&systematic| Cause {
            name: candidate_name(systematic).to_string(),
            systematic,
            size: calibrated(systematic, offset, physics),
        })
        .collect()
}

/// The passive configuration: wave vector up, at the operating field, temperature and Rabi
/// frequency.
fn passive_configuration() -> Result<InterferometerConfiguration<FloatType>, QuantumError> {
    InterferometerConfiguration::new(BIAS_FIELD, ATOM_TEMPERATURE, RABI_FREQUENCY)
}

/// The instrument context for `configuration` at `tick` s from the start, holding a reading of
/// the Earth tide at that time, or of none when `records_tide` is off.
fn session(
    name: &str,
    configuration: InterferometerConfiguration<FloatType>,
    tick: u64,
    records_tide: bool,
    interferometer: &InterferometerModel<FloatType>,
    physics: &Physics,
) -> Result<Session, QuantumError> {
    let mut context = interferometer_context(1, name, *interferometer, configuration)?;
    let tide = if records_tide {
        tide_at(tick, physics)
    } else {
        FloatType::zero()
    };
    record_environment(
        &mut context,
        tick,
        TimeScale::Second,
        EnvironmentReading::new(tide, AMBIENT_TEMPERATURE, FloatType::zero())?,
    )?;
    Ok(Session { context, tick })
}

/// The passive experiment at `tick`, observed first at `draws` effective draws.
///
/// # Errors
///
/// The configuration's, the context's or the experiment's refusal.
pub fn passive(
    tick: u64,
    records_tide: bool,
    draws: u64,
    interferometer: &InterferometerModel<FloatType>,
    physics: &Physics,
) -> Result<ConfiguredExperiment<FloatType, Session>, QuantumError> {
    let name = "E0 passive";
    let at = session(
        name,
        passive_configuration()?,
        tick,
        records_tide,
        interferometer,
        physics,
    )?;
    ConfiguredExperiment::new(name, FloatType::zero(), draws, at, 0)
}

/// The experiment family at `tick`, each one knob turned from the passive configuration and priced
/// at its setup time.
///
/// # Errors
///
/// The configuration's, the context's or the experiment's refusal.
pub fn experiments(
    tick: u64,
    records_tide: bool,
    interferometer: &InterferometerModel<FloatType>,
    physics: &Physics,
) -> Result<Vec<ConfiguredExperiment<FloatType, Session>>, QuantumError> {
    let base = passive_configuration()?;
    let turned = [
        (
            "E1 k reversed",
            SETUP_K_REVERSAL,
            base.with_wave_vector(WaveVector::Down),
        ),
        (
            "E2 turned 180°",
            SETUP_TURN,
            base.with_heading(FloatType::pi())?,
        ),
        (
            "E3 accelerometer correction",
            SETUP_ACCELEROMETER,
            base.with_accelerometer_correction(true),
        ),
        ("E4 tilted", SETUP_TILT, base.with_tilt_offset(TILT_STEP)?),
        (
            "E5 field halved",
            SETUP_FIELD,
            base.with_bias_field(BIAS_FIELD * FIELD_STEP)?,
        ),
        (
            "E6 extrapolated to 0 K",
            SETUP_TEMPERATURE,
            base.with_atom_temperature(FloatType::zero())?,
        ),
        (
            "E7 Rabi frequency halved",
            SETUP_RABI,
            base.with_rabi_frequency(RABI_FREQUENCY * RABI_STEP)?,
        ),
        (
            "E8 cloud displaced",
            SETUP_CLOUD,
            base.with_cloud_position([CLOUD_DISPLACEMENT, FloatType::zero(), FloatType::zero()])?,
        ),
    ];
    turned
        .into_iter()
        .map(|(name, cost, configuration)| {
            let at = session(
                name,
                configuration,
                tick,
                records_tide,
                interferometer,
                physics,
            )?;
            ConfiguredExperiment::new(name, cost, DECLARED_SHOTS, at, 0)
        })
        .collect()
}

/// The effective draws at which the baseline separates `offset` from no offset at the floor.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] when the draws do not fit a count.
pub fn baseline_draws(
    offset: FloatType,
    interferometer: &InterferometerModel<FloatType>,
    physics: &Physics,
) -> Result<u64, QuantumError> {
    let t = interferometer.interrogation_time();
    let phase_per_acceleration = interferometer.k_eff() * t * t;
    let bits = separation_bits(
        read_out(offset, phase_per_acceleration, physics),
        read_out(FloatType::zero(), phase_per_acceleration, physics),
        1,
    );
    to_count((FLOOR_BITS / bits).ceil()).ok_or_else(|| {
        QuantumError::CalculationError("the baseline's draws do not fit a count".into())
    })
}
