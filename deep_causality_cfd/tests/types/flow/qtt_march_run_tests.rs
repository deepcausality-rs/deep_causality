/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    CfdFlow, MarchStop, QttIncompressible2d, QttMarchConfigBuilder, QttObserve,
};
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

    let report = CfdFlow::qtt_march(&cfg).run().unwrap();
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
    let report = CfdFlow::qtt_march(&cfg).run().unwrap();
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

    let report = CfdFlow::qtt_march(&cfg).run().unwrap();
    let n = report.series("kinetic_energy").unwrap().len();
    assert!(n < 151, "steady stop did not terminate early (n = {n})");
    assert!(n > 2, "steady stop terminated immediately (n = {n})");
}

#[test]
fn per_step_hook_fires_each_step_and_leaves_report_identical() {
    let (nu, dt, steps) = (0.05f64, 0.02f64, 6usize);
    let observe = QttObserve::default().kinetic_energy();

    let cfg = taylor_green_config(nu, dt, steps, observe);
    let baseline = CfdFlow::qtt_march(&cfg).run().unwrap();

    let cfg2 = taylor_green_config(nu, dt, steps, observe);
    let mut seen = Vec::new();
    let hooked = CfdFlow::qtt_march(&cfg2)
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
    let report = CfdFlow::qtt_march(&cfg)
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

    let report = CfdFlow::qtt_march(&cfg).run().unwrap();
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
