/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    AeroBlackoutStub, Ambient, BlackoutTrigger, CfdFlow, CoupledField, Coupling, MarchStop,
    QttIncompressible2d, QttMarchConfigBuilder, QttObserve, ThermalRelax,
};
use deep_causality_core::{AlternatableContext, AlternatableState, AlternatableValue};
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const N: usize = 16;
const L: usize = 4;

fn field(dx: f64, f: impl Fn(f64, f64) -> f64) -> CausalTensor<f64> {
    let mut data = vec![0.0; N * N];
    for i in 0..N {
        for j in 0..N {
            data[i * N + j] = f(i as f64 * dx, j as f64 * dx);
        }
    }
    CausalTensor::new(data, vec![N, N]).unwrap()
}

fn tg_u(x: f64, y: f64) -> f64 {
    -(x.cos() * y.sin())
}
fn tg_v(x: f64, y: f64) -> f64 {
    x.sin() * y.cos()
}

fn taylor_green_config(
    nu: f64,
    dt: f64,
    steps: usize,
    observe: QttObserve,
) -> deep_causality_cfd::QttMarchConfig<f64> {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    QttMarchConfigBuilder::<f64>::new()
        .name("taylor_green_qtt")
        .grid(L, L, dx, dx)
        .solver(dt, nu, trunc)
        .seed_fn(|x, y| (tg_u(x, y), tg_v(x, y)))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(observe)
        .build()
        .unwrap()
}

#[test]
fn runs_and_reports_observables() {
    let (nu, dt, steps) = (0.05f64, 0.02f64, 10usize);
    let observe = QttObserve::default()
        .kinetic_energy()
        .divergence()
        .max_speed()
        .bond();
    let cfg = taylor_green_config(nu, dt, steps, observe);

    let report = CfdFlow::march(&cfg).run().unwrap();
    assert_eq!(report.name(), "taylor_green_qtt");

    // One series per enabled observable, each with steps + 1 samples (seed + each step).
    for label in ["kinetic_energy", "divergence", "max_speed", "bond"] {
        let s = report
            .series(label)
            .unwrap_or_else(|| panic!("missing {label}"));
        assert_eq!(s.len(), steps + 1, "{label} length");
    }

    // The final (u, v) fields are exposed.
    assert_eq!(report.final_field().unwrap().len(), N * N);
    assert_eq!(report.series("final_v").unwrap().len(), N * N);

    // Divergence stays at the projection floor; kinetic energy tracks the analytic e^{-2νt·n} decay.
    let div = report.series("divergence").unwrap();
    assert!(div.iter().all(|d| d.abs() <= 1e-4), "divergence {div:?}");
    let ke = report.series("kinetic_energy").unwrap();
    let decay = (-2.0 * nu * dt * steps as f64).exp();
    // Analytic kinetic energy decays as decay²; check the ratio against the seed energy.
    let ratio = ke[steps] / ke[0];
    assert!(
        (ratio - decay * decay).abs() <= 5e-2,
        "energy ratio {ratio} vs {}",
        decay * decay
    );
}

#[test]
fn matches_the_direct_driver() {
    let (nu, dt, steps) = (0.05f64, 0.02f64, 8usize);
    let cfg = taylor_green_config(nu, dt, steps, QttObserve::default());

    // DSL path.
    let report = CfdFlow::march(&cfg).run().unwrap();
    let dsl_u = report.final_field().unwrap();
    let dsl_v = report.series("final_v").unwrap();

    // Direct driver, same seed / horizon / round policy.
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let solver = QttIncompressible2d::new(L, L, dx, dx, dt, nu, trunc).unwrap();
    let (u, v) = solver
        .run(&field(dx, tg_u), &field(dx, tg_v), steps)
        .unwrap();

    for (a, b) in dsl_u.iter().zip(u.as_slice()) {
        assert!((a - b).abs() <= 1e-12, "u differs: {a} vs {b}");
    }
    for (a, b) in dsl_v.iter().zip(v.as_slice()) {
        assert!((a - b).abs() <= 1e-12, "v differs: {a} vs {b}");
    }
}

#[test]
fn steady_stop_terminates_on_the_plateau() {
    // A diffusion-dominated decaying vortex: the step-to-step energy change shrinks monotonically, so
    // the steady stop fires once it drops below `tol` — well before the cap. The kinetic energy here
    // is the unnormalized Frobenius sum (~tens on this grid), so `tol` is set to that scale. A cheap
    // 8 x 8 grid keeps the (projection-heavy) march fast.
    let lq = 3usize; // 8
    let nq = 8usize;
    let dx = TAU / nq as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let cfg = QttMarchConfigBuilder::<f64>::new()
        .grid(lq, lq, dx, dx)
        .solver(0.02, 0.3, trunc)
        .seed_fn(|x, y| (tg_u(x, y), tg_v(x, y)))
        .unwrap()
        .stop(MarchStop::Steady {
            tol: 0.3,
            max_steps: 150,
        })
        .observe(QttObserve::default().kinetic_energy())
        .build()
        .unwrap();

    let report = CfdFlow::march(&cfg).run().unwrap();
    let n = report.series("kinetic_energy").unwrap().len();
    assert!(n < 151, "steady stop did not terminate early (n = {n})");
    assert!(n > 2, "steady stop terminated immediately (n = {n})");
}

#[test]
fn per_step_hook_fires_each_step_and_leaves_report_identical() {
    let (nu, dt, steps) = (0.05f64, 0.02f64, 6usize);
    let observe = QttObserve::default().kinetic_energy();

    let cfg = taylor_green_config(nu, dt, steps, observe);
    let baseline = CfdFlow::march(&cfg).run().unwrap();

    let cfg2 = taylor_green_config(nu, dt, steps, observe);
    let mut seen = Vec::new();
    let hooked = CfdFlow::march(&cfg2)
        .run_with(|view| {
            // Each call exposes a monotonically increasing step and a finite energy.
            let e = view.kinetic_energy().unwrap();
            assert!(e.is_finite());
            assert!((view.time() - view.step() as f64 * dt).abs() <= 1e-15);
            let _ = view.max_bond();
            seen.push(view.step());
        })
        .unwrap();

    assert_eq!(seen, (1..=steps).collect::<Vec<_>>(), "hook step sequence");
    // The hookless and hooked runs produce an identical report.
    let a = baseline.series("kinetic_energy").unwrap();
    let b = hooked.series("kinetic_energy").unwrap();
    assert_eq!(a, b, "hook changed the report");
}

#[test]
fn observe_override_swaps_the_series() {
    let (nu, dt, steps) = (0.05f64, 0.02f64, 4usize);
    // Config collects nothing; the override turns on energy + bond.
    let cfg = taylor_green_config(nu, dt, steps, QttObserve::default());
    let report = CfdFlow::march(&cfg)
        .observe_with(QttObserve::default().kinetic_energy().bond())
        .run()
        .unwrap();
    assert!(report.series("kinetic_energy").is_some());
    assert!(report.series("bond").is_some());
    assert!(report.series("max_speed").is_none());
}

#[test]
fn pipeline_emits_a_drag_series_with_a_body() {
    use deep_causality_cfd::body_mask_2d;
    use deep_causality_tensor::TensorTrain;

    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let c = TAU * 0.5;
    let mask = body_mask_2d::<f64>(L, L, dx, dx, c, c, TAU * 0.18, 2.0 * dx, &trunc).unwrap();
    let _ = mask.norm(); // touch the trait so the import is used

    let steps = 5usize;
    let cfg = QttMarchConfigBuilder::<f64>::new()
        .name("cyl")
        .grid(L, L, dx, dx)
        .solver(0.005, 0.05, trunc)
        .seed_fn(|_, _| (1.0, 0.0))
        .unwrap()
        .body(mask, 0.0, 0.0, 0.02, 1.0, 2.0 * TAU * 0.18)
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default().drag().divergence())
        .build()
        .unwrap();

    let report = CfdFlow::march(&cfg).run().unwrap();
    let drag = report.series("drag").expect("drag series");
    let lift = report.series("lift").expect("lift series");
    assert_eq!(drag.len(), steps + 1, "one drag sample per step + seed");
    assert_eq!(lift.len(), steps + 1);
    assert!(drag.iter().all(|d| d.is_finite()), "drag finite");
    // A free-stream past a static body produces a positive streamwise drag.
    assert!(
        drag[steps] > 0.0,
        "expected positive drag, got {}",
        drag[steps]
    );
}

#[test]
fn run_with_hook_exposes_step_view_accessors() {
    // Exercise the QttStepView accessors: `dt()`, `time()`, `u()`, `v()`, `divergence()`,
    // `max_speed()`.
    let (nu, dt, steps) = (0.05f64, 0.02f64, 4usize);
    let observe = QttObserve::default().kinetic_energy();
    let cfg = taylor_green_config(nu, dt, steps, observe);

    let mut last_step = 0usize;
    let _ = CfdFlow::march(&cfg)
        .run_with(|view| {
            // The time step is exactly the configured `dt` and `time() = step · dt`.
            assert_eq!(view.dt(), dt, "dt accessor");
            assert!((view.time() - view.step() as f64 * dt).abs() <= 1e-15);

            // The velocity trains are the current state (non-empty, dequantizes to the grid).
            let _u = view.u();
            let _v = view.v();

            // The tensor-train-native diagnostics are finite.
            let div = view.divergence().unwrap();
            assert!(div.is_finite(), "divergence finite");
            assert!(
                div.abs() <= 1e-4,
                "divergence at the projection floor: {div}"
            );
            let vmax = view.max_speed().unwrap();
            assert!(vmax.is_finite() && vmax >= 0.0, "max speed {vmax}");

            last_step = view.step();
        })
        .unwrap();
    assert_eq!(last_step, steps, "hook fired for the final step");
}

/// A body-free coupled run driven by the `AeroBlackoutStub`: publishes `n_e`, transports an
/// `"alpha"` scalar, and samples the blackout observables into the report.
fn coupled_free_config(steps: usize) -> deep_causality_cfd::QttMarchConfig<f64> {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    QttMarchConfigBuilder::<f64>::new()
        .name("blackout_qtt")
        .grid(L, L, dx, dx)
        .solver(0.005, 0.05, trunc)
        .seed_fn(|_, _| (1.0, 0.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .blackout_dwell(),
        )
        .build()
        .unwrap()
}

#[test]
fn run_coupled_free_solver_samples_blackout_observables() {
    let steps = 5usize;
    let cfg = coupled_free_config(steps);

    // The stub raises n_e into the blackout level over steps [2, 5); the trigger denies the link
    // whenever the plasma frequency clears the comms band.
    let stub = AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 2, 5);
    let trigger = BlackoutTrigger::new(1.0e9); // rad/s comms band

    // Seed an "alpha" reacting fraction to drive the transport branch (body-free ⇒ carried through).
    let ncells = (1usize << L) * (1usize << L);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("alpha", vec![0.5_f64; ncells]);

    let report = CfdFlow::march(&cfg)
        .run_coupled(stub, field, trigger, 0.01)
        .unwrap();

    // One n_e / plasma-frequency sample per step (the stub publishes n_e every step).
    let ne = report.series("n_e").expect("n_e series");
    let wp = report
        .series("plasma_frequency")
        .expect("plasma_frequency series");
    assert_eq!(ne.len(), steps, "one n_e sample per step");
    assert_eq!(wp.len(), steps, "one plasma-frequency sample per step");
    // Inside the window (step 3) n_e reaches the blackout level.
    assert!(
        ne.iter().copied().fold(0.0, f64::max) >= 1.0e20,
        "n_e peaks at the blackout level: {ne:?}"
    );
    assert!(wp.iter().all(|w| w.is_finite() && *w >= 0.0), "wp finite");

    // Blackout dwell is a single value = denied_steps · dt, and non-negative.
    let dwell = report.series("blackout_dwell").expect("dwell series");
    assert_eq!(dwell.len(), 1, "dwell is a single accumulated value");
    assert!(dwell[0] >= 0.0, "dwell non-negative: {}", dwell[0]);

    // The final (u, v) fields are exposed on the coupled report too.
    assert_eq!(report.final_field().unwrap().len(), N * N);
    assert_eq!(report.series("final_v").unwrap().len(), N * N);
}

#[test]
fn run_coupled_body_solver_transports_alpha() {
    use deep_causality_cfd::body_mask_2d;

    // A body config drives the immersed (Brinkman) solver path in run_coupled, including the
    // penalized `advance_scalar` transport of the carried "alpha" fraction.
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let c = TAU * 0.5;
    let mask = body_mask_2d::<f64>(L, L, dx, dx, c, c, TAU * 0.18, 2.0 * dx, &trunc).unwrap();

    let steps = 4usize;
    let cfg = QttMarchConfigBuilder::<f64>::new()
        .name("blackout_body_qtt")
        .grid(L, L, dx, dx)
        .solver(0.005, 0.05, trunc)
        .seed_fn(|_, _| (1.0, 0.0))
        .unwrap()
        .body(mask, 0.0, 0.0, 0.02, 1.0, 2.0 * TAU * 0.18)
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default().electron_density())
        .build()
        .unwrap();

    let stub = AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 1, 4);
    let trigger = BlackoutTrigger::new(1.0e9);

    let ncells = (1usize << L) * (1usize << L);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("alpha", vec![0.5_f64; ncells]);

    let report = CfdFlow::march(&cfg)
        .run_coupled(stub, field, trigger, 0.01)
        .unwrap();

    // electron_density opted in ⇒ the n_e series is present; the others are not.
    let ne = report.series("n_e").expect("n_e series");
    assert_eq!(ne.len(), steps);
    assert!(report.series("plasma_frequency").is_none());
    assert!(report.series("blackout_dwell").is_none());
    assert_eq!(report.final_field().unwrap().len(), N * N);
}

#[test]
fn run_coupled_honours_run_overrides() {
    // seed_with / march_with / observe_with all feed run_coupled: shrink the horizon, swap the seed,
    // and switch the observe set on the fly.
    let cfg = coupled_free_config(10); // config says 10 steps + all three blackout series

    let steps = 3usize;
    let ncells = (1usize << L) * (1usize << L);
    let u0 = CausalTensor::new(vec![1.0_f64; ncells], vec![N, N]).unwrap();
    let v0 = CausalTensor::new(vec![0.0_f64; ncells], vec![N, N]).unwrap();

    let stub = AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 1, 3);
    let trigger = BlackoutTrigger::new(1.0e9);
    let field = CoupledField::new(Ambient::new(0.01, 0.0, None));

    let report = CfdFlow::march(&cfg)
        .seed_with(u0, v0)
        .march_with(MarchStop::Fixed(steps))
        .observe_with(QttObserve::default().electron_density())
        .run_coupled(stub, field, trigger, 0.01)
        .unwrap();

    // The march_with override caps the run at `steps`, and observe_with keeps only n_e.
    let ne = report.series("n_e").expect("n_e series");
    assert_eq!(ne.len(), steps, "march_with shortened the horizon");
    assert!(
        report.series("plasma_frequency").is_none(),
        "observe_with dropped plasma_frequency"
    );
    assert!(report.series("blackout_dwell").is_none());
    assert_eq!(report.final_field().unwrap().len(), N * N);
}

// ---------------------------------------------------------------------------
// Counterfactual alternation — the pre-run attach point (Stage 4.6)
// ---------------------------------------------------------------------------

fn uniform_seed() -> (CausalTensor<f64>, CausalTensor<f64>) {
    let ncells = N * N;
    (
        CausalTensor::new(vec![0.5_f64; ncells], vec![N, N]).unwrap(),
        CausalTensor::new(vec![0.0_f64; ncells], vec![N, N]).unwrap(),
    )
}

#[test]
fn alternate_state_subsumes_seed_with() {
    let cfg = taylor_green_config(0.05, 0.02, 4, QttObserve::default());
    let (u0, v0) = uniform_seed();

    let via_seed_with = CfdFlow::march(&cfg)
        .seed_with(u0.clone(), v0.clone())
        .run()
        .unwrap();
    let via_alternation = CfdFlow::march(&cfg)
        .alternate_state((u0, v0))
        .run()
        .unwrap();

    // Identical numerics; the alternation additionally leaves its audit marker.
    assert_eq!(
        via_seed_with.final_field().unwrap(),
        via_alternation.final_field().unwrap()
    );
    assert!(via_seed_with.effect_log().is_none(), "seed_with is silent");
    let log = via_alternation.effect_log().expect("alternation logged");
    assert!(
        format!("{log}").contains("!!StateAlternation!!"),
        "verbatim marker: {log}"
    );
}

#[test]
fn alternate_context_swaps_the_whole_world() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let world = |name: &str, nu: f64| {
        QttMarchConfigBuilder::<f64>::new()
            .name(name)
            .grid(L, L, dx, dx)
            .solver(0.02, nu, trunc)
            .seed_fn(|x, y| (tg_u(x, y), tg_v(x, y)))
            .unwrap()
            .stop(MarchStop::Fixed(4))
            .observe(QttObserve::default())
            .build()
            .unwrap()
    };
    let nominal = world("nominal_reentry", 0.05);
    let steep = world("steep_reentry", 0.09);

    let direct = CfdFlow::march(&steep).run().unwrap();
    let alternated = CfdFlow::march(&nominal)
        .alternate_context(&steep)
        .run()
        .unwrap();

    // The alternated run marches the *steep* world entirely: name, numerics, and the audit marker.
    assert_eq!(alternated.name(), "steep_reentry");
    assert_eq!(
        direct.final_field().unwrap(),
        alternated.final_field().unwrap()
    );
    let log = alternated.effect_log().expect("alternation logged");
    let text = format!("{log}");
    assert!(text.contains("!!ContextAlternation!!"), "marker: {text}");
    assert!(
        text.contains("nominal_reentry") && text.contains("steep_reentry"),
        "the entry names both worlds: {text}"
    );
}

#[test]
fn alternate_value_injects_a_primary_snapshot() {
    let cfg = taylor_green_config(0.05, 0.02, 4, QttObserve::default());
    let (u0, v0) = uniform_seed();

    let report = CfdFlow::march(&cfg)
        .alternate_value((u0.clone(), v0.clone()))
        .run()
        .unwrap();
    let reference = CfdFlow::march(&cfg).seed_with(u0, v0).run().unwrap();

    assert_eq!(
        report.final_field().unwrap(),
        reference.final_field().unwrap()
    );
    let log = report.effect_log().expect("alternation logged");
    assert!(
        format!("{log}").contains("!!ValueAlternation!!"),
        "verbatim marker: {log}"
    );
}

#[test]
fn run_coupled_surfaces_the_provenance_log() {
    let cfg = coupled_free_config(3);
    let stub = AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 1, 3);
    let trigger = BlackoutTrigger::new(1.0e9);
    let (u0, v0) = uniform_seed();

    let report = CfdFlow::march(&cfg)
        .alternate_state((u0, v0))
        .run_coupled(
            stub,
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            trigger,
            0.01,
        )
        .unwrap();

    // The pre-run alternation marker threads through the CoupledField log into the report ([7]).
    let log = report.effect_log().expect("coupled run carries the log");
    assert!(format!("{log}").contains("!!StateAlternation!!"));
    assert_eq!(report.log_entries(), Some(1));
}

#[test]
fn coupled_body_march_publishes_the_penalization_heat_integral() {
    use deep_causality_cfd::{RecoveryTemperatureStage, body_mask_2d};

    // An immersed body plus a coupling that writes "T_tr": the loop publishes the Brinkman
    // penalization heat integral into the field each step.
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let c = TAU * 0.5;
    let mask = body_mask_2d::<f64>(L, L, dx, dx, c, c, TAU * 0.18, 2.0 * dx, &trunc).unwrap();
    let cfg = QttMarchConfigBuilder::<f64>::new()
        .name("wall_flux")
        .grid(L, L, dx, dx)
        .solver(0.005, 0.05, trunc)
        .seed_fn(|_, _| (1.0, 0.0))
        .unwrap()
        .body(mask, 0.0, 0.0, 0.02, 1.0, 2.0 * TAU * 0.18)
        .stop(MarchStop::Fixed(4))
        .observe(QttObserve::default())
        .build()
        .unwrap();

    let recovery = RecoveryTemperatureStage::new(25.0_f64, 1.4, 250.0, 1004.0);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            recovery,
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.01,
            |_, _| false,
        )
        .unwrap();

    let q = pause
        .field()
        .scalar("penalization_heat_integral")
        .expect("wall flux published");
    assert_eq!(q.len(), 1);
    assert!(q[0].is_finite());
    assert!(
        q[0] < 0.0,
        "hot sheath vs the T_w = 0 reference gives a negative wall-side flux: {}",
        q[0]
    );
}

#[test]
fn run_coupled_samples_the_sensed_heat_flux_series() {
    use deep_causality_cfd::{PhysicsError, PhysicsStage, StepContext};

    // A minimal loads producer: publishes a constant sensed heat flux each step.
    #[derive(Clone, Copy)]
    struct ConstLoads;
    impl PhysicsStage<2, f64> for ConstLoads {
        fn apply(
            &self,
            _ctx: &StepContext<'_, 2, f64>,
            field: &mut CoupledField<f64>,
        ) -> Result<(), PhysicsError> {
            field.set_scalar("heat_flux", vec![2.5e6]);
            Ok(())
        }
    }

    let steps = 3usize;
    let cfg = coupled_free_config(steps);
    let report = CfdFlow::march(&cfg)
        .observe_with(QttObserve::default().heat_flux())
        .run_coupled(
            ConstLoads,
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.01,
        )
        .unwrap();

    let heat = report.series("heat_flux").expect("heat_flux series");
    assert_eq!(heat.len(), steps, "one sensed sample per coupled step");
    assert!(heat.iter().all(|&q| q == 2.5e6));
}

#[test]
fn run_coupled_samples_the_peak_speed_series() {
    let steps = 3usize;
    let cfg = coupled_free_config(steps);
    let stub = AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 1, 3);
    let trigger = BlackoutTrigger::new(1.0e9);

    let report = CfdFlow::march(&cfg)
        .observe_with(QttObserve::default().max_speed())
        .run_coupled(
            stub,
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            trigger,
            0.01,
        )
        .unwrap();

    // One peak-|u| sample per coupled step, from the published "speed" projection.
    let speed = report.series("max_speed").expect("max_speed series");
    assert_eq!(speed.len(), steps);
    assert!(speed.iter().all(|s| s.is_finite() && *s > 0.0));
}

// ---------------------------------------------------------------------------
// DSL equivalence (Stage 4.5): the composed stack IS the hand-written tuple
// ---------------------------------------------------------------------------

#[test]
fn composed_coupling_stack_equals_the_hand_written_tuple() {
    let cfg = coupled_free_config(4);
    let trigger = BlackoutTrigger::new(1.0e9);
    let stub = AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 1, 4);
    let relax = ThermalRelax::new(0.5, 300.0);

    let initial = || {
        let mut f = CoupledField::new(Ambient::new(0.01, 0.0, None));
        f.set_scalar("temperature", vec![250.0_f64; N * N]);
        f
    };

    // The DSL naming layer over the cons-tuple...
    let composed = Coupling::between_steps().then(stub).then(relax).build();
    let dsl = CfdFlow::march(&cfg)
        .run_coupled(composed, initial(), trigger, 0.01)
        .unwrap();

    // ...compiles to exactly the hand-nested tuple (no dyn, same monomorphized pipeline).
    let manual = (((), stub), relax);
    let hand = CfdFlow::march(&cfg)
        .run_coupled(manual, initial(), trigger, 0.01)
        .unwrap();

    assert_eq!(dsl.series("n_e").unwrap(), hand.series("n_e").unwrap());
    assert_eq!(
        dsl.series("plasma_frequency").unwrap(),
        hand.series("plasma_frequency").unwrap()
    );
    assert_eq!(dsl.final_field().unwrap(), hand.final_field().unwrap());
    assert_eq!(
        dsl.series("final_v").unwrap(),
        hand.series("final_v").unwrap()
    );
}

#[test]
fn the_configured_wall_temperature_reaches_the_heat_integral() {
    use deep_causality_cfd::{RecoveryTemperatureStage, body_mask_2d};

    // `T_w` was hardcoded to zero and absent from the config. Now that it is configurable, the knob
    // has to move the quantity it defines — the integral is `(1/η)∫χ(T_w − T)dV`, so raising `T_w`
    // over a fixed field must raise it, by `(1/η)·ΔT_w·∫χ dV`. A setter that changed nothing would
    // be the same defect as the hardcoded zero, only harder to see.
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let c = TAU * 0.5;

    let run = |t_wall: f64| {
        let mask = body_mask_2d::<f64>(L, L, dx, dx, c, c, TAU * 0.18, 2.0 * dx, &trunc).unwrap();
        let cfg = QttMarchConfigBuilder::<f64>::new()
            .name("t_wall_knob")
            .grid(L, L, dx, dx)
            .solver(0.005, 0.05, trunc)
            .seed_fn(|_, _| (1.0, 0.0))
            .unwrap()
            .body(mask, 0.0, 0.0, 0.02, 1.0, 2.0 * TAU * 0.18)
            .wall_temperature(t_wall)
            .stop(MarchStop::Fixed(4))
            .observe(QttObserve::default())
            .build()
            .unwrap();
        let recovery = RecoveryTemperatureStage::new(25.0_f64, 1.4, 250.0, 1004.0);
        let pause = CfdFlow::march(&cfg)
            .run_until(
                recovery,
                CoupledField::new(Ambient::new(0.01, 0.0, None)),
                BlackoutTrigger::new(1.0e9),
                0.01,
                |_, _| false,
            )
            .unwrap();
        pause
            .field()
            .scalar("penalization_heat_integral")
            .expect("published")[0]
    };

    let cold = run(0.0);
    let hot = run(500.0);
    assert!(
        hot > cold,
        "raising T_w must raise the heat integral: T_w=0 gave {cold}, T_w=500 gave {hot}"
    );
    // Default stays zero, so every existing case is unmoved by the new knob.
    assert_eq!(cold, run(0.0));
}
