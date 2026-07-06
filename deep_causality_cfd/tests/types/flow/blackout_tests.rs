/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-A Park-2T plasma-blackout coupling: the LER closed-form mechanism, the
//! recovery-temperature reconstruction, ionization lag, and static stage composition.

use deep_causality_cfd::{
    Ambient, BlackoutTrigger, CoupledField, Coupling, EosStage, IonizationStage, PhysicsStage,
    RecoveryTemperatureStage, StepContext, VibrationalLagStage, ler_relax_scalar, ler_step,
};
use deep_causality_physics::ElectronDensity;
use deep_causality_physics::{
    SolenoidalField, Temperature, VelocityOneForm, park2t_ionization_surrogate_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn empty_context() -> (Manifold<LatticeComplex<2, f64>, f64>, SolenoidalField<f64>) {
    let n = 4;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    let n1 = manifold.complex().num_cells(1);
    let zero = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let velocity = VelocityOneForm::from_raw(zero);
    let (state, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();
    (manifold, state)
}

// ── LER closed-form helper ───────────────────────────────────────────────

#[test]
fn test_ler_step_exact_on_linear_relaxation() {
    let (x, x_eq, tau, dt) = (300.0_f64, 7000.0_f64, 0.01_f64, 0.003_f64);
    let expected = x_eq - (x_eq - x) * (-(dt / tau)).exp();
    assert_eq!(ler_step(x, x_eq, tau, dt), expected);
}

#[test]
fn test_ler_step_equilibrium_limit() {
    // τ → 0 (and the degenerate τ ≤ 0) jumps exactly to the target.
    assert_eq!(ler_step(300.0, 7000.0, 0.0, 0.1), 7000.0);
    assert_eq!(ler_step(300.0, 7000.0, -1.0, 0.1), 7000.0);
}

#[test]
fn test_ler_step_zero_dt_is_identity() {
    assert_eq!(ler_step(300.0, 7000.0, 0.01, 0.0), 300.0);
}

#[test]
fn test_ler_step_stable_at_stiffness_where_euler_diverges() {
    // τ = dt/1000: the LER update stays bounded in [x0, x_eq] and monotone,
    // where an explicit-Euler rate step massively overshoots.
    let (dt, x_eq) = (1.0_f64, 7000.0);
    let tau = dt / 1000.0;
    let mut x = 300.0_f64;
    for _ in 0..50 {
        let nx = ler_step(x, x_eq, tau, dt);
        assert!(nx >= x - 1e-6 && nx <= x_eq + 1e-6, "overshoot: {nx}");
        x = nx;
    }
    assert!((x - x_eq).abs() < 1.0);

    // Explicit Euler with the same stiffness ratio dt/τ = 1000 diverges.
    let euler = 300.0 + (dt / tau) * (x_eq - 300.0);
    assert!(euler > x_eq * 100.0);
}

#[test]
fn test_ler_relax_scalar_relaxes_named_field() {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("alpha", vec![0.0, 0.0]);
    ler_relax_scalar(&mut field, "alpha", 1.0, &[1.0, 0.5], &[0.0, 0.0]).unwrap();
    // τ = 0 ⇒ each cell jumps to its target.
    assert_eq!(field.scalar("alpha"), Some(&[1.0, 0.5][..]));
}

#[test]
fn test_ler_relax_scalar_absent_is_noop_and_mismatch_errors() {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    // Absent field is a no-op.
    ler_relax_scalar(&mut field, "alpha", 1.0, &[], &[]).unwrap();
    // Length mismatch is an error.
    field.set_scalar("alpha", vec![0.0, 0.0]);
    assert!(ler_relax_scalar(&mut field, "alpha", 1.0, &[1.0], &[0.0]).is_err());
}

// ── Recovery-temperature reconstruction ──────────────────────────────────

#[test]
fn test_recovery_temperature_reaches_ionization_band() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = RecoveryTemperatureStage::new(25.0, 1.4, 200.0, 1004.0);

    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("speed", vec![0.0, 1000.0, 2000.0]);
    stage.apply(&ctx, &mut field).unwrap();

    let t_tr = field.scalar("T_tr").expect("T_tr built");
    // The stagnation cell (u = 0) carries the full post-shock jump, in the ~10⁴ K band.
    assert!(t_tr[0] > 1.0e4, "T_post = {}", t_tr[0]);
    // Higher speed ⇒ more kinetic enthalpy removed ⇒ cooler.
    assert!(t_tr[1] > t_tr[2]);
}

// ── Ionization lag + electron production ─────────────────────────────────

#[test]
fn test_ionization_lags_then_catches_up() {
    let (manifold, state) = empty_context();
    let n_tot = 1.0e22_f64;
    let t_hot = 8000.0_f64;
    let alpha_eq = park2t_ionization_surrogate_kernel(Temperature::new(t_hot).unwrap(), n_tot)
        .unwrap()
        .value();
    let stage = IonizationStage::new(n_tot);

    // Small dt ≪ τ_ion: α lags well below equilibrium.
    let ctx_fast = StepContext::new(&manifold, &state, 1.0e-5, 1);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("T_tr", vec![t_hot]);
    stage.apply(&ctx_fast, &mut field).unwrap();
    let alpha_lagged = field.scalar("alpha").unwrap()[0];
    assert!(alpha_lagged < alpha_eq, "{alpha_lagged} vs {alpha_eq}");
    assert!(field.scalar("n_e").unwrap()[0] > 0.0);

    // A long dwell drives α toward equilibrium (gap closes).
    let ctx_slow = StepContext::new(&manifold, &state, 10.0, 2);
    stage.apply(&ctx_slow, &mut field).unwrap();
    let alpha_settled = field.scalar("alpha").unwrap()[0];
    assert!(alpha_settled > alpha_lagged);
    assert!((alpha_settled - alpha_eq).abs() < alpha_eq * 0.05);
}

#[test]
fn test_eos_stage_writes_pressure() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = EosStage::new(1.0e22_f64);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("T_tr", vec![7000.0]);
    stage.apply(&ctx, &mut field).unwrap();
    let p = field.scalar("pressure").expect("pressure written");
    assert!(p[0] > 0.0);
}

// ── Static composition (no dyn) ──────────────────────────────────────────

#[test]
fn test_static_composition_of_tier_a_stages() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-3, 1);

    // Recovery → ionization → EOS composes statically by cons-tuple.
    let coupling = Coupling::between_steps()
        .then(RecoveryTemperatureStage::new(25.0, 1.4, 200.0, 1004.0))
        .then(IonizationStage::new(1.0e22))
        .then(EosStage::new(1.0e22))
        .build();

    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("speed", vec![0.0, 500.0]);
    coupling.apply(&ctx, &mut field).expect("pipeline applies");

    assert!(field.scalar("T_tr").is_some());
    assert!(field.scalar("n_e").is_some());
    assert!(field.scalar("pressure").is_some());
}

// ── Blackout trigger ─────────────────────────────────────────────────────

#[test]
fn test_blackout_trigger_no_plasma_keeps_link() {
    // GPS L-band ≈ 1.5 GHz → ω ≈ 9.4e9 rad/s.
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    let state = trigger
        .evaluate(ElectronDensity::<f64>::new(0.0).unwrap())
        .unwrap();
    assert_eq!(state.plasma_frequency, 0.0);
    assert!(!state.denied);
}

#[test]
fn test_blackout_trigger_above_band_denies() {
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    // n_e = 1e18 ⇒ ω_p ≈ 5.6e10 rad/s > band ⇒ denied.
    let state = trigger
        .evaluate(ElectronDensity::<f64>::new(1.0e18).unwrap())
        .unwrap();
    assert!(state.plasma_frequency > 9.4e9);
    assert!(state.denied);
}

#[test]
fn test_blackout_trigger_below_band_keeps_link() {
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    // n_e = 1e15 ⇒ ω_p ≈ 1.8e9 rad/s < band ⇒ link available.
    let state = trigger
        .evaluate(ElectronDensity::<f64>::new(1.0e15).unwrap())
        .unwrap();
    assert!(state.plasma_frequency < 9.4e9);
    assert!(!state.denied);
}

#[test]
fn test_blackout_trigger_classify_is_pure_effect() {
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    let effect = trigger.classify(ElectronDensity::<f64>::new(1.0e18).unwrap());
    assert!(effect.is_ok());
    if let Some(state) = effect.value() {
        assert!(state.denied);
    } else {
        panic!("expected Value");
    }
}

// ── No-op guard branches (prerequisite scalar absent) ────────────────────

#[test]
fn test_recovery_temperature_without_speed_is_a_noop() {
    // No "speed" field present → the stage returns Ok without writing "T_tr".
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = RecoveryTemperatureStage::new(25.0, 1.4, 200.0, 1004.0);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    stage.apply(&ctx, &mut field).expect("noop without speed");
    assert!(field.scalar("T_tr").is_none());
}

#[test]
fn test_ionization_without_t_tr_is_a_noop() {
    // No "T_tr" field present → the ionization stage returns Ok without writing "alpha"/"n_e".
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = IonizationStage::new(1.0e22_f64);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    stage.apply(&ctx, &mut field).expect("noop without T_tr");
    assert!(field.scalar("alpha").is_none());
    assert!(field.scalar("n_e").is_none());
}

#[test]
fn test_eos_without_t_tr_is_a_noop() {
    // No "T_tr" field present → the EOS stage returns Ok without writing "pressure".
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = EosStage::new(1.0e22_f64);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    stage.apply(&ctx, &mut field).expect("noop without T_tr");
    assert!(field.scalar("pressure").is_none());
}

// ── Frozen-chemistry timescale (vanishing forward rate) ──────────────────

#[test]
fn test_ionization_frozen_chemistry_leaves_alpha_unchanged() {
    // At a very low temperature the Arrhenius forward rate exp(−T_a/T) underflows to zero,
    // so the concentration-scaled denominator vanishes and τ falls to the frozen branch
    // (τ ≫ dt). The LER step then leaves α essentially unchanged (no spurious equilibrium jump).
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = IonizationStage::new(1.0e22_f64);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("T_tr", vec![1.0_f64]); // 1 K ⇒ exp(−32400) == 0.0 ⇒ frozen τ
    stage
        .apply(&ctx, &mut field)
        .expect("frozen-chemistry step applies");
    let alpha = field
        .scalar("alpha")
        .expect("alpha seeded on first contact");
    // Cold start at α = 0; the frozen step keeps it at (essentially) zero.
    assert!(alpha[0].abs() < 1e-12, "frozen α stayed put: {}", alpha[0]);
    assert_eq!(field.scalar("n_e").expect("n_e written")[0], 0.0);
}

// ── classify error path (kernel overflow) ────────────────────────────────

#[test]
fn test_blackout_trigger_classify_propagates_kernel_error() {
    // An enormous but finite electron density overflows the plasma-frequency kernel to a
    // non-finite ω_p, which PlasmaFrequency::new rejects — classify carries the error effect.
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    let effect = trigger.classify(ElectronDensity::<f64>::new(f64::MAX).unwrap());
    assert!(
        effect.is_err(),
        "the kernel overflow surfaces as an error effect"
    );
    assert!(effect.value().is_none());
    // The plain-Result form errors the same way.
    assert!(
        trigger
            .evaluate(ElectronDensity::<f64>::new(f64::MAX).unwrap())
            .is_err()
    );
}

// ── VibrationalLagStage (the Park-2T rate-controlling temperature) ───────

#[test]
fn vibrational_lag_is_a_noop_without_t_tr() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    let stage = VibrationalLagStage::new(250.0_f64, 0.04, 7.0, 3393.0, 5.0e-6);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    stage.apply(&ctx, &mut field).expect("applies");
    assert!(field.scalar("T_ve").is_none());
    assert!(field.scalar("T_a").is_none());
}

#[test]
fn vibrational_lag_suppresses_the_controller_at_short_residence() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    // A short residence at low pressure leaves the bath cold: T_ve << T_tr, so T_a << T_tr.
    let stage = VibrationalLagStage::new(250.0_f64, 1.0e-3, 7.0, 3393.0, 1.0e-7);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("T_tr", vec![30_000.0_f64, 25_000.0]);
    stage.apply(&ctx, &mut field).expect("applies");

    let t_ve = field.scalar("T_ve").expect("T_ve written");
    let t_a = field.scalar("T_a").expect("T_a written");
    assert_eq!(t_ve.len(), 2);
    for (i, &t_tr) in [30_000.0_f64, 25_000.0].iter().enumerate() {
        assert!(t_ve[i] < t_tr, "the bath lags: {} < {t_tr}", t_ve[i]);
        assert!(
            t_a[i] < t_tr && t_a[i] > t_ve[i],
            "geometric mean sits between: {} in ({}, {t_tr})",
            t_a[i],
            t_ve[i]
        );
        let expected = (t_tr * t_ve[i]).sqrt();
        assert!((t_a[i] - expected).abs() < 1e-9, "T_a = sqrt(T_tr*T_ve)");
    }
}

#[test]
fn vibrational_lag_saturates_at_long_residence() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    // A long residence at high pressure fully relaxes the bath: T_a -> T_tr.
    let stage = VibrationalLagStage::new(250.0_f64, 1.0, 7.0, 3393.0, 10.0);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("T_tr", vec![20_000.0_f64]);
    stage.apply(&ctx, &mut field).expect("applies");
    let t_a = field.scalar("T_a").expect("T_a written")[0];
    assert!(
        (t_a - 20_000.0).abs() / 20_000.0 < 1e-3,
        "saturated controller approaches T_tr: {t_a}"
    );
}

#[test]
fn ionization_driven_by_the_lagged_controller_produces_fewer_electrons() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    let n_tot = 1.0e22_f64;

    // Hot translation drives near-saturation.
    let mut hot = CoupledField::new(Ambient::new(0.01, 0.0, None));
    hot.set_scalar("T_tr", vec![30_000.0_f64]);
    IonizationStage::new(n_tot)
        .apply(&ctx, &mut hot)
        .expect("applies");
    let ne_hot = hot.scalar("n_e").expect("n_e")[0];

    // The lagged controller suppresses both the Saha target and the rate.
    let mut lagged = CoupledField::new(Ambient::new(0.01, 0.0, None));
    lagged.set_scalar("T_tr", vec![30_000.0_f64]);
    lagged.set_scalar("T_a", vec![4_000.0_f64]);
    IonizationStage::new(n_tot)
        .driven_by("T_a")
        .apply(&ctx, &mut lagged)
        .expect("applies");
    let ne_lagged = lagged.scalar("n_e").expect("n_e")[0];

    assert!(
        ne_lagged < ne_hot,
        "the controller suppresses ionization: {ne_lagged} < {ne_hot}"
    );
}

#[test]
fn sheath_renewal_limits_ionization_to_one_residence_time() {
    let (manifold, state) = empty_context();
    let n_tot = 1.0e22_f64;

    // Accumulating relaxation: many steps drive the carried fraction to equilibrium.
    let carried = IonizationStage::new(n_tot);
    let mut accumulated = CoupledField::new(Ambient::new(0.01, 0.0, None));
    accumulated.set_scalar("T_tr", vec![8_000.0_f64]);
    for s in 0..200 {
        let ctx = StepContext::new(&manifold, &state, 0.004, s + 1);
        carried.apply(&ctx, &mut accumulated).expect("applies");
    }
    let ne_accumulated = accumulated.scalar("n_e").expect("n_e")[0];

    // Sheath renewal: the exposure stays one residence time however long the march runs.
    let renewed = IonizationStage::new(n_tot).with_sheath_renewal(2.0e-5);
    let mut sheath = CoupledField::new(Ambient::new(0.01, 0.0, None));
    sheath.set_scalar("T_tr", vec![8_000.0_f64]);
    let mut first = 0.0;
    for s in 0..200 {
        let ctx = StepContext::new(&manifold, &state, 0.004, s + 1);
        renewed.apply(&ctx, &mut sheath).expect("applies");
        if s == 0 {
            first = sheath.scalar("n_e").expect("n_e")[0];
        }
    }
    let ne_renewed = sheath.scalar("n_e").expect("n_e")[0];

    assert!(
        ne_renewed < ne_accumulated,
        "renewal caps the exposure: {ne_renewed} < {ne_accumulated}"
    );
    assert_eq!(
        ne_renewed, first,
        "the renewed sheath is stateless per step"
    );
    assert!(ne_renewed > 0.0, "one residence time still ionizes");
}

// ── Evolved per-cell inputs (the compressible-carrier chemistry path) ─────

#[test]
fn vibrational_lag_reads_the_evolved_pressure_per_cell() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    // Identical translation, but the evolved pressure differs per cell: the
    // Millikan-White clock runs faster at higher pressure, so the bath relaxes
    // further there.
    let stage =
        VibrationalLagStage::new(250.0_f64, 1.0e-3, 7.0, 3393.0, 1.0e-5).with_pressure_field("p");
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("T_tr", vec![20_000.0_f64, 20_000.0]);
    field.set_scalar("p", vec![1.0e-3_f64, 1.0]);
    stage.apply(&ctx, &mut field).expect("applies");

    let t_ve = field.scalar("T_ve").expect("T_ve written");
    assert!(
        t_ve[1] > t_ve[0],
        "higher evolved pressure relaxes the bath further: {} > {}",
        t_ve[1],
        t_ve[0]
    );
}

#[test]
fn vibrational_lag_falls_back_to_the_config_pressure() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    let constant = VibrationalLagStage::new(250.0_f64, 0.04, 7.0, 3393.0, 1.0e-5);
    // The named field is absent, so the per-cell reader falls back to the constant.
    let evolved = constant.with_pressure_field("pressure_atm");

    let mut a = CoupledField::new(Ambient::new(0.01, 0.0, None));
    a.set_scalar("T_tr", vec![20_000.0_f64]);
    constant.apply(&ctx, &mut a).expect("applies");

    let mut b = CoupledField::new(Ambient::new(0.01, 0.0, None));
    b.set_scalar("T_tr", vec![20_000.0_f64]);
    evolved.apply(&ctx, &mut b).expect("applies");

    assert_eq!(
        a.scalar("T_ve").unwrap()[0],
        b.scalar("T_ve").unwrap()[0],
        "absent field means the config constant"
    );
}

#[test]
fn ionization_reads_the_evolved_density_per_cell() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    // Identical temperature, but the evolved density differs per cell: the dense
    // cell both ionizes faster (shorter τ_ion) and carries more heavy particles,
    // so its electron density comes out higher.
    let stage = IonizationStage::new(1.0e22_f64)
        .with_density_field("n_tot")
        .with_sheath_renewal(2.0e-5);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("T_tr", vec![8_000.0_f64, 8_000.0]);
    field.set_scalar("n_tot", vec![1.0e20_f64, 1.0e22]);
    stage.apply(&ctx, &mut field).expect("applies");

    let n_e = field.scalar("n_e").expect("n_e written");
    assert!(n_e.iter().all(|&x| x > 0.0), "both cells ionize");
    assert!(
        n_e[1] > n_e[0],
        "the dense cell produces more electrons: {} > {}",
        n_e[1],
        n_e[0]
    );
}

#[test]
fn vibrational_lag_rejects_a_mismatched_pressure_field_length() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    // Length n is per-cell, length 1 broadcasts; a length-3 field against a
    // 2-cell grid is a shape bug that must surface, not silently read cell 0.
    let stage =
        VibrationalLagStage::new(250.0_f64, 1.0e-3, 7.0, 3393.0, 1.0e-5).with_pressure_field("p");
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("T_tr", vec![20_000.0_f64, 20_000.0]);
    field.set_scalar("p", vec![1.0e-3_f64, 1.0, 0.5]);
    let err = stage
        .apply(&ctx, &mut field)
        .expect_err("a length-3 pressure field against 2 cells is a shape bug");
    let msg = err.to_string();
    assert!(msg.contains("'p'"), "the error names the field: {msg}");
}

#[test]
fn ionization_rejects_a_mismatched_density_field_length() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    let stage = IonizationStage::new(1.0e22_f64).with_density_field("n_tot");
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("T_tr", vec![8_000.0_f64, 8_000.0]);
    field.set_scalar("n_tot", vec![1.0e20_f64, 1.0e22, 1.0e21]);
    let err = stage
        .apply(&ctx, &mut field)
        .expect_err("a length-3 density field against 2 cells is a shape bug");
    let msg = err.to_string();
    assert!(msg.contains("n_tot"), "the error names the field: {msg}");
}

#[test]
fn ionization_density_field_matches_the_scalar_config_when_equal() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.004, 1);
    let n_tot = 1.0e22_f64;

    let mut scalar_cfg = CoupledField::new(Ambient::new(0.01, 0.0, None));
    scalar_cfg.set_scalar("T_tr", vec![8_000.0_f64, 8_000.0]);
    IonizationStage::new(n_tot)
        .apply(&ctx, &mut scalar_cfg)
        .expect("applies");

    // A single-cell density field broadcasts the same value across the grid.
    let mut evolved = CoupledField::new(Ambient::new(0.01, 0.0, None));
    evolved.set_scalar("T_tr", vec![8_000.0_f64, 8_000.0]);
    evolved.set_scalar("n_tot", vec![n_tot]);
    IonizationStage::new(1.0_f64)
        .with_density_field("n_tot")
        .apply(&ctx, &mut evolved)
        .expect("applies");

    for i in 0..2 {
        assert_eq!(
            scalar_cfg.scalar("n_e").unwrap()[i],
            evolved.scalar("n_e").unwrap()[i],
            "the broadcast field reproduces the scalar config"
        );
    }
}
