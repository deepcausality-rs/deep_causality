/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{MarchStop, QttMarchConfigBuilder, QttObserve};
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;

#[test]
fn seed_closure_materializes_over_the_grid() {
    let lx = 3usize; // 8
    let ly = 2usize; // 4
    let (nx, ny) = (8usize, 4usize);
    let dx = TAU / nx as f64;
    let dy = TAU / ny as f64;
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();

    let cfg = QttMarchConfigBuilder::<f64>::new()
        .grid(lx, ly, dx, dy)
        .solver(0.01, 0.05, trunc)
        .seed_fn(|x, y| (-(x.cos() * y.sin()), x.sin() * y.cos()))
        .unwrap()
        .stop(MarchStop::Fixed(5))
        .observe(QttObserve::default().kinetic_energy().bond())
        .build()
        .unwrap();

    // The materialized seed is the closure evaluated at each grid node (row-major [nx, ny]).
    let us = cfg.seed_u().as_slice();
    let vs = cfg.seed_v().as_slice();
    assert_eq!(cfg.seed_u().shape(), [nx, ny]);
    for i in 0..nx {
        for j in 0..ny {
            let (x, y) = (i as f64 * dx, j as f64 * dy);
            assert!((us[i * ny + j] - (-(x.cos() * y.sin()))).abs() <= 1e-15);
            assert!((vs[i * ny + j] - (x.sin() * y.cos())).abs() <= 1e-15);
        }
    }
}

#[test]
fn taylor_green_convenience_matches_the_closure() {
    let (lx, ly) = (3usize, 2usize);
    let (nx, ny) = (8usize, 4usize);
    let dx = TAU / nx as f64;
    let dy = TAU / ny as f64;
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();

    let via_tg = QttMarchConfigBuilder::<f64>::new()
        .grid(lx, ly, dx, dy)
        .solver(0.01, 0.05, trunc)
        .taylor_green()
        .unwrap()
        .build()
        .unwrap();
    let via_fn = QttMarchConfigBuilder::<f64>::new()
        .grid(lx, ly, dx, dy)
        .solver(0.01, 0.05, trunc)
        .seed_fn(|x, y| (-(x.cos() * y.sin()), x.sin() * y.cos()))
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(via_tg.seed_u().as_slice(), via_fn.seed_u().as_slice());
    assert_eq!(via_tg.seed_v().as_slice(), via_fn.seed_v().as_slice());
}

#[test]
fn seed_fn_requires_grid_first() {
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    let r = QttMarchConfigBuilder::<f64>::new()
        .solver(0.01, 0.05, trunc)
        .seed_fn(|_, _| (0.0, 0.0));
    assert!(r.is_err(), "seed_fn without a grid must error");
}

#[test]
fn rejects_mismatched_seed_shape() {
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    // Grid says 8 x 4, but the supplied fields are 4 x 4.
    let u0 = CausalTensor::new(vec![0.0; 16], vec![4, 4]).unwrap();
    let v0 = CausalTensor::new(vec![0.0; 16], vec![4, 4]).unwrap();
    let r = QttMarchConfigBuilder::<f64>::new()
        .grid(3, 2, 0.1, 0.1)
        .solver(0.01, 0.05, trunc)
        .seed_fields(u0, v0)
        .build();
    assert!(r.is_err(), "mismatched seed shape must be rejected");
}

#[test]
fn rejects_missing_grid_or_solver() {
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    // Missing solver.
    let r = QttMarchConfigBuilder::<f64>::new()
        .grid(2, 2, 0.1, 0.1)
        .seed_fields(
            CausalTensor::new(vec![0.0; 16], vec![4, 4]).unwrap(),
            CausalTensor::new(vec![0.0; 16], vec![4, 4]).unwrap(),
        )
        .build();
    assert!(r.is_err());
    // Missing grid.
    let r = QttMarchConfigBuilder::<f64>::new()
        .solver(0.01, 0.05, trunc)
        .build();
    assert!(r.is_err());
}

#[test]
fn rejects_missing_seed() {
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    // Grid + solver set, but no seed supplied.
    let r = QttMarchConfigBuilder::<f64>::new()
        .grid(2, 2, 0.1, 0.1)
        .solver(0.01, 0.05, trunc)
        .build();
    assert!(r.is_err(), "a missing seed must be rejected");
}

#[test]
fn config_accessors_report_name_and_modes() {
    let lx = 3usize; // 8
    let ly = 2usize; // 4
    let (nx, ny) = (8usize, 4usize);
    let dx = TAU / nx as f64;
    let dy = TAU / ny as f64;
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();

    let cfg = QttMarchConfigBuilder::<f64>::new()
        .name("named_case")
        .grid(lx, ly, dx, dy)
        .solver(0.01, 0.05, trunc)
        .seed_fn(|_, _| (0.0, 0.0))
        .unwrap()
        .build()
        .unwrap();

    // `name()` and `modes()` accessors.
    assert_eq!(cfg.name(), "named_case");
    assert_eq!(cfg.modes(), (lx, ly));
    assert_eq!(cfg.seed_u().shape(), [nx, ny]);
}

#[test]
fn default_name_when_unset() {
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    let cfg = QttMarchConfigBuilder::<f64>::new()
        .grid(2, 2, 0.1, 0.1)
        .solver(0.01, 0.05, trunc)
        .seed_fn(|_, _| (0.0, 0.0))
        .unwrap()
        .build()
        .unwrap();
    // No `.name(...)` call → the default case name.
    assert_eq!(cfg.name(), "qtt_march");
}

#[test]
fn blackout_observe_flags_chain_and_build() {
    // The blackout observe builders (`electron_density`/`plasma_frequency`/`blackout_dwell`) each
    // opt a series in; here we exercise the fluent chain and confirm the config still builds.
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    let observe = QttObserve::default()
        .electron_density()
        .plasma_frequency()
        .blackout_dwell();
    let cfg = QttMarchConfigBuilder::<f64>::new()
        .grid(2, 2, 0.1, 0.1)
        .solver(0.01, 0.05, trunc)
        .seed_fn(|_, _| (0.0, 0.0))
        .unwrap()
        .observe(observe)
        .build()
        .unwrap();
    assert_eq!(cfg.modes(), (2, 2));
}
