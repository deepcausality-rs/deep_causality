/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The descent-world builder, the atmosphere rows, the initial coupled field, and the composed
//! coupling stack both blackout examples fly.

use super::FloatType;
use super::constants::{
    AIR_MEAN_MOLECULAR_MASS_KG, ATMOSPHERE, CDA_OVER_M, DT_FLIGHT, DT_SOLVER, FALLBACK_N_TOT,
    FALLBACK_PRESSURE_ATM, GAMMA_EFF, GNSS_VAR, IMU_ACCEL_BIAS, IMU_GYRO_BIAS, L, L_CHAR, L_OVER_D,
    MAX_BANK_RAD, MAX_G_LOAD, MAX_HEAT_FLUX, N_REF, NAV_INIT_ERR, OPTICAL_VAR, P0_DIAG, Q_DIAG,
    REDUCED_MASS_AMU, RHO_REF, S_REF, SEED_P_HAT, SEED_RHO_HAT, SEED_U_HAT, SHEATH_PEAK_AGE_S,
    T_REF, T_VE_INITIAL, THETA_VIB, TRUTH_ALTITUDE_0, TRUTH_V0, U_REF,
};
use super::constants::{
    BURN_CORRIDOR_Q_INF, CHAMBER_PRESSURE_MAX, CHAMBER_TEMPERATURE, CONTACT_SPEED_MS, G0,
    IGNITION_MACH_MAX, IGNITION_MACH_MIN, IGNITION_Q_MAX, IGNITION_Q_MIN, IMPRINT_AXIS_Y,
    IMPRINT_DOMAIN_M, IMPRINT_ETA, IMPRINT_FACE_X, IMPRINT_MAX_REFRESHES, IMPRINT_SMOOTHING_CELLS,
    IMPRINT_TARGET, IMPRINT_THROTTLE_TOL, JET_GAMMA, JET_R_SPECIFIC, MAX_CT, MAX_DESCENT_RATE,
    NOZZLE_CONE_L, NOZZLE_EXIT_MACH, NOZZLE_EXIT_R, NOZZLE_HALF_ANGLE_RAD, NOZZLE_THROAT_D,
    PLUME_GAMMA_INF, PLUME_MACH_INF, PLUME_P_INF, PLUME_S_REF_M2, PROPELLANT_FLOOR_KG,
    PROPELLANT_KG, RETRO_ISP_S, RETRO_THRUST_N, TERMINAL_MAX_CT, TERMINAL_MAX_DESCENT_RATE,
    TERMINAL_THROTTLE_FLOOR, THROTTLE_CEILING, THROTTLE_FLOOR, TOUCHDOWN_ALTITUDE_M,
    VEHICLE_MASS_KG,
};
use super::constants::{
    GAMMA_TERMINAL, N_REF_TERMINAL, S_REF_TERMINAL, SEED_P_HAT_TERMINAL, SEED_RHO_HAT_TERMINAL,
    SEED_U_HAT_TERMINAL, T_REF_TERMINAL, TERMINAL_REBUILD_BUDGET, U_REF_TERMINAL,
};
use super::stages::{
    CommandedBank, FreestreamFeeds, SuttonGravesLoads, TruthGnss, WeatherTelemetry,
};
use super::utils;
use deep_causality_algebra::Real;
use deep_causality_cfd::{
    Ambient, AtmosphereRow, BankSteeredLift, BurnEnvelope, CompressibleMarchConfig,
    CompressibleMarchConfigBuilder, CoupledField, Coupling, CyberneticCorrect, DescentSchedule,
    FiniteRateIonizationStage, FlightSensors, IgnitionCorridor, ImuModel, InsErrorState, MarchStop,
    NavFilter, PhysicsError, PhysicsStage, PlumeImprint, PlumeNozzle, PlumeObstruction, QttObserve,
    ReentryNavEngine, ReferenceScales, RegimeClassify, RetroThrust, SafetyEnvelope,
    ThrottleGuidance, TrajectoryNav, VibrationalLagStage,
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
    descent_world_with(name, rows, steps, constants, false)
}

/// `descent_world`, with the marched-layer plume imprint optionally enabled.
pub fn descent_world_with(
    name: &'static str,
    rows: Vec<AtmosphereRow<FloatType>>,
    steps: usize,
    constants: &[(&'static str, FloatType)],
    imprint: bool,
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
    if imprint {
        builder = builder.plume_imprint(plume_imprint());
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

// ── Powered descent (M5) ────────────────────────────────────────────────────────────────────

/// The **powered-descent coupling stack**: the corridor stack extended with the burn.
///
/// Reads top to bottom, with the M2/M4 ordering contracts made explicit:
/// - the chemistry, feeds, and classifier as the corridor flies them — but the classifier now opts
///   into the **flight axes** (`with_flight_axes`), without which the Mach, thrust-state, and
///   touchdown axes stay `Unknown` even though the carrier publishes `"flight_mach"` every step,
///   and the regime cascade never logs;
/// - `FlightSensors` publishes `"q_inf"` and `"descent_rate"`. Nothing else produces them, and the
///   gate reads each through a peak folded from zero, so without this stage the descent-rate bound
///   cannot fire and the dynamic `C_T` cap collapses to the static ceiling — two safety axes
///   reporting as enforcing while unable to;
/// - `RetroThrust` and `PlumeObstruction` compose **after** the ④-writing lift stage and **before**
///   the force consumers (loads, truth propagator, navigation kick), so every consumer reads one
///   summed vector. The M2 `PropulsionStub` is deliberately absent: it bundles thrust, depletion,
///   and the A0 decrement, so composing it beside these two would apply thrust and depletion twice
///   and compound the multiplicative drag rescale to `f²`;
/// - `ThrottleGuidance` composes after the navigation stage, so it reads this step's nav quality,
///   and before the gate, so the gate clamps what it wrote. The thrust stages, composed earlier,
///   fly the previous step's clamped command — the one-step actuation lag the bank channel has;
/// - `CyberneticCorrect` carries the burn axes **and** `with_burn_sensing`, without which the
///   dynamic `C_T` cap silently never binds.
///
/// `margin_m` is the table-sized ignition margin (`drift_mean + k·drift_sd` from the interpolated
/// dispersion row) — a caller-supplied input, because the physics crate's ignition kernel takes its
/// margin as an input by design.
pub fn powered_descent_coupling(
    imu_bias_departure: f64,
    noise_draw: usize,
    margin_m: FloatType,
) -> impl PhysicsStage<2, FloatType> {
    powered_descent_coupling_with(imu_bias_departure, noise_draw, margin_m, false)
}

/// The powered-descent stack, with `terminal` selecting the **subsonic** burn envelope for the
/// landing leg (the SRP-regime axes released — see the terminal-envelope constants).
pub fn powered_descent_coupling_with(
    imu_bias_departure: f64,
    noise_draw: usize,
    margin_m: FloatType,
    terminal: bool,
) -> impl PhysicsStage<2, FloatType> {
    let imu = ImuModel::new(
        core::array::from_fn(|i| utils::ft(IMU_ACCEL_BIAS[i] * imu_bias_departure)),
        core::array::from_fn(|i| utils::ft(IMU_GYRO_BIAS[i])),
        core::array::from_fn(|i| utils::ft(Q_DIAG[i])),
    );
    // The supersonic leg's burn is a *deceleration* burn — it flies the Jarvinen-Adams band to shed
    // entry velocity, and there is no landing site in its stopping-distance sense. Only the terminal
    // leg flies a stopping burn, so only it coasts to the ignition altitude first: applied earlier,
    // the hold-off would suppress the retropropulsion the whole example exists to show.
    let guidance = ThrottleGuidance::new(utils::ft(RETRO_THRUST_N), utils::ft(G0)).with_corridor(
        IgnitionCorridor::new(
            utils::ft(IGNITION_MACH_MIN),
            utils::ft(IGNITION_MACH_MAX),
            utils::ft(IGNITION_Q_MIN),
            utils::ft(IGNITION_Q_MAX),
            margin_m,
        ),
    );
    let guidance = if terminal {
        // Aimed at the touchdown plane the regime classifier calls the ground, so the vehicle
        // arrives stopped where the descent rate is actually sampled.
        guidance
            .with_stopping_burn(margin_m)
            .with_target_altitude(utils::ft(TOUCHDOWN_ALTITUDE_M))
            .with_contact_speed(utils::ft(CONTACT_SPEED_MS))
    } else {
        guidance
    };

    let burn = if terminal {
        // The SRP axes are released: they are bow-shock-interaction constraints and there is no
        // bow shock subsonically. See the terminal-envelope constants for the full reasoning.
        BurnEnvelope::new(
            utils::ft(TERMINAL_THROTTLE_FLOOR),
            utils::ft(THROTTLE_CEILING),
            utils::ft(TERMINAL_MAX_CT),
            utils::ft(IGNITION_Q_MIN),
            utils::ft(IGNITION_Q_MAX),
            utils::ft(PROPELLANT_FLOOR_KG),
            utils::ft(TERMINAL_MAX_DESCENT_RATE),
        )
    } else {
        BurnEnvelope::new(
            utils::ft(THROTTLE_FLOOR),
            utils::ft(THROTTLE_CEILING),
            utils::ft(MAX_CT),
            utils::ft(IGNITION_Q_MIN),
            utils::ft(IGNITION_Q_MAX),
            utils::ft(PROPELLANT_FLOOR_KG),
            utils::ft(MAX_DESCENT_RATE),
        )
    };
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
            FiniteRateIonizationStage::new(utils::ft(FALLBACK_N_TOT))
                .with_density_field("n_tot")
                .with_sheath_renewal(utils::ft(SHEATH_PEAK_AGE_S)),
        )
        .then(FreestreamFeeds)
        .then(FlightSensors::new(utils::ft(AIR_MEAN_MOLECULAR_MASS_KG)))
        .then(
            RegimeClassify::new(utils::ft(L_CHAR), utils::trigger()).with_flight_axes(
                utils::ft(0.8),
                utils::ft(1.2),
                utils::ft(TOUCHDOWN_ALTITUDE_M),
            ),
        )
        .then(
            BankSteeredLift::new(
                utils::ft(RHO_REF),
                utils::ft(CDA_OVER_M),
                utils::ft(L_OVER_D),
            )
            .with_speed_field("equivalent_airspeed"),
        )
        // Order is load-bearing: the plume decrement scales the along-velocity component of
        // the force channel, and the thrust term is also along −v̂ — so the decrement must be
        // applied to the aerodynamic drag alone, before the thrust is added to the channel.
        // `PlumeObstruction` takes its dynamic pressure at CONSTRUCTION, not from the sensed
        // `"q_inf"` the gate reads — so the world supplies a representative burn-corridor value.
        // The A0 correlation is digitized over a bounded `C_T` domain and errors outside it, so a
        // placeholder normalization here does not merely bias the decrement, it fails the step.
        // The optional geometry publication is what lets the marched layer learn the plume: the
        // carrier's re-imprint reader consumes `"plume_max_radius"`/`"plume_penetration"` from here.
        // It is **state realism only** — the force-channel drag decrement is the A0 correlation with
        // or without it (the AMBER verdict).
        .then(
            PlumeObstruction::new(
                utils::ft(RETRO_THRUST_N),
                utils::ft(BURN_CORRIDOR_Q_INF),
                utils::ft(PLUME_S_REF_M2),
            )
            .with_plume_geometry(plume_nozzle()),
        )
        .then(RetroThrust::new(
            utils::ft(RETRO_THRUST_N),
            utils::ft(RETRO_ISP_S),
        ))
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
        .then(guidance)
        .then(WeatherTelemetry)
        .then(
            CyberneticCorrect::new(
                SafetyEnvelope::new(
                    utils::ft(MAX_HEAT_FLUX),
                    utils::ft(MAX_G_LOAD),
                    utils::ft(MAX_BANK_RAD),
                )
                .with_burn(burn),
            )
            .with_burn_sensing(
                "q_inf",
                "propellant",
                "descent_rate",
                utils::ft(RETRO_THRUST_N),
                utils::ft(PLUME_S_REF_M2),
            ),
        )
        .build()
}

/// The initial field for a powered descent: the corridor's seeded field plus the carried propulsion
/// state.
///
/// `"mass"` and `"propellant"` are seeded **once, onto the field** — never through the world's
/// `publish_constant` seam. The carrier re-publishes every world constant with `set_scalar` at the
/// head of each step, *before* the coupling runs, so a published mass would be restored to its seed
/// after every step's depletion: the burn would never consume propellant, the propellant floor
/// could never trip, and the mass-aware thrust normalization would be frozen at its initial value.
pub fn powered_initial_field() -> CoupledField<FloatType> {
    let mut field = initial_field();
    field.set_scalar("mass", Vec::from([utils::ft(VEHICLE_MASS_KG)]));
    field.set_scalar("propellant", Vec::from([utils::ft(PROPELLANT_KG)]));
    field
}

/// The **terminal-leg world**: the descent world retuned for cool low-Mach air.
///
/// Both gammas move: the schedule's effective gamma (the Rankine-Hugoniot jump) and the marcher's
/// own. The corridor pins both to the reacting-shock recipe, which hides that they are separate
/// knobs. The acoustic reference is retuned with them, since at low Mach the sound speed dominates
/// the required wave-speed envelope, and the terminal leg carries a rebuild budget so a leg that
/// cannot settle on an envelope fails loudly instead of reporting numbers marched on an undersized
/// one.
pub fn terminal_descent_world(
    name: &'static str,
    rows: Vec<AtmosphereRow<FloatType>>,
    steps: usize,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    let schedule = DescentSchedule::new(rows, utils::ft(GAMMA_TERMINAL))?
        .with_rebuild_budget(TERMINAL_REBUILD_BUDGET);
    let n = 1usize << L;
    let dx = utils::ft(1.0) / utils::ft(n as f64);
    CompressibleMarchConfigBuilder::<FloatType>::new()
        .name(name)
        .grid(L, L, dx, dx)
        .solver(
            utils::ft(DT_SOLVER),
            utils::ft(S_REF_TERMINAL),
            utils::ft(GAMMA_TERMINAL),
            utils::trunc(),
        )
        .flight_dt(utils::ft(DT_FLIGHT))
        .seed_fn(|_, _| {
            (
                utils::ft(SEED_RHO_HAT_TERMINAL),
                utils::ft(SEED_U_HAT_TERMINAL),
                utils::ft(0.0),
                utils::ft(SEED_P_HAT_TERMINAL),
            )
        })?
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default().heat_flux())
        .schedule(schedule)
        .reference(ReferenceScales {
            t_ref: utils::ft(T_REF_TERMINAL),
            n_ref: utils::ft(N_REF_TERMINAL),
            u_ref: utils::ft(U_REF_TERMINAL),
        })
        .publish_constant("commanded_bank", utils::ft(0.0))
        .build()
}

/// The nozzle and freestream the analytic plume boundary is built from. Values sit inside the
/// Cordell–Braun validated envelope (jet gamma in [1.2, 1.4], freestream Mach in [2, 4]) — the
/// discipline pin that keeps a surprising result attributable to physics rather than extrapolation.
pub fn plume_nozzle() -> PlumeNozzle<FloatType> {
    PlumeNozzle {
        chamber_pressure_max: utils::ft(CHAMBER_PRESSURE_MAX),
        chamber_temperature: utils::ft(CHAMBER_TEMPERATURE),
        r_specific: utils::ft(JET_R_SPECIFIC),
        gamma_jet: utils::ft(JET_GAMMA),
        exit_mach: utils::ft(NOZZLE_EXIT_MACH),
        nozzle_half_angle_rad: utils::ft(NOZZLE_HALF_ANGLE_RAD),
        throat_diameter: utils::ft(NOZZLE_THROAT_D),
        exit_radius: utils::ft(NOZZLE_EXIT_R),
        cone_length: utils::ft(NOZZLE_CONE_L),
        p_inf: utils::ft(PLUME_P_INF),
        mach_inf: utils::ft(PLUME_MACH_INF),
        gamma_inf: utils::ft(PLUME_GAMMA_INF),
    }
}

/// The marched-layer plume imprint spec.
///
/// This is M3's opt-in **state-realism** seam: the carrier refreshes a masked forcing region from
/// the geometry `PlumeObstruction` published, on a tolerance-sized throttle move, capped and logged.
/// Without it a branch's throttle reaches only the force channel and the marched layer never learns
/// the plume exists — so the branch flow observables cannot spread, and the flow-spread witness has
/// nothing to measure. It never touches the drag closure.
pub fn plume_imprint() -> PlumeImprint<FloatType> {
    PlumeImprint {
        throttle_tolerance: utils::ft(IMPRINT_THROTTLE_TOL),
        max_refreshes: IMPRINT_MAX_REFRESHES,
        face_x: utils::ft(IMPRINT_FACE_X),
        axis_y: utils::ft(IMPRINT_AXIS_Y),
        smoothing_cells: utils::ft(IMPRINT_SMOOTHING_CELLS),
        domain_m: utils::ft(IMPRINT_DOMAIN_M),
        target: core::array::from_fn(|i| utils::ft(IMPRINT_TARGET[i])),
        eta: utils::ft(IMPRINT_ETA),
    }
}
