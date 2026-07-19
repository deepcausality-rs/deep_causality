/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage 3 corridor composition stages: the [`RegimeClassify`] governing-model selector, the
//! [`BranchAccumulator`]/[`BranchOutcome`] counterfactual branch vocabulary, and the
//! [`CyberneticCorrect`] bounded-correction gate.

use deep_causality_cfd::{
    Ambient, BankCorrection, BankSteeredLift, BlackoutTrigger, BranchAccumulator, BurnEnvelope,
    CoupledField, CyberneticCorrect, GoverningModel, InsErrorState, MachRegime, NavFilter,
    PhysicsStage, ReentryNavEngine, RegimeClass, RegimeClassify, SafetyEnvelope, StepContext,
    ThrustState, TrajectoryNav,
};
use deep_causality_haft::LogSize;
use deep_causality_physics::EARTH_GM;

fn field() -> CoupledField<f64> {
    CoupledField::new(Ambient::new(0.01_f64, 0.0, None))
}

fn ctx(step: usize) -> StepContext<'static, 2, f64> {
    StepContext::<2, f64>::qtt(0.1, step)
}

// A band that denies the link for any real plasma (ω_p ≫ 1 rad/s for any positive n_e).
fn denying_trigger() -> BlackoutTrigger<f64> {
    BlackoutTrigger::new(1.0)
}

// ---------------------------------------------------------------------------
// GoverningModel
// ---------------------------------------------------------------------------

#[test]
fn governing_model_names_are_stable() {
    assert_eq!(GoverningModel::Continuum.name(), "continuum");
    assert_eq!(GoverningModel::Slip.name(), "slip");
    assert_eq!(GoverningModel::Transitional.name(), "transitional");
    assert_eq!(GoverningModel::FreeMolecular.name(), "free-molecular");
}

// ---------------------------------------------------------------------------
// RegimeClassify (3.1)
// ---------------------------------------------------------------------------

#[test]
fn classify_is_a_noop_without_mean_free_path() {
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert!(f.regime().is_none(), "no classification without λ");
    assert!(f.log().is_empty(), "nothing logged");
}

#[test]
fn classify_selects_each_knudsen_band() {
    // L = 1 m, so Kn = λ directly. One representative λ per band.
    let cases = [
        (0.005_f64, GoverningModel::Continuum),
        (0.05, GoverningModel::Slip),
        (1.0, GoverningModel::Transitional),
        (20.0, GoverningModel::FreeMolecular),
    ];
    for (lambda, expected) in cases {
        let stage = RegimeClassify::new(1.0, denying_trigger());
        let mut f = field();
        f.set_scalar("mean_free_path", vec![lambda * 0.5, lambda]); // peak is `lambda`
        stage.apply(&ctx(0), &mut f).expect("applies");
        let class = f.regime().expect("classified");
        assert_eq!(class.model, expected, "λ={lambda}");
        assert!((class.knudsen - lambda).abs() < 1e-12);
    }
}

#[test]
fn classify_uses_configured_thresholds() {
    // Push the slip band up to 1.0: a Kn of 0.5 is now still continuum.
    let stage = RegimeClassify::new(1.0, denying_trigger()).with_thresholds(1.0, 5.0, 50.0);
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.5]);
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.regime().unwrap().model, GoverningModel::Continuum);
}

#[test]
fn classify_flags_gnss_denial_from_electron_density() {
    let stage = RegimeClassify::new(1.0, denying_trigger());

    // Dense plasma → denied.
    let mut denied = field();
    denied.set_scalar("mean_free_path", vec![0.005]);
    denied.set_scalar("n_e", vec![1.0e18]);
    stage.apply(&ctx(0), &mut denied).expect("applies");
    let c = denied.regime().unwrap();
    assert!(c.gnss_denied, "dense plasma denies the link");
    assert!(c.plasma_frequency > 0.0);

    // No plasma → available.
    let mut avail = field();
    avail.set_scalar("mean_free_path", vec![0.005]);
    avail.set_scalar("n_e", vec![0.0]);
    stage.apply(&ctx(0), &mut avail).expect("applies");
    let c = avail.regime().unwrap();
    assert!(!c.gnss_denied, "no plasma leaves the link available");
    assert_eq!(c.plasma_frequency, 0.0);
}

#[test]
fn classify_logs_only_genuine_regime_changes() {
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();

    // First classification is always a change (None -> Continuum): one entry.
    f.set_scalar("mean_free_path", vec![0.005]);
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    // Re-applying with the same regime logs nothing new.
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1, "unchanged regime is not re-logged");

    // A band change (Continuum -> FreeMolecular) logs a second entry.
    f.set_scalar("mean_free_path", vec![20.0]);
    stage.apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "a genuine transition is logged");
}

#[test]
fn classify_logs_a_comms_denial_transition() {
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("n_e", vec![0.0]);
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    // Same flow band, but now denied: a regime change (the key includes comms denial).
    f.set_scalar("n_e", vec![1.0e18]);
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2);
}

// ---------------------------------------------------------------------------
// BranchAccumulator / BranchOutcome (3.2)
// ---------------------------------------------------------------------------

#[test]
fn branch_accumulator_folds_peak_load_and_dwell() {
    let mut acc = BranchAccumulator::<f64>::new(0.35);
    acc.observe(100.0, false, 0.1); // heat 100, comms up
    acc.observe(300.0, true, 0.1); // heat 300 (new peak), denied
    acc.observe(200.0, true, 0.2); // heat 200, denied
    let out = acc.finish(42.0);

    assert_eq!(out.bank_angle, 0.35);
    assert_eq!(out.peak_heat_flux, 300.0);
    // thermal load = 100*0.1 + 300*0.1 + 200*0.2 = 10 + 30 + 40 = 80.
    assert!((out.thermal_load - 80.0).abs() < 1e-9);
    // dwell = 0.1 + 0.2 = 0.3 (only the denied steps).
    assert!((out.blackout_dwell - 0.3).abs() < 1e-9);
    assert_eq!(out.miss_distance, 42.0);
}

#[test]
fn branch_accumulator_with_no_steps_is_zero() {
    let out = BranchAccumulator::<f64>::new(0.0).finish(5.0);
    assert_eq!(out.peak_heat_flux, 0.0);
    assert_eq!(out.thermal_load, 0.0);
    assert_eq!(out.blackout_dwell, 0.0);
    assert_eq!(out.miss_distance, 5.0);
}

// ---------------------------------------------------------------------------
// CyberneticCorrect (3.3)
// ---------------------------------------------------------------------------

fn envelope() -> SafetyEnvelope<f64> {
    // heat ≤ 1e6 W/m², g ≤ 12, |bank| ≤ 0.5 rad.
    SafetyEnvelope::new(1.0e6, 12.0, 0.5)
}

fn gate() -> CyberneticCorrect<f64> {
    CyberneticCorrect::new(envelope())
}

#[test]
fn correction_clamps_bank_into_the_envelope() {
    let mut f = field();
    f.set_scalar("heat_flux", vec![1.0e5]); // within
    f.set_scalar("g_load", vec![3.0]); // within
    f.set_control_action(1.2); // desired bank beyond the 0.5 cap

    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(0.5), "bank clamped to +max");
    assert_eq!(f.log().len(), 1, "the bounding is logged");
}

#[test]
fn correction_clamps_negative_bank() {
    let mut f = field();
    f.set_control_action(-3.0);
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(-0.5), "bank clamped to -max");
}

#[test]
fn in_envelope_command_passes_through_unchanged() {
    let mut f = field();
    f.set_scalar("heat_flux", vec![2.0e5]);
    f.set_control_action(0.2); // already inside [-0.5, 0.5]
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(0.2));
    assert!(f.log().is_empty(), "an unchanged command is not logged");
}

#[test]
fn heat_breach_returns_entropy_and_logs() {
    let mut f = field();
    f.set_scalar("heat_flux", vec![2.0e6]); // above the 1e6 ceiling
    f.set_control_action(0.1);
    let result = gate().apply(&ctx(0), &mut f);
    assert!(result.is_err(), "an unrecoverable breach short-circuits");
    assert_eq!(f.log().len(), 1);
}

#[test]
fn g_load_breach_returns_entropy() {
    let mut f = field();
    f.set_scalar("g_load", vec![20.0]); // above the 12 g ceiling
    f.set_control_action(0.1);
    assert!(gate().apply(&ctx(0), &mut f).is_err());
}

// ---------------------------------------------------------------------------
// CyberneticCorrect — powered-descent burn axes (powered-descent-envelope)
// ---------------------------------------------------------------------------

// Burn envelope: throttle ∈ [0.1, 0.9], max_ct 2.0, ignition q window [1000, 5000],
// propellant floor 10 kg, max descent rate 100 m/s. Sensing: thrust_ref 2000 N, S_ref 0.785 m².
// The dynamic cap is ct_ceiling = max_ct·q·S_ref/thrust_ref = q·7.85e-4.
fn burn_gate() -> CyberneticCorrect<f64> {
    let envelope = SafetyEnvelope::new(1.0e6, 12.0, 0.5).with_burn(BurnEnvelope::new(
        0.1, 0.9, 2.0, 1000.0, 5000.0, 10.0, 100.0,
    ));
    CyberneticCorrect::new(envelope).with_burn_sensing(
        "q_inf",
        "propellant",
        "descent_rate",
        2000.0,
        0.785,
    )
}

#[test]
fn dynamic_ct_cap_binds_below_the_static_ceiling() {
    let mut f = field();
    f.set_scalar("q_inf", vec![1_000.0]); // ct_ceiling = 0.785 < static 0.9
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(0.85); // inside the static ceiling, above the dynamic cap
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        Some(0.785),
        "throttle capped by the C_T ceiling"
    );
    assert_eq!(f.log().len(), 1, "the dynamic cap is logged");
}

#[test]
fn static_ceiling_binds_when_dynamic_pressure_is_high() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]); // ct_ceiling = 1.57 > static 0.9
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(1.2); // above the static ceiling
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        Some(0.9),
        "throttle capped by the static ceiling"
    );
}

#[test]
fn throttle_below_the_floor_clamps_up() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(0.05); // below the 0.1 floor
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.throttle_action(), Some(0.1));
}

#[test]
fn propellant_floor_breach_refuses_not_clamps() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![5.0]); // at/below the 10 kg floor
    f.set_throttle_action(0.5); // positive throttle commanded
    let result = burn_gate().apply(&ctx(0), &mut f);
    assert!(result.is_err(), "a propellant-floor breach short-circuits");
    assert!(!f.log().is_empty(), "the breach is logged");
}

#[test]
fn descent_rate_bound_breach_returns_entropy() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![100.0]);
    f.set_scalar("descent_rate", vec![150.0]); // above the 100 m/s bound
    f.set_throttle_action(0.5);
    assert!(burn_gate().apply(&ctx(0), &mut f).is_err());
}

#[test]
fn burn_none_leaves_the_throttle_channel_untouched() {
    // With `burn: None` the gate never reads or writes the throttle channel: a throttle written
    // upstream survives verbatim and no burn-related provenance appears (the corridor gate path).
    let mut f = field();
    f.set_throttle_action(5.0); // an absurd command the burn gate would clamp — but burn is None
    f.set_control_action(0.2); // inside the bank envelope
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        Some(5.0),
        "throttle untouched with burn: None"
    );
    assert_eq!(f.control_action(), Some(0.2));
    assert!(f.log().is_empty(), "no burn log traffic");
}

#[test]
fn burn_axes_without_a_throttle_command_are_inert() {
    // Burn axes present, but no throttle write: the throttle channel stays absent and no
    // burn-related log appears (only the ordinary bank path runs).
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_control_action(0.2);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        None,
        "no throttle command ⇒ no throttle write"
    );
    assert!(f.log().is_empty(), "no burn log traffic without a command");
}

#[test]
fn absent_sensor_fields_are_treated_as_zero_and_safe() {
    let mut f = field();
    f.set_control_action(0.3);
    // No heat_flux / g_load fields → sensed as zero → inside envelope.
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(0.3));
}

#[test]
fn gate_is_deterministic() {
    let build = || {
        let mut f = field();
        f.set_scalar("heat_flux", vec![5.0e5]);
        f.set_scalar("g_load", vec![6.0]);
        f.set_control_action(0.9);
        f
    };
    let mut a = build();
    let mut b = build();
    gate().apply(&ctx(0), &mut a).expect("applies");
    gate().apply(&ctx(0), &mut b).expect("applies");
    assert_eq!(a.control_action(), b.control_action());
    assert_eq!(a.control_action(), Some(0.5));
}

#[test]
fn bank_correction_value_equality() {
    assert_eq!(
        BankCorrection::Clamped(0.5_f64),
        BankCorrection::Clamped(0.5)
    );
    assert_ne!(
        BankCorrection::Clamped(0.5_f64),
        BankCorrection::NoSafeAction
    );
}

// ---------------------------------------------------------------------------
// TrajectoryNav (4.2)
// ---------------------------------------------------------------------------

fn nav_engine() -> ReentryNavEngine<f64> {
    // The bound LEO-ish state the nav module's own tests use.
    let (r0, v0) = ([7.0e6, 1.0e6, 2.0e6], [-1.0e3, 6.5e3, 3.0e3]);
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]);
    ReentryNavEngine::new(r0, v0, EARTH_GM, filter)
}

fn nav_stage() -> TrajectoryNav<f64> {
    // Q diagonal, GNSS 5 m 1σ (25 m² variance), through-plasma optical 50 m 1σ.
    TrajectoryNav::new([1.0e-6; 17], 25.0, 2500.0)
}

fn denied_regime() -> RegimeClass<f64> {
    RegimeClass {
        model: GoverningModel::Continuum,
        knudsen: 1.0e-3,
        plasma_frequency: 1.0e10,
        gnss_denied: true,
        // A corridor-class regime carries the powered-descent axes neutral.
        mach_regime: MachRegime::Unknown,
        thrust_state: ThrustState::Unknown,
        touchdown: false,
    }
}

#[test]
fn trajectory_nav_is_a_noop_without_an_engine() {
    let mut f = field();
    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert!(f.scalar("nav_position").is_none());
    assert!(f.log().is_empty());
}

#[test]
fn trajectory_nav_dead_reckons_and_publishes_witnesses() {
    let mut f = field();
    f.set_nav(nav_engine());

    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.scalar("nav_position").unwrap().len(), 3);
    assert_eq!(f.scalar("nav_position_variance").unwrap().len(), 1);
    assert!(f.nav().is_some(), "the engine threads back into the field");
    assert_eq!(f.log().len(), 1, "the dead-reckoning transition is logged");

    // A second dead-reckoning step is not a transition: no new entry.
    nav_stage().apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);
}

#[test]
fn trajectory_nav_reads_the_aero_force_channel() {
    let stage = nav_stage();

    let mut coasting = field();
    coasting.set_nav(nav_engine());
    stage.apply(&ctx(1), &mut coasting).expect("applies");

    let mut dragged = field();
    dragged.set_nav(nav_engine());
    dragged.set_aero_force([-1.0e3, 0.0, 0.0]);
    stage.apply(&ctx(1), &mut dragged).expect("applies");

    let a = coasting.scalar("nav_position").unwrap();
    let b = dragged.scalar("nav_position").unwrap();
    assert!(
        (a[0] - b[0]).abs() > 1.0,
        "the ④ aero kick perturbs the predicted position: {} vs {}",
        a[0],
        b[0]
    );
}

#[test]
fn gnss_fix_is_folded_when_available() {
    let stage = nav_stage();

    // Twin runs: one dead-reckons, one folds a GNSS fix at the propagated position.
    let mut dead = field();
    dead.set_nav(nav_engine());
    stage.apply(&ctx(1), &mut dead).expect("applies");

    let mut aided = field();
    aided.set_nav(nav_engine());
    let predicted = dead.scalar("nav_position").unwrap().to_vec();
    aided.set_scalar("gnss_fix", predicted);
    stage.apply(&ctx(1), &mut aided).expect("applies");

    assert_eq!(aided.scalar("nav_mode").unwrap()[0], 1.0, "aided");
    assert_eq!(dead.scalar("nav_mode").unwrap()[0], 0.0, "dead reckoning");
    // The fix collapses the position variance below the predict-only twin.
    let v_aided = aided.scalar("nav_position_variance").unwrap()[0];
    let v_dead = dead.scalar("nav_position_variance").unwrap()[0];
    assert!(
        v_aided < v_dead,
        "fix collapses variance: {v_aided} vs {v_dead}"
    );
}

#[test]
fn gnss_fix_is_gated_by_the_denial_flag() {
    let mut f = field();
    f.set_nav(nav_engine());
    f.set_regime(denied_regime());
    f.set_scalar("gnss_fix", vec![7.0e6, 1.0e6, 2.0e6]);

    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(
        f.scalar("nav_mode").unwrap()[0],
        0.0,
        "a denied link never folds the GNSS fix"
    );
    assert!(
        f.scalar("gnss_fix").is_none(),
        "the denied-step broadcast is consumed unread, not latched for reacquisition"
    );
}

#[test]
fn optical_fix_rides_through_the_blackout() {
    let mut f = field();
    f.set_nav(nav_engine());
    f.set_regime(denied_regime());
    f.set_scalar("optical_fix", vec![7.0e6, 1.0e6, 2.0e6]);

    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(
        f.scalar("nav_mode").unwrap()[0],
        1.0,
        "the through-plasma optical fix folds even when GNSS is denied"
    );
    assert!(
        f.scalar("optical_fix").is_none(),
        "the folded optical fix is consumed"
    );
}

#[test]
fn a_fix_is_consumed_and_not_refolded_when_the_publisher_goes_quiet() {
    let stage = nav_stage();
    let mut f = field();
    f.set_nav(nav_engine());

    // Step 1: a published fix is folded (aided) and consumed off the field.
    f.set_scalar("gnss_fix", vec![7.0e6, 1.0e6, 2.0e6]);
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.scalar("nav_mode").unwrap()[0], 1.0, "aided");
    assert!(f.scalar("gnss_fix").is_none(), "the fix is consumed");
    let v_folded = f.scalar("nav_position_variance").unwrap()[0];

    // Step 2: the publisher goes quiet. The old fix must not be re-folded as fresh — the
    // filter drops to dead reckoning and the position variance grows instead of collapsing.
    stage.apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.scalar("nav_mode").unwrap()[0], 0.0, "dead reckoning");
    let v_quiet = f.scalar("nav_position_variance").unwrap()[0];
    assert!(
        v_quiet > v_folded,
        "a quiet publisher grows the variance: {v_quiet} vs {v_folded}"
    );
}

#[test]
fn with_imu_senses_the_specific_force_through_the_bias() {
    use deep_causality_cfd::ImuModel;

    // Twin engines, same true aero force. The biased IMU's dead-reckoned position diverges from
    // the unbiased twin: the real INS drift mechanism.
    let aero = [-30.0_f64, 0.0, 0.0];
    let imu = ImuModel::new([0.5, -0.3, 0.2], [0.0; 3], [1.0e-4; 17]);

    let mut clean = field();
    clean.set_nav(nav_engine());
    clean.set_aero_force(aero);
    nav_stage().apply(&ctx(1), &mut clean).expect("applies");

    let mut biased = field();
    biased.set_nav(nav_engine());
    biased.set_aero_force(aero);
    TrajectoryNav::new([1.0e-6; 17], 25.0, 2500.0)
        .with_imu(imu)
        .apply(&ctx(1), &mut biased)
        .expect("applies");

    let a = clean.scalar("nav_position").unwrap();
    let b = biased.scalar("nav_position").unwrap();
    assert!(
        (a[0] - b[0]).abs() > 1e-6 || (a[1] - b[1]).abs() > 1e-6,
        "the accelerometer bias drifts the dead-reckoned position"
    );
}

#[test]
fn nav_predict_failure_short_circuits_but_threads_the_engine_back() {
    // An unbound (hyperbolic) state cannot re-lift onto the KS manifold: predict fails.
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]);
    let unbound = ReentryNavEngine::new([7.0e6, 0.0, 0.0], [1.0e5, 0.0, 0.0], EARTH_GM, filter);

    let mut f = field();
    f.set_nav(unbound);
    let result = nav_stage().apply(&ctx(1), &mut f);
    assert!(result.is_err(), "an unbound predict short-circuits");
    assert!(
        f.nav().is_some(),
        "the engine threads back so a pause captures a whole state"
    );
}

// ---------------------------------------------------------------------------
// BankSteeredLift (the 3-DOF ④ producer)
// ---------------------------------------------------------------------------

// A truth vehicle on the +x radial flying tangentially along +y: the lift plane's
// in-plane "up" is +x and the side direction v̂ × n̂ is −z.
fn steered_field(bank: Option<f64>) -> CoupledField<f64> {
    let mut f = field();
    f.set_scalar("speed", vec![100.0]);
    f.set_scalar("truth_state", vec![7.0e6, 0.0, 0.0, 0.0, 7.5e3, 0.0]);
    if let Some(b) = bank {
        f.set_control_action(b);
    }
    f
}

// rho_ref = 1, C_d·A/m = 1, L/D = 1 over U_max = 100: a_drag = a_lift = q = 5000.
fn steered_stage() -> BankSteeredLift<f64> {
    BankSteeredLift::new(1.0, 1.0, 1.0)
}

#[test]
fn bank_steered_lift_writes_zero_force_without_speed() {
    let mut f = field();
    f.set_scalar("truth_state", vec![7.0e6, 0.0, 0.0, 0.0, 7.5e3, 0.0]);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(
        f.aero_force(),
        Some([0.0, 0.0, 0.0]),
        "no dynamic pressure, zero force"
    );
}

#[test]
fn missing_speed_zeroes_a_previously_written_force() {
    // Step 1 writes a real aero force; step 2's speed field is gone. The stale force must not
    // stay latched and keep kicking the trajectory: the stage zeroes the ④ channel.
    let mut f = steered_field(None);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_ne!(f.aero_force(), Some([0.0, 0.0, 0.0]), "a real force landed");

    assert!(f.take_scalar("speed").is_some(), "the publisher goes quiet");
    steered_stage().apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(
        f.aero_force(),
        Some([0.0, 0.0, 0.0]),
        "no force this step, not the latched one"
    );
}

#[test]
fn bank_steered_lift_falls_back_to_axis_drag_without_a_truth_state() {
    let mut f = field();
    f.set_scalar("speed", vec![100.0]);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    let a = f.aero_force().expect("force written");
    assert_eq!(a, [-5000.0, 0.0, 0.0], "the AeroForceCoupling behavior");
}

#[test]
fn zero_bank_keeps_the_lift_in_plane() {
    let mut f = steered_field(None);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    let a = f.aero_force().expect("force written");
    // Drag opposes +y; the zero-bank lift points up the local radial (+x); nothing leaves
    // the orbital plane.
    assert!((a[0] - 5000.0).abs() < 1e-9, "lift up the radial: {}", a[0]);
    assert!((a[1] + 5000.0).abs() < 1e-9, "drag against v: {}", a[1]);
    assert!(a[2].abs() < 1e-9, "in-plane at zero bank: {}", a[2]);
}

#[test]
fn opposite_banks_curve_the_trajectory_oppositely() {
    let bank = 0.5_f64;
    let mut left = steered_field(Some(bank));
    steered_stage().apply(&ctx(1), &mut left).expect("applies");
    let mut right = steered_field(Some(-bank));
    steered_stage().apply(&ctx(1), &mut right).expect("applies");

    let al = left.aero_force().expect("force");
    let ar = right.aero_force().expect("force");
    assert!(al[2] != 0.0, "banking leaves the plane");
    assert!(
        (al[2] + ar[2]).abs() < 1e-9,
        "mirror banks push out-of-plane oppositely: {} vs {}",
        al[2],
        ar[2]
    );
    // The in-plane lift shrinks by cos φ identically on both.
    assert!((al[0] - ar[0]).abs() < 1e-9);
    assert!((al[0] - 5000.0 * bank.cos()).abs() < 1e-6);
}

#[test]
fn the_clamped_command_actuates_not_the_raw_one() {
    // A raw guidance command far beyond the envelope's bank cap: the gate clamps the channel,
    // and the next step's lift flies the clamped value (the one-step actuation lag).
    let cap = 0.2_f64;
    let gate = CyberneticCorrect::new(SafetyEnvelope::new(1.0e9, 100.0, cap));

    let mut f = steered_field(Some(10.0));
    gate.apply(&ctx(1), &mut f).expect("gate clamps");
    steered_stage().apply(&ctx(2), &mut f).expect("applies");
    let clamped = f.aero_force().expect("force");

    let mut reference = steered_field(Some(cap));
    steered_stage()
        .apply(&ctx(2), &mut reference)
        .expect("applies");
    let expected = reference.aero_force().expect("force");

    assert_eq!(
        clamped, expected,
        "the actuated bank is the gate's clamp, not the raw command"
    );
}

#[test]
fn finish_at_derives_the_miss_from_the_terminal_state() {
    let aim = [6.4e6_f64, 0.0, 0.0];

    // On the aim point: zero miss.
    let on_target = BranchAccumulator::new(0.0).finish_at(aim, aim);
    assert_eq!(on_target.miss_distance, 0.0);

    // Distinct terminal states (distinct banks steer distinct trajectories) yield
    // distinct, dynamics-derived misses.
    let short = BranchAccumulator::new(0.3).finish_at([6.4e6, 3.0e3, -4.0e3], aim);
    let wide = BranchAccumulator::new(-0.3).finish_at([6.4e6, 9.0e3, -1.2e4], aim);
    assert_eq!(short.miss_distance, 5.0e3);
    assert_eq!(wide.miss_distance, 1.5e4);
    assert!(short.miss_distance != wide.miss_distance);
}

// ---------------------------------------------------------------------------
// RegimeClassify — powered-descent flight axes (flight-regime-classifier)
// ---------------------------------------------------------------------------

/// A classifier with explicit flight bands: subsonic ≤ 0.8, supersonic ≥ 1.2, touchdown ≤ 10 m.
fn flight_classifier() -> RegimeClassify<f64> {
    RegimeClassify::new(1.0, denying_trigger()).with_flight_axes(0.8, 1.2, 10.0)
}

#[test]
fn each_flight_axis_reads_its_published_scalar() {
    let cases = [
        (
            2.5_f64,
            1.0_f64,
            50_000.0_f64,
            MachRegime::Supersonic,
            ThrustState::Burn,
            false,
        ),
        (
            1.0,
            0.0,
            50_000.0,
            MachRegime::Transonic,
            ThrustState::Coast,
            false,
        ),
        (
            0.5,
            0.0,
            5.0,
            MachRegime::Subsonic,
            ThrustState::Coast,
            true,
        ),
    ];
    for (mach, ignited, alt, want_mach, want_thrust, want_touchdown) in cases {
        let mut f = field();
        f.set_scalar("mean_free_path", vec![0.005]);
        f.set_scalar("flight_mach", vec![mach]);
        f.set_scalar("ignited", vec![ignited]);
        f.set_scalar("flight_altitude", vec![alt]);
        flight_classifier().apply(&ctx(0), &mut f).expect("applies");
        let c = f.regime().expect("classified");
        assert_eq!(c.mach_regime, want_mach, "mach {mach}");
        assert_eq!(c.thrust_state, want_thrust, "ignited {ignited}");
        assert_eq!(c.touchdown, want_touchdown, "altitude {alt}");
    }
}

#[test]
fn the_corridor_classification_is_unchanged_without_the_opt_in() {
    // The compressible carrier publishes "flight_mach" every step, so neutrality cannot depend on
    // the scalar being absent: the flight axes are **opt-in**. A classifier built without
    // `with_flight_axes` ignores the published flight scalars entirely, so the regime key reduces
    // to today's (model, gnss_denied) pair and the logged message is exactly the pre-change text.
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("n_e", vec![0.0]);
    f.set_scalar("flight_mach", vec![2.5]); // published, as the carrier really does
    f.set_scalar("ignited", vec![1.0]);
    f.set_scalar("flight_altitude", vec![5.0]);
    RegimeClassify::new(1.0, denying_trigger())
        .apply(&ctx(0), &mut f)
        .expect("applies");

    let c = f.regime().expect("classified");
    assert_eq!(c.mach_regime, MachRegime::Unknown);
    assert_eq!(c.thrust_state, ThrustState::Unknown);
    assert!(!c.touchdown);
    assert_eq!(f.log().len(), 1);
    let msg: Vec<&str> = f.log().messages().collect();
    assert!(
        msg[0].starts_with("regime -> continuum (GNSS-available), Kn="),
        "pre-change message text preserved: {}",
        msg[0]
    );
    assert!(
        !msg[0].contains("mach-unknown"),
        "no flight-phase suffix when the axes are neutral: {}",
        msg[0]
    );
}

#[test]
fn a_mach_crossing_under_thrust_logs_once() {
    let stage = flight_classifier();
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("ignited", vec![1.0]);
    f.set_scalar("flight_mach", vec![2.5]); // supersonic
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    // Same band on the next step: nothing new.
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1, "an unchanged band is not re-logged");

    // Cross into transonic: one new entry.
    f.set_scalar("flight_mach", vec![1.0]);
    stage.apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "a Mach crossing is a regime change");
}

#[test]
fn a_burn_to_coast_transition_logs() {
    let stage = flight_classifier();
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("flight_mach", vec![2.5]);
    f.set_scalar("ignited", vec![1.0]); // burn
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    f.set_scalar("ignited", vec![0.0]); // cutoff → coast, same Mach band
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "burn↔coast is a regime change");
}

#[test]
fn a_touchdown_logs_and_appears_in_the_message() {
    let stage = flight_classifier();
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("flight_mach", vec![0.5]);
    f.set_scalar("ignited", vec![0.0]);
    f.set_scalar("flight_altitude", vec![100.0]); // above the floor
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    f.set_scalar("flight_altitude", vec![5.0]); // at/below the 10 m floor
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "touchdown is a regime change");
    let msg: Vec<&str> = f.log().messages().collect();
    assert!(
        msg[1].contains("touchdown"),
        "the phase suffix names it: {}",
        msg[1]
    );
}

// ---------------------------------------------------------------------------
// CyberneticCorrect — M4 live enforcement (powered-descent-envelope)
// ---------------------------------------------------------------------------

/// A field inside every burn axis, with the engine not yet lit.
fn burn_field() -> CoupledField<f64> {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![100.0]);
    f
}

fn log_text(f: &CoupledField<f64>) -> String {
    f.log().messages().collect::<Vec<_>>().join(" | ")
}

#[test]
fn ignition_outside_the_dynamic_pressure_window_refuses() {
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![200.0]); // below the 1000 Pa window floor
    f.set_throttle_action(0.5); // a throttle rising from zero: the engine is not lit
    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("dynamic-pressure window"));
    assert!(log_text(&f).contains("ignition dynamic pressure"));
}

#[test]
fn ignition_inside_the_window_is_admitted() {
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![1_500.0]);
    f.set_throttle_action(0.5);
    burn_gate()
        .apply(&ctx(0), &mut f)
        .expect("inside the window");
}

#[test]
fn the_window_does_not_bound_a_burn_already_under_way() {
    // The engine is lit, so `q∞` leaving the window as the vehicle decelerates is not an ignition
    // decision and must not refuse — the running axes bound a burn in progress, not this one.
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![200.0]);
    f.set_scalar("ignited", vec![1.0]);
    f.set_throttle_action(0.5);
    burn_gate()
        .apply(&ctx(0), &mut f)
        .expect("a running burn is not re-gated on the ignition window");
}

#[test]
fn an_absent_dynamic_pressure_sensor_does_not_trip_the_window() {
    // Absent sensors read as zero and stay safe; the producer-side gap is the flight-sensor
    // stage's business, not a reason to fail closed here.
    let mut f = field();
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(0.5);
    burn_gate()
        .apply(&ctx(0), &mut f)
        .expect("absent q is safe");
}

#[test]
fn a_crossed_throttle_window_refuses_rather_than_choosing_a_bound() {
    // ct_ceiling = q·7.85e-4; at q = 100 Pa that is 0.0785, below the 0.1 floor.
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![100.0]);
    f.set_scalar("ignited", vec![1.0]); // isolate the crossed window from the ignition window
    f.set_throttle_action(0.5); // at/above the floor: the pre-change code clamped DOWN past it
    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("throttle window crossed"));
}

#[test]
fn neither_crossed_window_branch_emits_an_out_of_envelope_throttle() {
    // The pre-change clamp tested its lower bound first, so a command BETWEEN the crossed bounds
    // was pushed UP past the C_T cap while one at or above the floor was pushed DOWN below the
    // floor. Both must now refuse, and neither may leave a bounded command behind.
    for commanded in [0.09_f64, 0.5] {
        let mut f = burn_field();
        f.set_scalar("q_inf", vec![100.0]);
        f.set_scalar("ignited", vec![1.0]);
        f.set_throttle_action(commanded);
        assert!(
            burn_gate().apply(&ctx(0), &mut f).is_err(),
            "commanded {commanded} must refuse on a crossed window"
        );
        assert_eq!(
            f.throttle_action(),
            Some(commanded),
            "a refused step leaves no bounded command"
        );
    }
}

#[test]
fn simultaneous_breaches_are_all_logged_and_the_first_is_returned() {
    // A heat breach used to return before the burn block was reached, so the propellant breach on
    // the same step was never logged. Both must appear; the error names the first in axis order.
    let mut f = burn_field();
    f.set_scalar("heat_flux", vec![2.0e6]); // above the 1e6 ceiling
    f.set_scalar("propellant", vec![5.0]); // at/below the 10 kg floor
    f.set_scalar("ignited", vec![1.0]);
    f.set_throttle_action(0.5);

    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    let log = log_text(&f);
    assert!(log.contains("no recoverable bank correction"), "log: {log}");
    assert!(log.contains("propellant"), "log: {log}");
    assert!(format!("{err:?}").contains("no recoverable bank correction"));
}

#[test]
fn the_returned_error_is_deterministic_across_runs() {
    let build = || {
        let mut f = burn_field();
        f.set_scalar("heat_flux", vec![2.0e6]);
        f.set_scalar("descent_rate", vec![150.0]);
        f.set_scalar("ignited", vec![1.0]);
        f.set_throttle_action(0.5);
        f
    };
    let mut a = build();
    let mut b = build();
    let ea = burn_gate().apply(&ctx(0), &mut a).unwrap_err();
    let eb = burn_gate().apply(&ctx(0), &mut b).unwrap_err();
    assert_eq!(format!("{ea:?}"), format!("{eb:?}"));
}

#[test]
fn scalar_driven_thrust_with_burn_axes_attached_refuses() {
    // The propulsion stages honour the published scalar; the gate senses the channel alone. A world
    // driving only the scalar would otherwise fly its full propulsion path unenforced.
    let mut f = burn_field();
    f.set_scalar("commanded_throttle", vec![0.7]);
    // No `set_throttle_action`: the channel is absent.
    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("not on the throttle channel"));
    assert!(log_text(&f).contains("cannot enforce a throttle it cannot see"));
}

#[test]
fn the_gate_never_writes_the_channel_from_a_scalar_source() {
    // Writing the channel here would outrank the world's published constant on every later step,
    // freezing a counterfactual branch at its first clamped value.
    let mut f = burn_field();
    f.set_scalar("commanded_throttle", vec![0.7]);
    let _ = burn_gate().apply(&ctx(0), &mut f);
    assert_eq!(
        f.throttle_action(),
        None,
        "the counterfactual seam must survive the gate"
    );
}

#[test]
fn a_zero_published_scalar_with_burn_axes_stays_silent() {
    // Neither seam driven: the inactive-axes guarantee holds.
    let mut f = burn_field();
    f.set_scalar("commanded_throttle", vec![0.0]);
    f.set_control_action(0.2);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert!(f.log().is_empty(), "no new log traffic: {}", log_text(&f));
}

#[test]
fn burn_axes_without_burn_sensing_refuse_rather_than_silently_disabling_the_cap() {
    let envelope = SafetyEnvelope::new(1.0e6, 12.0, 0.5).with_burn(BurnEnvelope::new(
        0.1, 0.9, 2.0, 1000.0, 5000.0, 10.0, 100.0,
    ));
    // No `with_burn_sensing`: thrust_ref and s_ref stay zero, so the dynamic C_T cap cannot bind.
    let gate = CyberneticCorrect::new(envelope);
    let mut f = burn_field();
    f.set_scalar("ignited", vec![1.0]);
    f.set_throttle_action(0.5);

    let err = gate.apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("with_burn_sensing"));
}

#[test]
fn a_commanded_shutdown_is_not_clamped_up_to_the_throttle_floor() {
    // The floor is a stability constraint for a running engine — below it the central-nozzle
    // jet-penetration flow is unsteady — so it bounds how softly a burn may run, not whether one
    // must run at all. Clamping a commanded zero up to the floor would light the engine.
    let mut f = burn_field();
    f.set_throttle_action(0.0);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.throttle_action(), Some(0.0));
    assert!(f.log().is_empty(), "a shutdown needs no bounding entry");
}

#[test]
fn a_positive_throttle_below_the_floor_still_clamps_up() {
    // The shutdown carve-out must not weaken the floor for a genuinely commanded burn.
    let mut f = burn_field();
    f.set_throttle_action(0.02);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.throttle_action(), Some(0.1));
}
