/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The descent-world builder, the atmosphere rows, the initial coupled field, and the composed
//! coupling stack both blackout examples fly.

use super::FloatType;
use super::constants::{
    ATMOSPHERE, CDA_OVER_M, DT_FLIGHT, DT_SOLVER, FALLBACK_N_TOT, FALLBACK_PRESSURE_ATM, GAMMA_EFF,
    GNSS_VAR, IMU_ACCEL_BIAS, IMU_GYRO_BIAS, L, L_CHAR, L_OVER_D, MAX_BANK_RAD, MAX_G_LOAD,
    MAX_HEAT_FLUX, N_REF, NAV_INIT_ERR, OPTICAL_VAR, P0_DIAG, Q_DIAG, REDUCED_MASS_AMU, RHO_REF,
    S_REF, SEED_P_HAT, SEED_RHO_HAT, SEED_U_HAT, SHEATH_PEAK_AGE_S, T_REF, T_VE_INITIAL, THETA_VIB,
    TRUTH_ALTITUDE_0, TRUTH_V0, U_REF,
};
use super::stages::{
    CommandedBank, FreestreamFeeds, SuttonGravesLoads, TruthGnss, WeatherTelemetry,
};
use super::utils;
use deep_causality_algebra::Real;
use deep_causality_cfd::{
    Ambient, AtmosphereRow, BankSteeredLift, CompressibleMarchConfig,
    CompressibleMarchConfigBuilder, CoupledField, Coupling, CyberneticCorrect, DescentSchedule,
    FiniteRateIonizationStage, ImuModel, InsErrorState, MarchStop, NavFilter, PhysicsError,
    PhysicsStage, QttObserve, ReentryNavEngine, ReferenceScales, RegimeClassify, SafetyEnvelope,
    TrajectoryNav, VibrationalLagStage,
};
use deep_causality_physics::{EARTH_GM, EARTH_RADIUS};

/// The baseline atmosphere as typed rows.
pub fn standard_atmosphere() -> Vec<AtmosphereRow<FloatType>> {
    weather_atmosphere(0.0, 1.0)
}

/// A weather-dispersed atmosphere: the baseline table with a temperature offset added and a
/// density scale multiplied per row, the sound speed rescaled by `sqrt(T'/T)`.
pub fn weather_atmosphere(d_temp: f64, rho_scale: f64) -> Vec<AtmosphereRow<FloatType>> {
    ATMOSPHERE
        .iter()
        .map(|&(alt, n, t, a)| {
            let t_new = t + d_temp;
            AtmosphereRow {
                altitude_m: utils::ft(alt),
                n_tot: utils::ft(n * rho_scale),
                temperature: utils::ft(t_new),
                sound_speed: utils::ft(a) * Real::sqrt(utils::ft(t_new / t)),
            }
        })
        .collect()
}

/// A named descent world over `rows`, marching up to `steps` coupled steps, publishing the
/// world's `constants` (name, value) into the field each step. The published constants are how
/// a counterfactual world carries its commanded inputs (a candidate bank, a bias departure)
/// through the shared coupling stack.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn descent_world(
    name: &'static str,
    rows: Vec<AtmosphereRow<FloatType>>,
    steps: usize,
    constants: &[(&'static str, FloatType)],
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    let schedule = DescentSchedule::new(rows, utils::ft(GAMMA_EFF))?;
    let n = 1usize << L;
    let dx = utils::ft(1.0) / utils::ft(n as f64);
    let mut builder = CompressibleMarchConfigBuilder::<FloatType>::new()
        .name(name)
        .grid(L, L, dx, dx)
        .solver(
            utils::ft(DT_SOLVER),
            utils::ft(S_REF),
            utils::ft(GAMMA_EFF),
            utils::trunc(),
        )
        .flight_dt(utils::ft(DT_FLIGHT))
        .seed_fn(|_, _| {
            (
                utils::ft(SEED_RHO_HAT),
                utils::ft(SEED_U_HAT),
                utils::ft(0.0),
                utils::ft(SEED_P_HAT),
            )
        })?
        .stop(MarchStop::Fixed(steps))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .heat_flux()
                .blackout_dwell(),
        )
        .schedule(schedule)
        .reference(ReferenceScales {
            t_ref: utils::ft(T_REF),
            n_ref: utils::ft(N_REF),
            u_ref: utils::ft(U_REF),
        });
    for &(cname, value) in constants {
        builder = builder.publish_constant(cname, value);
    }
    builder.build()
}

/// The per-step coupling stack, the loop body every leg, branch, and weather world iterates.
/// Reads top to bottom: the vibrational bath and the finite-rate ionization network on the
/// **evolved** marched state, the freestream feeds, the regime classifier, the 3-DOF
/// bank-steered aero force, the
/// Sutton-Graves loads, truth/GNSS, navigation with an IMU-sensed specific force, the
/// commanded-bank guidance, the telemetry accumulator, and the cybernetic bounded-correction
/// gate. A static cons-tuple; no `dyn`. An `Err`, such as an envelope breach, short-circuits
/// the whole step.
///
/// `imu_bias_departure` scales the accelerometer bias away from its calibration point (1.0 at
/// standard conditions) while the filter's priors stay standard-day: the weather study's INS
/// thermal model. The corridor flies 1.0. `noise_draw` selects the deterministic receiver-noise
/// realization (0 reproduces the original sequence); it is the Monte Carlo dimension of the
/// weather table's error bars.
pub fn corridor_coupling(
    imu_bias_departure: f64,
    noise_draw: usize,
) -> impl PhysicsStage<2, FloatType> {
    let imu = ImuModel::new(
        core::array::from_fn(|i| utils::ft(IMU_ACCEL_BIAS[i] * imu_bias_departure)),
        core::array::from_fn(|i| utils::ft(IMU_GYRO_BIAS[i])),
        core::array::from_fn(|i| utils::ft(Q_DIAG[i])),
    );
    Coupling::between_steps()
        .then(
            VibrationalLagStage::new(
                utils::ft(T_VE_INITIAL),
                utils::ft(FALLBACK_PRESSURE_ATM),
                utils::ft(REDUCED_MASS_AMU),
                utils::ft(THETA_VIB),
                utils::ft(SHEATH_PEAK_AGE_S),
            )
            .with_pressure_field("pressure_atm"),
        )
        .then(
            // The finite-rate network reads "T_tr" and "T_ve" directly and
            // builds each channel's controlling temperature internally; there
            // is no Saha calibration target anywhere in this stage. Renewal
            // mode is kept per the stagnation-line A/B: its fixed-point clock
            // is the network's true Riccati timescale, and the carried arm
            // under-relaxes young sheath gas. The exposure is the transit-age
            // profile's observable peak, the same age the stagline gate reads.
            FiniteRateIonizationStage::new(utils::ft(FALLBACK_N_TOT))
                .with_density_field("n_tot")
                .with_sheath_renewal(utils::ft(SHEATH_PEAK_AGE_S)),
        )
        .then(FreestreamFeeds)
        .then(RegimeClassify::new(utils::ft(L_CHAR), utils::trigger()))
        .then(
            BankSteeredLift::new(
                utils::ft(RHO_REF),
                utils::ft(CDA_OVER_M),
                utils::ft(L_OVER_D),
            )
            .with_speed_field("equivalent_airspeed"),
        )
        .then(SuttonGravesLoads)
        .then(TruthGnss { noise_draw })
        .then(
            TrajectoryNav::new(
                core::array::from_fn(|i| utils::ft(Q_DIAG[i])),
                utils::ft(GNSS_VAR),
                utils::ft(OPTICAL_VAR),
            )
            .with_imu(imu),
        )
        .then(CommandedBank)
        .then(WeatherTelemetry)
        .then(CyberneticCorrect::new(SafetyEnvelope::new(
            utils::ft(MAX_HEAT_FLUX),
            utils::ft(MAX_G_LOAD),
            utils::ft(MAX_BANK_RAD),
        )))
        .build()
}

/// The initial coupled field: the truth vehicle at the descent start and the navigation engine
/// seeded with the standard-day priors and a 50-m-class initial INS error. Everything else is
/// evolved from here; no station constants.
pub fn initial_field() -> CoupledField<FloatType> {
    let mut field = CoupledField::new(Ambient::new(utils::ft(0.01), utils::ft(0.0), None));
    let r0 = [EARTH_RADIUS + TRUTH_ALTITUDE_0, 0.0, 0.0];
    field.set_scalar(
        "truth_state",
        Vec::from([
            utils::ft(r0[0]),
            utils::ft(r0[1]),
            utils::ft(r0[2]),
            utils::ft(TRUTH_V0[0]),
            utils::ft(TRUTH_V0[1]),
            utils::ft(TRUTH_V0[2]),
        ]),
    );
    let nav_r0: [FloatType; 3] = core::array::from_fn(|i| utils::ft(r0[i] + NAV_INIT_ERR[i]));
    let nav_v0: [FloatType; 3] = core::array::from_fn(|i| utils::ft(TRUTH_V0[i]));
    let filter = NavFilter::new(
        InsErrorState::<FloatType>::zero(),
        core::array::from_fn(|i| utils::ft(P0_DIAG[i])),
    );
    field.set_nav(ReentryNavEngine::new(
        nav_r0,
        nav_v0,
        utils::ft(EARTH_GM),
        filter,
    ));
    field
}
